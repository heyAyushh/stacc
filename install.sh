#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

SCRIPT_REF="${BASH_SOURCE[0]-$0}"
SCRIPT_DIR="$(cd "$(dirname "${SCRIPT_REF}")" && pwd)"
ROOT_DIR=""
PROJECT_ROOT="$(pwd)"
TMP_ROOT=""
TTY_DEVICE=""
REPO_URL="https://github.com/heyAyushh/stacc.git"

NON_INTERACTIVE=0
DRY_RUN=0
VERBOSE=0
SELECTED_EDITORS=""
SELECTED_SCOPE=""
SELECTED_CATEGORIES=""
CONFLICT_MODE=""

cleanup() {
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

COLORS_ENABLED=0
COLOR_RESET=""
COLOR_BOLD=""
COLOR_DIM=""
COLOR_CYAN=""
COLOR_GREEN=""
COLOR_YELLOW=""
COLOR_RED=""

init_colors() {
  if [ -t 2 ] && [ -z "${NO_COLOR-}" ]; then
    COLORS_ENABLED=1
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

strip_ansi() {
  printf '%s' "$1" | sed 's/\x1b\[[0-9;]*m//g'
}

ui_wrap_count() {
  local text="$1"
  local cols="$2"
  local clean
  clean="$(strip_ansi "${text}")"
  local len=${#clean}
  if [ "${len}" -le 0 ]; then
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

menu_single_lines() {
  local title="$1"
  local instructions="$2"
  shift 2
  local -a items=("$@")
  local cols
  local total=0
  local i

  cols="$(tput cols 2>/dev/null || printf '80')"
  total=$((total + $(ui_count_lines "${title}" "${cols}")))
  total=$((total + $(ui_count_lines "${instructions}" "${cols}")))
  total=$((total + 1))
  for i in "${!items[@]}"; do
    total=$((total + $(ui_count_lines "   ${items[$i]}" "${cols}")))
  done

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
  local i

  cols="$(tput cols 2>/dev/null || printf '80')"
  total=$((total + $(ui_count_lines "${title}" "${cols}")))
  total=$((total + $(ui_count_lines "${instructions}" "${cols}")))
  if [ -n "${footer}" ]; then
    total=$((total + $(ui_count_lines "${footer}" "${cols}")))
    total=$((total + 1))
  else
    total=$((total + 1))
  fi
  for i in "${!items[@]}"; do
    total=$((total + $(ui_count_lines "   [ ] ${items[$i]}" "${cols}")))
  done

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

  ui_out "${COLOR_BOLD}${COLOR_CYAN}${title}${COLOR_RESET}\n"
  ui_out "${COLOR_DIM}${instructions}${COLOR_RESET}\n"
  local i
  for i in "${!items[@]}"; do
    if [ "${i}" -eq "${cursor}" ]; then
      ui_out " ${COLOR_YELLOW}>${COLOR_RESET} ${COLOR_BOLD}${items[$i]}${COLOR_RESET}\n"
    else
      ui_out "   ${items[$i]}\n"
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
  local -a selected=("${SELECTED_FLAGS[@]}")

  ui_out "${COLOR_BOLD}${COLOR_CYAN}${title}${COLOR_RESET}\n"
  ui_out "${COLOR_DIM}${instructions}${COLOR_RESET}\n"
  if [ -n "${footer}" ]; then
    ui_out "${COLOR_RED}${footer}${COLOR_RESET}\n\n"
  else
    ui_out "\n"
  fi
  local i
  for i in "${!items[@]}"; do
    local marker="[ ]"
    local label="${items[$i]}"
    if [ "${selected[$i]}" = "1" ]; then
      marker="[x]"
      label="${COLOR_GREEN}${label}${COLOR_RESET}"
    fi
    if [ "${i}" -eq "${cursor}" ]; then
      ui_out " ${COLOR_YELLOW}>${COLOR_RESET} ${marker} ${label}\n"
    else
      ui_out "   ${marker} ${label}\n"
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
  local stty_state

  stty_state="$(stty -g < "${TTY_DEVICE}")"
  stty -echo -icanon time 0 min 1 < "${TTY_DEVICE}"
  tput civis > "${TTY_DEVICE}" 2>/dev/null || true

  lines="$(menu_single_lines "${title}" "${instructions}" "${items[@]}")"
  render_menu_single "${title}" "${instructions}" "${cursor}" "${items[@]}"
  while true; do
    key="$(read_key)"
    case "${key}" in
      up) cursor=$(( (cursor + total - 1) % total )) ;;
      down) cursor=$(( (cursor + 1) % total )) ;;
      enter) break ;;
      *) ;;
    esac
    ui_out "$(tput cuu "${lines}" 2>/dev/null || true)"
    ui_out "$(tput ed 2>/dev/null || true)"
    render_menu_single "${title}" "${instructions}" "${cursor}" "${items[@]}"
  done

  ui_out "$(tput cuu "${lines}" 2>/dev/null || true)"
  ui_out "$(tput ed 2>/dev/null || true)"
  tput cnorm > "${TTY_DEVICE}" 2>/dev/null || true
  stty "${stty_state}" < "${TTY_DEVICE}"
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
  local stty_state
  local i

  SELECTED_FLAGS=()
  for i in "${!items[@]}"; do
    if [ "${default_all}" -eq 1 ]; then
      SELECTED_FLAGS[$i]="1"
    else
      SELECTED_FLAGS[$i]="0"
    fi
  done

  stty_state="$(stty -g < "${TTY_DEVICE}")"
  stty -echo -icanon time 0 min 1 < "${TTY_DEVICE}"
  tput civis > "${TTY_DEVICE}" 2>/dev/null || true

  local lines
  lines="$(menu_multi_lines "${title}" "${instructions}" "${footer}" "${items[@]}")"
  render_menu_multi "${title}" "${instructions}" "${cursor}" "${footer}" "${items[@]}"
  while true; do
    key="$(read_key)"
    case "${key}" in
      up) cursor=$(( (cursor + total - 1) % total )) ;;
      down) cursor=$(( (cursor + 1) % total )) ;;
      space)
        if [ "${SELECTED_FLAGS[$cursor]}" = "1" ]; then
          SELECTED_FLAGS[$cursor]="0"
        else
          SELECTED_FLAGS[$cursor]="1"
        fi
        ;;
      a)
        local all_selected=1
        for i in "${!items[@]}"; do
          if [ "${SELECTED_FLAGS[$i]}" != "1" ]; then
            all_selected=0
            break
          fi
        done
        for i in "${!items[@]}"; do
          if [ "${all_selected}" -eq 1 ]; then
            SELECTED_FLAGS[$i]="0"
          else
            SELECTED_FLAGS[$i]="1"
          fi
        done
        ;;
      enter) break ;;
      *) ;;
    esac
    ui_out "$(tput cuu "${lines}" 2>/dev/null || true)"
    ui_out "$(tput ed 2>/dev/null || true)"
    render_menu_multi "${title}" "${instructions}" "${cursor}" "${footer}" "${items[@]}"
  done

  ui_out "$(tput cuu "${lines}" 2>/dev/null || true)"
  ui_out "$(tput ed 2>/dev/null || true)"
  tput cnorm > "${TTY_DEVICE}" 2>/dev/null || true
  stty "${stty_state}" < "${TTY_DEVICE}"

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
  --both               Install to both Cursor and Claude Code
  --all                Install to all supported editors
  --global             Install to global locations
  --project            Install to project locations
  --categories LIST    Comma-separated categories (commands,rules,agents,skills,stack,hooks,mcps)
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
  if [ -t 0 ]; then
    TTY_DEVICE="/dev/tty"
  elif [ -r /dev/tty ]; then
    TTY_DEVICE="/dev/tty"
  fi
}

prompt_read() {
  local var_name="$1"
  if [ -z "${TTY_DEVICE}" ]; then
    die "non-interactive shell; use --yes or pass options"
  fi
  IFS= read -r "${var_name}" < "${TTY_DEVICE}"
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
      --both)
        SELECTED_EDITORS="cursor claude"
        shift
        ;;
      --all)
        SELECTED_EDITORS="cursor claude opencode codex"
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
    SELECTED_EDITORS="cursor claude opencode codex"
    return 0
  fi

  local -a editor_items=("Cursor" "Claude Code" "OpenCode" "Codex")
  local footer=""
  while true; do
    SELECTED_EDITORS=""
    menu_multi "Select editors" "Use ↑/↓ to move, Space to toggle, A for all, Enter to continue." 0 "|" "${footer}" "${editor_items[@]}"
    case "${MENU_RESULT}" in
      *Cursor*) SELECTED_EDITORS="${SELECTED_EDITORS} cursor" ;;
    esac
    case "${MENU_RESULT}" in
      *"Claude Code"*) SELECTED_EDITORS="${SELECTED_EDITORS} claude" ;;
    esac
    case "${MENU_RESULT}" in
      *OpenCode*) SELECTED_EDITORS="${SELECTED_EDITORS} opencode" ;;
    esac
    case "${MENU_RESULT}" in
      *Codex*) SELECTED_EDITORS="${SELECTED_EDITORS} codex" ;;
    esac
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

  local -a scope_items=("Global (~/.cursor or ~/.claude)" "Project (.cursor or .claude in current directory)")
  menu_single "Select scope" "Use ↑/↓ to move, Enter to select." 0 "${scope_items[@]}"
  case "${MENU_RESULT}" in
    "Global"*) SELECTED_SCOPE="global" ;;
    "Project"*) SELECTED_SCOPE="project" ;;
    *) die "invalid scope selection" ;;
  esac
}

select_categories() {
  if [ -n "${SELECTED_CATEGORIES}" ]; then
    return 0
  fi

  if [ "${NON_INTERACTIVE}" -eq 1 ]; then
    SELECTED_CATEGORIES="commands,rules,agents,skills,stack,hooks,mcps"
    return 0
  fi

  local -a category_items=("commands" "rules" "agents" "skills" "stack" "hooks" "mcps")
  local footer=""
  while true; do
    menu_multi "Select categories" "Use ↑/↓ to move, Space to toggle, A for all, Enter to continue." 0 "," "${footer}" "${category_items[@]}"
    SELECTED_CATEGORIES="${MENU_RESULT}"
    if [ -n "${SELECTED_CATEGORIES}" ]; then
      break
    fi
    footer="Please select at least one category."
  done
}

confirm_summary() {
  if [ "${NON_INTERACTIVE}" -eq 1 ]; then
    return 0
  fi

  print_heading "Summary"
  log_info "  Editors:    ${SELECTED_EDITORS}"
  log_info "  Scope:      ${SELECTED_SCOPE}"
  log_info "  Categories: ${SELECTED_CATEGORIES}"
  log_info ""
  printf "Proceed? [y/N] " > "${TTY_DEVICE}"
  prompt_read confirm
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

  log_info "Conflict detected. Choose action:"
  log_info "  1) Overwrite"
  log_info "  2) Backup existing"
  log_info "  3) Skip"
  log_info "  4) Overwrite all"
  log_info "  5) Backup all"
  log_info "  6) Skip all"
  printf "> " > "${TTY_DEVICE}"
  prompt_read choice
  case "${choice}" in
    1) CONFLICT_MODE="overwrite" ;;
    2) CONFLICT_MODE="backup" ;;
    3) CONFLICT_MODE="skip" ;;
    4) CONFLICT_MODE="overwrite_all" ;;
    5) CONFLICT_MODE="backup_all" ;;
    6) CONFLICT_MODE="skip_all" ;;
    *) die "invalid selection" ;;
  esac
}

prompt_dir_conflict_mode() {
  if [ "${CONFLICT_MODE}" = "selective" ]; then
    return 0
  fi
  if [ -n "${CONFLICT_MODE}" ] || [ "${NON_INTERACTIVE}" -eq 1 ]; then
    return 0
  fi

  log_info "Category already exists. Choose action:"
  log_info "  1) Overwrite category"
  log_info "  2) Backup existing category"
  log_info "  3) Skip category"
  log_info "  4) Selective (install file-by-file)"
  log_info "  5) Overwrite all"
  log_info "  6) Backup all"
  log_info "  7) Skip all"
  printf "> " > "${TTY_DEVICE}"
  prompt_read choice
  case "${choice}" in
    1) CONFLICT_MODE="overwrite" ;;
    2) CONFLICT_MODE="backup" ;;
    3) CONFLICT_MODE="skip" ;;
    4) CONFLICT_MODE="selective" ;;
    5) CONFLICT_MODE="overwrite_all" ;;
    6) CONFLICT_MODE="backup_all" ;;
    7) CONFLICT_MODE="skip_all" ;;
    *) die "invalid selection" ;;
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
    local ts
    ts="$(date +%Y%m%d%H%M%S)"
    local backup="${target}.bak.${ts}"
    log_verbose "Backing up ${target} -> ${backup}"
    run_cmd mv "${target}" "${backup}"
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
    local ts
    ts="$(date +%Y%m%d%H%M%S)"
    local backup="${target}.bak.${ts}"
    log_verbose "Backing up ${target} -> ${backup}"
    run_cmd mv "${target}" "${backup}"
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
    local rel="${file#${src_dir}/}"
    local dest="${dest_dir}/${rel}"
    copy_file "${file}" "${dest}"
  done < <(find "${src_dir}" -type f ! -name ".DS_Store" -print0)
}

install_category() {
  local category="$1"
  local target_root="$2"
  local src="${ROOT_DIR}/configs/${category}"
  local dest="${target_root}/${category}"

  [ -d "${src}" ] || die "source category not found: ${src}"
  if [ -d "${dest}" ] && find "${dest}" -mindepth 1 -print -quit | grep -q .; then
    if ! handle_dir_conflict "${dest}"; then
      return 0
    fi
  fi
  copy_tree "${src}" "${dest}"
}

merge_mcp() {
  local src="$1"
  local dest="$2"

  if command -v jq >/dev/null 2>&1; then
    if [ "${NON_INTERACTIVE}" -eq 0 ]; then
      printf "Merge MCP config into existing %s? [y/N] " "${dest}" > "${TTY_DEVICE}"
      local confirm
      prompt_read confirm
      case "${confirm}" in
        y|Y|yes|YES) ;;
        *) return 1 ;;
      esac
    fi

    local tmp
    tmp="$(mktemp)"
    run_cmd jq -s '.[0] * .[1]' "${dest}" "${src}" > "${tmp}"
    if [ -e "${dest}" ]; then
      handle_conflict "${dest}" || { rm -f "${tmp}"; return 0; }
    fi
    log_verbose "Writing merged MCP config to ${dest}"
    run_cmd mv "${tmp}" "${dest}"
    return 0
  fi

  return 1
}

install_mcp() {
  local target_root="$1"
  local dest="$2"
  local src="${ROOT_DIR}/configs/mcps/mcp.json"

  [ -f "${src}" ] || die "missing MCP config: ${src}"
  mkdir -p "${target_root}"

  if [ -e "${dest}" ] && merge_mcp "${src}" "${dest}"; then
    return 0
  fi

  copy_file "${src}" "${dest}"
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
        printf '%s\n' "${HOME}/.opencode"
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
    *)
      die "unknown editor: ${editor}"
      ;;
  esac
}

mcp_path_for() {
  local editor="$1"
  local target_root="$2"

  case "${editor}" in
    claude)
      printf '%s\n' "${target_root}/.mcp.json"
      ;;
    cursor|opencode|codex)
      printf '%s\n' "${target_root}/mcp.json"
      ;;
    *)
      die "unknown editor: ${editor}"
      ;;
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
    if [ "${category}" = "mcps" ]; then
      install_mcp "${target_root}" "$(mcp_path_for "${editor}" "${target_root}")"
    else
      install_category "${category}" "${target_root}"
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
  ensure_repo_root

  select_editors
  select_scope
  select_categories
  set_conflict_mode_default
  confirm_summary

  local editor
  for editor in ${SELECTED_EDITORS}; do
    install_for_target "${editor}" "${SELECTED_SCOPE}"
  done

  log_info "Done."
}

main "$@"
