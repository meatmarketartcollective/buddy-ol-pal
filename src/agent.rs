//! The buddy that works: tiered agent commands.
//!
//! `tb ask` (tier 1+): a read-only adviser over the event ledger. Answers are
//! never clamped: the 140-character rule is the voice's, never the agent's.
//! `tb agent` (tier 2+): the buddy reads files and runs commands with
//! captured output; tier 3 unlocks edits. Every action appends an agent_*
//! event to the same ledger your commands write to, so the buddy's history
//! shows what it did. Tier assignment is a probe (`tb ai test`) or an
//! explicit `tier = N` in ai.toml; no key, no agent, ever a silent fallback.

use crate::ai;
use crate::state::{self, Event, Mood};
use crate::species::{self, Species};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub const AGENT_MAX_STEPS: usize = 12;
pub const MAX_TOOL_OUTPUT: usize = 4000;
pub const AGENT_READ_CAP: usize = 6000;
const CONTEXT_EVENTS: usize = 30;

pub struct Flags {
    pub full: bool,
    pub recap: bool,
    pub yes: bool,
    pub max_steps: usize,
    pub inner: bool,
    pub rest: Vec<String>,
}

pub fn parse_flags(rest: &[String]) -> Result<Flags, String> {
    let mut f = Flags {
        full: false,
        recap: false,
        yes: false,
        max_steps: AGENT_MAX_STEPS,
        inner: false,
        rest: Vec::new(),
    };
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--full" => f.full = true,
            "--recap" => f.recap = true,
            "--yes" => f.yes = true,
            "--inner" => f.inner = true,
            "--max-steps" => match rest.get(i + 1).and_then(|v| v.parse::<usize>().ok()) {
                Some(n) if n > 0 => {
                    f.max_steps = n;
                    i += 1;
                }
                _ => return Err("--max-steps needs a number".to_string()),
            },
            other if other.starts_with("--") => {
                return Err(format!("unknown flag: {other}"));
            }
            other => f.rest.push(other.to_string()),
        }
        i += 1;
    }
    Ok(f)
}

#[derive(Debug)]
struct Gate {
    cfg: ai::AiConfig,
    tier: u8,
    dir: PathBuf,
}

fn gate(dir: &Path, min_tier: u8, what: &str) -> Result<Gate, String> {
    let cfg = ai::load_config(dir).ok_or_else(|| {
        format!("hatch {what}: no AI voice configured; run tb ai setup first.")
    })?;
    if cfg.enabled == Some(false) {
        return Err(format!("hatch {what}: the AI voice is off (tb ai on)."));
    }
    let tier = ai::effective_tier(dir, &cfg).ok_or_else(|| {
        format!(
            "hatch {what}: no capability tier yet; run `tb ai test`, or set tier = 1..3 in ai.toml."
        )
    })?;
    if tier < min_tier {
        return Err(format!(
            "hatch {what}: needs tier {min_tier}, yours is {tier} (tier = N in ai.toml raises it)."
        ));
    }
    Ok(Gate {
        cfg,
        tier,
        dir: dir.to_path_buf(),
    })
}

fn audit(dir: &Path, kind: &str, exit_code: i32, duration_secs: u64) {
    let ev = Event {
        ts: state::now_secs(),
        kind: kind.to_string(),
        duration_secs,
        exit_code,
    };
    if let Err(e) = state::append_event(dir, &ev) {
        eprintln!("hatch: could not write event: {e}");
    }
}

pub fn events_digest(events: &[Event]) -> String {
    if events.is_empty() {
        return "(no events yet)".to_string();
    }
    events
        .iter()
        .map(|e| format!("- {} exit={} {}s", e.kind, e.exit_code, e.duration_secs))
        .collect::<Vec<_>>()
        .join("\n")
}

fn ask_prompt(name: &str, sp: &str, mood: &str, events: &[Event], q: &str) -> String {
    format!(
        "You are {name} the {sp}, feeling {mood}. Recent terminal events (oldest first):\n{}\n\nThe user asks: {q}",
        events_digest(events)
    )
}

fn recap_prompt(events: &[Event], now: i64) -> String {
    let hour_ago = now - 3600;
    let recent: Vec<Event> = events
        .iter()
        .filter(|e| e.ts >= hour_ago)
        .cloned()
        .collect();
    let recent = if recent.len() > CONTEXT_EVENTS {
        recent[recent.len() - CONTEXT_EVENTS..].to_vec()
    } else {
        recent
    };
    format!(
        "Summarize the last hour in this terminal. Events (oldest first):\n{}\n\nCover: what ran, what failed and why it may have failed, and anything that took unusually long. Plain text, short paragraphs.",
        events_digest(&recent)
    )
}

pub fn ask_cmd(rest: &[String]) -> i32 {
    let flags = match parse_flags(rest) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("hatch ask: {e}");
            return 2;
        }
    };
    let dir = state::config_dir();
    let g = match gate(&dir, 1, "ask") {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    let st = state::load_or_hatch(&dir);
    let events = state::read_events_tail(&dir, CONTEXT_EVENTS);
    let now = state::now_secs();
    let sp = species::get(&st.species).unwrap_or_else(species::fallback);
    let active = state::is_active(&dir);
    let mood = state::current_mood(active, &events, now).label().to_string();
    let user = if flags.recap {
        recap_prompt(&events, now)
    } else {
        let q = flags.rest.join(" ");
        if q.trim().is_empty() {
            eprintln!("usage: tb ask \"question\"  |  tb ask --recap");
            return 2;
        }
        ask_prompt(&st.name, &sp.name, &mood, &events, &q)
    };
    let sys = format!(
        "You are {} the {}, a tiny terminal companion. You answer questions about what has been happening in the user's terminal, using the event log you are given. Be concrete and useful, plain text, no preamble.",
        st.name, sp.name
    );
    let msgs = vec![
        ("system".to_string(), sys),
        ("user".to_string(), user),
    ];
    let max_tokens = g
        .cfg
        .ask_max_tokens
        .unwrap_or(if g.tier >= 2 { 512 } else { 256 });
    let timeout = Duration::from_secs(g.cfg.ask_timeout_secs.unwrap_or(15));
    let r = ai::resolve(&g.cfg);
    match ai::chat(&r, &msgs, Some(max_tokens), timeout) {
        Ok(answer) => {
            println!("{answer}");
            if flags.full {
                let path = dir.join("last_answer.md");
                match std::fs::write(&path, &answer) {
                    Ok(()) => println!("\nfull answer saved to {}", path.display()),
                    Err(e) => eprintln!("hatch ask: could not save the answer: {e}"),
                }
            }
            audit(&dir, "asked", 0, 0);
            0
        }
        Err(e) => {
            eprintln!("hatch ask: {e}");
            1
        }
    }
}

// ---- the agent loop ----

enum Action {
    Run { command: String },
    Read { path: String },
    Write { path: String, content: String },
    Done { message: String, success: bool },
}

fn action_from(v: &Value) -> Result<Action, String> {
    let kind = v
        .get("action")
        .and_then(|a| a.as_str())
        .ok_or_else(|| "no \"action\" field".to_string())?;
    match kind {
        "run" => Ok(Action::Run {
            command: v
                .get("command")
                .and_then(|c| c.as_str())
                .ok_or_else(|| "run needs \"command\"".to_string())?
                .to_string(),
        }),
        "read" => Ok(Action::Read {
            path: v
                .get("path")
                .and_then(|p| p.as_str())
                .ok_or_else(|| "read needs \"path\"".to_string())?
                .to_string(),
        }),
        "write" => Ok(Action::Write {
            path: v
                .get("path")
                .and_then(|p| p.as_str())
                .ok_or_else(|| "write needs \"path\"".to_string())?
                .to_string(),
            content: v
                .get("content")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_string(),
        }),
        "done" => Ok(Action::Done {
            message: v
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("done")
                .to_string(),
            success: v
                .get("success")
                .and_then(|s| s.as_bool())
                .unwrap_or(true),
        }),
        other => Err(format!(
            "unknown action \"{other}\" (use run, read, write, done)"
        )),
    }
}

/// Strip markdown fences and prose, then parse the first JSON object.
fn parse_agent_json(text: &str) -> Option<Value> {
    let mut t = text.trim();
    if let Some(rest) = t.strip_prefix("```json") {
        t = rest;
    } else if let Some(rest) = t.strip_prefix("```") {
        t = rest;
    }
    if let Some(rest) = t.strip_suffix("```") {
        t = rest;
    }
    let t = t.trim();
    let start = t.find('{')?;
    let end = t.rfind('}')?;
    if end < start {
        return None;
    }
    serde_json::from_str(&t[start..=end]).ok()
}

const DESTRUCTIVE_TOKENS: [&str; 6] = ["rm", "mkfs", "dd", "shutdown", "reboot", "halt"];
const DESTRUCTIVE_PHRASES: [&str; 3] = [
    "git push --force",
    "git push -f",
    "git reset --hard",
];

fn is_destructive(cmd: &str) -> bool {
    let mut c = cmd.trim_start();
    if let Some(rest) = c.strip_prefix("sudo ") {
        c = rest.trim_start();
    }
    if DESTRUCTIVE_PHRASES.iter().any(|p| c.contains(p)) {
        return true;
    }
    // Judge each pipeline/chain segment by its leading command: `echo rm`
    // is harmless, `cat x; rm -rf y` is not.
    c.split(|ch| ch == ';' || ch == '|' || ch == '&')
        .any(|seg| match seg.trim_start().split_whitespace().next() {
            Some(tok) => {
                DESTRUCTIVE_TOKENS.contains(&tok.trim_end_matches('/'))
                    || tok.starts_with("mkfs")
            }
            None => false,
        })
}

fn clip(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        return s.to_string();
    }
    let t: String = s.chars().take(n).collect();
    format!("{t}\n... (output truncated)")
}

fn run_capture(cmd: &str) -> (String, i32, u64) {
    let start = Instant::now();
    match Command::new("sh").arg("-c").arg(cmd).output() {
        Ok(o) => {
            let mut text = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if !err.trim().is_empty() {
                if !text.trim().is_empty() {
                    text.push('\n');
                }
                text.push_str(&err);
            }
            (
                clip(&text, MAX_TOOL_OUTPUT),
                o.status.code().unwrap_or(-1),
                start.elapsed().as_secs(),
            )
        }
        Err(e) => (
            format!("could not spawn: {e}"),
            -1,
            start.elapsed().as_secs(),
        ),
    }
}

pub fn agent_cmd(rest: &[String]) -> i32 {
    let flags = match parse_flags(rest) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("hatch agent: {e}");
            return 2;
        }
    };
    let dir = state::config_dir();
    let g = match gate(&dir, 2, "agent") {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    let mut task = flags.rest.join(" ");
    if let Ok(t) = std::env::var("HATCH_AGENT_TASK") {
        if !t.trim().is_empty() {
            task = t;
        }
    }
    if task.trim().is_empty() {
        eprintln!("usage: tb agent \"task\"  (tier 2: reads + runs; tier 3: edits too)");
        return 2;
    }
    if !flags.inner && std::env::var_os("TMUX").is_some() {
        return open_agent_window(&g, &task, &flags);
    }
    run_agent(&g, &task, &flags)
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn b64_encode(data: &[u8]) -> String {
    let mut out = String::new();
    for chunk in data.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            B64[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            B64[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// `tb agent` from inside tmux: the buddy gets its own window and hops back
/// to the caller's terminal the moment it finishes (the window closes).
fn open_agent_window(g: &Gate, task: &str, flags: &Flags) -> i32 {
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("tb"));
    let b64 = b64_encode(task.as_bytes());
    let mut cmd = format!(
        "HATCH_AGENT_TASK={b64} {} agent --inner",
        shell_quote(&exe.display().to_string())
    );
    if flags.yes {
        cmd.push_str(" --yes");
    }
    if flags.max_steps != AGENT_MAX_STEPS {
        cmd.push_str(&format!(" --max-steps {}", flags.max_steps));
    }
    let status = Command::new("tmux")
        .args(["new-window", "-n", "hatch-agent", &cmd])
        .status();
    match status {
        Ok(s) if s.success() => {
            println!("agent window opened; the buddy is on it there.");
            0
        }
        _ => {
            eprintln!("hatch agent: could not open a tmux window; running inline instead.");
            run_agent(g, task, flags)
        }
    }
}

// ---- the inner agent run: TUI in a tty, honest printing otherwise ----

struct Transcript {
    lines: Vec<String>,
}

impl Transcript {
    fn push(&mut self, l: String) {
        self.lines.push(l);
    }
}

fn tool_result_msg(label: &str, body: &str) -> String {
    format!("Tool result for {label}:\n{body}\nRespond with the next JSON step, or done when finished.")
}

fn run_agent(g: &Gate, task: &str, flags: &Flags) -> i32 {
    let dir = g.dir.clone();
    let st = state::load_or_hatch(&dir);
    let sp = species::get(&st.species).unwrap_or_else(species::fallback);
    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "?".to_string());
    let sys = format!(
        "You are {} the {}, a terminal agent working on the user's machine. Work step by step toward the task. Reply ONLY with one JSON object per turn:\n{{\"thought\": \"<=20 words\", \"action\": \"run\"|\"read\"|\"write\"|\"done\"}}\n- run: add \"command\": \"<shell command>\" (POSIX sh); commands run with captured output.\n- read: add \"path\": \"<file>\" to read a file.\n- write: add \"path\" and \"content\" to write a file.{}\n- done: add \"message\": \"<final answer>\" and \"success\": true|false.\nNever touch ~/.ssh, credentials, or API keys. Never send file contents anywhere except writing them locally. Keep each step small.",
        st.name,
        sp.name,
        if g.tier >= 3 {
            ""
        } else {
            " Writing files is NOT allowed at your tier: use done with the proposed change as a diff or clear instructions instead."
        }
    );
    let user0 = format!("Working directory: {cwd}\nPlatform: {}\n\nTask: {task}", std::env::consts::OS);
    let mut msgs: Vec<(String, String)> = vec![
        ("system".to_string(), sys),
        ("user".to_string(), user0),
    ];
    let mut t = Transcript {
        lines: vec![format!("task: {task}"), format!("cwd: {cwd}")],
    };
    let mut ui = if is_tty() { Some(AgentUi::new()) } else { None };
    let mut parse_failures = 0usize;
    let mut exit_code = 0i32;
    let mut final_msg = String::new();
    let started_all = Instant::now();

    for step in 1..=flags.max_steps {
        let r = ai::resolve(&g.cfg);
        let max_tokens = g.cfg.agent_max_tokens.unwrap_or(if g.tier >= 3 { 0 } else { 1024 });
        let timeout = Duration::from_secs(g.cfg.agent_timeout_secs.unwrap_or(60));
        let msgs_snapshot = msgs.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(ai::chat(
                &r,
                &msgs_snapshot,
                if max_tokens == 0 { None } else { Some(max_tokens) },
                timeout,
            ));
        });
        let step_started = Instant::now();
        let answer = loop {
            match rx.recv_timeout(Duration::from_millis(150)) {
                Ok(res) => break res,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if let Some(u) = ui.as_mut() {
                        u.draw(&t, sp, &st.name, step, step_started.elapsed().as_secs(), false, "");
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break Err("agent: model call thread died".to_string())
                }
            }
        };
        let text = match answer {
            Ok(x) => x,
            Err(e) => {
                t.push(format!("model call failed: {e}"));
                exit_code = 1;
                break;
            }
        };
        let v = match parse_agent_json(&text) {
            Some(v) => v,
            None => {
                parse_failures += 1;
                t.push(format!("step {step}: reply was not a step (no JSON object)"));
                if parse_failures >= 2 {
                    exit_code = 1;
                    break;
                }
                msgs.push((
                    "user".to_string(),
                    "Your reply was not a JSON step object. Reply ONLY with the JSON object: thought, action (run/read/write/done) and its fields.".to_string(),
                ));
                continue;
            }
        };
        parse_failures = 0;
        let thought = v
            .get("thought")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        let action = match action_from(&v) {
            Ok(a) => a,
            Err(e) => {
                msgs.push((
                    "user".to_string(),
                    format!("Malformed step: {e}. Reply ONLY with the JSON object."),
                ));
                t.push(format!("step {step}: malformed step: {e}"));
                continue;
            }
        };
        msgs.push(("assistant".to_string(), text.clone()));
        match action {
            Action::Done { message, success } => {
                t.push(format!("done: {message}"));
                final_msg = message;
                exit_code = if success { 0 } else { 1 };
                break;
            }
            Action::Run { command } => {
                t.push(format!("step {step}: {thought}"));
                t.push(format!("  run: {command}"));
                if needs_confirm_run(&g, &command) && !may_proceed(&mut ui, &t, sp, &st.name, step, &format!("run: {command}"), flags) {
                    t.push("  (declined)".to_string());
                    msgs.push((
                        "user".to_string(),
                        "The user declined that command. Choose a different approach or use done.".to_string(),
                    ));
                    audit(&dir, "agent_run", 2, 0);
                    continue;
                }
                let (out, code, secs) = run_capture(&command);
                audit(&dir, "agent_run", code, secs);
                t.push(format!("  -> exit {code}"));
                msgs.push((
                    "user".to_string(),
                    tool_result_msg(&format!("run: {command}"), &out),
                ));
            }
            Action::Read { path } => {
                t.push(format!("step {step}: {thought}"));
                t.push(format!("  read: {path}"));
                let body = match std::fs::read_to_string(&path) {
                    Ok(s) => clip(&s, AGENT_READ_CAP),
                    Err(e) => format!("read failed: {e}"),
                };
                audit(&dir, "agent_read", 0, 0);
                msgs.push((
                    "user".to_string(),
                    tool_result_msg(&format!("read: {path}"), &body),
                ));
            }
            Action::Write { path, content } => {
                t.push(format!("step {step}: {thought}"));
                t.push(format!("  write: {path}"));
                if g.tier < 3 {
                    t.push("  -> refused: writes need tier 3".to_string());
                    msgs.push((
                        "user".to_string(),
                        "Writes are not allowed at your tier. Use done with the proposed change as a diff instead.".to_string(),
                    ));
                    continue;
                }
                if needs_confirm_write(&g) && !may_proceed(&mut ui, &t, sp, &st.name, step, &format!("write: {path} ({} bytes)", content.len()), flags) {
                    t.push("  (declined)".to_string());
                    msgs.push((
                        "user".to_string(),
                        "The user declined that write. Choose a different approach or use done.".to_string(),
                    ));
                    audit(&dir, "agent_write", 2, 0);
                    continue;
                }
                let ok = std::fs::write(&path, &content).is_ok();
                let code = if ok { 0 } else { 1 };
                audit(&dir, "agent_write", code, 0);
                t.push(format!("  -> exit {code}"));
                msgs.push((
                    "user".to_string(),
                    tool_result_msg(&format!("write: {path}"), if ok { "written" } else { "write failed" }),
                ));
            }
        }
    }
    if exit_code != 0 && final_msg.is_empty() {
        final_msg = "hit the step limit or failed".to_string();
    }
    audit(&dir, "agent_done", exit_code, started_all.elapsed().as_secs());
    if let Some(u) = ui.as_mut() {
        u.draw(&t, sp, &st.name, 0, started_all.elapsed().as_secs(), false, "done. q to close");
        let _ = wait_for_q();
        u.teardown();
    } else {
        println!("{final_msg}");
    }
    exit_code
}

fn is_tty() -> bool {
    // A TUI needs a writable tty: crossterm would fail on a plain pipe.
    std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/tty")
        .is_ok()
        && std::env::var_os("TERM").is_some()
}

fn needs_confirm_run(g: &Gate, command: &str) -> bool {
    g.cfg.confirm_destructive != Some(false) && is_destructive(command)
}

fn needs_confirm_write(g: &Gate) -> bool {
    g.cfg.confirm_writes != Some(false)
}

/// Ask the human before a one-way door. In a tty this draws a prompt and
/// waits for y/n; headless there is no one to ask, so only --yes proceeds.
fn may_proceed(
    ui: &mut Option<AgentUi>,
    t: &Transcript,
    sp: &Species,
    name: &str,
    step: usize,
    what: &str,
    flags: &Flags,
) -> bool {
    if ui.is_none() {
        if flags.yes {
            return true;
        }
        eprintln!("hatch agent: refusing {what}: no terminal to confirm in; pass --yes to allow it.");
        return false;
    }
    let u = ui.as_mut().unwrap();
    u.draw(t, sp, name, step, 0, true, &format!("{what}  -- allow? [y/n]"));
    loop {
        if let Some(k) = poll_key(Duration::from_millis(150)) {
            match k.code {
                crossterm::event::KeyCode::Char('y') | crossterm::event::KeyCode::Char('Y') => {
                    return true
                }
                crossterm::event::KeyCode::Char('n')
                | crossterm::event::KeyCode::Char('N')
                | crossterm::event::KeyCode::Esc => return false,
                crossterm::event::KeyCode::Char('q') => return false,
                _ => {}
            }
        }
    }
}

fn poll_key(timeout: Duration) -> Option<crossterm::event::KeyEvent> {
    if crossterm::event::poll(timeout).unwrap_or(false) {
        if let Ok(crossterm::event::Event::Key(k)) = crossterm::event::read() {
            if k.kind == crossterm::event::KeyEventKind::Press {
                return Some(k);
            }
        }
    }
    None
}

fn wait_for_q() -> bool {
    loop {
        if let Some(k) = poll_key(Duration::from_millis(150)) {
            match k.code {
                crossterm::event::KeyCode::Char('q') | crossterm::event::KeyCode::Esc => {
                    return true
                }
                _ => return false,
            }
        }
    }
}

struct AgentUi {
    terminal:
        ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
}

impl AgentUi {
    fn new() -> AgentUi {
        let mut stdout = std::io::stdout();
        let _ = crossterm::terminal::enable_raw_mode();
        let _ = crossterm::execute!(
            stdout,
            crossterm::terminal::EnterAlternateScreen
        );
        let terminal =
            ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(stdout))
                .expect("agent: terminal");
        AgentUi { terminal }
    }

    fn draw(
        &mut self,
        t: &Transcript,
        sp: &Species,
        name: &str,
        step: usize,
        elapsed: u64,
        confirming: bool,
        prompt: &str,
    ) {
        let _ = self.terminal.draw(|f| {
            let area = f.area();
            // transcript fills the window; the sprite box draws over it later.
            let shown: usize = t.lines.len();
            let max_rows = area.height as usize;
            let skip = shown.saturating_sub(max_rows);
            let lines: Vec<ratatui::text::Line> = t.lines[skip..]
                .iter()
                .map(|l| {
                    ratatui::text::Line::from(ratatui::text::Span::styled(
                        l.clone(),
                        ratatui::style::Style::default()
                            .fg(ratatui::style::Color::White),
                    ))
                })
                .collect();
            f.render_widget(
                ratatui::widgets::Paragraph::new(lines),
                area,
            );

            // the buddy, docked bottom-right while it works.
            let art = sp.art.frame_for(Mood::Working, 0);
            let art_h = art.lines().count() as u16;
            let w = 28u16.min(area.width);
            let h = (art_h + 4).min(area.height);
            let rect = ratatui::layout::Rect {
                x: area.width.saturating_sub(w),
                y: area.height.saturating_sub(h),
                width: w,
                height: h,
            };
            f.render_widget(ratatui::widgets::Clear, rect);
            let mut sprite: Vec<ratatui::text::Line> = Vec::new();
            for l in art.lines() {
                sprite.push(ratatui::text::Line::from(ratatui::text::Span::styled(
                    l.to_string(),
                    ratatui::style::Style::default().fg(ratatui::style::Color::Green),
                )));
            }
            sprite.push(ratatui::text::Line::from(ratatui::text::Span::styled(
                format!("{name} the {}", sp.name),
                ratatui::style::Style::default()
                    .fg(ratatui::style::Color::White)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )));
            sprite.push(ratatui::text::Line::from(ratatui::text::Span::styled(
                if step == 0 {
                    "finished".to_string()
                } else {
                    format!("step {step} - {elapsed}s")
                },
                ratatui::style::Style::default().fg(ratatui::style::Color::Cyan),
            )));
            f.render_widget(
                ratatui::widgets::Paragraph::new(sprite).block(
                    ratatui::widgets::Block::bordered().title("hatch agent"),
                ),
                rect,
            );

            if confirming || !prompt.is_empty() {
                let bar = ratatui::layout::Rect {
                    x: 0,
                    y: area.height.saturating_sub(1),
                    width: area.width.saturating_sub(w),
                    height: 1,
                };
                f.render_widget(
                    ratatui::widgets::Paragraph::new(ratatui::text::Line::from(
                        ratatui::text::Span::styled(
                            format!(" {prompt}"),
                            ratatui::style::Style::default()
                                .fg(ratatui::style::Color::Yellow),
                        ),
                    )),
                    bar,
                );
            }
        });
    }

    fn teardown(&mut self) {
        let _ = crossterm::execute!(
            self.terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen
        );
        crossterm::terminal::disable_raw_mode().ok();
        self.terminal.show_cursor().ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn flags_parse() {
        let f = parse_flags(&v(&["--full", "--yes", "why", "did", "it", "fail"]))
            .unwrap();
        assert!(f.full && f.yes && !f.recap && !f.inner);
        assert_eq!(f.rest, v(&["why", "did", "it", "fail"]));
        let f = parse_flags(&v(&["--recap"])).unwrap();
        assert!(f.recap && f.rest.is_empty());
        let f = parse_flags(&v(&["--max-steps", "3", "task"])).unwrap();
        assert_eq!(f.max_steps, 3);
        assert!(parse_flags(&v(&["--nope"])).is_err());
        assert!(parse_flags(&v(&["--max-steps"])).is_err());
        assert!(parse_flags(&v(&["--max-steps", "0"])).is_err());
    }

    #[test]
    fn agent_json_parses_bare_fenced_and_prose() {
        let bare = r#"{"thought":"t","action":"done","message":"m","success":true}"#;
        assert!(parse_agent_json(bare).is_some());
        let fenced = "```json\n{\"action\":\"run\",\"command\":\"ls\"}\n```";
        assert!(parse_agent_json(fenced).is_some());
        let prose = "Sure! {\"action\":\"read\",\"path\":\"x\"} hope that helps";
        assert!(parse_agent_json(prose).is_some());
        assert!(parse_agent_json("no json here").is_none());
        assert!(parse_agent_json("{ broken").is_none());
        let multiline = "{\n  \"action\": \"write\",\n  \"path\": \"a.txt\",\n  \"content\": \"hi\\nthere\"\n}";
        assert!(parse_agent_json(multiline).is_some());
    }

    #[test]
    fn actions_parse_with_fields() {
        let a = action_from(&parse_agent_json("{\"action\":\"run\",\"command\":\"ls -la\"}").unwrap()).unwrap();
        assert!(matches!(a, Action::Run { command } if command == "ls -la"));
        let a = action_from(&parse_agent_json("{\"action\":\"read\",\"path\":\"/x\"}").unwrap()).unwrap();
        assert!(matches!(a, Action::Read { path } if path == "/x"));
        let a = action_from(
            &parse_agent_json("{\"action\":\"write\",\"path\":\"/x\",\"content\":\"hi\"}").unwrap(),
        )
        .unwrap();
        assert!(matches!(a, Action::Write { ref path, content } if path == "/x" && content == "hi"));
        let a = action_from(&parse_agent_json("{\"action\":\"done\",\"message\":\"m\",\"success\":false}").unwrap()).unwrap();
        assert!(matches!(a, Action::Done { ref message, success } if message == "m" && !success));
        assert!(action_from(&parse_agent_json("{\"action\":\"dance\"}").unwrap()).is_err());
        assert!(action_from(&parse_agent_json("{\"thought\":\"x\"}").unwrap()).is_err());
        assert!(action_from(&parse_agent_json("{\"action\":\"run\"}").unwrap()).is_err());
    }

    #[test]
    fn destructive_commands_are_caught() {
        assert!(is_destructive("rm -rf /"));
        assert!(is_destructive("sudo rm -rf /tmp/x"));
        assert!(is_destructive("rm x"));
        assert!(is_destructive("mkfs.ext4 /dev/sda"));
        assert!(is_destructive("shutdown now"));
        assert!(is_destructive("git push --force origin main"));
        assert!(is_destructive("git reset --hard"));
        assert!(!is_destructive("cargo test"));
        assert!(!is_destructive("ls -la"));
        assert!(!is_destructive("echo rm"));
        assert!(!is_destructive("git push origin feat/agent"));
        assert!(!is_destructive("git commit -m \"rm anything\""));
        assert!(is_destructive("cat x; rm -rf y"));
        assert!(is_destructive("echo hi && sudo mkfs.btrfs /dev/sda"));
        assert!(!is_destructive("ls file | wc -l"));
    }

    #[test]
    fn clip_truncates_with_marker() {
        assert_eq!(clip("short", 10), "short");
        let long = "x".repeat(20);
        let c = clip(&long, 10);
        assert!(c.starts_with("xxxxxxxxxx"));
        assert!(c.contains("output truncated"));
    }

    #[test]
    fn digest_and_recap_shape() {
        assert_eq!(events_digest(&[]), "(no events yet)");
        let ev = Event { ts: state::now_secs() - 10, kind: "build_finished".into(), duration_secs: 3, exit_code: 0 };
        let d = events_digest(&[ev.clone()]);
        assert!(d.contains("build_finished") && d.contains("exit=0"));
        let old = Event { ts: state::now_secs() - 7200, kind: "old".into(), duration_secs: 0, exit_code: 0 };
        let r = recap_prompt(&[ev, old], state::now_secs());
        assert!(r.contains("build_finished"));
        assert!(!r.contains("- old "), "events older than an hour stay out of the recap");
    }

    #[test]
    fn run_capture_reports_exit_and_output() {
        let (out, code, _) = run_capture("echo ok");
        assert_eq!(code, 0);
        assert!(out.contains("ok"));
        let (_, code, _) = run_capture("false");
        assert_eq!(code, 1);
    }

    #[test]
    fn b64_output_is_shell_safe() {
        for s in [
            "",
            "hi",
            "task with 'quotes' and spaces",
            "emoji-less but long: aaaaaaaaaaaaaaaaaaaaaaaa",
        ] {
            let enc = b64_encode(s.as_bytes());
            assert!(
                enc.chars().all(|c| B64.contains(&(c as u8)) || c == '='),
                "only the safe alphabet and padding survive: {enc}"
            );
            assert_eq!(b64_encode(s.as_bytes()), enc, "deterministic");
            // base64 padding is length-correct: (len+2)/3*4
            assert_eq!(enc.len(), (s.len() + 2) / 3 * 4);
        }
    }

    #[test]
    fn gate_demands_config_and_tier() {
        let dir = tempfile::tempdir().unwrap();
        let e = gate(dir.path(), 1, "ask").unwrap_err();
        assert!(e.contains("no AI voice configured"));
        std::fs::write(dir.path().join("ai.toml"), "endpoint = \"http://127.0.0.1:9/v1\"\nmodel = \"m\"\napi_key = \"k\"\n").unwrap();
        let e = gate(dir.path(), 1, "ask").unwrap_err();
        assert!(e.contains("no capability tier yet"), "got: {e}");
        std::fs::write(dir.path().join("ai_tier"), "2").unwrap();
        let g = gate(dir.path(), 2, "agent").unwrap();
        assert_eq!(g.tier, 2);
        let e = gate(dir.path(), 3, "agent").unwrap_err();
        assert!(e.contains("needs tier 3"), "got: {e}");
    }
}
