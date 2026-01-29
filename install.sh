#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

ROOT_DIR=""
PROJECT_ROOT="$(pwd)" || { echo "error: failed to determine current directory" >&2; exit 1; }
TMP_ROOT=""
TTY_DEVICE=""
SAVED_TTY_STATE=""
REPO_URL="https://github.com/heyAyushh/stacc.git"

NON_INTERACTIVE=0
DRY_RUN=0
VERBOSE=0
SELECTED_EDITORS=""
SELECTED_SCOPE=""
SELECTED_CATEGORIES=""
SELECTED_MCP_SERVERS=""
SELECTED_STACKS=""
CONFLICT_MODE=""
DISABLED_FLAGS=()

cleanup() {
  # Restore terminal if needed
  if [ -n "${SAVED_TTY_STATE}" ] && [ -n "${TTY_DEVICE}" ]; then
    stty "${SAVED_TTY_STATE}" < "${TTY_DEVICE}" 2>/dev/null || true
    tput cnorm > "${TTY_DEVICE}" 2>/dev/null || true
  fi

  # Clean temporary files
  if [ -n "${TMP_ROOT}" ] && [ -d "${TMP_ROOT}" ]; then
    rm -rf "${TMP_ROOT}"
  fi
}
trap cleanup EXIT

log_info() { printf '%s\n' "$*" >&2; }
log_error() { printf '%s\n' "$*" >&2; }
log_verbose() {
  if [ "${VERBOSE}" -eq 1 ]; then
    printf '%s\n' "$*" >&2
  fi
}
die() { log_error "error: $*"; exit 1; }

backup_target() {
  local target="$1"
  local ts backup counter
  ts="$(date +%Y%m%d%H%M%S)" || ts="$$"
  backup="${target}.bak.${ts}"

  # If backup already exists, add counter to prevent overwriting
  counter=1
  while [ -e "${backup}" ]; do
    backup="${target}.bak.${ts}.${counter}"
    counter=$((counter + 1))
  done

  log_verbose "Backing up ${target} -> ${backup}"
  run_cmd mv "${target}" "${backup}"
}

COLOR_RESET=""
COLOR_BOLD=""
COLOR_DIM=""
COLOR_CYAN=""
COLOR_GREEN=""
COLOR_YELLOW=""
COLOR_RED=""
CAPTION_SHOWN=0

init_colors() {
  if [ -t 2 ] && [ -z "${NO_COLOR-}" ]; then
    COLOR_RESET=$'\033[0m'
    COLOR_BOLD=$'\033[1m'
    COLOR_DIM=$'\033[2m'
    COLOR_CYAN=$'\033[36m'
    COLOR_GREEN=$'\033[32m'
    COLOR_YELLOW=$'\033[33m'
    COLOR_RED=$'\033[31m'
  fi
}

print_divider() { printf '%s\n' "----------------------------------------" >&2; }
print_heading() { printf '\n%b\n' "${COLOR_BOLD}${COLOR_CYAN}$*${COLOR_RESET}" >&2; }

ui_out() { printf '%b' "$*" > "${TTY_DEVICE}"; }

ui_move_up() {
  local count="$1"
  [ "${count}" -gt 0 ] || return 0
  local seq
  seq="$(tput cuu "${count}" 2>/dev/null || true)"
  if [ -z "${seq}" ]; then
    seq="$(printf '\033[%dA' "${count}")"
  fi
  ui_out "${seq}"
}

ui_move_down() {
  local count="${1:-1}"
  [ "${count}" -gt 0 ] || return 0
  local seq
  if [ "${count}" -eq 1 ]; then
    seq="$(tput cud1 2>/dev/null || true)"
    if [ -z "${seq}" ]; then
      seq=$'\033[1B'
    fi
  else
    seq="$(tput cud "${count}" 2>/dev/null || true)"
    if [ -z "${seq}" ]; then
      seq="$(printf '\033[%dB' "${count}")"
    fi
  fi
  ui_out "${seq}"
}

ui_clear_line() {
  local seq
  seq="$(tput el 2>/dev/null || true)"
  if [ -z "${seq}" ]; then
    seq=$'\033[2K'
  fi
  ui_out "${seq}"
}

ui_carriage_return() {
  local seq
  seq="$(tput cr 2>/dev/null || true)"
  if [ -z "${seq}" ]; then
    seq=$'\r'
  fi
  ui_out "${seq}"
}

ui_clear_to_end() {
  local seq
  seq="$(tput ed 2>/dev/null || true)"
  if [ -z "${seq}" ]; then
    seq=$'\033[J'
  fi
  ui_out "${seq}"
}

ui_save_cursor() {
  local seq
  seq="$(tput sc 2>/dev/null || true)"
  if [ -z "${seq}" ]; then
    seq=$'\0337'
  fi
  ui_out "${seq}"
}

ui_restore_cursor() {
  local seq
  seq="$(tput rc 2>/dev/null || true)"
  if [ -z "${seq}" ]; then
    seq=$'\0338'
  fi
  ui_out "${seq}"
}

truncate_label() {
  local label="$1"
  local max="$2"
  if [ "${max}" -le 0 ]; then
    printf '%s' ""
    return
  fi
  local len=${#label}
  if [ "${len}" -le "${max}" ]; then
    printf '%s' "${label}"
    return
  fi
  if [ "${max}" -le 3 ]; then
    printf '%s' "${label:0:${max}}"
    return
  fi
  printf '%s...' "${label:0:$((max - 3))}"
}

menu_label_max_width() {
  local prefix_len="$1"
  local cols
  cols="$(tput cols 2>/dev/null || printf '80')"
  printf '%s' $((cols - prefix_len))
}

menu_single_item_line() {
  local label="$1"
  local is_cursor="$2"
  local pad="${3:-0}"
  local max label_trim
  max="$(menu_label_max_width 3)"
  label_trim="$(truncate_label "${label}" "${max}")"
  if [ "${is_cursor}" -eq 1 ]; then
    printf '%*s %b>%b %b%s%b' "${pad}" "" "${COLOR_YELLOW}" "${COLOR_RESET}" "${COLOR_BOLD}" "${label_trim}" "${COLOR_RESET}"
  else
    printf '%*s   %s' "${pad}" "" "${label_trim}"
  fi
}

menu_single_item_block_lines() {
  printf '%s' 1
}

center_menu_items_single() {
  local -a items=("$@")
  local max_width=0
  local i item_width clean

  # Find the widest menu item (accounting for " > " prefix)
  for i in "${!items[@]}"; do
    clean="$(strip_ansi "${items[$i]}")"
    item_width=$((3 + ${#clean}))
    if [ "${item_width}" -gt "${max_width}" ]; then
      max_width="${item_width}"
    fi
  done

  local cols
  cols="$(tput cols 2>/dev/null || printf '80')"
  local pad=$(( (cols - max_width) / 2 ))
  if [ "${pad}" -lt 0 ]; then
    pad=0
  fi
  printf '%s' "${pad}"
}

center_menu_items_multi() {
  local -a items=("$@")
  local max_width=0
  local i item_width clean

  # Find the widest menu item (accounting for " > [x] " prefix - 6 chars)
  for i in "${!items[@]}"; do
    clean="$(strip_ansi "${items[$i]}")"
    item_width=$((6 + ${#clean}))
    if [ "${item_width}" -gt "${max_width}" ]; then
      max_width="${item_width}"
    fi
  done

  local cols
  cols="$(tput cols 2>/dev/null || printf '80')"
  local pad=$(( (cols - max_width) / 2 ))
  if [ "${pad}" -lt 0 ]; then
    pad=0
  fi
  printf '%s' "${pad}"
}

menu_multi_item_line() {
  local label="$1"
  local is_cursor="$2"
  local is_selected="$3"
  local is_disabled="$4"
  local pad="${5:-0}"
  local marker="[ ]"
  local max label_trim
  max="$(menu_label_max_width 6)"
  label_trim="$(truncate_label "${label}" "${max}")"
  if [ "${is_disabled}" -eq 1 ]; then
    marker="${COLOR_DIM}${marker}${COLOR_RESET}"
    label_trim="${COLOR_DIM}${label_trim}${COLOR_RESET}"
  elif [ "${is_selected}" -eq 1 ]; then
    marker="[x]"
    label_trim="${COLOR_GREEN}${label_trim}${COLOR_RESET}"
  fi
  if [ "${is_cursor}" -eq 1 ]; then
    printf '%*s %b>%b %s %s' "${pad}" "" "${COLOR_YELLOW}" "${COLOR_RESET}" "${marker}" "${label_trim}"
  else
    printf '%*s   %s %s' "${pad}" "" "${marker}" "${label_trim}"
  fi
}

menu_multi_item_block_lines() {
  printf '%s' 1
}

menu_single_item_line_index() {
  local title="$1"
  local instructions="$2"
  local index="$3"
  shift 3
  local -a items=("$@")
  local cols
  local base
  cols="$(tput cols 2>/dev/null || printf '80')"
  base=$(( $(ui_count_lines "${title}" "${cols}") + $(ui_count_lines "${instructions}" "${cols}") + 1 ))
  printf '%s' $((base + index + 1))
}

menu_multi_item_line_index() {
  local title="$1"
  local instructions="$2"
  local footer="$3"
  local index="$4"
  shift 4
  local -a items=("$@")
  local cols
  local base
  cols="$(tput cols 2>/dev/null || printf '80')"
  base=$(( $(ui_count_lines "${title}" "${cols}") + $(ui_count_lines "${instructions}" "${cols}") ))
  if [ -n "${footer}" ]; then
    base=$((base + $(ui_count_lines "${footer}" "${cols}") + 1))
  else
    base=$((base + 1))
  fi
  printf '%s' $((base + index + 1))
}

menu_update_block() {
  local total_lines="$1"
  local line_index="$2"
  local clear_lines="$3"
  local content="$4"
  local up=$((total_lines - line_index + 1))
  ui_save_cursor
  ui_move_up "${up}"
  ui_carriage_return
  local j
  for j in $(seq 1 "${clear_lines}"); do
    ui_clear_line
    if [ "${j}" -lt "${clear_lines}" ]; then
      ui_move_down 1
      ui_carriage_return
    fi
  done
  if [ "${clear_lines}" -gt 1 ]; then
    ui_move_up $((clear_lines - 1))
    ui_carriage_return
  fi
  ui_out "${content}"
  ui_restore_cursor
}

show_caption_only() {
  local caption="  here bro, hold my collection of AI agent configurations for coding "
  if [ -n "${TTY_DEVICE}" ] && [ -w "${TTY_DEVICE}" ] && [ -t 0 ]; then
    local i partial rendered
    for i in $(seq 1 "${#caption}"); do
      partial="${caption:0:${i}}"
      rendered="$(center_line "${COLOR_BOLD}${COLOR_GREEN}${partial}${COLOR_RESET}")"
      ui_carriage_return
      ui_clear_line
      ui_out "${rendered}"
      sleep 0.02
    done
    ui_out "\n"
  else
    printf '%b\n' "$(center_line "${COLOR_BOLD}${COLOR_GREEN}${caption}${COLOR_RESET}")" >&2
  fi
  CAPTION_SHOWN=1
}

show_animals() {
  # Single static ASCII art
  local chosen=$'                                      :                                                                       \n                                     :*=:  .=-                                                                \n                                     -**====--:                                                               \n                                    :====++==*=:                                                              \n                                   :==-::----+=:                                                              \n                                  :===----==+%*:                                                              \n                                 :==--=--=====::                                                              \n                                :=-------====-:.                                                              \n                              .:==-----=-===--:-:.                                                            \n                 .::--============-:---=====-:-=======--::.                                                   \n               :-==-----========-=-------===--==========---==:                                                \n             .-=------------------============------------=----                                               \n             :---------------================-----=-===--------:                                              \n            .------------------=============----===--==--------:                                              \n            :--------==-=----------=-----===--===----==---------                                              \n           :=--------===-===-=---=-=------===-=---=====--=-----=.                                             \n          :===-------====---===----------===------=-===--------=:                                             \n          :===----=-=========------------====--========--=----==-                                             \n          :==---==--===--====-------------=============--=----==:                          .=...             \n          :==----====:.:--======--=================--===--=---==.                         .=********-.       \n          :=--------==. .--====---===========-----=-..=====---=-                         .=+*+******#*.      \n          ---==-==---=-. .=------=======--====----=: .-==-------                         :***++==******.     \n         .===----===-==:  .=====-========-====-=-:=. -=---=-==--                        =%#+++====+*****=.   \n          -=-----=--===:   -====-========-====-=-:=::====-=---=-                        .=***+===+**##***+   \n          :=---=-----==:   :====----=====-==--==-==::===--=---=:                          .=====**##*****#*: \n           :---------=:    :===-=----====-=-=---==. .==----=--=:                           ===-=+*++++**###*= \n            :--------==-   :====-==---===========:   -=-------:                           .========++***##***:\n             .:--------==: :=----=--=-==..------=:.-==------::                            :==========+**#**##-\n               :--------==  -----=----=.  :=-----.==-------::                            .==+=======+*****##*-\n                ::-:::-=+=. .=-------:    .=---=: :==--::-::                             :=+======+****#**##*:\n                 .-:-=---.   :=---==.     .==-=-.  :----:-:                               :+==**+++***%%####= \n                   . ..      .----=:      :=---=.   ..:. .                             .:=++==*#*+++**%%###- \n                              :----.      :----.                                      :==:.=+**+===+**#%#=:  \n                               ---:       .--=                                         :=+++**===+****-.     \n                               .--:.       ---:.                                      *+====.     -**:       \n                                                                                       :--.     .:==+.       \n                                                                                                =++=.        \n                                                                                                 .            \n                                                                                                              \n'

  local caption="  here bro, hold my collection of AI agent configurations for coding "

  if [ -n "${TTY_DEVICE}" ] && [ -w "${TTY_DEVICE}" ] && [ -t 0 ]; then
    local lines cols art_lines pad_lines i min_indent trim_prefix max_width art_pad
    cols="$(tput cols 2>/dev/null || printf '0')"
    lines="$(tput lines 2>/dev/null || printf '0')"
    art_lines="$(printf '%s\n' "${chosen}" | awk 'END {print NR}')"
    min_indent="$(printf '%s\n' "${chosen}" | awk 'NF { match($0, /^[ ]*/); len=RLENGTH; if (min == "" || len < min) min = len } END { print (min == "" ? 0 : min) }')"
    trim_prefix="$(printf '%*s' "${min_indent}" "")"
    max_width="$(printf '%s\n' "${chosen}" | awk -v min="${min_indent}" '{
      line = $0
      if (min > 0) {
        line = substr(line, min + 1)
      }
      if (length(line) > max) {
        max = length(line)
      }
    } END { print (max == "" ? 0 : max) }')"
    art_pad=0
    if [ "${cols}" -gt 0 ] && [ "${max_width}" -gt 0 ] && [ "${cols}" -gt "${max_width}" ]; then
      art_pad=$(( (cols - max_width) / 2 ))
    fi
    pad_lines=0
    if [ "${CAPTION_SHOWN}" -eq 0 ] && [ "${lines}" -gt 0 ]; then
      # total lines: caption + spacer + art
      local total=$(( art_lines + 1 + 1 ))
      if [ "${lines}" -gt "${total}" ]; then
        pad_lines=$(( (lines - total) / 2 ))
      fi
    fi
    for i in $(seq 1 "${pad_lines}"); do
      printf '\n' > "${TTY_DEVICE}"
    done
    {
      if [ "${CAPTION_SHOWN}" -eq 0 ]; then
        printf '%b\n\n' "$(center_line "${COLOR_BOLD}${COLOR_GREEN}${caption}${COLOR_RESET}")"
      fi
      printf '%s\n' "${chosen}" | while IFS= read -r line; do
        if [ "${min_indent}" -gt 0 ]; then
          line="${line#"${trim_prefix}"}"
        fi
        printf '%*s%b\n' "${art_pad}" "" "${COLOR_DIM}${line}${COLOR_RESET}"
      done
    } > "${TTY_DEVICE}"
  else
    local min_indent trim_prefix max_width art_pad
    min_indent="$(printf '%s\n' "${chosen}" | awk 'NF { match($0, /^[ ]*/); len=RLENGTH; if (min == "" || len < min) min = len } END { print (min == "" ? 0 : min) }')"
    trim_prefix="$(printf '%*s' "${min_indent}" "")"
    max_width="$(printf '%s\n' "${chosen}" | awk -v min="${min_indent}" '{
      line = $0
      if (min > 0) {
        line = substr(line, min + 1)
      }
      if (length(line) > max) {
        max = length(line)
      }
    } END { print (max == "" ? 0 : max) }')"
    art_pad=0
    if [ "${max_width}" -gt 0 ]; then
      local fallback_cols
      fallback_cols="$(tput cols 2>/dev/null || printf '0')"
      if [ "${fallback_cols}" -gt 0 ] && [ "${fallback_cols}" -gt "${max_width}" ]; then
        art_pad=$(( (fallback_cols - max_width) / 2 ))
      fi
    fi
    {
      if [ "${CAPTION_SHOWN}" -eq 0 ]; then
        printf '%b\n\n' "$(center_line "${COLOR_BOLD}${COLOR_GREEN}${caption}${COLOR_RESET}")"
      fi
      printf '%s\n' "${chosen}" | while IFS= read -r line; do
        if [ "${min_indent}" -gt 0 ]; then
          line="${line#"${trim_prefix}"}"
        fi
        printf '%*s%b\n' "${art_pad}" "" "${COLOR_DIM}${line}${COLOR_RESET}"
      done
    } >&2
  fi
}

clear_animals() {
  if [ -n "${TTY_DEVICE}" ] && [ -w "${TTY_DEVICE}" ] && [ -t 0 ]; then
    tput clear > "${TTY_DEVICE}" 2>/dev/null || printf '\033c' > "${TTY_DEVICE}"
  fi
}


strip_ansi() {
  printf '%s' "$1" | sed 's/\x1b\[[0-9;]*m//g'
}

center_line() {
  local text="$1"
  local cols clean len pad
  cols="$(tput cols 2>/dev/null || printf '80')"
  clean="$(strip_ansi "${text}")"
  len=${#clean}
  if [ "${len}" -ge "${cols}" ]; then
    printf '%s' "${text}"
    return
  fi
  pad=$(( (cols - len) / 2 ))
  printf '%*s%s' "${pad}" "" "${text}"
}

ui_wrap_count() {
  local text="$1"
  local cols="$2"
  local clean
  clean="$(strip_ansi "${text}")"
  local len=${#clean}
  if [ "${len}" -le 0 ] || [ "${cols}" -le 0 ]; then
    printf '%s' 1
    return
  fi
  printf '%s' $(( (len - 1) / cols + 1 ))
}

ui_count_lines() {
  local text="$1"
  local cols="$2"
  local total=0
  local line

  while IFS= read -r line; do
    total=$((total + $(ui_wrap_count "${line}" "${cols}")))
  done <<< "${text}"

  if [ "${text}" = "" ]; then
    total=1
  fi

  printf '%s' "${total}"
}

ui_print_line() {
  local line="${1-}"
  ui_carriage_return
  ui_clear_line
  ui_out "${line}\n"
}

LAST_UI_BLOCK_LINES=0

clear_last_ui_block() {
  if [ "${LAST_UI_BLOCK_LINES}" -gt 0 ]; then
    ui_move_up "${LAST_UI_BLOCK_LINES}"
    ui_clear_to_end
    LAST_UI_BLOCK_LINES=0
  fi
}

menu_single_lines() {
  local title="$1"
  local instructions="$2"
  shift 2
  local -a items=("$@")
  local cols
  local total=0

  cols="$(tput cols 2>/dev/null || printf '80')"
  total=$((total + $(ui_count_lines "${title}" "${cols}")))
  total=$((total + $(ui_count_lines "${instructions}" "${cols}")))
  total=$((total + 1))
  total=$((total + ${#items[@]}))

  printf '%s' "${total}"
}

menu_multi_lines() {
  local title="$1"
  local instructions="$2"
  local footer="$3"
  shift 3
  local -a items=("$@")
  local cols
  local total=0

  cols="$(tput cols 2>/dev/null || printf '80')"
  total=$((total + $(ui_count_lines "${title}" "${cols}")))
  total=$((total + $(ui_count_lines "${instructions}" "${cols}")))
  if [ -n "${footer}" ]; then
    total=$((total + $(ui_count_lines "${footer}" "${cols}")))
    total=$((total + 1))
  else
    total=$((total + 1))
  fi
  total=$((total + ${#items[@]}))

  printf '%s' "${total}"
}

join_by() {
  local IFS="$1"
  shift
  printf '%s' "$*"
}

read_key() {
  local key
  IFS= read -rsn1 key < "${TTY_DEVICE}" || return 1
  if [ "${key}" = $'\033' ]; then
    local rest
    IFS= read -rsn2 rest < "${TTY_DEVICE}" || true
    case "${rest}" in
      "[A") printf '%s' "up" ;;
      "[B") printf '%s' "down" ;;
      *) printf '%s' "esc" ;;
    esac
    return 0
  fi
  if [ -z "${key}" ]; then
    printf '%s' "enter"
    return 0
  fi
  case "${key}" in
    " ") printf '%s' "space" ;;
    $'\n'|$'\r') printf '%s' "enter" ;;
    *) printf '%s' "${key}" ;;
  esac
}

render_menu_single() {
  local title="$1"
  local instructions="$2"
  local cursor="$3"
  shift 3
  local -a items=("$@")

  ui_out "$(center_line "${COLOR_BOLD}${COLOR_CYAN}${title}${COLOR_RESET}")\n"
  ui_out "$(center_line "${COLOR_DIM}${instructions}${COLOR_RESET}")\n"
  ui_out "\n"

  local pad
  pad="$(center_menu_items_single "${items[@]}")"

  local i
  for i in "${!items[@]}"; do
    if [ "${i}" -eq "${cursor}" ]; then
      ui_out "$(menu_single_item_line "${items[$i]}" 1 "${pad}")\n"
    else
      ui_out "$(menu_single_item_line "${items[$i]}" 0 "${pad}")\n"
    fi
  done
}

render_menu_multi() {
  local title="$1"
  local instructions="$2"
  local cursor="$3"
  local footer="$4"
  shift 4
  local -a items=("$@")

  ui_out "$(center_line "${COLOR_BOLD}${COLOR_CYAN}${title}${COLOR_RESET}")\n"
  ui_out "$(center_line "${COLOR_DIM}${instructions}${COLOR_RESET}")\n"
  if [ -n "${footer}" ]; then
    ui_out "$(center_line "${COLOR_RED}${footer}${COLOR_RESET}")\n\n"
  else
    ui_out "\n"
  fi

  local pad
  pad="$(center_menu_items_multi "${items[@]}")"

  local i
  for i in "${!items[@]}"; do
    local disabled="${DISABLED_FLAGS[$i]:-0}"
    if [ "${i}" -eq "${cursor}" ]; then
      ui_out "$(menu_multi_item_line "${items[$i]}" 1 "${SELECTED_FLAGS[$i]}" "${disabled}" "${pad}")\n"
    else
      ui_out "$(menu_multi_item_line "${items[$i]}" 0 "${SELECTED_FLAGS[$i]}" "${disabled}" "${pad}")\n"
    fi
  done
}

menu_single() {
  local title="$1"
  local instructions="$2"
  local default_index="$3"
  shift 3
  local -a items=("$@")
  local cursor="${default_index}"
  local total="${#items[@]}"
  local lines
  local key=""
  local pad

  SAVED_TTY_STATE="$(stty -g < "${TTY_DEVICE}")"
  stty -echo -icanon time 0 min 1 < "${TTY_DEVICE}"
  tput civis > "${TTY_DEVICE}" 2>/dev/null || true

  pad="$(center_menu_items_single "${items[@]}")"
  lines="$(menu_single_lines "${title}" "${instructions}" "${items[@]}")"
  render_menu_single "${title}" "${instructions}" "${cursor}" "${items[@]}"
  local last_cursor="${cursor}"
  while true; do
    key="$(read_key)"
    case "${key}" in
      up) cursor=$(( (cursor + total - 1) % total )) ;;
      down) cursor=$(( (cursor + 1) % total )) ;;
      enter) break ;;
      *) ;;
    esac
    if [ "${cursor}" -ne "${last_cursor}" ]; then
      local old_line new_line block_lines
      old_line="$(menu_single_item_line "${items[$last_cursor]}" 0 "${pad}")"
      new_line="$(menu_single_item_line "${items[$cursor]}" 1 "${pad}")"
      block_lines="$(menu_single_item_block_lines "${items[$last_cursor]}")"
      menu_update_block "${lines}" "$(menu_single_item_line_index "${title}" "${instructions}" "${last_cursor}" "${items[@]}")" "${block_lines}" "${old_line}"
      block_lines="$(menu_single_item_block_lines "${items[$cursor]}")"
      menu_update_block "${lines}" "$(menu_single_item_line_index "${title}" "${instructions}" "${cursor}" "${items[@]}")" "${block_lines}" "${new_line}"
      last_cursor="${cursor}"
    fi
  done

  ui_move_up "${lines}"
  ui_clear_to_end
  tput cnorm > "${TTY_DEVICE}" 2>/dev/null || true
  stty "${SAVED_TTY_STATE}" < "${TTY_DEVICE}"
  SAVED_TTY_STATE=""
  MENU_RESULT="${items[$cursor]}"
}

menu_multi() {
  local title="$1"
  local instructions="$2"
  local default_all="$3"
  local delimiter="$4"
  local footer="$5"
  shift 5
  local -a items=("$@")
  local total="${#items[@]}"
  local cursor=0
  local key=""
  local i
  local pad

  if [ "${#DISABLED_FLAGS[@]}" -ne "${#items[@]}" ]; then
    DISABLED_FLAGS=()
    for i in "${!items[@]}"; do
      DISABLED_FLAGS[i]="0"
    done
  fi

  SELECTED_FLAGS=()
  for i in "${!items[@]}"; do
    if [ "${default_all}" -eq 1 ] && [ "${DISABLED_FLAGS[$i]}" != "1" ]; then
      SELECTED_FLAGS[i]="1"
    else
      SELECTED_FLAGS[i]="0"
    fi
  done

  SAVED_TTY_STATE="$(stty -g < "${TTY_DEVICE}")"
  stty -echo -icanon time 0 min 1 < "${TTY_DEVICE}"
  tput civis > "${TTY_DEVICE}" 2>/dev/null || true

  pad="$(center_menu_items_multi "${items[@]}")"
  local lines
  lines="$(menu_multi_lines "${title}" "${instructions}" "${footer}" "${items[@]}")"
  render_menu_multi "${title}" "${instructions}" "${cursor}" "${footer}" "${items[@]}"
  local last_cursor="${cursor}"
  while true; do
    key="$(read_key)"
    case "${key}" in
      up) cursor=$(( (cursor + total - 1) % total )) ;;
      down) cursor=$(( (cursor + 1) % total )) ;;
      space)
        if [ "${DISABLED_FLAGS[$cursor]}" != "1" ]; then
          if [ "${SELECTED_FLAGS[$cursor]}" = "1" ]; then
            SELECTED_FLAGS[cursor]="0"
          else
            SELECTED_FLAGS[cursor]="1"
          fi
        fi
        ;;
      a)
        local all_selected=1
        for i in "${!items[@]}"; do
          if [ "${DISABLED_FLAGS[$i]}" = "1" ]; then
            continue
          fi
          if [ "${SELECTED_FLAGS[$i]}" != "1" ]; then
            all_selected=0
            break
          fi
        done
        for i in "${!items[@]}"; do
          if [ "${DISABLED_FLAGS[$i]}" = "1" ]; then
            continue
          fi
          if [ "${all_selected}" -eq 1 ]; then
            SELECTED_FLAGS[i]="0"
          else
            SELECTED_FLAGS[i]="1"
          fi
        done
        ;;
      enter) break ;;
      *) ;;
    esac
    if [ "${key}" = "a" ]; then
      for i in "${!items[@]}"; do
        local line
        local block_lines
        local disabled="${DISABLED_FLAGS[$i]:-0}"
        if [ "${i}" -eq "${cursor}" ]; then
          line="$(menu_multi_item_line "${items[$i]}" 1 "${SELECTED_FLAGS[$i]}" "${disabled}" "${pad}")"
        else
          line="$(menu_multi_item_line "${items[$i]}" 0 "${SELECTED_FLAGS[$i]}" "${disabled}" "${pad}")"
        fi
        block_lines="$(menu_multi_item_block_lines "${items[$i]}")"
        menu_update_block "${lines}" "$(menu_multi_item_line_index "${title}" "${instructions}" "${footer}" "${i}" "${items[@]}")" "${block_lines}" "${line}"
      done
      last_cursor="${cursor}"
      continue
    fi

    if [ "${key}" = "space" ]; then
      local line
      local block_lines
      local disabled="${DISABLED_FLAGS[$cursor]:-0}"
      line="$(menu_multi_item_line "${items[$cursor]}" 1 "${SELECTED_FLAGS[$cursor]}" "${disabled}" "${pad}")"
      block_lines="$(menu_multi_item_block_lines "${items[$cursor]}")"
      menu_update_block "${lines}" "$(menu_multi_item_line_index "${title}" "${instructions}" "${footer}" "${cursor}" "${items[@]}")" "${block_lines}" "${line}"
      last_cursor="${cursor}"
      continue
    fi

    if [ "${cursor}" -ne "${last_cursor}" ]; then
      local old_line new_line block_lines
      local old_disabled="${DISABLED_FLAGS[$last_cursor]:-0}"
      local new_disabled="${DISABLED_FLAGS[$cursor]:-0}"
      old_line="$(menu_multi_item_line "${items[$last_cursor]}" 0 "${SELECTED_FLAGS[$last_cursor]}" "${old_disabled}" "${pad}")"
      new_line="$(menu_multi_item_line "${items[$cursor]}" 1 "${SELECTED_FLAGS[$cursor]}" "${new_disabled}" "${pad}")"
      block_lines="$(menu_multi_item_block_lines "${items[$last_cursor]}")"
      menu_update_block "${lines}" "$(menu_multi_item_line_index "${title}" "${instructions}" "${footer}" "${last_cursor}" "${items[@]}")" "${block_lines}" "${old_line}"
      block_lines="$(menu_multi_item_block_lines "${items[$cursor]}")"
      menu_update_block "${lines}" "$(menu_multi_item_line_index "${title}" "${instructions}" "${footer}" "${cursor}" "${items[@]}")" "${block_lines}" "${new_line}"
      last_cursor="${cursor}"
    fi
  done

  ui_move_up "${lines}"
  ui_clear_to_end
  tput cnorm > "${TTY_DEVICE}" 2>/dev/null || true
  stty "${SAVED_TTY_STATE}" < "${TTY_DEVICE}"
  SAVED_TTY_STATE=""

  local -a selected_items
  selected_items=()
  for i in "${!items[@]}"; do
    if [ "${SELECTED_FLAGS[$i]}" = "1" ]; then
      selected_items+=("${items[$i]}")
    fi
  done

  MENU_RESULT=""
  if [ "${#selected_items[@]}" -gt 0 ]; then
    MENU_RESULT="$(join_by "${delimiter}" "${selected_items[@]}")"
  fi
}

menu_confirm() {
  local title="$1"
  local instructions="$2"
  local default_index="$3"
  local yes_label="$4"
  local no_label="$5"

  if [ "${NON_INTERACTIVE}" -eq 1 ]; then
    return 1
  fi

  ui_clear_to_end
  if [ -z "${yes_label}" ]; then
    yes_label="Yes"
  fi
  if [ -z "${no_label}" ]; then
    no_label="No"
  fi
  if [ -z "${default_index}" ]; then
    default_index=1
  fi

  menu_single "${title}" "${instructions}" "${default_index}" "${yes_label}" "${no_label}"
  if [ "${MENU_RESULT}" = "${yes_label}" ]; then
    return 0
  fi
  return 1
}

usage() {
  cat <<'EOF'
stacc installer

Usage:
  ./install.sh [options]

Options:
  --root PATH          Use PATH as repo root (must contain configs/)
  --cursor             Install to Cursor only
  --claude             Install to Claude Code only
  --opencode           Install to OpenCode only
  --codex              Install to Codex only
  --ampcode            Install to AMP Code only
  --both               Install to both Cursor and Claude Code
  --all                Install to all supported editors
  --global             Install to global locations
  --project            Install to project locations
  --categories LIST    Comma-separated categories (commands,rules,agents,skills,stack,hooks,mcps)
  --stacks LIST        Comma-separated stack skills from configs/stack (or "all")
  --conflict MODE      Conflict mode: overwrite, backup, skip, selective
  --yes                Non-interactive with safe defaults
  --dry-run            Print actions without changing files
  --verbose            Print verbose installation logs
  --help               Show this help

Examples:
  ./install.sh --yes
  ./install.sh --cursor --project --categories commands,rules
  ./install.sh --conflict backup
EOF
}

run_cmd() {
  if [ "${DRY_RUN}" -eq 1 ]; then
    printf '[dry-run] %s\n' "$*"
  else
    "$@"
  fi
}

init_tty() {
  if [ -t 0 ] && [ -w /dev/tty ]; then
    TTY_DEVICE="/dev/tty"
  elif [ -r /dev/tty ] && [ -w /dev/tty ]; then
    TTY_DEVICE="/dev/tty"
  fi
}

prompt_read() {
  local var_name="$1"
  if [ -z "${TTY_DEVICE}" ]; then
    die "non-interactive shell; use --yes or pass options"
  fi
  IFS= read -r "${var_name?}" < "${TTY_DEVICE}"
}

download_repo() {
  command -v git >/dev/null 2>&1 || die "git is required to download stacc"

  TMP_ROOT="$(mktemp -d)"
  log_verbose "Temp path: ${TMP_ROOT}"
  if [ "${VERBOSE}" -eq 1 ]; then
    log_verbose "Cloning stacc repository..."
    run_cmd git clone --depth 1 --branch main "${REPO_URL}" "${TMP_ROOT}/stacc"
  else
    run_cmd git clone --quiet --depth 1 --branch main "${REPO_URL}" "${TMP_ROOT}/stacc"
  fi
  ROOT_DIR="${TMP_ROOT}/stacc"
}

ensure_repo_root() {
  if [ -n "${ROOT_DIR}" ]; then
    [ -d "${ROOT_DIR}/configs" ] || die "configs/ not found in ${ROOT_DIR}"
    return 0
  fi

  download_repo
  [ -d "${ROOT_DIR}/configs" ] || die "downloaded repo missing configs/"
}

parse_args() {
  while [ $# -gt 0 ]; do
    case "$1" in
      --root)
        [ $# -ge 2 ] || die "--root requires a path"
        ROOT_DIR="$2"
        shift 2
        ;;
      --cursor)
        SELECTED_EDITORS="cursor"
        shift
        ;;
      --claude)
        SELECTED_EDITORS="claude"
        shift
        ;;
      --opencode)
        SELECTED_EDITORS="opencode"
        shift
        ;;
      --codex)
        SELECTED_EDITORS="codex"
        shift
        ;;
      --ampcode)
        SELECTED_EDITORS="ampcode"
        shift
        ;;
      --both)
        SELECTED_EDITORS="cursor claude"
        shift
        ;;
      --all)
        SELECTED_EDITORS="cursor claude opencode codex ampcode"
        shift
        ;;
      --global)
        SELECTED_SCOPE="global"
        shift
        ;;
      --project)
        SELECTED_SCOPE="project"
        shift
        ;;
      --categories)
        [ $# -ge 2 ] || die "--categories requires a list"
        SELECTED_CATEGORIES="$2"
        # Strip all whitespace to handle "commands, rules" -> "commands,rules"
        SELECTED_CATEGORIES="${SELECTED_CATEGORIES// /}"
        shift 2
        ;;
      --stacks)
        [ $# -ge 2 ] || die "--stacks requires a list"
        SELECTED_STACKS="$2"
        SELECTED_STACKS="${SELECTED_STACKS// /}"
        shift 2
        ;;
      --conflict)
        [ $# -ge 2 ] || die "--conflict requires a mode"
        CONFLICT_MODE="$2"
        shift 2
        ;;
      --yes)
        NON_INTERACTIVE=1
        shift
        ;;
      --dry-run)
        DRY_RUN=1
        shift
        ;;
      --verbose)
        VERBOSE=1
        shift
        ;;
      --help|-h)
        usage
        exit 0
        ;;
      *)
        die "unknown option: $1"
        ;;
    esac
  done
}

select_editors() {
  if [ -n "${SELECTED_EDITORS}" ]; then
    return 0
  fi

  if [ "${NON_INTERACTIVE}" -eq 1 ]; then
    SELECTED_EDITORS="cursor claude opencode codex ampcode"
    return 0
  fi

  local -a editor_items=("Cursor" "Claude Code" "OpenCode" "Codex" "AMP Code")
  local -a editor_keys=("cursor" "claude" "opencode" "codex" "ampcode")
  local footer=""
  while true; do
    SELECTED_EDITORS=""
    menu_multi "Select editors" "Use ↑/↓ to move, Space to toggle, A for all, Enter to continue." 0 "|" "${footer}" "${editor_items[@]}"
    local i
    for i in "${!editor_items[@]}"; do
      case "${MENU_RESULT}" in
        *"${editor_items[$i]}"*) SELECTED_EDITORS="${SELECTED_EDITORS} ${editor_keys[$i]}" ;;
      esac
    done
    SELECTED_EDITORS="${SELECTED_EDITORS# }"
    if [ -n "${SELECTED_EDITORS}" ]; then
      break
    fi
    footer="Please select at least one editor."
  done
}

select_scope() {
  if [ -n "${SELECTED_SCOPE}" ]; then
    return 0
  fi

  if [ "${NON_INTERACTIVE}" -eq 1 ]; then
    SELECTED_SCOPE="global"
    return 0
  fi

  local -a scope_items=("Global (~/.cursor, ~/.claude, etc.)" "Project (.cursor, .claude, etc. in current directory)")
  menu_single "Select scope" "Use ↑/↓ to move, Enter to select." 0 "${scope_items[@]}"
  case "${MENU_RESULT}" in
    "Global"*) SELECTED_SCOPE="global" ;;
    "Project"*) SELECTED_SCOPE="project" ;;
    *) die "invalid scope selection" ;;
  esac
}

array_contains() {
  local needle="$1"
  shift
  local item
  for item in "$@"; do
    if [ "${item}" = "${needle}" ]; then
      return 0
    fi
  done
  return 1
}

get_supported_categories() {
  local editor="$1"
  local scope="$2"

  case "${editor}" in
    cursor|claude)
      printf '%s' "commands rules agents skills stack hooks mcps"
      ;;
    opencode)
      printf '%s' "commands rules agents skills stack mcps"
      ;;
    codex)
      if [ "${scope}" = "global" ]; then
        printf '%s' "commands rules skills stack mcps"
      else
        printf '%s' "commands rules skills stack"
      fi
      ;;
    ampcode)
      printf '%s' "commands rules skills stack"
      ;;
    *)
      printf '%s' ""
      ;;
  esac
}

is_category_supported() {
  local category="$1"
  local supported="$2"
  local item
  for item in ${supported}; do
    if [ "${item}" = "${category}" ]; then
      return 0
    fi
  done
  return 1
}

is_category_disabled() {
  return 1
}

get_available_categories() {
  local scope="$1"
  local -a base=("commands" "rules" "agents" "skills" "stack" "hooks" "mcps")
  local -a available=()
  local cat editor supported

  for cat in "${base[@]}"; do
    local ok=1
    for editor in ${SELECTED_EDITORS}; do
      supported="$(get_supported_categories "${editor}" "${scope}")"
      if ! is_category_supported "${cat}" "${supported}"; then
        ok=0
        break
      fi
    done
    if [ "${ok}" -eq 1 ]; then
      available+=("${cat}")
    fi
  done

  printf '%s' "${available[*]}"
}

normalize_selected_categories() {
  local scope="$1"
  local -a requested
  local -a available
  local -a filtered=()
  local cat

  IFS=',' read -r -a requested <<< "${SELECTED_CATEGORIES}"
  IFS=' ' read -r -a available <<< "$(get_available_categories "${scope}")"

  for cat in "${requested[@]}"; do
    cat="${cat// /}"
    if [ -z "${cat}" ]; then
      continue
    fi
    if array_contains "${cat}" "${available[@]}"; then
      filtered+=("${cat}")
    else
      log_verbose "Skipping unsupported category: ${cat}"
    fi
  done

  if [ "${#filtered[@]}" -eq 0 ]; then
    die "no supported categories for selected editors and scope"
  fi

  SELECTED_CATEGORIES="$(join_by "," "${filtered[@]}")"
}

select_categories() {
  if [ -n "${SELECTED_CATEGORIES}" ]; then
    normalize_selected_categories "${SELECTED_SCOPE}"
    return 0
  fi

  if [ "${NON_INTERACTIVE}" -eq 1 ]; then
    local -a available enabled
    IFS=' ' read -r -a available <<< "$(get_available_categories "${SELECTED_SCOPE}")"
    enabled=()
    local cat
    for cat in "${available[@]}"; do
      enabled+=("${cat}")
    done
    SELECTED_CATEGORIES="$(join_by "," "${enabled[@]}")"
    return 0
  fi

  local -a category_items=()
  IFS=' ' read -r -a category_items <<< "$(get_available_categories "${SELECTED_SCOPE}")"
  local -a category_display_items=()
  local item first_char rest_chars
  for item in "${category_items[@]}"; do
    first_char="$(printf '%s' "${item}" | cut -c1 | tr '[:lower:]' '[:upper:]')"
    rest_chars="$(printf '%s' "${item}" | cut -c2-)"
    category_display_items+=("${first_char}${rest_chars}")
  done
  local footer=""
  while true; do
    menu_multi "Select categories" "Use ↑/↓ to move, Space to toggle, A for all, Enter to continue." 0 "," "${footer}" "${category_display_items[@]}"
    SELECTED_CATEGORIES="$(printf '%s' "${MENU_RESULT}" | tr '[:upper:]' '[:lower:]')"
    if [ -n "${SELECTED_CATEGORIES}" ]; then
      break
    fi
    footer="Please select at least one category."
  done
}

mcp_category_selected() {
  case ",${SELECTED_CATEGORIES}," in
    *,mcps,*) return 0 ;;
    *) return 1 ;;
  esac
}

extract_mcp_keys_fallback() {
  local src="$1"
  awk '
    BEGIN { in=0; depth=0 }
    /"mcpServers"[[:space:]]*:[[:space:]]*{/ {
      in=1
      depth=1
      next
    }
    in {
      if (depth == 1 && match($0, /"[^"]+"[[:space:]]*:[[:space:]]*{/)) {
        key=substr($0, RSTART + 1, RLENGTH - 3)
        print key
      }
      depth += gsub(/{/, "{")
      depth -= gsub(/}/, "}")
      if (depth <= 0) {
        exit
      }
    }
  ' "${src}"
}

get_mcp_server_keys() {
  local src="${ROOT_DIR}/configs/mcps/mcp.json"
  [ -f "${src}" ] || return 0
  if command -v jq >/dev/null 2>&1; then
    jq -r '.mcpServers | keys[]' "${src}"
    return 0
  fi
  extract_mcp_keys_fallback "${src}"
}

select_mcp_servers() {
  if ! mcp_category_selected; then
    return 0
  fi

  if [ -n "${SELECTED_MCP_SERVERS}" ]; then
    return 0
  fi

  if [ "${NON_INTERACTIVE}" -eq 1 ]; then
    SELECTED_MCP_SERVERS="$(get_mcp_server_keys | paste -sd "," -)"
    return 0
  fi

  local -a server_items=()
  IFS=$'\n' read -r -d '' -a server_items < <(get_mcp_server_keys && printf '\0')
  if [ "${#server_items[@]}" -eq 0 ]; then
    return 0
  fi

  local footer=""
  while true; do
    menu_multi "Select MCP servers" "Use ↑/↓ to move, Space to toggle, A for all, Enter to continue." 0 "," "${footer}" "${server_items[@]}"
    SELECTED_MCP_SERVERS="${MENU_RESULT}"
    if [ -n "${SELECTED_MCP_SERVERS}" ]; then
      break
    fi
    footer="Please select at least one MCP server."
  done
}

get_available_stacks() {
  local stack_dir="${ROOT_DIR}/configs/stacks"
  [ -d "${stack_dir}" ] || return 0
  find "${stack_dir}" -mindepth 1 -maxdepth 1 -type d -print0 \
    | while IFS= read -r -d '' dir; do basename "${dir}"; done \
    | sort || true
}

normalize_selected_stacks() {
  local -a requested
  local -a available
  local -a filtered=()
  local stack

  IFS=',' read -r -a requested <<< "${SELECTED_STACKS}"
  IFS=$'\n' read -r -d '' -a available < <(get_available_stacks && printf '\0')

  for stack in "${requested[@]}"; do
    stack="${stack// /}"
    if [ -z "${stack}" ]; then
      continue
    fi
    if [ "${stack}" = "all" ]; then
      filtered=("${available[@]}")
      break
    fi
    if array_contains "${stack}" "${available[@]}"; then
      filtered+=("${stack}")
    else
      log_verbose "Skipping unknown stack: ${stack}"
    fi
  done

  if [ "${#filtered[@]}" -eq 0 ]; then
    SELECTED_STACKS=""
    return 0
  fi

  SELECTED_STACKS="$(join_by "," "${filtered[@]}")"
}

select_stacks() {
  case ",${SELECTED_CATEGORIES}," in
    *,stack,*) ;;
    *) SELECTED_STACKS=""; return 0 ;;
  esac

  if [ -n "${SELECTED_STACKS}" ]; then
    normalize_selected_stacks
    return 0
  fi

  if [ "${NON_INTERACTIVE}" -eq 1 ]; then
    if [ -n "${SELECTED_STACKS}" ]; then
      normalize_selected_stacks
    else
      SELECTED_STACKS=""
    fi
    return 0
  fi

  local -a stack_items=()
  IFS=$'\n' read -r -d '' -a stack_items < <(get_available_stacks && printf '\0')
  if [ "${#stack_items[@]}" -eq 0 ]; then
    return 0
  fi

  menu_multi "Select stacks" "Use ↑/↓ to move, Space to toggle, A for all, Enter to continue." 0 "," "" "${stack_items[@]}"
  SELECTED_STACKS="${MENU_RESULT}"
}

confirm_summary() {
  if [ "${NON_INTERACTIVE}" -eq 1 ]; then
    return 0
  fi

  if [ -n "${TTY_DEVICE}" ] && [ -w "${TTY_DEVICE}" ] && [ -t 0 ]; then
    local line_title line_editors line_scope line_categories line_mcps line_stacks line_prompt
    local summary_lines
    local cols max pad
    line_title="${COLOR_BOLD}${COLOR_CYAN}Summary${COLOR_RESET}"
    line_editors="  Editors:    ${SELECTED_EDITORS}"
    line_scope="  Scope:      ${SELECTED_SCOPE}"
    line_categories="  Categories: ${SELECTED_CATEGORIES}"
    if mcp_category_selected && [ -n "${SELECTED_MCP_SERVERS}" ]; then
      line_mcps="  MCPs:       ${SELECTED_MCP_SERVERS}"
    else
      line_mcps=""
    fi
    if [ -n "${SELECTED_STACKS}" ]; then
      line_stacks="  Stacks:     ${SELECTED_STACKS}"
    else
      line_stacks=""
    fi
    line_prompt="Proceed? [y/N] "
    cols="$(tput cols 2>/dev/null || printf '80')"
    max=${#line_editors}
    if [ ${#line_scope} -gt "${max}" ]; then
      max=${#line_scope}
    fi
    if [ ${#line_categories} -gt "${max}" ]; then
      max=${#line_categories}
    fi
    if [ -n "${line_mcps}" ] && [ ${#line_mcps} -gt "${max}" ]; then
      max=${#line_mcps}
    fi
    if [ -n "${line_stacks}" ] && [ ${#line_stacks} -gt "${max}" ]; then
      max=${#line_stacks}
    fi
    if [ ${#line_prompt} -gt "${max}" ]; then
      max=${#line_prompt}
    fi
    pad=0
    if [ "${cols}" -gt "${max}" ]; then
      pad=$(( (cols - max) / 2 ))
    fi
    ui_carriage_return
    ui_clear_to_end
    ui_out "\n"
    summary_lines=1
    ui_print_line "$(center_line "${line_title}")"
    summary_lines=$((summary_lines + 1))
    ui_print_line ""
    summary_lines=$((summary_lines + 1))
    ui_print_line "$(printf '%*s%s' "${pad}" "" "${line_editors}")"
    summary_lines=$((summary_lines + 1))
    ui_print_line "$(printf '%*s%s' "${pad}" "" "${line_scope}")"
    summary_lines=$((summary_lines + 1))
    ui_print_line "$(printf '%*s%s' "${pad}" "" "${line_categories}")"
    summary_lines=$((summary_lines + 1))
    if [ -n "${line_mcps}" ]; then
      ui_print_line "$(printf '%*s%s' "${pad}" "" "${line_mcps}")"
      summary_lines=$((summary_lines + 1))
    fi
    if [ -n "${line_stacks}" ]; then
      ui_print_line "$(printf '%*s%s' "${pad}" "" "${line_stacks}")"
      summary_lines=$((summary_lines + 1))
    fi
    summary_lines=$((summary_lines + 1))
    ui_print_line ""
    ui_carriage_return
    ui_clear_line
    ui_out "$(printf '%*s%s' "${pad}" "" "${line_prompt}")"
    summary_lines=$((summary_lines + 1))
    LAST_UI_BLOCK_LINES="${summary_lines}"
  else
    print_heading "Summary"
    log_info "  Editors:    ${SELECTED_EDITORS}"
    log_info "  Scope:      ${SELECTED_SCOPE}"
    log_info "  Categories: ${SELECTED_CATEGORIES}"
    if mcp_category_selected && [ -n "${SELECTED_MCP_SERVERS}" ]; then
      log_info "  MCPs:       ${SELECTED_MCP_SERVERS}"
    fi
    if [ -n "${SELECTED_STACKS}" ]; then
      log_info "  Stacks:     ${SELECTED_STACKS}"
    fi
    log_info ""
    printf "Proceed? [y/N] " > "${TTY_DEVICE}"
  fi
  prompt_read confirm
  clear_last_ui_block
  case "${confirm}" in
    y|Y|yes|YES) ;;
    *) die "aborted" ;;
  esac
}

set_conflict_mode_default() {
  if [ -n "${CONFLICT_MODE}" ]; then
    if [ "${NON_INTERACTIVE}" -eq 1 ] && [ "${CONFLICT_MODE}" = "selective" ]; then
      CONFLICT_MODE="backup"
    fi
    return 0
  fi
  if [ "${NON_INTERACTIVE}" -eq 1 ]; then
    CONFLICT_MODE="backup"
  fi
}

prompt_conflict_mode() {
  if [ "${CONFLICT_MODE}" = "selective" ]; then
    CONFLICT_MODE=""
  fi
  if [ -n "${CONFLICT_MODE}" ] || [ "${NON_INTERACTIVE}" -eq 1 ]; then
    return 0
  fi

  ui_clear_to_end
  menu_single "Conflict detected" "Use ↑/↓ to move, Enter to select." 1 \
    "Overwrite" \
    "Backup existing" \
    "Skip" \
    "Overwrite all" \
    "Backup all" \
    "Skip all"
  case "${MENU_RESULT}" in
    "Overwrite") CONFLICT_MODE="overwrite" ;;
    "Backup existing") CONFLICT_MODE="backup" ;;
    "Skip") CONFLICT_MODE="skip" ;;
    "Overwrite all") CONFLICT_MODE="overwrite_all" ;;
    "Backup all") CONFLICT_MODE="backup_all" ;;
    "Skip all") CONFLICT_MODE="skip_all" ;;
  esac
}

prompt_dir_conflict_mode() {
  if [ "${CONFLICT_MODE}" = "selective" ]; then
    return 0
  fi
  if [ -n "${CONFLICT_MODE}" ] || [ "${NON_INTERACTIVE}" -eq 1 ]; then
    return 0
  fi

  ui_clear_to_end
  menu_single "Category already exists" "Use ↑/↓ to move, Enter to select." 1 \
    "Overwrite category" \
    "Backup existing category" \
    "Skip category" \
    "Selective (install file-by-file)" \
    "Overwrite all" \
    "Backup all" \
    "Skip all"
  case "${MENU_RESULT}" in
    "Overwrite category") CONFLICT_MODE="overwrite" ;;
    "Backup existing category") CONFLICT_MODE="backup" ;;
    "Skip category") CONFLICT_MODE="skip" ;;
    "Selective (install file-by-file)") CONFLICT_MODE="selective" ;;
    "Overwrite all") CONFLICT_MODE="overwrite_all" ;;
    "Backup all") CONFLICT_MODE="backup_all" ;;
    "Skip all") CONFLICT_MODE="skip_all" ;;
  esac
}

apply_conflict_mode() {
  local mode="$1"
  case "${mode}" in
    overwrite_all) echo "overwrite" ;;
    backup_all) echo "backup" ;;
    skip_all) echo "skip" ;;
    selective) echo "selective" ;;
    overwrite|backup|skip) echo "${mode}" ;;
    *) echo "" ;;
  esac
}

handle_conflict() {
  local target="$1"
  prompt_conflict_mode
  local mode
  mode="$(apply_conflict_mode "${CONFLICT_MODE}")"

  if [ -z "${mode}" ]; then
    die "conflict mode not set for ${target}"
  fi

  if [ "${mode}" = "skip" ]; then
    log_verbose "Skipping ${target}"
    return 1
  fi

  if [ "${mode}" = "backup" ]; then
    backup_target "${target}"
  fi

  return 0
}

handle_dir_conflict() {
  local target="$1"
  local mode

  prompt_dir_conflict_mode
  mode="$(apply_conflict_mode "${CONFLICT_MODE}")"

  if [ -z "${mode}" ]; then
    die "conflict mode not set for ${target}"
  fi

  if [ "${mode}" = "selective" ]; then
    return 0
  fi

  if [ "${mode}" = "skip" ]; then
    log_verbose "Skipping ${target}"
    return 1
  fi

  if [ "${mode}" = "backup" ]; then
    backup_target "${target}"
    return 0
  fi

  if [ "${mode}" = "overwrite" ]; then
    log_verbose "Removing existing ${target}"
    run_cmd rm -rf "${target}"
    return 0
  fi

  return 0
}

copy_file() {
  local src="$1"
  local dest="$2"

  mkdir -p "$(dirname "${dest}")"

  if [ -e "${dest}" ]; then
    if ! handle_conflict "${dest}"; then
      return 0
    fi
  fi

  log_verbose "Installing ${dest}"
  run_cmd cp "${src}" "${dest}"
}

copy_tree() {
  local src_dir="$1"
  local dest_dir="$2"

  [ -d "${src_dir}" ] || die "missing source directory: ${src_dir}"

  while IFS= read -r -d '' file; do
    local rel="${file#"${src_dir}/"}"
    local dest="${dest_dir}/${rel}"
    copy_file "${file}" "${dest}"
  done < <(find "${src_dir}" -type f ! -name ".DS_Store" -print0)
}

rules_summary_path() {
  printf '%s\n' "${ROOT_DIR}/configs/rules/summary.md"
}

rules_summary_marker() {
  printf '%s\n' "<!-- stacc:rules-summary -->"
}

append_rules_summary() {
  local target="$1"
  local summary marker
  summary="$(rules_summary_path)"
  marker="$(rules_summary_marker)"

  if [ ! -f "${summary}" ]; then
    log_info "Rules summary not found at ${summary}; skipping summary append."
    return 0
  fi

  if [ -f "${target}" ] && rg -q "${marker}" "${target}"; then
    log_verbose "Rules summary already present in ${target}"
    return 0
  fi

  mkdir -p "$(dirname "${target}")"

  if [ "${DRY_RUN}" -eq 1 ]; then
    log_verbose "[dry-run] Would append rules summary to ${target}"
    return 0
  fi

  if [ -f "${target}" ]; then
    printf '\n\n' >> "${target}"
    cat "${summary}" >> "${target}"
  else
    cat "${summary}" > "${target}"
  fi
}

rules_summary_target_for() {
  local editor="$1"
  local scope="$2"
  local target_root="$3"

  if [ "${editor}" = "claude" ]; then
    if [ "${scope}" = "project" ]; then
      local claude_file="${PROJECT_ROOT}/CLAUDE.md"
      if [ -f "${claude_file}" ]; then
        local line_count
        line_count="$(wc -l < "${claude_file}" | tr -d ' ')"
        if [ "${line_count}" -gt 3 ]; then
          printf '%s\n' "${claude_file}"
          return 0
        fi
      fi
      printf '%s\n' "${PROJECT_ROOT}/AGENTS.md"
      return 0
    fi
    printf '%s\n' "${target_root}/CLAUDE.md"
    return 0
  fi

  if [ "${scope}" = "project" ]; then
    printf '%s\n' "${PROJECT_ROOT}/AGENTS.md"
  else
    printf '%s\n' "${target_root}/AGENTS.md"
  fi
}

install_cursor_rules() {
  local src="$1"
  local dest="$2"

  if [ -d "${dest}" ] && [ -n "$(find "${dest}" -mindepth 1 -print -quit)" ]; then
    if ! handle_dir_conflict "${dest}"; then
      return 0
    fi
  fi

  while IFS= read -r -d '' file; do
    local rel="${file#"${src}/"}"
    local dest_file="${dest}/${rel}"
    copy_file "${file}" "${dest_file}"
  done < <(find "${src}" -type f ! -name ".DS_Store" ! -name "summary.md" -print0)
}

install_codex_rules() {
  local src="$1"
  local dest="$2"

  if [ -d "${dest}" ] && [ -n "$(find "${dest}" -mindepth 1 -print -quit)" ]; then
    if ! handle_dir_conflict "${dest}"; then
      return 0
    fi
  fi

  mkdir -p "${dest}"

  while IFS= read -r -d '' file; do
    local base
    base="$(basename "${file}")"
    base="${base%.*}.rules"
    copy_file "${file}" "${dest}/${base}"
  done < <(find "${src}" -type f ! -name ".DS_Store" ! -name "summary.md" -print0)
}

install_rules() {
  local editor="$1"
  local scope="$2"
  local target_root="$3"
  local src="${ROOT_DIR}/configs/rules"
  local should_append=1

  [ -d "${src}" ] || die "source category not found: ${src}"

  if [ "${editor}" = "cursor" ]; then
    install_cursor_rules "${src}" "${target_root}/rules"
    should_append=0
  elif [ "${editor}" = "codex" ]; then
    if [ "${scope}" = "global" ]; then
      install_codex_rules "${src}" "${target_root}/rules"
    fi
    should_append=0
  fi

  local summary_target
  if [ "${should_append}" -eq 1 ]; then
    summary_target="$(rules_summary_target_for "${editor}" "${scope}" "${target_root}")"
    append_rules_summary "${summary_target}"
  fi
}

install_category() {
  local category="$1"
  local target_root="$2"
  local dest_subdir="${3:-${category}}"
  local src="${ROOT_DIR}/configs/${category}"
  local dest="${target_root}/${dest_subdir}"

  [ -d "${src}" ] || die "source category not found: ${src}"
  if [ -d "${dest}" ] && [ -n "$(find "${dest}" -mindepth 1 -print -quit)" ]; then
    if ! handle_dir_conflict "${dest}"; then
      return 0
    fi
  fi
  copy_tree "${src}" "${dest}"
}

category_dest_for() {
  local editor="$1"
  local category="$2"

  case "${editor}:${category}" in
    *)
      printf '%s' "${category}"
      ;;
  esac
}

skills_root_for() {
  local editor="$1"
  local scope="$2"
  local target_root="$3"

  case "${editor}" in
    ampcode)
      if [ "${scope}" = "global" ]; then
        printf '%s\n' "${HOME}/.config/agents/skills"
      else
        printf '%s\n' "${target_root}/skills"
      fi
      ;;
    *)
      printf '%s\n' "${target_root}/skills"
      ;;
  esac
}

install_skills() {
  local editor="$1"
  local scope="$2"
  local target_root="$3"
  local src="${ROOT_DIR}/configs/skills"
  local dest
  dest="$(skills_root_for "${editor}" "${scope}" "${target_root}")"

  [ -d "${src}" ] || die "source category not found: ${src}"
  if [ -d "${dest}" ] && [ -n "$(find "${dest}" -mindepth 1 -print -quit)" ]; then
    # Commands and stacks share the skills root for some editors; avoid wiping the whole dir.
    if [ "${editor}" != "claude" ] && [ "${editor}" != "codex" ] && [ "${editor}" != "ampcode" ]; then
      if ! handle_dir_conflict "${dest}"; then
        return 0
      fi
    else
      log_verbose "Shared skills root detected; resolving conflicts per-file."
    fi
  fi
  copy_tree "${src}" "${dest}"
}

install_commands() {
  local editor="$1"
  local scope="$2"
  local target_root="$3"
  local src
  local dest

  case "${editor}" in
    codex|claude|ampcode)
      src="${ROOT_DIR}/configs/commands/skills"
      dest="$(skills_root_for "${editor}" "${scope}" "${target_root}")"
      ;;
    *)
      src="${ROOT_DIR}/configs/commands"
      dest="${target_root}/commands"
      ;;
  esac

  [ -d "${src}" ] || die "source category not found: ${src}"
  if [ -d "${dest}" ] && [ -n "$(find "${dest}" -mindepth 1 -print -quit)" ]; then
    # For claude/codex/ampcode, commands live inside the shared skills root.
    if [ "${editor}" != "claude" ] && [ "${editor}" != "codex" ] && [ "${editor}" != "ampcode" ]; then
      if ! handle_dir_conflict "${dest}"; then
        return 0
      fi
    else
      log_verbose "Shared skills root detected; resolving conflicts per-file."
    fi
  fi
  copy_tree "${src}" "${dest}"
}

install_stack_skill() {
  local stack="$1"
  local editor="$2"
  local scope="$3"
  local target_root="$4"
  local src="${ROOT_DIR}/configs/stacks/${stack}"
  local dest_root
  local dest

  dest_root="$(skills_root_for "${editor}" "${scope}" "${target_root}")"
  dest="${dest_root}/${stack}"

  [ -d "${src}" ] || die "source stack not found: ${src}"
  if [ -d "${dest}" ] && [ -n "$(find "${dest}" -mindepth 1 -print -quit)" ]; then
    if ! handle_dir_conflict "${dest}"; then
      return 0
    fi
  fi
  copy_tree "${src}" "${dest}"
}

write_mcp_subset_fallback() {
  local src="$1"
  local dest="$2"
  local selected="$3"
  awk -v selected="${selected}" '
    BEGIN {
      split(selected, keys, ",")
      for (i in keys) {
        if (keys[i] != "") {
          wanted[keys[i]] = 1
        }
      }
      print "{"
      print "  \"mcpServers\": {"
      first=1
    }
    /"mcpServers"[[:space:]]*:[[:space:]]*{/ {
      inside=1
      depth=1
      next
    }
    inside {
      if (depth == 1 && match($0, /"[^"]+"[[:space:]]*:[[:space:]]*{/)) {
        key=substr($0, RSTART + 1, RLENGTH - 3)
        keep=(key in wanted)
        if (keep && !first) {
          print ","
        }
        if (keep) {
          first=0
          print "    \"" key "\": {"
        }
        depth += gsub(/{/, "{")
        depth -= gsub(/}/, "}")
        next
      }
      if (keep) {
        line=$0
        sub(/^[[:space:]]*/, "", line)
        print "      " line
      }
      depth += gsub(/{/, "{")
      depth -= gsub(/}/, "}")
      if (depth == 1) {
        keep=0
      }
      if (depth <= 0) {
        inside=0
      }
    }
    END {
      print "  }"
      print "}"
    }
  ' "${src}" > "${dest}"
}

build_selected_mcp_source() {
  local src="$1"
  local selected="${SELECTED_MCP_SERVERS}"

  if [ -z "${selected}" ]; then
    printf '%s' "${src}"
    return 0
  fi

  if [ -z "${TMP_ROOT}" ]; then
    TMP_ROOT="$(mktemp -d)"
  fi

  local tmp="${TMP_ROOT}/mcp_selected.$$"
  if command -v jq >/dev/null 2>&1; then
    local keys_json
    keys_json="$(printf '%s' "${selected}" | awk -F',' '{
      printf "["
      for (i = 1; i <= NF; i++) {
        if ($i != "") {
          if (i > 1) printf ","
          printf "\"" $i "\""
        }
      }
      printf "]"
    }')"
    jq --argjson keys "${keys_json}" '{mcpServers: (.mcpServers | with_entries(select(.key as $k | $keys | index($k))))}' "${src}" > "${tmp}"
  else
    write_mcp_subset_fallback "${src}" "${tmp}" "${selected}"
  fi

  printf '%s' "${tmp}"
}

wrap_amp_mcp_source() {
  local src="$1"
  local dest="$2"
  if command -v jq >/dev/null 2>&1; then
    jq '{amp: {mcpServers: .mcpServers}}' "${src}" > "${dest}"
    return 0
  fi

  {
    printf '{\n  "amp": {\n'
    sed '1s/^[[:space:]]*{//; $s/}[[:space:]]*$//' "${src}"
    printf '\n  }\n}\n'
  } > "${dest}"
}

build_codex_mcp_block() {
  local src="$1"
  local keys_csv="${2-}"
  if ! command -v jq >/dev/null 2>&1; then
    build_codex_mcp_block_fallback "${src}" "${keys_csv}"
    return 0
  fi
  if [ -n "${keys_csv}" ]; then
    jq -r --argjson keys "$(printf '%s' "${keys_csv}" | awk -F',' '{
      printf "["
      for (i = 1; i <= NF; i++) {
        if ($i != "") {
          if (i > 1) printf ","
          printf "\"" $i "\""
        }
      }
      printf "]"
    }')" '
      .mcpServers
      | with_entries(select(.key as $k | $keys | index($k)))
      | to_entries[]
      | .key as $name
      | (["[mcp_servers." + $name + "]"]
         + (if .value.command then ["command = " + (.value.command | @json)] else [] end)
         + (if .value.args then ["args = [" + (.value.args | map(@json) | join(", ")) + "]"] else [] end)
         + (if .value.type then ["type = " + (.value.type | @json)] else [] end)
         + (if .value.url then ["url = " + (.value.url | @json)] else [] end)
        )
      | . + [""]
      | .[]
    ' "${src}"
  else
    jq -r '
      .mcpServers
      | to_entries[]
      | .key as $name
      | (["[mcp_servers." + $name + "]"]
         + (if .value.command then ["command = " + (.value.command | @json)] else [] end)
         + (if .value.args then ["args = [" + (.value.args | map(@json) | join(", ")) + "]"] else [] end)
         + (if .value.type then ["type = " + (.value.type | @json)] else [] end)
         + (if .value.url then ["url = " + (.value.url | @json)] else [] end)
        )
      | . + [""]
      | .[]
    ' "${src}"
  fi
}

merge_codex_mcp() {
  local src="$1"
  local dest="$2"

  if ! command -v jq >/dev/null 2>&1; then
    return 1
  fi

  if [ "${DRY_RUN}" -eq 1 ]; then
    log_verbose "[dry-run] Would merge MCP config ${src} into ${dest}"
    return 0
  fi

  if [ -z "${TMP_ROOT}" ]; then
    TMP_ROOT="$(mktemp -d)"
  fi

  local tmp="${TMP_ROOT}/mcp_codex.$$"
  local -a selected_keys=()
  local key
  while IFS= read -r key; do
    [ -n "${key}" ] || continue
    selected_keys+=("${key}")
  done < <(jq -r '.mcpServers | keys[]' "${src}")

  local existing_keys_csv=""
  if [ -e "${dest}" ]; then
    existing_keys_csv="$(awk '
      /^\[mcp_servers\./ {
        line=$0
        sub(/^\[mcp_servers\./, "", line)
        sub(/\]$/, "", line)
        print line
      }
    ' "${dest}" | paste -sd "," -)"
  fi

  local -a keys_to_add=()
  local -a keys_to_remove=()
  for key in "${selected_keys[@]}"; do
    if [ -n "${existing_keys_csv}" ] && printf '%s\n' "${existing_keys_csv}" | tr ',' '\n' | grep -qx "${key}"; then
      if [ "${NON_INTERACTIVE}" -eq 1 ]; then
        log_verbose "Skipping existing Codex MCP server ${key}"
        continue
      fi
      if menu_confirm "MCP server '${key}' exists" "Overwrite in ${dest}? Use ↑/↓ to move, Enter to select." 1 "Overwrite" "Skip"; then
        keys_to_remove+=("${key}")
        keys_to_add+=("${key}")
      fi
    else
      keys_to_add+=("${key}")
    fi
  done

  if [ -e "${dest}" ]; then
    if [ "${#keys_to_remove[@]}" -gt 0 ]; then
      local remove_csv
      remove_csv="$(IFS=','; printf '%s' "${keys_to_remove[*]}")"
      awk -v remove_csv="${remove_csv}" '
        BEGIN {
          split(remove_csv, arr, ",")
          for (i in arr) {
            if (arr[i] != "") {
              remove[arr[i]] = 1
            }
          }
          skip=0
        }
        {
          if ($0 ~ /^\[mcp_servers\./) {
            key=$0
            sub(/^\[mcp_servers\./, "", key)
            sub(/\]$/, "", key)
            if (remove[key]) {
              skip=1
              next
            }
            skip=0
          }
          if (!skip) print
        }
      ' "${dest}" > "${tmp}"
    else
      cp "${dest}" "${tmp}"
    fi
  else
    : > "${tmp}"
  fi

  if [ "${#keys_to_add[@]}" -eq 0 ]; then
    log_verbose "No new Codex MCP servers to add."
    rm -f "${tmp}"
    return 0
  fi

  printf '\n' >> "${tmp}"
  build_codex_mcp_block "${src}" "$(IFS=','; printf '%s' "${keys_to_add[*]}")" >> "${tmp}"
  log_verbose "Writing merged Codex MCP config to ${dest}"
  mv "${tmp}" "${dest}"
  return 0
}
merge_mcp() {
  local src="$1"
  local dest="$2"

  if command -v jq >/dev/null 2>&1; then
    # Handle dry-run early to avoid creating temp files
    if [ "${DRY_RUN}" -eq 1 ]; then
      log_verbose "[dry-run] Would merge MCP config ${src} into ${dest}"
      return 0
    fi

    if [ "${NON_INTERACTIVE}" -eq 0 ]; then
      if ! menu_confirm "Merge MCP config?" "Target: ${dest}. Use ↑/↓ to move, Enter to select." 1 "Merge" "Skip"; then
        return 1
      fi
    fi

    # Ensure we have a temp directory for cleanup
    if [ -z "${TMP_ROOT}" ]; then
      TMP_ROOT="$(mktemp -d)"
    fi

    local tmp
    tmp="${TMP_ROOT}/mcp_merge.$$"
    jq -s '.[0] * .[1]' "${dest}" "${src}" > "${tmp}" || {
      rm -f "${tmp}"
      return 1
    }
    if [ -e "${dest}" ]; then
      handle_conflict "${dest}" || { rm -f "${tmp}"; return 0; }
    fi
    log_verbose "Writing merged MCP config to ${dest}"
    mv "${tmp}" "${dest}"
    return 0
  fi

  return 1
}

install_mcp() {
  local editor="$1"
  local scope="$2"
  local target_root="$3"
  local dest="$4"
  local src="${ROOT_DIR}/configs/mcps/mcp.json"

  [ -f "${src}" ] || die "missing MCP config: ${src}"

  local selected_src
  selected_src="$(build_selected_mcp_source "${src}")"

  if [ "${editor}" = "ampcode" ]; then
    if [ -z "${TMP_ROOT}" ]; then
      TMP_ROOT="$(mktemp -d)"
    fi
    local wrapped_src="${TMP_ROOT}/mcp_amp.$$"
    wrap_amp_mcp_source "${selected_src}" "${wrapped_src}"
    selected_src="${wrapped_src}"
  fi

  mkdir -p "$(dirname "${dest}")"

  if [ "${editor}" = "codex" ]; then
    if [ -e "${dest}" ] && merge_codex_mcp "${selected_src}" "${dest}"; then
      return 0
    fi
    if [ "${DRY_RUN}" -eq 1 ]; then
      log_verbose "[dry-run] Would write Codex MCP config to ${dest}"
      return 0
    fi
    if [ -e "${dest}" ]; then
      handle_conflict "${dest}" || return 0
    fi
    if ! build_codex_mcp_block "${selected_src}" > "${dest}"; then
      die "unable to build Codex MCP config; jq is required"
    fi
    return 0
  fi

  if [ -e "${dest}" ] && merge_mcp "${selected_src}" "${dest}"; then
    return 0
  fi

  copy_file "${selected_src}" "${dest}"
}

target_root_for() {
  local editor="$1"
  local scope="$2"

  case "${editor}" in
    cursor)
      if [ "${scope}" = "global" ]; then
        printf '%s\n' "${HOME}/.cursor"
      else
        printf '%s\n' "${PROJECT_ROOT}/.cursor"
      fi
      ;;
    claude)
      if [ "${scope}" = "global" ]; then
        printf '%s\n' "${HOME}/.claude"
      else
        printf '%s\n' "${PROJECT_ROOT}/.claude"
      fi
      ;;
    opencode)
      if [ "${scope}" = "global" ]; then
        printf '%s\n' "${HOME}/.config/opencode"
      else
        printf '%s\n' "${PROJECT_ROOT}/.opencode"
      fi
      ;;
    codex)
      if [ "${scope}" = "global" ]; then
        printf '%s\n' "${HOME}/.codex"
      else
        printf '%s\n' "${PROJECT_ROOT}/.codex"
      fi
      ;;
    ampcode)
      if [ "${scope}" = "global" ]; then
        printf '%s\n' "${HOME}/.config/amp"
      else
        printf '%s\n' "${PROJECT_ROOT}/.agents"
      fi
      ;;
    *)
      die "unknown editor: ${editor}"
      ;;
  esac
}

build_codex_mcp_block_fallback() {
  local src="$1"
  local keys_csv="${2-}"
  awk -v keys_csv="${keys_csv}" '
    BEGIN {
      have_filter = (keys_csv != "")
      if (have_filter) {
        n = split(keys_csv, tmp, ",")
        for (i = 1; i <= n; i++) {
          if (tmp[i] != "") {
            keep[tmp[i]] = 1
          }
        }
      }
      current = ""
      skip = 0
      in_args = 0
      first_block = 1
      args_count = 0
    }
    function toml_escape(s) {
      gsub(/\\/, "\\\\", s)
      gsub(/"/, "\\\"", s)
      return s
    }
    function flush_args() {
      if (skip || !in_args) return
      printf "args = ["
      for (i = 1; i <= args_count; i++) {
        if (i > 1) printf ", "
        printf "\"" toml_escape(args[i]) "\""
      }
      printf "]\n"
      in_args = 0
      args_count = 0
    }
    function start_block(name) {
      if (skip) return
      if (!first_block) printf "\n"
      first_block = 0
      printf "[mcp_servers.%s]\n", name
    }
    /^[[:space:]]*"mcpServers"[[:space:]]*:/ {
      in_servers = 1
    }
    in_servers && match($0, /^[[:space:]]*"([^"]+)"[[:space:]]*:[[:space:]]*{/, m) {
      current = m[1]
      skip = (have_filter && !keep[current])
      start_block(current)
      next
    }
    current != "" {
      if (match($0, /^[[:space:]]*"command"[[:space:]]*:[[:space:]]*"([^"]*)"/, m)) {
        if (!skip) printf "command = \"%s\"\n", toml_escape(m[1])
        next
      }
      if (match($0, /^[[:space:]]*"type"[[:space:]]*:[[:space:]]*"([^"]*)"/, m)) {
        if (!skip) printf "type = \"%s\"\n", toml_escape(m[1])
        next
      }
      if (match($0, /^[[:space:]]*"url"[[:space:]]*:[[:space:]]*"([^"]*)"/, m)) {
        if (!skip) printf "url = \"%s\"\n", toml_escape(m[1])
        next
      }
      if (match($0, /^[[:space:]]*"args"[[:space:]]*:[[:space:]]*\[/)) {
        in_args = 1
        args_count = 0
        if (match($0, /\[[^]]*]/)) {
          tmp = $0
          while (match(tmp, /"([^"]*)"/, m)) {
            args[++args_count] = m[1]
            tmp = substr(tmp, RSTART + RLENGTH)
          }
          flush_args()
        }
        next
      }
      if (in_args) {
        tmp = $0
        while (match(tmp, /"([^"]*)"/, m)) {
          args[++args_count] = m[1]
          tmp = substr(tmp, RSTART + RLENGTH)
        }
        if (index($0, "]")) {
          flush_args()
        }
        next
      }
      if ($0 ~ /^[[:space:]]*}[,]?[[:space:]]*$/) {
        current = ""
        skip = 0
        next
      }
    }
  ' "${src}"
}

mcp_path_for() {
  local editor="$1"
  local scope="$2"
  local target_root="$3"

  case "${editor}" in
    claude)
      if [ "${scope}" = "global" ]; then
        printf '%s\n' "${HOME}/.claude.json"
      else
        printf '%s\n' "${PROJECT_ROOT}/.mcp.json"
      fi
      ;;
    cursor)
      printf '%s\n' "${target_root}/mcp.json"
      ;;
    opencode)
      if [ "${scope}" = "global" ]; then
        printf '%s\n' "${target_root}/.opencode.json"
      else
        printf '%s\n' "${PROJECT_ROOT}/.opencode.json"
      fi
      ;;
    codex)
      printf '%s\n' "${target_root}/config.toml"
      ;;
    ampcode)
      printf '%s\n' "${target_root}/settings.json"
      ;;
    *)
      die "unknown editor: ${editor}"
      ;;
  esac
}

validate_category() {
  local cat="$1"
  case "${cat}" in
    commands|rules|agents|skills|stack|hooks|mcps) return 0 ;;
    *) die "invalid category: ${cat}" ;;
  esac
}

install_for_target() {
  local editor="$1"
  local scope="$2"
  local target_root
  target_root="$(target_root_for "${editor}" "${scope}")"

  mkdir -p "${target_root}"

  log_info ""
  log_info "Installing for ${editor} (${scope})..."
  local category
  IFS=',' read -r -a cats <<< "${SELECTED_CATEGORIES}"
  for category in "${cats[@]}"; do
    validate_category "${category}"
    if [ "${category}" = "mcps" ]; then
      install_mcp "${editor}" "${scope}" "${target_root}" "$(mcp_path_for "${editor}" "${scope}" "${target_root}")"
    elif [ "${category}" = "rules" ]; then
      install_rules "${editor}" "${scope}" "${target_root}"
    elif [ "${category}" = "skills" ]; then
      install_skills "${editor}" "${scope}" "${target_root}"
    elif [ "${category}" = "commands" ]; then
      install_commands "${editor}" "${scope}" "${target_root}"
    elif [ "${category}" = "stack" ]; then
      if [ -n "${SELECTED_STACKS}" ]; then
        local stack
        IFS=',' read -r -a stacks <<< "${SELECTED_STACKS}"
        for stack in "${stacks[@]}"; do
          [ -n "${stack}" ] || continue
          install_stack_skill "${stack}" "${editor}" "${scope}" "${target_root}"
        done
      fi
    else
      install_category "${category}" "${target_root}" "$(category_dest_for "${editor}" "${category}")"
    fi
  done
}

main() {
  parse_args "$@"
  init_tty
  init_colors
  if [ "${NON_INTERACTIVE}" -eq 0 ] && [ -z "${TTY_DEVICE}" ]; then
    NON_INTERACTIVE=1
  fi
  if [ "${NON_INTERACTIVE}" -eq 0 ]; then
    show_caption_only
  fi
  ensure_repo_root
  if [ "${NON_INTERACTIVE}" -eq 0 ]; then
    show_animals
  fi

  select_editors
  select_scope
  select_categories
  select_mcp_servers
  select_stacks
  set_conflict_mode_default
  confirm_summary

  local editor
  local saved_ifs="${IFS}"
  IFS=' '
  for editor in ${SELECTED_EDITORS}; do
    IFS="${saved_ifs}"
    install_for_target "${editor}" "${SELECTED_SCOPE}"
  done
  IFS="${saved_ifs}"

  log_info "Done."
}

main "$@"
