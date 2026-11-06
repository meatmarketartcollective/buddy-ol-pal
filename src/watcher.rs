use crate::species;
use crate::state::{self, Event};
use std::process::Command;
use std::time::Instant;

pub fn threshold_secs() -> u64 {
    std::env::var("BUDDY_THRESHOLD_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10)
}

/// `buddy run -- <command> [args...]`: spawns the command with inherited stdio,
/// times it, marks the companion working while the child is live, and on exit
/// appends one event if it qualifies. Never logs command content, only
/// outcome, duration, and exit code.
pub fn run_and_log(argv: &[String]) -> i32 {
    if argv.is_empty() {
        eprintln!("usage: buddy run -- <command> [args...]");
        return 2;
    }
    let dir = state::config_dir();
    let _ = state::mark_active(&dir);

    let start = Instant::now();
    let status = Command::new(&argv[0]).args(&argv[1..]).status();
    let duration_secs = start.elapsed().as_secs();
    let _ = std::fs::remove_file(dir.join("active"));

    let code = match status {
        Ok(s) => s.code().unwrap_or(-1),
        Err(e) => {
            eprintln!("buddy: could not run '{}': {}", argv[0], e);
            -1
        }
    };

    let kind = state::classify(code, duration_secs, threshold_secs()).map(|k| k.to_string());
    settle(&dir, kind.as_deref(), duration_secs, code);
    code
}

/// Fold one logged event into the companion: write it, pay the XP, apply the
/// chaos effect, save, and notify while the buddy is on. Shared by `buddy run`
/// and `buddy emit` so adapters get exactly the same reactions.
/// Voice: print one spoken line, but only when stdout is a real terminal.
/// Piped output stays machine-clean; that lesson cost a bug already.
pub fn speak_text(msg: &str) {
    use std::io::IsTerminal;
    if !std::io::stdout().is_terminal() {
        return;
    }
    println!("{msg}");
}

pub fn settle(dir: &std::path::Path, kind: Option<&str>, duration_secs: u64, code: i32) -> i32 {
    let _lock = state::lock_state(dir);

    // The kinds a wrapped run can produce. An adapter may emit its own kind:
    // it is logged, but it stays silent and calm (no XP, no chaos, no ping),
    // so a typo cannot mint rewards and a stranger event cannot startle the
    // buddy.
    let known = kind
        .map(|k| matches!(k, "build_finished" | "command_failed"))
        .unwrap_or(true);
    if !known {
        eprintln!(
            "buddy: unknown kind '{}'. known kinds: build_finished, command_failed. logging it, but it stays silent and calm.",
            kind.unwrap_or("")
        );
        if let Some(k) = kind {
            let ev = Event {
                ts: state::now_secs(),
                kind: k.to_string(),
                duration_secs,
                exit_code: code,
            };
            if let Err(e) = state::append_event(dir, &ev) {
                eprintln!("buddy: could not write event: {}", e);
            }
        }
        return code;
    }

    if let Some(k) = kind {
        let ev = Event {
            ts: state::now_secs(),
            kind: k.to_string(),
            duration_secs,
            exit_code: code,
        };
        if let Err(e) = state::append_event(dir, &ev) {
            eprintln!("buddy: could not write event: {}", e);
        }
    }

    // XP accrual against the daily cap. Work pays up to XP_DAILY_CAP per
    // unix day; past the cap events still log and still move chaos, they
    // just stop printing XP, so a loop of toy runs cannot mint a level 5
    // buddy. Also persists any pending migration on the first run.
    let mut st = state::load_or_hatch(dir);
    let threshold = threshold_secs();
    let gain = state::xp_gain(code, duration_secs, threshold);
    let outcome = state::apply_xp(&mut st, gain);
    let prev_level = st.level;
    if let Some(k) = kind {
        state::apply_event_chaos(&mut st, k);
    }
    if let Err(e) = state::save_state(dir, &st) {
        eprintln!("buddy: could not save state: {}", e);
    }

    // Notifications: only while the buddy is on. The OSC 9 sequence asks the
    // terminal for a desktop notification; inside tmux the message also lands
    // on the status line so other panes see it.
    if st.enabled {
        if let Some(k) = kind {
            // Voice: the buddy speaks a line right where the run happened.
            // The ping is the notification half; this is the text half. Both
            // stay gated on the off switch and on stdout being a terminal.
            let mood = if k == "command_failed" {
                state::Mood::Miffed
            } else {
                state::Mood::Celebrate
            };
            let sp = species::get(&st.species).unwrap_or_else(species::fallback);
            let line = species::speak(
                sp,
                mood,
                (state::now_secs() as u64) + st.chaos.unsigned_abs(),
            );
            if !line.is_empty() {
                speak_text(&format!(
                    "{} the {}: {}",
                    st.name,
                    species::display_name(&st.species),
                    line
                ));
            }
            match outcome {
                state::XpOutcome::Leveled => speak_text(&format!(
                    "{} the {} grew to level {}!",
                    st.name,
                    species::display_name(&st.species),
                    st.level
                )),
                state::XpOutcome::Ascended => speak_text(&format!(
                    "{} ascended! Rehatched as a {} with rarer species odds ahead.",
                    st.name,
                    species::display_name(&st.species)
                )),
                state::XpOutcome::Nothing => {}
            }
            notify(&notification_message(k, duration_secs, code));
        }
    }

    code
}


/// `buddy emit [kind] [--exit N] [--secs S]`: the adapter surface. With a kind,
/// any harness can feed buddy a named event. With no kind, the outcome is
/// classified like a wrapped run: slow successes and any failure become
/// events, fast successes are skipped (nothing worth reacting to). Same
/// effects as a real run either way. Unknown kinds are logged but stay
/// silent and calm.
pub fn parse_emit_args(argv: &[String]) -> Result<(Option<String>, i32, u64), String> {
    let mut kind: Option<String> = None;
    let mut exit_code = 0i32;
    let mut secs = 0u64;
    let mut it = argv.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--exit" => match it.next().and_then(|v| v.parse().ok()) {
                Some(v) => exit_code = v,
                None => return Err("--exit needs a number".into()),
            },
            "--secs" => match it.next().and_then(|v| v.parse().ok()) {
                Some(v) => secs = v,
                None => return Err("--secs needs a number".into()),
            },
            _ if kind.is_none() => kind = Some(a.clone()),
            _ => return Err("too many arguments".into()),
        }
    }
    Ok((kind, exit_code, secs))
}

pub fn emit(argv: &[String]) -> i32 {
    let (kind, exit_code, secs) = match parse_emit_args(argv) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("buddy: {}. usage: buddy emit [kind] [--exit N] [--secs S]", e);
            return 2;
        }
    };
    let kind = match kind {
        Some(k) => Some(k),
        None => match state::classify(exit_code, secs, threshold_secs()) {
            Some(k) => Some(k.to_string()),
            None => return 0,
        },
    };
    settle(&state::config_dir(), kind.as_deref(), secs, exit_code);
    0
}

/// The notification line for an event, in the buddy's voice.
pub fn notification_message(kind: &str, duration_secs: u64, code: i32) -> String {
    match kind {
        "build_finished" => format!("buddy: build finished in {}s", duration_secs),
        "command_failed" => format!("buddy: command failed (exit {})", code),
        other => format!("buddy: {}", other),
    }
}

/// Emit a notification: the OSC 9 desktop-notification escape on stdout, and,
/// inside tmux, a display-message so the other panes see it too. The escape
/// only goes to a real terminal (a pipe gets clean output); BUDDY_NOTIFY=off
/// silences everything and BUDDY_NOTIFY=force emits even piped, for testing.
pub fn notify(msg: &str) {
    match std::env::var("BUDDY_NOTIFY").as_deref() {
        Ok("off") => return,
        Ok("force") => {}
        _ => {
            use std::io::IsTerminal;
            if !std::io::stdout().is_terminal() {
                return;
            }
        }
    }
    print!("\x1b]9;{}\x07", msg);
    use std::io::Write;
    let _ = std::io::stdout().flush();
    if std::env::var_os("TMUX").is_some() {
        let _ = Command::new("tmux")
            .args(["display-message", "-d", "4000", msg])
            .spawn();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_messages_name_the_outcome() {
        assert_eq!(notification_message("build_finished", 42, 0), "buddy: build finished in 42s");
        assert_eq!(notification_message("command_failed", 3, 2), "buddy: command failed (exit 2)");
        assert_eq!(notification_message("petted", 0, 0), "buddy: petted");
    }

    #[test]
    fn emit_args_parse() {
        let a: Vec<String> = vec!["build_finished".into()];
        assert_eq!(parse_emit_args(&a).unwrap(), (Some("build_finished".into()), 0, 0));
        let b: Vec<String> = vec!["command_failed".into(), "--exit".into(), "2".into(), "--secs".into(), "7".into()];
        assert_eq!(parse_emit_args(&b).unwrap(), (Some("command_failed".into()), 2, 7));
        assert_eq!(parse_emit_args(&[]).unwrap(), (None, 0, 0), "no kind means classify it");
        let c: Vec<String> = vec!["k".into(), "--exit".into()];
        assert!(parse_emit_args(&c).is_err());
        let d: Vec<String> = vec!["k".into(), "extra".into()];
        assert!(parse_emit_args(&d).is_err());
    }

    #[test]
    fn unknown_emit_kind_is_logged_but_calm() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("h");
        let before = state::load_or_hatch(&home).xp;
        settle(&home, Some("build_finshed"), 5, 0);
        let st = state::load_or_hatch(&home);
        assert_eq!(st.xp, before, "unknown kind must not pay XP");
        let tail = state::read_events_tail(&home, 10);
        assert!(tail.iter().any(|e| e.kind == "build_finshed"), "unknown kind is still logged");
    }
}
