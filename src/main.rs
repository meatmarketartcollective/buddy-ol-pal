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
        Some("run") => watcher::run_and_log(&args[1..]),
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
