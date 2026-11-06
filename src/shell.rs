//! Passive capture snippets for interactive shells. `eval "$(tb shell bash)"`
//! in .bashrc (or the zsh variant in .zshrc) feeds every interactive
//! command's outcome to tb: no wrapping, no aliasing, nothing to remember.
//! Fast successes are skipped; failures and slow successes become events,
//! exactly as `tb run` reports them. Commands starting with tb are ignored so
//! the capture does not chase itself.

pub const BASH: &str = r##"# tb: your terminal companion. Every command feeds it now.
__tb_prompt_done=1
__tb_preexec() {
  local c=$BASH_COMMAND
  [ "$c" = "__tb_preexec" ] && return
  [ -z "$__tb_busy" ] || return
  [ "$__tb_prompt_done" = 1 ] || return
  case "$c" in "__tb_"*|"trap "*|PROMPT_COMMAND*) return ;; esac
  __tb_prompt_done=0
  __tb_t0=${EPOCHSECONDS:-$(date +%s)}
  __tb_cmd=$c
}
__tb_capture() {
  __tb_busy=1
  local code=$? first secs
  __tb_prompt_done=1
  [ -n "$__tb_t0" ] || { __tb_busy=; return; }
  first=${__tb_cmd%% *}
  if [ "$first" != tb ]; then
    secs=$(( ${EPOCHSECONDS:-$(date +%s)} - __tb_t0 ))
    if [ "$code" -ne 0 ] || [ "$secs" -ge "${TB_THRESHOLD_SECS:-10}" ]; then
      command tb emit --exit "$code" --secs "$secs" >/dev/null 2>&1 &
    fi
  fi
  __tb_t0=
  __tb_busy=
}
trap '__tb_preexec' DEBUG
case ";$PROMPT_COMMAND;" in
  *";__tb_capture;"*) ;;
  *) PROMPT_COMMAND="__tb_capture${PROMPT_COMMAND:+;$PROMPT_COMMAND}" ;;
esac
"##;

pub const ZSH: &str = r##"# tb: your terminal companion. Every command feeds it now.
# zsh reports the command itself, so this version is simpler than bash's.
typeset -g __tb_t0 __tb_cmd
__tb_preexec() { __tb_t0=${EPOCHSECONDS:-$(date +%s)}; __tb_cmd=$1; }
__tb_precmd() {
  local code=$? first secs
  [ -n "$__tb_t0" ] || return
  first=${${=__tb_cmd}[1]}
  if [ "$first" != tb ]; then
    secs=$(( ${EPOCHSECONDS:-$(date +%s)} - __tb_t0 ))
    if [ "$code" -ne 0 ] || [ "$secs" -ge "${TB_THRESHOLD_SECS:-10}" ]; then
      command tb emit --exit "$code" --secs "$secs" >/dev/null 2>&1 &
    fi
  fi
  __tb_t0=
}
autoload -Uz add-zsh-hook
add-zsh-hook preexec __tb_preexec
add-zsh-hook precmd __tb_precmd
"##;
