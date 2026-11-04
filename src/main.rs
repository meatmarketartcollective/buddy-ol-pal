mod species;
mod state;
mod tui;
mod watcher;

use std::fs;

fn help() {
    println!("hatch - a terminal companion that reacts to real events\n");
    println!("  tb ui              open the companion (plain `tb` also works)");
    println!("  tb run -- <cmd>    run a command; long runs and failures get logged as events");
    println!("  tb status          one-line companion status for scripts and panes");
    println!("  tb dex             the discovery list; undiscovered species show as ???");
    println!("  tb tail [-n N]     print the last N events as JSON lines (default 10)\n");
    println!("state lives in ~/.config/hatch (override with HATCH_HOME)");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(|s| s.as_str()) {
        None | Some("ui") => tui::run_ui(),
        Some("run") => watcher::run_and_log(&run_argv(&args[1..])),
        Some("status") => status_cmd(),
        Some("dex") => dex_cmd(),
        Some("tail") => tail_cmd(&args[1..]),
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

/// `tb run -- <cmd>`: the conventional `--` separator before the command is
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

/// `tb status`: one plain line, read-only. Never hatches a companion.
fn status_cmd() -> i32 {
    let dir = state::config_dir();
    let st = fs::read_to_string(dir.join("state.toml"))
        .ok()
        .and_then(|s| state::parse_state(&s));
    let Some(st) = st else {
        println!("no companion yet; run tb");
        return 0;
    };
    let now = state::now_secs();
    let events = state::read_events_tail(&dir, 50);
    let active = state::is_active(&dir);
    let mood = state::current_mood(active, &events, now);
    println!(
        "{}",
        state::format_status(&st, species::display_name(&st.species), mood, events.last(), now)
    );
    0
}

/// `tb dex`: the discovery list. Discovered species show name, species and
/// tier; undiscovered ones are hidden behind ???.
fn dex_cmd() -> i32 {
    let dir = state::config_dir();
    let st = state::load_or_hatch(&dir);
    for row in species::dex_rows(&st.discovered) {
        println!("{row}");
    }
    0
}

/// `tb tail [-n N]`: the last N events as JSON lines. Read-only.
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
