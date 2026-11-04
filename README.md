# hatch

A terminal companion that lives in your terminal and reacts to real events.
One creature, three moods, zero accounts. It remembers your builds, not your
business: events record that a command ran and how long, never what it was.

Status: milestone one. Rust + ratatui, single binary.

## Usage

    tb ui                # open the companion (plain `tb` also works)
    tb run -- cargo build  # wrap a command; long successes and any failure become events

State lives in `~/.config/hatch/` (`state.toml` + append-only `events.jsonl`,
override the directory with `HATCH_HOME`). A truncated final line in the event
log is skipped, never fatal. Corrupt or version-mismatched `state.toml` is
backed up to `state.toml.bak-<unix>` and your companion re-hatches.

Milestone one deliberately excludes: multi-harness adapters, extra species,
customization UI, XP curves, cross-pane notifications, packaging.

Inspired by the community's reaction to the Claude Code buddy removal
(anthropics/claude-code#45596): an open alternative, yours to own.

License: MIT
