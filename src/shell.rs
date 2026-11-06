//! Passive capture snippets for interactive shells. `eval "$(buddy shell bash)"`
//! in .bashrc (or the zsh variant in .zshrc) feeds every interactive
//! command's outcome to buddy: no wrapping, no aliasing, nothing to remember.
//! Fast successes are skipped; failures and slow successes become events,
//! exactly as `buddy run` reports them. Commands starting with buddy are ignored so
//! the capture does not chase itself.

pub const BASH: &str = r##"# buddy: your terminal companion. Every command feeds it now.
__buddy_prompt_done=1
__buddy_preexec() {
  local c=$BASH_COMMAND
  [ "$c" = "__buddy_preexec" ] && return
  [ -z "$__buddy_busy" ] || return
  [ "$__buddy_prompt_done" = 1 ] || return
  case "$c" in "__buddy_"*|"trap "*|PROMPT_COMMAND*) return ;; esac
  __buddy_prompt_done=0
  __buddy_t0=${EPOCHSECONDS:-$(date +%s)}
  __buddy_cmd=$c
}
__buddy_capture() {
  __buddy_busy=1
  local code=$? first secs
  __buddy_prompt_done=1
  [ -n "$__buddy_t0" ] || { __buddy_busy=; return; }
  first=${__buddy_cmd%% *}
  if [ "$first" != buddy ]; then
    secs=$(( ${EPOCHSECONDS:-$(date +%s)} - __buddy_t0 ))
    if [ "$code" -ne 0 ] || [ "$secs" -ge "${BUDDY_THRESHOLD_SECS:-10}" ]; then
      command buddy emit --exit "$code" --secs "$secs" >/dev/null 2>&1 &
    fi
  fi
  __buddy_t0=
  __buddy_busy=
}
trap '__buddy_preexec' DEBUG
case ";$PROMPT_COMMAND;" in
  *";__buddy_capture;"*) ;;
  *) PROMPT_COMMAND="__buddy_capture${PROMPT_COMMAND:+;$PROMPT_COMMAND}" ;;
esac
"##;

pub const ZSH: &str = r##"# buddy: your terminal companion. Every command feeds it now.
# zsh reports the command itself, so this version is simpler than bash's.
typeset -g __buddy_t0 __buddy_cmd
__buddy_preexec() { __buddy_t0=${EPOCHSECONDS:-$(date +%s)}; __buddy_cmd=$1; }
__buddy_precmd() {
  local code=$? first secs
  [ -n "$__buddy_t0" ] || return
  first=${${=__buddy_cmd}[1]}
  if [ "$first" != buddy ]; then
    secs=$(( ${EPOCHSECONDS:-$(date +%s)} - __buddy_t0 ))
    if [ "$code" -ne 0 ] || [ "$secs" -ge "${BUDDY_THRESHOLD_SECS:-10}" ]; then
      command buddy emit --exit "$code" --secs "$secs" >/dev/null 2>&1 &
    fi
  fi
  __buddy_t0=
}
autoload -Uz add-zsh-hook
add-zsh-hook preexec __buddy_preexec
add-zsh-hook precmd __buddy_precmd
"##;

#[cfg(test)]
mod tests {
    use super::{BASH, ZSH};

    fn tokens(s: &str) -> Vec<&str> {
        s.split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .filter(|t| !t.is_empty())
            .collect()
    }

    #[test]
    fn bash_capture_wires_trap_and_prompt_command() {
        assert!(BASH.contains("trap '__buddy_preexec' DEBUG"));
        assert!(BASH.contains("PROMPT_COMMAND=\"__buddy_capture"));
        assert!(BASH.contains("__buddy_capture() {"));
    }

    #[test]
    fn zsh_capture_uses_zsh_hooks() {
        assert!(ZSH.contains("add-zsh-hook preexec __buddy_preexec"));
        assert!(ZSH.contains("add-zsh-hook precmd __buddy_precmd"));
        assert!(ZSH.contains("__buddy_cmd=$1"), "zsh must report the command itself");
    }

    #[test]
    fn both_snippets_ignore_buddy_commands() {
        for s in [BASH, ZSH] {
            assert!(
                s.contains("!= buddy ]"),
                "capture must not chase its own emit"
            );
        }
    }

    #[test]
    fn both_snippets_emit_exit_and_secs() {
        for s in [BASH, ZSH] {
            assert!(s.contains(r#"command buddy emit --exit "$code" --secs "$secs""#));
        }
    }

    #[test]
    fn threshold_is_env_overridable_with_sane_default() {
        for s in [BASH, ZSH] {
            assert!(s.contains("${BUDDY_THRESHOLD_SECS:-10}"));
        }
    }

    #[test]
    fn fast_successes_are_skipped_slow_or_failed_are_kept() {
        for s in [BASH, ZSH] {
            assert!(s.contains("\"$code\" -ne 0"), "failures must be captured");
            assert!(s.contains("-ge \"${BUDDY_THRESHOLD_SECS"), "slow runs must be captured");
        }
    }

    #[test]
    fn bash_guard_fires_only_once_per_prompt() {
        assert!(BASH.contains("__buddy_prompt_done=1"));
        assert!(BASH.contains("__buddy_busy=1"));
    }

    #[test]
    fn snippets_carry_no_legacy_tb_name() {
        for s in [BASH, ZSH] {
            for t in tokens(s) {
                assert_ne!(t, "tb", "legacy tb token in shell snippet");
                assert_ne!(t, "TB_HOME");
            }
            assert!(!s.contains("TB_"));
        }
    }
}
