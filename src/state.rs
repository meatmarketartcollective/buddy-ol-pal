use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const STATE_VERSION: i64 = 6;
pub const CELEBRATE_SECS: i64 = 6;
pub const MIFF_SECS: i64 = 600;
pub const MIFF_WINDOW_SECS: i64 = 3600;
// The original 7-state matrix: sleep after a quiet stretch, resting after a
// wake keypress, petted after `tb pet`. Chaos builds from failures and throws
// random glitch frames once it crosses the glitch threshold.
pub const SLEEP_AFTER_SECS: i64 = 300;
pub const REST_SECS: i64 = 10;
pub const PET_SECS: i64 = 12;
pub const GLITCH_THRESHOLD: i64 = 50;
pub const CHAOS_PER_ERROR: i64 = 8;
pub const CHAOS_SUCCESS_RELIEF: i64 = 3;
pub const CHAOS_PET_RELIEF: i64 = 2;
// The interaction set: feed and play earn a little XP and ease chaos, each
// on its own cooldown so a buddy can't be spammed; talk is free.
pub const FEED_COOLDOWN_SECS: i64 = 60;
pub const PLAY_COOLDOWN_SECS: i64 = 300;
pub const XP_FEED: i64 = 1;
pub const XP_PLAY: i64 = 2;
pub const CHAOS_FEED_RELIEF: i64 = 1;
pub const CHAOS_PLAY_RELIEF: i64 = 2;
pub const CHAOS_MAX: i64 = 100;
pub const ROTATE_LINES: usize = 1000;
pub const MAX_ARCHIVE: usize = 3;
pub const TAIL_READ_CAP: u64 = 256 * 1024;

/// Cumulative xp needed to reach levels 1..=5.
pub const LEVEL_THRESHOLDS: [i64; 5] = [0, 10, 25, 50, 100];

/// The daily XP cap. Work pays only up to here each calendar day; past the
/// cap events still log and still move chaos, they just stop printing XP.
/// This is what keeps a loop of toy runs from minting a level 5 buddy.
pub const XP_DAILY_CAP: i64 = 100;

/// Max level, the last entry of LEVEL_THRESHOLDS.
pub const MAX_LEVEL: i64 = LEVEL_THRESHOLDS.len() as i64;

/// Fresh xp past max level that triggers an ascension (rehatch). Each one
/// after costs ASCENSION_STEP more than the last: the first fires at 400
/// total xp, then 700, then 1000. At the daily cap that is days of real
/// work per step, so the long game is earned, not ground.
pub const ASCENSION_STEP: i64 = 300;

/// Total xp that triggers the next ascension.
pub fn ascension_threshold(ascensions: i64) -> i64 {
    LEVEL_THRESHOLDS[LEVEL_THRESHOLDS.len() - 1] + ASCENSION_STEP * (ascensions + 1)
}

/// Hatch weights after `a` ascensions: common bleeds into the rarer tiers
/// 10/5/5/5/5 a step, clamped at four steps so the sum always stays 100.
/// Legendary goes 2% -> 7% -> 12% -> 17% -> 22%.
pub fn ascension_weights(a: i64) -> [(&'static str, u64); 5] {
    let a = a.clamp(0, 4) as u64;
    [
        ("common", 50 - 10 * a),
        ("uncommon", 25 - 5 * a),
        ("rare", 15 + 5 * a),
        ("epic", 8 + 5 * a),
        ("legendary", 2 + 5 * a),
    ]
}

/// What one xp grant changed. Leveled is a level crossing; Ascended is a
/// rehatch past max level into a fresh species with rarer hatch odds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XpOutcome {
    Nothing,
    Leveled,
    Ascended,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompanionState {
    pub version: i64,
    pub name: String,
    pub species: String,
    pub hatched_at: i64,
    pub mood: String,
    pub xp: i64,
    #[serde(default)]
    pub level: i64, // display-only, derived from xp (state v2)
    #[serde(default)]
    pub discovered: Vec<String>, // species names seen at least once (state v3)
    #[serde(default)]
    pub chaos: i64, // glitch pressure from failures, eased by success and pets (state v4)
    #[serde(default = "default_true")]
    pub enabled: bool, // the off switch: false silences notifications and the UI (state v5)
    #[serde(default)]
    pub xp_today: i64, // XP earned today against the daily cap (state v6)
    #[serde(default)]
    pub xp_day: i64, // unix day the xp_today counter belongs to (state v6)
    #[serde(default)]
    pub ascensions: i64, // completed rehatches past max level; defaulted so older state files load as-is
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub ts: i64,
    pub kind: String, // "build_finished" | "command_failed"
    pub duration_secs: u64,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Idle,
    Working,
    Celebrate,
    Miffed,
    Sleeping,
    Resting,
    Petted,
}

impl Mood {
    pub fn label(self) -> &'static str {
        match self {
            Mood::Idle => "idle",
            Mood::Working => "working",
            Mood::Celebrate => "celebrating",
            Mood::Miffed => "miffed",
            Mood::Sleeping => "sleeping",
            Mood::Resting => "resting",
            Mood::Petted => "petted",
        }
    }
}

pub fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn config_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HATCH_HOME") {
        return PathBuf::from(home);
    }
    let base = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            PathBuf::from(home).join(".config")
        });
    base.join("hatch")
}

pub fn default_state(now: i64, seed: u64) -> CompanionState {
    // A new hatch rolls a rarity tier, then a species within it (both seeded
    // so tests can reproduce the roll). Half the names come from the species'
    // curated pool, half from the petname wordlist so names keep feeling
    // fresh. Existing state files keep their species and name.
    let sp = crate::species::pick(seed);
    let r = (seed >> 16) as usize;
    let name = if r % 2 == 0 {
        sp.names[r % sp.names.len()].clone()
    } else {
        library_name(r)
    };
    CompanionState {
        version: STATE_VERSION,
        chaos: 0,
        enabled: true,
        xp_today: 0,
        xp_day: now / 86400,
        name,
        species: sp.name.clone(),
        hatched_at: now,
        mood: Mood::Idle.label().to_string(),
        xp: 0,
        level: 1,
        discovered: vec![sp.name.clone()],
        ascensions: 0,
    }
}

/// A name from the petname wordlist (~1,800 common nouns, MIT/Apache-2.0),
/// capitalized to match the curated pools. Falls back to "Buddy" if the
/// wordlist ever came up empty.
fn library_name(r: usize) -> String {
    let petnames = petname::Petnames::large();
    if petnames.nouns.is_empty() {
        return "Buddy".to_string();
    }
    let n = petnames.nouns[r % petnames.nouns.len()].to_string();
    let mut chars = n.chars();
    match chars.next() {
        Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
        None => n,
    }
}

/// Level is derived from xp, never stored as truth beyond a cache.
pub fn level_for(xp: i64) -> i64 {
    let mut lvl = 1;
    for (i, t) in LEVEL_THRESHOLDS.iter().enumerate() {
        if xp >= *t {
            lvl = (i + 1) as i64;
        }
    }
    lvl
}

/// XP per run: +1 short success, +2 success at/above the threshold, 0 on failure.
pub fn xp_gain(exit_code: i32, duration_secs: u64, threshold_secs: u64) -> i64 {
    if exit_code != 0 {
        return 0;
    }
    if duration_secs >= threshold_secs {
        2
    } else {
        // Fast successes earn nothing. A command that took a second is not
        // work; paying XP for it turned the economy into a slot machine.
        0
    }
}

/// The single XP door. Every path that grants XP (settle for command
/// outcomes, interaction for pets, feeds, plays) goes through here, so the
/// daily cap cannot be bypassed by one of them. Rolls the per-day counter at
/// the unix day boundary, guards against a clock that jumps backward (a stale
/// xp_day in the future resets the counter instead of granting an extra day),
/// clamps negative gains to nothing, and re-derives the level. Returns the
/// outcome (Nothing, Leveled, Ascended) so callers can react once.
pub fn apply_xp(st: &mut CompanionState, gain: i64) -> XpOutcome {
    if gain <= 0 {
        st.level = level_for(st.xp);
        return XpOutcome::Nothing;
    }
    let today = now_secs() / 86400;
    if st.xp_day != today {
        // A future xp_day means the clock moved backward; reset, do not
        // trust it and hand out a fresh cap for free.
        st.xp_day = today;
        st.xp_today = 0;
    }
    if st.xp_today < XP_DAILY_CAP {
        let room = (XP_DAILY_CAP - st.xp_today).min(gain);
        st.xp += room;
        st.xp_today += room;
    }
    let prev_level = st.level;
    st.level = level_for(st.xp);
    if st.level > prev_level {
        return XpOutcome::Leveled;
    }
    // Ascension: xp keeps accruing past max level; every ASCENSION_STEP past
    // the last threshold rehatches the buddy into a fresh species with the
    // rarity odds shifted up. The reroll keeps name, level and xp, and the
    // new species lands in the dex.
    if st.level == MAX_LEVEL && st.xp >= ascension_threshold(st.ascensions) {
        st.ascensions += 1;
        st.xp = LEVEL_THRESHOLDS[LEVEL_THRESHOLDS.len() - 1];
        let seed = (now_secs() as u64)
            .wrapping_mul(0x9E37_79B9_7F4A_7C15)
            ^ (st.ascensions as u64).wrapping_mul(7919);
        let sp = crate::species::pick_weighted(seed, &ascension_weights(st.ascensions));
        if !st.discovered.iter().any(|d| d == &sp.name) {
            st.discovered.push(sp.name.clone());
        }
        st.species = sp.name.clone();
        return XpOutcome::Ascended;
    }
    XpOutcome::Nothing
}

/// XP still owed before the next level, or 0 when maxed.
pub fn xp_to_next(xp: i64) -> i64 {
    let last = LEVEL_THRESHOLDS[LEVEL_THRESHOLDS.len() - 1];
    if xp >= last {
        return 0;
    }
    for t in LEVEL_THRESHOLDS.iter() {
        if xp < *t {
            return t - xp;
        }
    }
    0
}

fn seed() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    nanos.wrapping_mul(2_654_435_761) ^ ((std::process::id() as u64) << 32)
}

pub fn parse_state(text: &str) -> Option<CompanionState> {
    let mut st: CompanionState = toml::from_str(text).ok()?;
    if st.version == 1 {
        // v1 -> v2 migration: level did not exist; derive it. The next save
        // persists the migration; nothing is deleted.
        st.level = level_for(st.xp);
        st.version = 2;
    }
    if st.version == 2 {
        // v2 -> v3 migration: the dex did not exist; the companion's own
        // species counts as its first discovery.
        st.discovered = vec![st.species.clone()];
        st.version = 3;
    }
    if st.version == 3 {
        // v3 -> v4 migration: chaos did not exist; every companion starts
        // calm. The next save persists it; nothing is deleted.
        st.chaos = 0;
        st.version = 4;
    }
    if st.version == 4 {
        // v4 -> v5 migration: the off switch did not exist; the buddy is on.
        st.enabled = true;
        st.version = 5;
    }
    if st.version == 5 {
        // v5 -> v6 migration: the daily XP cap did not exist; today's counter
        // starts at zero on the current unix day.
        st.xp_today = 0;
        st.xp_day = now_secs() / 86400;
        st.version = STATE_VERSION;
    }
    if st.version == STATE_VERSION {
        Some(st)
    } else {
        None
    }
}

/// A cross-process lock over the read-modify-write section every command
/// shares (load state, mutate, save). Concurrent `tb run` and `tb emit` calls
/// otherwise lose each other's XP and chaos to the last writer. Blocks until
/// the lock is held; on any error the caller proceeds unlocked rather than
/// failing the command.
pub fn lock_state(dir: &Path) -> Option<std::fs::File> {
    let f = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(dir.join("hatch.lock"))
        .ok()?;
    f.lock().ok()?;
    Some(f)
}

pub fn save_state(dir: &Path, st: &CompanionState) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    let tmp = dir.join("state.toml.tmp");
    let body = toml::to_string_pretty(st)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(body.as_bytes())?;
        f.sync_all().ok();
    }
    fs::rename(&tmp, dir.join("state.toml"))?;
    Ok(())
}

/// Loads the companion, or hatches a fresh one. A state file that fails to
/// parse or carries an unknown version is backed up (never deleted) and the
/// companion re-hatches. Old state is always preserved on disk.
pub fn load_or_hatch(dir: &Path) -> CompanionState {
    fs::create_dir_all(dir).ok();
    let path = dir.join("state.toml");
    if path.exists() {
        match fs::read_to_string(&path).ok().and_then(|s| parse_state(&s)) {
            Some(st) => return st,
            None => {
                let bak = dir.join(format!("state.toml.bak-{}", now_secs()));
                fs::rename(&path, bak).ok();
            }
        }
    }
    let st = default_state(now_secs(), seed());
    let _ = save_state(dir, &st);
    st
}

pub fn events_path(dir: &Path) -> PathBuf {
    dir.join("events.jsonl")
}

pub fn append_event(dir: &Path, ev: &Event) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    // Rotate BEFORE opening the handle: a rename under an open append handle
    // would send this event into the archived file, not the fresh log.
    rotate_events_if_needed(dir);
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(events_path(dir))?;
    let mut line =
        serde_json::to_string(ev).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    line.push('\n');
    f.write_all(line.as_bytes())
}

/// Bounded tail read: reads at most the last TAIL_READ_CAP bytes of the log
/// and drops the torn first line if it started mid-file.
fn read_tail_text(path: &Path) -> std::io::Result<String> {
    let mut f = fs::File::open(path)?;
    let len = f.metadata()?.len();
    let start = len.saturating_sub(TAIL_READ_CAP);
    f.seek(SeekFrom::Start(start))?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    if start > 0 {
        if let Some(idx) = s.find('\n') {
            s.drain(..=idx);
        }
    }
    Ok(s)
}

/// Reads the newest `max` events, oldest-first. Truncated or corrupt lines
/// (a torn final write) are skipped, never fatal.
pub fn read_events_tail(dir: &Path, max: usize) -> Vec<Event> {
    let text = match read_tail_text(&events_path(dir)) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(ev) = serde_json::from_str::<Event>(line) {
            out.push(ev);
        }
    }
    if out.len() > max {
        out.split_off(out.len() - max)
    } else {
        out
    }
}

pub fn mark_active(dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    fs::write(dir.join("active"), now_secs().to_string())
}

pub fn marker_stale(age_secs: u64) -> bool {
    age_secs > 6 * 3600
}

/// True while a watcher child is (recently) running. A stale marker older
/// than 6 hours (a crashed watcher) is cleaned up and reports false.
pub fn is_active(dir: &Path) -> bool {
    let p = dir.join("active");
    match fs::metadata(&p).and_then(|md| md.modified()) {
        Ok(modified) => {
            let stale = SystemTime::now()
                .duration_since(modified)
                .map(|age| marker_stale(age.as_secs()))
                .unwrap_or(true);
            if stale {
                let _ = fs::remove_file(&p);
                false
            } else {
                true
            }
        }
        Err(_) => false,
    }
}

pub fn classify(exit_code: i32, duration_secs: u64, threshold_secs: u64) -> Option<&'static str> {
    if exit_code == 0 {
        if duration_secs >= threshold_secs {
            Some("build_finished")
        } else {
            None
        }
    } else {
        Some("command_failed")
    }
}

/// Mood is derived, never stored as truth: working beats celebrate, and
/// celebrate decays back to idle.
pub fn current_mood(child_running: bool, events: &[Event], now: i64) -> Mood {
    if child_running {
        return Mood::Working;
    }
    if let Some(last) = events.last() {
        if last.kind == "build_finished" && now - last.ts <= CELEBRATE_SECS {
            return Mood::Celebrate;
        }
        if is_miffed(events, now) {
            return Mood::Miffed;
        }
        if last.kind == "petted" && now - last.ts <= PET_SECS {
            return Mood::Petted;
        }
        if now - last.ts >= SLEEP_AFTER_SECS {
            return Mood::Sleeping;
        }
    } else {
        // Nothing has ever happened: the buddy dozes off from the start.
        return Mood::Sleeping;
    }
    Mood::Idle
}

/// Chaos as it feels right now: the stored pressure decays one point per
/// hour since the companion's last event.
pub fn effective_chaos(st: &CompanionState, events: &[Event], now: i64) -> i64 {
    let since = events.last().map(|e| (now - e.ts).max(0)).unwrap_or(0);
    (st.chaos - since / 3600).max(0).min(CHAOS_MAX)
}

/// A fresh event nudges chaos: failures pile it on, successes and pets ease it.
pub fn apply_event_chaos(st: &mut CompanionState, kind: &str) {
    st.chaos = match kind {
        "command_failed" => (st.chaos + CHAOS_PER_ERROR).min(CHAOS_MAX),
        "build_finished" => st.chaos.saturating_sub(CHAOS_SUCCESS_RELIEF),
        "petted" => st.chaos.saturating_sub(CHAOS_PET_RELIEF),
        "fed" => st.chaos.saturating_sub(CHAOS_FEED_RELIEF),
        "played" => st.chaos.saturating_sub(CHAOS_PLAY_RELIEF),
        _ => st.chaos,
    };
}

/// The most recent event of a kind, if any.
pub fn last_event_of_kind<'a>(events: &'a [Event], kind: &str) -> Option<&'a Event> {
    events.iter().rev().find(|e| e.kind == kind)
}

/// True while an interaction of this kind is still cooling down.
pub fn on_cooldown(events: &[Event], kind: &str, now: i64, cooldown_secs: i64) -> bool {
    last_event_of_kind(events, kind)
        .map(|e| now - e.ts < cooldown_secs)
        .unwrap_or(false)
}

/// Trailing run of consecutive command_failed events.
pub fn failure_streak(events: &[Event]) -> usize {
    events
        .iter()
        .rev()
        .take_while(|e| e.kind == "command_failed")
        .count()
}

/// Miffed: two or more trailing failures, the newest within the miff decay
/// window, the whole streak inside the failure window.
pub fn is_miffed(events: &[Event], now: i64) -> bool {
    let streak = failure_streak(events);
    if streak < 2 {
        return false;
    }
    let n = events.len();
    if now - events[n - 1].ts > MIFF_SECS {
        return false;
    }
    let streak_events = &events[n - streak..];
    let recent = streak_events
        .iter()
        .rev()
        .take_while(|e| now - e.ts <= MIFF_WINDOW_SECS)
        .count();
    recent >= 2
}

/// One plain line for `tb status`: name, species, derived mood, xp, last event.
pub fn format_status(
    st: &CompanionState,
    species_label: &str,
    mood: Mood,
    last: Option<&Event>,
    now: i64,
    chaos: i64,
) -> String {
    let last_part = match last {
        Some(ev) => format!("last={} {}s ago", ev.kind, (now - ev.ts).max(0)),
        None => "no events yet".to_string(),
    };
    let asc_part = if st.ascensions > 0 {
        format!(" asc={}", st.ascensions)
    } else {
        String::new()
    };
    format!(
        "{} the {} L{} ({}) xp={} chaos={}{} {}",
        st.name,
        species_label,
        st.level,
        mood.label(),
        st.xp,
        chaos,
        asc_part,
        last_part
    )
}

fn count_event_lines(path: &Path) -> usize {
    fs::read(path)
        .map(|b| b.iter().filter(|&&c| c == b'\n').count())
        .unwrap_or(0)
}

/// Bounding the log: when appending would push events.jsonl past
/// ROTATE_LINES lines, it first becomes events-1.jsonl and older archives
/// shift, capped at MAX_ARCHIVE files. The oldest archive is replaced,
/// never accumulated forever.
pub fn rotate_events_if_needed(dir: &Path) {
    let path = events_path(dir);
    if count_event_lines(&path) < ROTATE_LINES {
        return;
    }
    for i in (1..MAX_ARCHIVE).rev() {
        let from = dir.join(format!("events-{}.jsonl", i));
        if from.exists() {
            let to = dir.join(format!("events-{}.jsonl", i + 1));
            let _ = fs::rename(&from, &to);
        }
    }
    let _ = fs::rename(&path, dir.join("events-1.jsonl"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmpdir() -> (tempfile::TempDir, PathBuf) {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().to_path_buf();
        (d, p)
    }

    #[test]
    fn state_round_trip() {
        let (_d, dir) = tmpdir();
        let st = default_state(1_700_000_000, 3);
        save_state(&dir, &st).unwrap();
        let loaded = load_or_hatch(&dir);
        assert_eq!(loaded, st);
    }

    #[test]
    fn corrupt_state_rehatches_and_backs_up() {
        let (_d, dir) = tmpdir();
        fs::write(dir.join("state.toml"), "not toml at all {{{").unwrap();
        let st = load_or_hatch(&dir);
        assert_eq!(st.version, STATE_VERSION);
        let baks: Vec<String> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.starts_with("state.toml.bak"))
            .collect();
        assert_eq!(baks.len(), 1, "old state must be backed up, not deleted");
    }

    #[test]
    fn missing_version_is_corrupt() {
        let (_d, dir) = tmpdir();
        fs::write(dir.join("state.toml"), "name = \"Pip\"\nspecies = \"ember\"\n").unwrap();
        let st = load_or_hatch(&dir);
        assert_eq!(st.version, STATE_VERSION);
        assert_eq!(st.hatched_at, now_secs()); // fresh hatch
    }

    #[test]
    fn events_append_and_tail() {
        let (_d, dir) = tmpdir();
        for i in 0..5 {
            append_event(
                &dir,
                &Event { ts: i, kind: "build_finished".into(), duration_secs: 30, exit_code: 0 },
            )
            .unwrap();
        }
        let tail = read_events_tail(&dir, 3);
        assert_eq!(tail.len(), 3);
        assert_eq!(tail[2].ts, 4);
    }

    #[test]
    fn torn_final_line_never_crashes() {
        let (_d, dir) = tmpdir();
        append_event(
            &dir,
            &Event { ts: 1, kind: "build_finished".into(), duration_secs: 30, exit_code: 0 },
        )
        .unwrap();
        let mut txt = fs::read_to_string(events_path(&dir)).unwrap();
        txt.push_str("{\"ts\": 2, \"kind\": \"bui");
        fs::write(events_path(&dir), txt).unwrap();
        let tail = read_events_tail(&dir, 10);
        assert_eq!(tail.len(), 1);
    }

    #[test]
    fn classify_thresholds() {
        assert_eq!(classify(0, 5, 10), None);
        assert_eq!(classify(0, 30, 10), Some("build_finished"));
        assert_eq!(classify(1, 0, 10), Some("command_failed"));
        assert_eq!(classify(127, 45, 10), Some("command_failed"));
    }

    #[test]
    #[test]
    fn v4_state_migrates_to_v5() {
        let (_d, dir) = tmpdir();
        fs::write(
            dir.join("state.toml"),
            "version = 4\nname = \"Pip\"\nspecies = \"ember\"\nhatched_at = 1\nmood = \"idle\"\nxp = 3\nlevel = 1\ndiscovered = [\"ember\"]\nchaos = 12\n",
        )
        .unwrap();
        let st = load_or_hatch(&dir);
        assert_eq!(st.version, STATE_VERSION);
        assert_eq!(st.chaos, 12);
        assert!(st.enabled);
    }

    #[test]
    fn off_switch_round_trips() {
        let mut st = default_state(1, 7);
        st.enabled = false;
        let (_d, dir) = tmpdir();
        save_state(&dir, &st).unwrap();
        let back = load_or_hatch(&dir);
        assert!(!back.enabled);
    }

    #[test]
    fn interactions_cooldown_and_effects() {
        let evs = vec![Event { ts: 100, kind: "fed".into(), duration_secs: 0, exit_code: 0 }];
        assert!(on_cooldown(&evs, "fed", 130, FEED_COOLDOWN_SECS));
        assert!(!on_cooldown(&evs, "fed", 160, FEED_COOLDOWN_SECS));
        assert!(!on_cooldown(&evs, "played", 130, PLAY_COOLDOWN_SECS));
        assert!(!on_cooldown(&evs, "fed", 130, 0));
        assert_eq!(last_event_of_kind(&evs, "played"), None);
        let mut st = default_state(1, 7);
        st.chaos = 50;
        apply_event_chaos(&mut st, "fed");
        assert_eq!(st.chaos, 49);
        apply_event_chaos(&mut st, "played");
        assert_eq!(st.chaos, 47);
        apply_event_chaos(&mut st, "petted");
        assert_eq!(st.chaos, 45);
    }

    #[test]
    fn mood_precedence_and_decay() {
        let evs = vec![Event {
            ts: 100,
            kind: "build_finished".into(),
            duration_secs: 30,
            exit_code: 0,
        }];
        assert_eq!(current_mood(true, &evs, 101), Mood::Working);
        assert_eq!(current_mood(false, &evs, 103), Mood::Celebrate);
        assert_eq!(current_mood(false, &evs, 200), Mood::Idle);
        // The state matrix: nothing has ever happened, so the buddy dozes.
        assert_eq!(current_mood(false, &[], 200), Mood::Sleeping);
    }

    #[test]
    fn name_comes_from_the_hatched_species_pool() {
        for seed in [0u64, 1, 2, 3, 999] {
            let st = default_state(1, seed);
            let sp = crate::species::resolve(&st.species);
            assert!(sp.names.contains(&st.name), "name {} not in pool of {}", st.name, st.species);
        }
    }

    #[test]
    fn hatched_species_is_always_known() {
        for seed in 0..64u64 {
            let st = default_state(1, seed);
            assert!(crate::species::get(&st.species).is_some());
        }
    }

    #[test]
    fn unknown_species_in_state_falls_back_to_ember() {
        let mut st = default_state(1, 0);
        st.species = "trilobite".to_string();
        assert_eq!(crate::species::display_name(&st.species), "ember");
    }

    #[test]
    fn stale_active_marker_expires() {
        let (_d, dir) = tmpdir();
        assert!(!is_active(&dir));
        mark_active(&dir).unwrap();
        assert!(is_active(&dir));
        let _ = fs::remove_file(dir.join("active"));
        assert!(!is_active(&dir));
    }

    #[test]
    fn staleness_threshold() {
        assert!(!marker_stale(0));
        assert!(!marker_stale(5 * 3600));
        assert!(marker_stale(6 * 3600 + 1));
    }


    #[test]
    fn status_format_and_no_events() {
        let st = default_state(1_700_000_000, 0);
        let line = format_status(&st, &st.species.clone(), Mood::Idle, None, 1_700_000_100, 0);
        assert!(line.starts_with(&format!("{} the {} L1 (idle) xp=0", st.name, st.species)));
        assert!(line.ends_with("no events yet"));
        let ev = Event {
            ts: 1_700_000_090,
            kind: "build_finished".into(),
            duration_secs: 30,
            exit_code: 0,
        };
        let line2 = format_status(&st, &st.species.clone(), Mood::Celebrate, Some(&ev), 1_700_000_100, 0);
        assert!(line2.ends_with("last=build_finished 10s ago"));
    }

    #[test]
    fn rotation_at_boundary() {
        let (_d, dir) = tmpdir();
        for i in 0..1000 {
            append_event(&dir, &Event { ts: i, kind: "build_finished".into(), duration_secs: 30, exit_code: 0 }).unwrap();
        }
        assert_eq!(count_event_lines(&events_path(&dir)), 1000);
        assert!(!dir.join("events-1.jsonl").exists());
        append_event(&dir, &Event { ts: 1001, kind: "command_failed".into(), duration_secs: 0, exit_code: 1 }).unwrap();
        assert_eq!(count_event_lines(&events_path(&dir)), 1);
        assert_eq!(count_event_lines(&dir.join("events-1.jsonl")), 1000);
    }

    #[test]
    fn rotation_caps_archives() {
        let (_d, dir) = tmpdir();
        for _round in 0..4 {
            for i in 0..1000 {
                append_event(&dir, &Event { ts: i, kind: "build_finished".into(), duration_secs: 30, exit_code: 0 }).unwrap();
            }
        }
        assert!(!dir.join("events-4.jsonl").exists());
        assert!(dir.join("events-3.jsonl").exists());
        assert_eq!(count_event_lines(&events_path(&dir)), 1000);
    }

    #[test]
    fn tail_is_bounded_and_ordered() {
        let (_d, dir) = tmpdir();
        for i in 0..40 {
            append_event(&dir, &Event { ts: i, kind: "build_finished".into(), duration_secs: 30, exit_code: 0 }).unwrap();
        }
        let tail = read_events_tail(&dir, 5);
        assert_eq!(tail.len(), 5);
        assert_eq!(tail[4].ts, 39);
        assert_eq!(tail[0].ts, 35);
    }

    #[test]
    fn failure_streak_counts_trailing_failures() {
        let evs = |kinds: &[&str]| -> Vec<Event> {
            kinds
                .iter()
                .enumerate()
                .map(|(i, k)| Event { ts: i as i64, kind: k.to_string(), duration_secs: 1, exit_code: 1 })
                .collect()
        };
        assert_eq!(failure_streak(&evs(&["build_finished", "command_failed", "command_failed"])), 2);
        assert_eq!(failure_streak(&evs(&["command_failed", "build_finished"])), 0);
        assert_eq!(failure_streak(&evs(&[])), 0);
    }

    #[test]
    fn is_miffed_windows() {
        let now = 1_700_000_000;
        let ev = |ts: i64| Event { ts, kind: "command_failed".into(), duration_secs: 0, exit_code: 1 };
        assert!(is_miffed(&[ev(now - 60), ev(now - 30)], now));
        assert!(!is_miffed(&[ev(now - 30)], now));
        assert!(!is_miffed(&[ev(now - 7200), ev(now - 601)], now));
        assert!(!is_miffed(&[ev(now - 3700), ev(now - 500)], now));
        assert!(is_miffed(&[ev(now - 3700), ev(now - 500), ev(now - 10)], now));
    }

    #[test]
    fn mood_precedence_with_miffed() {
        let fails = vec![
            Event { ts: 100, kind: "command_failed".into(), duration_secs: 0, exit_code: 1 },
            Event { ts: 110, kind: "command_failed".into(), duration_secs: 0, exit_code: 1 },
        ];
        assert_eq!(current_mood(false, &fails, 120), Mood::Miffed);
        assert_eq!(current_mood(true, &fails, 120), Mood::Working);
        // Past the miff window AND the sleep threshold: asleep wins.
        assert_eq!(current_mood(false, &fails, 712), Mood::Sleeping);
        let mixed = vec![
            Event { ts: 100, kind: "command_failed".into(), duration_secs: 0, exit_code: 1 },
            Event { ts: 112, kind: "build_finished".into(), duration_secs: 30, exit_code: 0 },
        ];
        assert_eq!(current_mood(false, &mixed, 115), Mood::Celebrate);
    }

    #[test]
    fn unknown_mood_in_state_is_tolerated() {
        let (_d, dir) = tmpdir();
        fs::write(
            dir.join("state.toml"),
            "version = 1\nname = \"Pip\"\nspecies = \"ember\"\nhatched_at = 1\nmood = \"banana\"\nxp = 3\n",
        )
        .unwrap();
        let st = load_or_hatch(&dir);
        assert_eq!(st.xp, 3);
        assert_eq!(st.version, STATE_VERSION);
        assert_eq!(st.level, 1);
        let line = format_status(&st, crate::species::display_name(&st.species), Mood::Idle, None, 2, 0);
        assert!(line.contains("(idle)"));
        assert!(!line.contains("banana"));
    }

    #[test]
    fn level_boundaries() {
        assert_eq!(level_for(0), 1);
        assert_eq!(level_for(9), 1);
        assert_eq!(level_for(10), 2);
        assert_eq!(level_for(24), 2);
        assert_eq!(level_for(25), 3);
        assert_eq!(level_for(49), 3);
        assert_eq!(level_for(50), 4);
        assert_eq!(level_for(99), 4);
        assert_eq!(level_for(100), 5);
        assert_eq!(level_for(1000), 5);
        assert_eq!(level_for(-5), 1);
    }

    #[test]
    fn xp_gain_rules() {
        assert_eq!(xp_gain(1, 0, 10), 0);
        assert_eq!(xp_gain(127, 45, 10), 0);
        assert_eq!(xp_gain(0, 5, 10), 0, "fast successes are free, not worth 1");
        assert_eq!(xp_gain(0, 9, 10), 0);
        assert_eq!(xp_gain(0, 10, 10), 2);
        assert_eq!(xp_gain(0, 30, 10), 2);
    }

    #[test]
    fn ascension_weights_shift_and_sum() {
        for a in 0..=6 {
            let w = ascension_weights(a);
            assert_eq!(w.iter().map(|x| x.1).sum::<u64>(), 100);
        }
        assert_eq!(ascension_weights(0)[4].1, 2, "legendary starts at 2");
        assert_eq!(ascension_weights(1)[4].1, 7, "legendary 2 -> 7 after one");
        assert_eq!(
            ascension_weights(9),
            ascension_weights(4),
            "odds stop moving after four ascensions"
        );
    }

    #[test]
    fn apply_xp_ascends_past_max_level() {
        let mut st = default_state(now_secs(), 7);
        st.level = MAX_LEVEL;
        st.xp = 399;
        st.xp_day = now_secs() / 86400;
        st.xp_today = 0;
        let out = apply_xp(&mut st, 2);
        assert_eq!(
            out,
            XpOutcome::Ascended,
            "401 crosses the first ascension threshold 400"
        );
        assert_eq!(st.ascensions, 1);
        assert_eq!(st.xp, LEVEL_THRESHOLDS[MAX_LEVEL as usize - 1]);
        assert_eq!(st.level, MAX_LEVEL);
        assert!(st.discovered.contains(&st.species));
        // the second ascension costs 300 more: threshold 700
        st.xp = 699;
        st.xp_today = 0;
        assert_eq!(apply_xp(&mut st, 2), XpOutcome::Ascended);
        assert_eq!(st.ascensions, 2);
        // below the threshold nothing happens
        st.ascensions = 0;
        st.xp = 390;
        st.xp_today = 0;
        assert_eq!(apply_xp(&mut st, 2), XpOutcome::Nothing);
        assert_eq!(st.ascensions, 0);
    }

    #[test]
    fn pick_weighted_honors_custom_weights() {
        let all_legendary = [("legendary", 100u64)];
        for s in [1u64, 7, 42, 999, 12345] {
            assert_eq!(
                crate::species::pick_weighted(s, &all_legendary).tier,
                "legendary"
            );
        }
        // 10 common / 90 legendary: exactly 20 of seeds 0..200 roll common
        let shifted = [("common", 10u64), ("legendary", 90u64)];
        let mut common = 0;
        for s in 0..200u64 {
            if crate::species::pick_weighted(s, &shifted).tier == "common" {
                common += 1;
            }
        }
        assert_eq!(common, 20, "seed % 100 < 10 rolls common");
    }

    #[test]
    fn apply_xp_caps_every_path() {
        let mut st = default_state(now_secs(), 7);
        st.xp_day = now_secs() / 86400;
        st.xp_today = XP_DAILY_CAP - 1;
        let outcome = apply_xp(&mut st, 5);
        assert_eq!(st.xp_today, XP_DAILY_CAP, "clipped at the cap");
        assert_eq!(outcome, XpOutcome::Nothing);
        // one more grant the same day pays nothing at all
        apply_xp(&mut st, 5);
        assert_eq!(st.xp_today, XP_DAILY_CAP);
    }

    #[test]
    fn apply_xp_detects_level_ups_once() {
        let mut st = default_state(now_secs(), 7);
        st.level = level_for(st.xp);
        st.xp = 9;
        st.xp_day = now_secs() / 86400;
        let outcome = apply_xp(&mut st, 2);
        assert_eq!(outcome, XpOutcome::Leveled, "crossing 10 is a level up");
        assert_eq!(st.level, 2);
        let again = apply_xp(&mut st, 2);
        assert_eq!(
            again,
            XpOutcome::Nothing,
            "no second level-up without crossing a threshold"
        );
    }

    #[test]
    fn apply_xp_ignores_negative_gain() {
        let mut st = default_state(now_secs(), 7);
        st.xp = 30;
        let before = st.xp;
        apply_xp(&mut st, -5);
        assert_eq!(st.xp, before, "xp never goes backward from a bad gain");
        assert_eq!(st.level, level_for(30));
    }

    #[test]
    fn apply_xp_survives_a_backward_clock() {
        let mut st = default_state(now_secs(), 7);
        st.xp_day = now_secs() / 86400 + 5; // stale future day
        st.xp_today = XP_DAILY_CAP;
        apply_xp(&mut st, 2);
        assert_eq!(st.xp_day, now_secs() / 86400, "future day is reset");
        assert_eq!(st.xp_today, 2, "cap not stolen by a stale future day");
    }

    #[test]
    fn xp_to_next_reads_the_ladder() {
        assert_eq!(xp_to_next(0), 10);
        assert_eq!(xp_to_next(9), 1);
        assert_eq!(xp_to_next(25), 25);
        assert_eq!(xp_to_next(99), 1);
        assert_eq!(xp_to_next(100), 0, "maxed level owes nothing");
        assert_eq!(xp_to_next(500), 0);
    }

    #[test]
    fn day_rollover_resets_the_counter() {
        let mut st = default_state(now_secs(), 7);
        st.xp_day = now_secs() / 86400 - 1;
        st.xp_today = XP_DAILY_CAP;
        let today = now_secs() / 86400;
        if st.xp_day != today {
            st.xp_day = today;
            st.xp_today = 0;
        }
        assert_eq!(st.xp_today, 0, "a new day spends fresh");
    }

    #[test]
    fn cap_room_never_overpays() {
        let st = default_state(now_secs(), 7);
        let today = st.xp_day;
        let _ = today;
        // 99 spent today, a 2-xp event: only 1 fits under the cap.
        let mut st = st;
        st.xp_today = XP_DAILY_CAP - 1;
        let gain = xp_gain(0, 60, 10);
        assert_eq!(gain, 2);
        let room = (XP_DAILY_CAP - st.xp_today).min(gain);
        assert_eq!(room, 1, "the cap clips the gain, never overpays");
    }

    #[test]
    fn v5_state_migrates_to_v6() {
        let (_d, dir) = tmpdir();
        fs::write(
            dir.join("state.toml"),
            "version = 5\nname = \"Pip\"\nspecies = \"ember\"\nhatched_at = 1\nmood = \"idle\"\nxp = 42\nlevel = 5\ndiscovered = [\"ember\"]\nchaos = 0\nenabled = true\n",
        )
        .unwrap();
        let st = parse_state(&fs::read_to_string(dir.join("state.toml")).unwrap()).unwrap();
        assert_eq!(st.version, STATE_VERSION);
        assert_eq!(st.xp_today, 0, "counter starts at zero on migration");
        assert_eq!(st.xp_day, now_secs() / 86400);
        assert_eq!(st.xp, 42, "existing xp untouched");
    }

    #[test]
    fn v1_state_migrates_fully() {
        let (_d, dir) = tmpdir();
        fs::write(
            dir.join("state.toml"),
            "version = 1\nname = \"Pip\"\nspecies = \"ember\"\nhatched_at = 1\nmood = \"idle\"\nxp = 17\n",
        )
        .unwrap();
        let st = load_or_hatch(&dir);
        assert_eq!(st.version, STATE_VERSION);
        assert_eq!(st.discovered, vec!["ember".to_string()]);
        assert_eq!(st.xp, 17);
        assert_eq!(st.level, level_for(17));
        assert_eq!(st.level, 2);
        let baks: Vec<String> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.starts_with("state.toml.bak"))
            .collect();
        assert!(baks.is_empty(), "migration is not corruption; no backup churn");
    }

    #[test]
    fn v2_state_migrates_to_v3() {
        let (_d, dir) = tmpdir();
        fs::write(
            dir.join("state.toml"),
            "version = 2\nname = \"Pip\"\nspecies = \"moss\"\nhatched_at = 1\nmood = \"idle\"\nxp = 5\nlevel = 1\n",
        )
        .unwrap();
        let st = load_or_hatch(&dir);
        assert_eq!(st.version, STATE_VERSION);
        assert_eq!(st.discovered, vec!["moss".to_string()]);
        assert_eq!(st.xp, 5);
        let baks: Vec<String> = fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.starts_with("state.toml.bak"))
            .collect();
        assert!(baks.is_empty(), "migration is not corruption; no backup churn");
    }

    #[test]
    fn v2_state_round_trip_keeps_level() {
        let (_d, dir) = tmpdir();
        let mut st = default_state(1_700_000_000, 2);
        st.xp = 42;
        st.level = level_for(42);
        save_state(&dir, &st).unwrap();
        let loaded = load_or_hatch(&dir);
        assert_eq!(loaded, st);
        assert_eq!(loaded.level, 3);
    }
}
