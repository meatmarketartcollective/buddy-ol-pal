mod agent;
mod ai;
mod anim;
mod shell;
mod species;
mod state;
mod tui;
mod watcher;

use std::fs;

fn help() {
    println!("buddy - a terminal companion that reacts to real events\n");
    println!("  buddy ui              open the companion (plain `buddy` also works)");
    println!("  buddy run -- <cmd>    run a command; long runs and failures get logged as events");
    println!("  buddy status          one-line companion status for scripts and panes");
    println!("  buddy dex             the discovery list; undiscovered species show as ???");
    println!("  buddy pet             show your buddy some love; eases the chaos");
    println!("  buddy feed            feed your buddy a treat (+1 xp, eases chaos, 1m cooldown)");
    println!("  buddy play            play with your buddy (+2 xp, eases chaos more, 5m cooldown)");
    println!("  buddy talk            say hi; your buddy answers with a line");
    println!("  buddy ask \"?\"         ask your buddy about recent terminal events (AI, tier 1+)");
    println!("  buddy agent \"task\"   the buddy works: reads files, runs commands; tier 3 edits too");
    println!("  buddy ai              give your buddy an AI voice: setup (free NVIDIA key), test, on/off, clear");
    println!("  buddy off / buddy on     silence notifications and hide the UI, or wake it back up");
    println!("  buddy emit <kind>     feed buddy an event from any harness (e.g. build_finished --exit 0 --secs 42)");
    println!("  buddy shell [bash|zsh]  print the passive-capture snippet; eval it in your rc file");
    println!("  buddy tail [-n N]     print the last N events as JSON lines (default 10)\n");
    println!("state lives in ~/.config/buddy-ol-pal (override with BUDDY_HOME)");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(|s| s.as_str()) {
        None | Some("ui") => tui::run_ui(),
        Some("run") => watcher::run_and_log(&run_argv(&args[1..])),
        Some("status") => status_cmd(),
        Some("pet") => pet_cmd(),
        Some("feed") => feed_cmd(),
        Some("play") => play_cmd(),
        Some("talk") => talk_cmd(),
        Some("ask") => agent::ask_cmd(&args[1..]),
        Some("agent") => agent::agent_cmd(&args[1..]),
        Some("ai") => ai::ai_cmd(&args[1..]),
        Some("off") => switch_cmd(false),
        Some("on") => switch_cmd(true),
        Some("dex") => dex_cmd(),
        Some("tail") => tail_cmd(&args[1..]),
        Some("emit") => watcher::emit(&args[1..]),
        Some("shell") => shell_cmd(args.get(1).map(|s| s.as_str())),
        Some("-h") | Some("--help") | Some("help") => {
            help();
            0
        }
        Some(other) => {
            eprintln!("unknown command: {other}\n");
            help();
            2
        }
    };
    std::process::exit(code);
}

/// `buddy run -- <cmd>`: the conventional `--` separator before the command is
/// optional; strip it so the documented form works.
fn run_argv(rest: &[String]) -> Vec<String> {
    if rest.first().map(|s| s.as_str()) == Some("--") {
        rest[1..].to_vec()
    } else {
        rest.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::run_argv;

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn dashee_before_command_is_stripped() {
        assert_eq!(run_argv(&v(&["--", "sleep", "12"])), v(&["sleep", "12"]));
    }

    #[test]
    fn bare_command_is_kept() {
        assert_eq!(run_argv(&v(&["sleep", "12"])), v(&["sleep", "12"]));
    }

    #[test]
    fn empty_args_survive() {
        assert_eq!(run_argv(&v(&[])), v(&[]));
    }
}

/// `buddy status`: one plain line, read-only. Never hatches a companion.
/// `buddy pet`: a little affection. Logs a petted event (the UI shows the petted
/// state for PET_SECS) and eases the chaos a touch.
fn pet_cmd() -> i32 {
    let dir = state::config_dir();
    let mut st = state::load_or_hatch(&dir);
    let ev = state::Event {
        ts: state::now_secs(),
        kind: "petted".to_string(),
        duration_secs: 0,
        exit_code: 0,
    };
    if let Err(e) = state::append_event(&dir, &ev) {
        eprintln!("buddy: could not write event: {}", e);
    }
    state::apply_event_chaos(&mut st, "petted");
    if let Err(e) = state::save_state(&dir, &st) {
        eprintln!("buddy: could not save state: {}", e);
    }
    println!(
        "{} the {} soaks it in. (petted)",
        st.name,
        species::display_name(&st.species)
    );
    0
}

/// The interaction set behind `buddy feed` / `buddy play` / `buddy talk`. Each logs
/// an event the UI, tail and status can see; feed and play earn a little XP
/// and ease the chaos, talk just gets a line back. Feed and play cool down.
fn interaction(kind: &str, cooldown_secs: i64, xp: i64, verb: &str, full: &str) -> i32 {
    let dir = state::config_dir();
    let _lock = state::lock_state(&dir);
    let mut st = state::load_or_hatch(&dir);
    let now = state::now_secs();
    let events = state::read_events_tail(&dir, 50);
    if state::on_cooldown(&events, kind, now, cooldown_secs) {
        println!("{}", full);
        return 0;
    }
    let ev = state::Event {
        ts: now,
        kind: kind.to_string(),
        duration_secs: 0,
        exit_code: 0,
    };
    if let Err(e) = state::append_event(&dir, &ev) {
        eprintln!("buddy: could not write event: {}", e);
    }
    let outcome = state::apply_xp(&mut st, xp);
    state::apply_event_chaos(&mut st, kind);
    if let Err(e) = state::save_state(&dir, &st) {
        eprintln!("buddy: could not save state: {}", e);
    }
    println!(
        "{} the {}: {}",
        st.name,
        species::display_name(&st.species),
        verb
    );
    match outcome {
        state::XpOutcome::Leveled => println!(
            "{} the {} grew to level {}!",
            st.name,
            species::display_name(&st.species),
            st.level
        ),
        state::XpOutcome::Ascended => println!(
            "{} ascended! It rehatched into a {} with rarer species odds ahead.",
            st.name,
            species::display_name(&st.species)
        ),
        state::XpOutcome::Nothing => {}
    }
    0
}

fn feed_cmd() -> i32 {
    interaction(
        "fed",
        state::FEED_COOLDOWN_SECS,
        state::XP_FEED,
        "munches happily. (fed)",
        "nope, full for now. try again in a minute",
    )
}

fn play_cmd() -> i32 {
    interaction(
        "played",
        state::PLAY_COOLDOWN_SECS,
        state::XP_PLAY,
        "chases the cursor around the screen. (played)",
        "worn out. give it five minutes",
    )
}

fn talk_cmd() -> i32 {
    // Voice: the buddy answers with a real line from its species pool,
    // matched to the mood it is in right now.
    let dir = state::config_dir();
    let st = state::load_or_hatch(&dir);
    let events = state::read_events_tail(&dir, 50);
    let now = state::now_secs();
    let active = state::is_active(&dir);
    let mood = state::current_mood(active, &events, now);
    let sp = species::get(&st.species).unwrap_or_else(species::fallback);
    let seed = (now as u64) + st.chaos.unsigned_abs() + st.xp.unsigned_abs();
    let last = events.last().map(|e| (e.kind.as_str(), e.exit_code, e.duration_secs));
    let voice = ai::Voice {
        pet_name: &st.name,
        species: &sp.name,
        mood: mood.label(),
        last_event: last,
    };
    let line = match ai::speak(&dir, &voice) {
        Some(l) if !l.is_empty() => l,
        _ => species::speak(sp, mood, seed).to_string(),
    };
    interaction("talked", 0, 0, &format!("{line} (talked)"), "")
}

/// `buddy off` / `buddy on`: the off switch. Events keep logging either way; off
/// only silences notifications and keeps the UI from opening.
fn switch_cmd(on: bool) -> i32 {
    let dir = state::config_dir();
    let mut st = state::load_or_hatch(&dir);
    st.enabled = on;
    if let Err(e) = state::save_state(&dir, &st) {
        eprintln!("buddy: could not save state: {}", e);
        return 1;
    }
    println!(
        "{}",
        if on {
            "the buddy is back on."
        } else {
            "the buddy is off. events still log; buddy on to wake it."
        }
    );
    0
}

/// `buddy shell [bash|zsh]`: prints the passive-capture snippet for that shell.
/// Eval its output in the rc file and buddy sees every interactive command.
fn shell_cmd(which: Option<&str>) -> i32 {
    match which.unwrap_or("bash") {
        "bash" => {
            print!("{}", shell::BASH);
            0
        }
        "zsh" => {
            print!("{}", shell::ZSH);
            0
        }
        other => {
            eprintln!("buddy: unsupported shell '{}'. try bash or zsh.", other);
            2
        }
    }
}

fn status_cmd() -> i32 {
    let dir = state::config_dir();
    let st = fs::read_to_string(dir.join("state.toml"))
        .ok()
        .and_then(|s| state::parse_state(&s));
    let Some(st) = st else {
        println!("no companion yet; run buddy");
        return 0;
    };
    let now = state::now_secs();
    let events = state::read_events_tail(&dir, 50);
    let active = state::is_active(&dir);
    let mood = state::current_mood(active, &events, now);
    let chaos = state::effective_chaos(&st, &events, now);
    println!(
        "{}",
        state::format_status(
            &st,
            species::display_name(&st.species),
            mood,
            events.last(),
            now,
            chaos
        )
    );
    let sp = species::get(&st.species).unwrap_or_else(species::fallback);
    let line = species::speak(
        sp,
        mood,
        (now as u64) + chaos.unsigned_abs() + st.xp.unsigned_abs(),
    );
    if !line.is_empty() {
        println!();
        println!("\"{line}\"");
    }
    let next = state::xp_to_next(st.xp);
    if next > 0 {
        println!("{} xp to level {}", next, st.level + 1);
    } else if st.xp >= state::LEVEL_THRESHOLDS[state::LEVEL_THRESHOLDS.len() - 1] {
        println!(
            "{} xp to the next rehatch",
            state::ascension_threshold(st.ascensions) - st.xp
        );
    }
    0
}

/// `buddy dex`: the discovery list. Discovered species show name, species and
/// tier; undiscovered ones are hidden behind ???.
fn dex_cmd() -> i32 {
    let dir = state::config_dir();
    let st = state::load_or_hatch(&dir);
    for row in species::dex_rows(&st.discovered) {
        println!("{row}");
    }
    0
}

/// `buddy tail [-n N]`: the last N events as JSON lines. Read-only.
fn tail_cmd(rest: &[String]) -> i32 {
    let mut n: usize = 10;
    let mut idx = 0;
    while idx < rest.len() {
        match rest[idx].as_str() {
            "-n" => {
                if let Some(v) = rest.get(idx + 1).and_then(|v| v.parse().ok()) {
                    n = v;
                }
                idx += 2;
            }
            other => {
                if let Ok(v) = other.parse::<usize>() {
                    n = v;
                }
                idx += 1;
            }
        }
    }
    let n = n.min(state::ROTATE_LINES);
    let dir = state::config_dir();
    for ev in state::read_events_tail(&dir, n) {
        if let Ok(line) = serde_json::to_string(&ev) {
            println!("{line}");
        }
    }
    0
}
