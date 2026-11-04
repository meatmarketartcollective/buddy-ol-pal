mod state;
mod tui;
mod watcher;

fn help() {
    println!("hatch - a terminal companion that reacts to real events\n");
    println!("  tb ui              open the companion (plain `tb` also works)");
    println!("  tb run -- <cmd>    run a command; long runs and failures get logged as events\n");
    println!("state lives in ~/.config/hatch (override with HATCH_HOME)");
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(|s| s.as_str()) {
        None | Some("ui") => tui::run_ui(),
        Some("run") => watcher::run_and_log(&run_argv(&args[1..])),
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
