_sanitize_filenames()
{
  local cur prev
  local -a words

  COMPREPLY=()
  cur="${COMP_WORDS[COMP_CWORD]}"
  prev="${COMP_WORDS[COMP_CWORD-1]}"
  words=("${COMP_WORDS[@]}")

  # Detect if we've seen the option terminator `--` before this position.
  local have_terminator=0
  local i
  for (( i=1; i<COMP_CWORD; i++ )); do
    if [[ ${words[i]} == -- ]]; then
      have_terminator=1
      break
    fi
  done

  # If the previous token expects a value for replacement, suggest common chars.
  if [[ $have_terminator -eq 0 && ( $prev == "-c" || $prev == "--replacement" ) ]]; then
    COMPREPLY=( $(compgen -W '_ - . +' -- "$cur") )
    return 0
  fi

  # If we're completing an option (and not after `--`), offer flags.
  if [[ $have_terminator -eq 0 && $cur == -* ]]; then
    local opts="--recursive -r --dry-run -n --replacement -c --full-sanitize -F --help -h --"
    COMPREPLY=( $(compgen -W "${opts}" -- "$cur") )
    return 0
  fi

  # Otherwise, complete filenames. After `--` we never suggest options.
  # Use compgen -f so both files and directories are suggested.
  COMPREPLY=( $(compgen -f -- "$cur") )
}

complete -F _sanitize_filenames sanitize_filenames
