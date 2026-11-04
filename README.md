# hatch

A terminal companion that lives in your terminal and reacts to real events.
One creature, 38 species across five rarity tiers, zero accounts. It remembers
your builds, not your business: events record that a command ran and how long,
never what it was.

Rust + ratatui, single binary.

## Install

    git clone <repo> && cd hatch
    ./install.sh

That builds the release binary and puts `tb` in `~/.local/bin`
(set `HATCH_BIN` to install somewhere else). First run hatches a companion
and saves it locally. No config, no account.

## Usage

    tb                          # open the companion UI
    tb run -- cargo build       # wrap a command; a slow success or any failure becomes an event
    tb status                   # one line: name, species, level, mood, xp, chaos
    tb tail                     # the recent events
    tb dex                      # species you have discovered
    tb pet / feed / play / talk # interactions, each on its own cooldown
    tb off / tb on              # the off switch: no notifications, no UI; events still log
    tb emit <kind> [--exit N] [--secs S]  # adapter: feed hatch an event from any harness

#### Passive capture (no wrapping)

Interactive shells can feed hatch without wrapping anything:

    eval "$(tb shell bash)"    # add to .bashrc
    eval "$(tb shell zsh)"     # add to .zshrc

Every command's outcome and duration then reach the buddy on their own.
Fast successes are skipped (nothing worth reacting to); failures and slow
successes become events, same as a wrapped run. Commands starting with tb
are ignored so the capture does not chase itself. Set HATCH_THRESHOLD_SECS
to move the slow-success line.

## Wiring it to real work

Wrap the commands you care about, and long successes plus any failure turn
into events:

    tb run -- cargo build
    tb run -- pytest

Or, from any harness or hook (Claude Code hooks, CI scripts, your own cron),
report the event without wrapping the command:

    tb emit build_finished --exit 0 --secs 42
    tb emit command_failed --exit 2

`emit` and `run` feed the same state: XP, mood, chaos, notifications.

### Notifications

A logged build event emits an OSC 9 desktop-notification escape on stdout,
and inside tmux also a `display-message` the other panes see. `tb off`
silences everything and keeps the UI from opening; `tb on` wakes the buddy
back up. Events keep logging either way.

## State

State lives in `~/.config/hatch/` (`state.toml` plus an append-only
`events.jsonl`; override the directory with `HATCH_HOME`). A truncated final
line in the event log is skipped, never fatal. Corrupt or version-mismatched
`state.toml` is backed up to `state.toml.bak-<unix>` and your companion
re-hatches. Older state files migrate forward (v1 through v5) without losing
anything.

Privacy: events never record command content, paths, or secrets. Only kind,
timestamp, duration, and exit code.

## Uninstall

Delete the binary (`~/.local/bin/tb` or `$HATCH_BIN/tb`) and the
`~/.config/hatch/` directory. That is everything hatch keeps.
