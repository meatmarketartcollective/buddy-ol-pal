use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const STATE_VERSION: i64 = 1;
pub const SPECIES: &str = "ember";
pub const CELEBRATE_SECS: i64 = 6;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompanionState {
    pub version: i64,
    pub name: String,
    pub species: String,
    pub hatched_at: i64,
    pub mood: String,
    pub xp: i64, // reserved, unused in milestone one
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
}

impl Mood {
    pub fn label(self) -> &'static str {
        match self {
            Mood::Idle => "idle",
            Mood::Working => "working",
            Mood::Celebrate => "celebrating",
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

const NAMES: &[&str] = &[
    "Pip", "Momo", "Biscuit", "Echo", "Waffle", "Pixel", "Sprout", "Noodle",
    "Comet", "Pebble", "Tofu", "Ziggy", "Maple", "Clover", "Dot", "Miso",
    "Bumble", "Fig", "Scooter", "Waldo", "Juniper", "Crumb", "Beacon", "Tango",
];

pub fn pick_name(seed: u64) -> &'static str {
    NAMES[(seed % NAMES.len() as u64) as usize]
}

pub fn default_state(now: i64, seed: u64) -> CompanionState {
    CompanionState {
        version: STATE_VERSION,
        name: pick_name(seed).to_string(),
        species: SPECIES.to_string(),
        hatched_at: now,
        mood: Mood::Idle.label().to_string(),
        xp: 0,
    }
}

fn seed() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64)
        .unwrap_or(0);
    nanos.wrapping_mul(2_654_435_761) ^ ((std::process::id() as u64) << 32)
}

pub fn parse_state(text: &str) -> Option<CompanionState> {
    let st: CompanionState = toml::from_str(text).ok()?;
    if st.version == STATE_VERSION {
        Some(st)
    } else {
        None
    }
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
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(events_path(dir))?;
    let mut line =
        serde_json::to_string(ev).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    line.push('\n');
    f.write_all(line.as_bytes())
}

/// Reads the newest `max` events, oldest-first. Truncated or corrupt lines
/// (a torn final write) are skipped, never fatal.
pub fn read_events_tail(dir: &Path, max: usize) -> Vec<Event> {
    let text = match fs::read_to_string(events_path(dir)) {
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
    }
    Mood::Idle
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
        assert_eq!(current_mood(false, &[], 200), Mood::Idle);
    }

    #[test]
    fn name_comes_from_list() {
        assert!(NAMES.contains(&default_state(1, 0).name.as_str()));
        assert!(NAMES.contains(&default_state(1, 999).name.as_str()));
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

}
