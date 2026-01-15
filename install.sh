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

log() { printf '%s\n' "$*" >&2; }
die() { log "error: $*"; exit 1; }

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
  log "Cloning stacc repository..."
  run_cmd git clone --depth 1 --branch main "${REPO_URL}" "${TMP_ROOT}/stacc"
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

  log "Select editor:"
  log "  1) Cursor"
  log "  2) Claude Code"
  log "  3) OpenCode"
  log "  4) Codex"
  log "  5) Both (Cursor + Claude Code)"
  log "  6) All"
  printf "> " > "${TTY_DEVICE}"
  prompt_read choice
  case "${choice}" in
    1) SELECTED_EDITORS="cursor" ;;
    2) SELECTED_EDITORS="claude" ;;
    3) SELECTED_EDITORS="opencode" ;;
    4) SELECTED_EDITORS="codex" ;;
    5) SELECTED_EDITORS="cursor claude" ;;
    6) SELECTED_EDITORS="cursor claude opencode codex" ;;
    *) die "invalid selection" ;;
  esac
}

select_scope() {
  if [ -n "${SELECTED_SCOPE}" ]; then
    return 0
  fi

  if [ "${NON_INTERACTIVE}" -eq 1 ]; then
    SELECTED_SCOPE="global"
    return 0
  fi

  log "Select scope:"
  log "  1) Global (~/.cursor or ~/.claude)"
  log "  2) Project (.cursor or .claude in current directory)"
  printf "> " > "${TTY_DEVICE}"
  prompt_read choice
  case "${choice}" in
    1) SELECTED_SCOPE="global" ;;
    2) SELECTED_SCOPE="project" ;;
    *) die "invalid selection" ;;
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

  log "Select categories (comma-separated numbers or 'all'):"
  log "  1) commands"
  log "  2) rules"
  log "  3) agents"
  log "  4) skills"
  log "  5) stack"
  log "  6) hooks"
  log "  7) mcps"
  printf "> " > "${TTY_DEVICE}"
  prompt_read choice

  if [ "${choice}" = "all" ]; then
    SELECTED_CATEGORIES="commands,rules,agents,skills,stack,hooks,mcps"
    return 0
  fi

  local selected=""
  local part
  IFS=',' read -r -a parts <<< "${choice}"
  for part in "${parts[@]}"; do
    case "$(printf '%s' "${part}" | tr -d '[:space:]')" in
      1) selected="${selected},commands" ;;
      2) selected="${selected},rules" ;;
      3) selected="${selected},agents" ;;
      4) selected="${selected},skills" ;;
      5) selected="${selected},stack" ;;
      6) selected="${selected},hooks" ;;
      7) selected="${selected},mcps" ;;
      *) die "invalid category selection: ${part}" ;;
    esac
  done
  SELECTED_CATEGORIES="${selected#,}"
}

confirm_summary() {
  if [ "${NON_INTERACTIVE}" -eq 1 ]; then
    return 0
  fi

  log ""
  log "Summary:"
  log "  Editors: ${SELECTED_EDITORS}"
  log "  Scope: ${SELECTED_SCOPE}"
  log "  Categories: ${SELECTED_CATEGORIES}"
  log ""
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

  log "Conflict detected. Choose action:"
  log "  1) Overwrite"
  log "  2) Backup existing"
  log "  3) Skip"
  log "  4) Overwrite all"
  log "  5) Backup all"
  log "  6) Skip all"
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

  log "Category already exists. Choose action:"
  log "  1) Overwrite category"
  log "  2) Backup existing category"
  log "  3) Skip category"
  log "  4) Selective (install file-by-file)"
  log "  5) Overwrite all"
  log "  6) Backup all"
  log "  7) Skip all"
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
    log "Skipping ${target}"
    return 1
  fi

  if [ "${mode}" = "backup" ]; then
    local ts
    ts="$(date +%Y%m%d%H%M%S)"
    local backup="${target}.bak.${ts}"
    log "Backing up ${target} -> ${backup}"
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
    log "Skipping ${target}"
    return 1
  fi

  if [ "${mode}" = "backup" ]; then
    local ts
    ts="$(date +%Y%m%d%H%M%S)"
    local backup="${target}.bak.${ts}"
    log "Backing up ${target} -> ${backup}"
    run_cmd mv "${target}" "${backup}"
    return 0
  fi

  if [ "${mode}" = "overwrite" ]; then
    log "Removing existing ${target}"
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

  log "Installing ${dest}"
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
    log "Writing merged MCP config to ${dest}"
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

  log "Done."
}

main "$@"
