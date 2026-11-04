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

    let threshold = threshold_secs();
    let mut logged_kind: Option<String> = None;

    if let Some(kind) = state::classify(code, duration_secs, threshold) {
        let ev = Event {
            ts: state::now_secs(),
            kind: kind.to_string(),
            duration_secs,
            exit_code: code,
        };
        if let Err(e) = state::append_event(&dir, &ev) {
            eprintln!("hatch: could not write event: {}", e);
        }
        logged_kind = Some(kind.to_string());
    }

    // XP accrual. Also persists the v1 -> v2 migration on the first run.
    let mut st = state::load_or_hatch(&dir);
    st.xp += state::xp_gain(code, duration_secs, threshold);
    st.level = state::level_for(st.xp);
    if let Some(k) = &logged_kind {
        state::apply_event_chaos(&mut st, k);
    }
    if let Err(e) = state::save_state(&dir, &st) {
        eprintln!("hatch: could not save state: {}", e);
    }

    code
}
