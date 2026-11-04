//! Passive capture snippets for interactive shells. `eval "$(tb shell bash)"`
//! in .bashrc (or the zsh variant in .zshrc) feeds every interactive
//! command's outcome to hatch: no wrapping, no aliasing, nothing to remember.
//! Fast successes are skipped; failures and slow successes become events,
//! exactly as `tb run` reports them. Commands starting with tb are ignored so
//! the capture does not chase itself.

pub const BASH: &str = r##"# hatch: your terminal companion. Every command feeds it now.
__hatch_prompt_done=1
__hatch_preexec() {
  local c=$BASH_COMMAND
  [ "$c" = "__hatch_preexec" ] && return
  [ -z "$__hatch_busy" ] || return
  [ "$__hatch_prompt_done" = 1 ] || return
  case "$c" in "__hatch_"*|"trap "*|PROMPT_COMMAND*) return ;; esac
  __hatch_prompt_done=0
  __hatch_t0=${EPOCHSECONDS:-$(date +%s)}
  __hatch_cmd=$c
}
__hatch_capture() {
  __hatch_busy=1
  local code=$? first secs
  __hatch_prompt_done=1
  [ -n "$__hatch_t0" ] || { __hatch_busy=; return; }
  first=${__hatch_cmd%% *}
  if [ "$first" != tb ]; then
    secs=$(( ${EPOCHSECONDS:-$(date +%s)} - __hatch_t0 ))
    if [ "$code" -ne 0 ] || [ "$secs" -ge "${HATCH_THRESHOLD_SECS:-10}" ]; then
      command tb emit --exit "$code" --secs "$secs" >/dev/null 2>&1 &
    fi
  fi
  __hatch_t0=
  __hatch_busy=
}
trap '__hatch_preexec' DEBUG
case ";$PROMPT_COMMAND;" in
  *";__hatch_capture;"*) ;;
  *) PROMPT_COMMAND="__hatch_capture${PROMPT_COMMAND:+;$PROMPT_COMMAND}" ;;
esac
"##;

pub const ZSH: &str = r##"# hatch: your terminal companion. Every command feeds it now.
# zsh reports the command itself, so this version is simpler than bash's.
typeset -g __hatch_t0 __hatch_cmd
__hatch_preexec() { __hatch_t0=${EPOCHSECONDS:-$(date +%s)}; __hatch_cmd=$1; }
__hatch_precmd() {
  local code=$? first secs
  [ -n "$__hatch_t0" ] || return
  first=${${=__hatch_cmd}[1]}
  if [ "$first" != tb ]; then
    secs=$(( ${EPOCHSECONDS:-$(date +%s)} - __hatch_t0 ))
    if [ "$code" -ne 0 ] || [ "$secs" -ge "${HATCH_THRESHOLD_SECS:-10}" ]; then
      command tb emit --exit "$code" --secs "$secs" >/dev/null 2>&1 &
    fi
  fi
  __hatch_t0=
}
autoload -Uz add-zsh-hook
add-zsh-hook preexec __hatch_preexec
add-zsh-hook precmd __hatch_precmd
"##;
