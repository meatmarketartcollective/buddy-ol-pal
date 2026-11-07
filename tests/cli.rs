//! End-to-end tests over the real binary. Each test gets its own BUDDY_HOME
//! tempdir so nothing touches a real config; every assertion reads the CLI's
//! own stdout/stderr and exit code, the same surface a user sees.

use std::process::Command;
use tempfile::TempDir;

struct Run {
    code: i32,
    out: String,
    err: String,
}

fn buddy(home: &TempDir, args: &[&str]) -> Run {
    let out = Command::new(env!("CARGO_BIN_EXE_buddy"))
        .args(args)
        .env("BUDDY_HOME", home.path())
        .env_remove("XDG_CONFIG_HOME")
        .output()
        .expect("spawn buddy");
    Run {
        code: out.status.code().unwrap_or(-1),
        out: String::from_utf8_lossy(&out.stdout).to_string(),
        err: String::from_utf8_lossy(&out.stderr).to_string(),
    }
}

/// `buddy dex` hatches a fresh companion on a clean home (load_or_hatch),
/// unlike `buddy status`, which must stay read-only.
fn hatch(home: &TempDir) -> String {
    let r = buddy(home, &["dex"]);
    assert_eq!(r.code, 0, "dex failed: {}", r.err);
    r.out
}

#[test]
fn fresh_status_hatches_nothing() {
    let home = TempDir::new().unwrap();
    let r = buddy(&home, &["status"]);
    assert_eq!(r.code, 0);
    assert!(r.out.contains("no companion yet"), "got: {}", r.out);
    assert!(!home.path().join("state.toml").exists(), "status must not hatch");
}

#[test]
fn status_after_hatch_names_the_pet() {
    let home = TempDir::new().unwrap();
    hatch(&home);
    let r = buddy(&home, &["status"]);
    assert_eq!(r.code, 0);
    assert!(r.out.contains("the "), "got: {}", r.out);
    assert!(!r.out.contains("no companion yet"));
    assert!(r.out.contains("tier="), "status must show the rarity tier: {}", r.out);
}

#[test]
fn feed_pays_one_xp_and_second_feed_hits_cooldown() {
    let home = TempDir::new().unwrap();
    hatch(&home);
    let r = buddy(&home, &["feed"]);
    assert_eq!(r.code, 0);
    assert!(r.out.contains("munches happily"), "got: {}", r.out);
    let name = r.out.split(" the ").next().unwrap().to_string();
    assert!(!name.is_empty());

    let st = buddy(&home, &["status"]);
    assert!(st.out.contains(&name), "status lost the name: {}", st.out);
    assert!(st.out.contains("xp=1"), "feed should pay 1 xp: {}", st.out);

    let again = buddy(&home, &["feed"]);
    assert!(again.out.contains("full for now"), "got: {}", again.out);
    let st = buddy(&home, &["status"]);
    assert!(st.out.contains("xp=1"), "cooldown feed must not pay xp");
}

#[test]
fn off_silences_and_on_wakes_while_state_survives() {
    let home = TempDir::new().unwrap();
    hatch(&home);
    assert!(buddy(&home, &["feed"]).out.contains("munches happily"));
    assert!(buddy(&home, &["off"]).out.contains("the buddy is off"));
    assert!(buddy(&home, &["on"]).out.contains("the buddy is back on"));
    // state survived the switch: the feed cooldown from before off/on still holds
    assert!(buddy(&home, &["feed"]).out.contains("full for now"));
}

#[test]
fn emit_logs_an_event_tail_can_show() {
    let home = TempDir::new().unwrap();
    hatch(&home);
    let r = buddy(&home, &["emit", "custom_event", "--exit", "1", "--secs", "2"]);
    assert_eq!(r.code, 0);
    let r = buddy(&home, &["tail", "-n", "1"]);
    assert!(r.out.contains("\"kind\":\"custom_event\""), "got: {}", r.out);
    assert!(r.out.contains("\"exit_code\":1"), "got: {}", r.out);
}

#[test]
fn emit_without_reportable_outcome_logs_nothing() {
    let home = TempDir::new().unwrap();
    hatch(&home);
    let r = buddy(&home, &["emit"]);
    assert_eq!(r.code, 0);
    let r = buddy(&home, &["tail", "-n", "10"]);
    assert!(r.out.trim().is_empty(), "exit 0 fast success should be skipped, got: {}", r.out);
}

#[test]
fn run_wraps_a_fast_success_silently_and_a_failure_loudly() {
    let home = TempDir::new().unwrap();
    hatch(&home);
    let r = buddy(&home, &["run", "--", "true"]);
    assert_eq!(r.code, 0);
    assert!(r.out.trim().is_empty(), "fast success must stay quiet: {}", r.out);
    assert!(buddy(&home, &["tail", "-n", "10"]).out.trim().is_empty());

    let r = buddy(&home, &["run", "--", "false"]);
    assert_eq!(r.code, 1);
    let r = buddy(&home, &["tail", "-n", "1"]);
    assert!(r.out.contains("command_failed"), "got: {}", r.out);
}

#[test]
fn run_rejects_an_empty_command() {
    let home = TempDir::new().unwrap();
    let r = buddy(&home, &["run"]);
    assert_eq!(r.code, 2);
    assert!(r.err.contains("usage"), "got: {}", r.err);
}

#[test]
fn shell_prints_bash_by_default_and_zsh_on_ask() {
    let home = TempDir::new().unwrap();
    let r = buddy(&home, &["shell"]);
    assert_eq!(r.code, 0);
    assert!(r.out.contains("__buddy_capture"), "got: {}", r.out);
    let r = buddy(&home, &["shell", "zsh"]);
    assert!(r.out.contains("add-zsh-hook"), "got: {}", r.out);
}

#[test]
fn shell_rejects_unsupported_shells() {
    let home = TempDir::new().unwrap();
    let r = buddy(&home, &["shell", "fish"]);
    assert_eq!(r.code, 2);
    assert!(r.err.contains("unsupported shell"), "got: {}", r.err);
}

#[test]
fn unknown_command_fails_with_help() {
    let home = TempDir::new().unwrap();
    let r = buddy(&home, &["frobnicate"]);
    assert_eq!(r.code, 2);
    assert!(r.err.contains("unknown command"), "got: {}", r.err);
    assert!(r.out.contains("buddy status"), "help should follow: {}", r.out);
}

#[test]
fn tail_respects_an_explicit_count() {
    let home = TempDir::new().unwrap();
    hatch(&home);
    for kind in ["a_one", "a_two", "a_three"] {
        assert_eq!(buddy(&home, &["emit", kind, "--exit", "1", "--secs", "1"]).code, 0);
    }
    let r = buddy(&home, &["tail", "-n", "2"]);
    let lines: Vec<&str> = r.out.trim().lines().collect();
    assert_eq!(lines.len(), 2, "got: {}", r.out);
    assert!(lines[1].contains("a_three"), "newest last: {}", r.out);
}

#[test]
fn talk_answers_from_the_canned_pool_without_ai() {
    let home = TempDir::new().unwrap();
    hatch(&home);
    let r = buddy(&home, &["talk"]);
    assert_eq!(r.code, 0, "stderr: {}", r.err);
    assert!(r.out.contains("(talked)"), "got: {}", r.out);
    assert!(r.out.contains(" the "), "the pet name should be in the reply");
}

#[test]
fn pet_logs_affection() {
    let home = TempDir::new().unwrap();
    hatch(&home);
    let r = buddy(&home, &["pet"]);
    assert_eq!(r.code, 0);
    assert!(r.out.contains("soaks it in"), "got: {}", r.out);
    let r = buddy(&home, &["tail", "-n", "1"]);
    assert!(r.out.contains("petted"), "got: {}", r.out);
}

#[test]
fn dex_lists_species_with_undiscovered_hidden() {
    let home = TempDir::new().unwrap();
    hatch(&home);
    let r = buddy(&home, &["dex"]);
    assert_eq!(r.code, 0);
    assert!(r.out.contains("???"), "undiscovered species should hide: {}", r.out);
    assert!(r.out.lines().count() > 1, "dex should list many species");
}
