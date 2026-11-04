use crate::state::{self, Event};
use std::process::Command;
use std::time::Instant;

pub fn threshold_secs() -> u64 {
    std::env::var("HATCH_THRESHOLD_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10)
}

/// `tb run -- <command> [args...]`: spawns the command with inherited stdio,
/// times it, marks the companion working while the child is live, and on exit
/// appends one event if it qualifies. Never logs command content, only
/// outcome, duration, and exit code.
pub fn run_and_log(argv: &[String]) -> i32 {
    if argv.is_empty() {
        eprintln!("usage: tb run -- <command> [args...]");
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
            eprintln!("hatch: could not run '{}': {}", argv[0], e);
            -1
        }
    };

    let kind = state::classify(code, duration_secs, threshold_secs()).map(|k| k.to_string());
    settle(&dir, kind.as_deref(), duration_secs, code);
    code
}

/// Fold one logged event into the companion: write it, pay the XP, apply the
/// chaos effect, save, and notify while the buddy is on. Shared by `tb run`
/// and `tb emit` so adapters get exactly the same reactions.
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
            "hatch: unknown kind '{}'. known kinds: build_finished, command_failed. logging it, but it stays silent and calm.",
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
                eprintln!("hatch: could not write event: {}", e);
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
            eprintln!("hatch: could not write event: {}", e);
        }
    }

    // XP accrual. Also persists any pending migration on the first run.
    let mut st = state::load_or_hatch(dir);
    let threshold = threshold_secs();
    st.xp += state::xp_gain(code, duration_secs, threshold);
    st.level = state::level_for(st.xp);
    if let Some(k) = kind {
        state::apply_event_chaos(&mut st, k);
    }
    if let Err(e) = state::save_state(dir, &st) {
        eprintln!("hatch: could not save state: {}", e);
    }

    // Notifications: only while the buddy is on. The OSC 9 sequence asks the
    // terminal for a desktop notification; inside tmux the message also lands
    // on the status line so other panes see it.
    if st.enabled {
        if let Some(k) = kind {
            notify(&notification_message(k, duration_secs, code));
        }
    }

    code
}


/// `tb emit <kind> [--exit N] [--secs S]`: the adapter surface. Any harness
/// can feed hatch an event without wrapping its command: kind names what
/// happened ("build_finished" or "command_failed"), --exit and --secs carry
/// the outcome. Unknown kinds are logged but stay silent and calm.
pub fn parse_emit_args(argv: &[String]) -> Result<(String, i32, u64), String> {
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
    kind.ok_or_else(|| "no kind given".to_string())
        .map(|k| (k, exit_code, secs))
}

pub fn emit(argv: &[String]) -> i32 {
    let (kind, exit_code, secs) = match parse_emit_args(argv) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("hatch: {}. usage: tb emit <kind> [--exit N] [--secs S]", e);
            return 2;
        }
    };
    settle(&state::config_dir(), Some(&kind), secs, exit_code);
    0
}

/// The notification line for an event, in the buddy's voice.
pub fn notification_message(kind: &str, duration_secs: u64, code: i32) -> String {
    match kind {
        "build_finished" => format!("hatch: build finished in {}s", duration_secs),
        "command_failed" => format!("hatch: command failed (exit {})", code),
        other => format!("hatch: {}", other),
    }
}

/// Emit a notification: the OSC 9 desktop-notification escape on stdout, and,
/// inside tmux, a display-message so the other panes see it too. The escape
/// only goes to a real terminal (a pipe gets clean output); HATCH_NOTIFY=off
/// silences everything and HATCH_NOTIFY=force emits even piped, for testing.
pub fn notify(msg: &str) {
    match std::env::var("HATCH_NOTIFY").as_deref() {
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
        assert_eq!(notification_message("build_finished", 42, 0), "hatch: build finished in 42s");
        assert_eq!(notification_message("command_failed", 3, 2), "hatch: command failed (exit 2)");
        assert_eq!(notification_message("petted", 0, 0), "hatch: petted");
    }

    #[test]
    fn emit_args_parse() {
        let a: Vec<String> = vec!["build_finished".into()];
        assert_eq!(parse_emit_args(&a).unwrap(), ("build_finished".into(), 0, 0));
        let b: Vec<String> = vec!["command_failed".into(), "--exit".into(), "2".into(), "--secs".into(), "7".into()];
        assert_eq!(parse_emit_args(&b).unwrap(), ("command_failed".into(), 2, 7));
        assert!(parse_emit_args(&[]).is_err());
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
