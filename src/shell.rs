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
