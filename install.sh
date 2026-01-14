#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
SRC_DIR="${ROOT_DIR}/configs"

DRY_RUN=0
FORCE=0
BACKUP=1
NON_INTERACTIVE=0
INSTALL_ROOT_FILES=1

SCOPE="" # project|global|both
PROJECT_DIR="${PWD}"

TOOLS_CLAUDE=0
TOOLS_CURSOR=0
TOOLS_OPENCODE=0

# Track whether values were supplied via CLI flags, so we don't prompt unnecessarily.
ANY_FLAGS=0
PROVIDED_TOOLS=0
PROVIDED_SCOPE=0
PROVIDED_PROJECT_DIR=0
PROVIDED_INSTALL_ROOT_FILES=0
PROVIDED_BACKUP_MODE=0

CLAUDE_HOME="${CLAUDE_HOME:-${HOME}/.claude}"
CURSOR_HOME="${CURSOR_HOME:-${HOME}/.cursor}"
OPENCODE_HOME="${OPENCODE_HOME:-${HOME}/.opencode}"

usage() {
  cat <<'EOF'
install.sh - install these configs into Claude Code, Cursor, and/or OpenCode

Usage:
  ./install.sh                  # interactive mode
  ./install.sh -y               # non-interactive defaults (all tools, project scope)

Options:
  --tool <list>                 Comma-separated: claude,cursor,opencode,all
  --scope <scope>               project|global|both
  --project-dir <path>          Target project directory (default: current directory)
  --claude-home <path>          Global Claude Code config dir (default: ~/.claude or $CLAUDE_HOME)
  --cursor-home <path>          Global Cursor config dir (default: ~/.cursor or $CURSOR_HOME)
  --opencode-home <path>        Global OpenCode config dir (default: ~/.opencode or $OPENCODE_HOME)
  --skip-root-files             For Claude installs, do not install CLAUDE.md/AGENTS.md
  --dry-run                     Print actions without writing
  --no-backup                   Overwrite without backups
  --force                       Same as --no-backup
  -y, --yes                     Non-interactive (accept defaults)
  -h, --help                    Show help

Notes:
  - Project installs go to:
      - Cursor:   <project>/.cursor/
      - Claude:   <project>/.claude/ (+ optional <project>/CLAUDE.md + <project>/AGENTS.md)
      - OpenCode: <project>/.opencode/
  - Global installs go to:
      - Cursor:   ~/.cursor/
      - Claude:   ~/.claude/
      - OpenCode: ~/.opencode/
EOF
}

die() {
  echo "Error: $*" >&2
  exit 1
}

log() {
  echo "==> $*"
}

have_cmd() {
  command -v "$1" >/dev/null 2>&1
}

timestamp() {
  date +"%Y%m%d%H%M%S"
}

sync_dir() {
  local src="$1"
  local dest="$2"
  local ts
  ts="$(timestamp)"

  [[ -d "$src" ]] || die "Missing source directory: ${src}"

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    log "[dry-run] Sync dir: ${src}/ -> ${dest}/"
    return 0
  fi

  mkdir -p "$dest"

  if have_cmd rsync; then
    local -a rsync_opts
    rsync_opts=(-a --exclude ".DS_Store")
    if [[ "${BACKUP}" -eq 1 ]]; then
      rsync_opts+=(--backup --suffix ".bak-${ts}")
    fi
    rsync "${rsync_opts[@]}" "${src}/" "${dest}/"
    return 0
  fi

  # Fallback: back up the entire destination directory (if present) before replacing.
  if [[ "${BACKUP}" -eq 1 && -e "${dest}" ]]; then
    mv "${dest}" "${dest}.bak-${ts}"
    mkdir -p "${dest}"
  fi

  # shellcheck disable=SC2115
  cp -a "${src}/." "${dest}/"
}

sync_file() {
  local src="$1"
  local dest="$2"
  local ts
  ts="$(timestamp)"

  [[ -f "$src" ]] || die "Missing source file: ${src}"

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    log "[dry-run] Sync file: ${src} -> ${dest}"
    return 0
  fi

  mkdir -p "$(dirname -- "${dest}")"

  if have_cmd rsync; then
    local -a rsync_opts
    rsync_opts=(-a)
    if [[ "${BACKUP}" -eq 1 ]]; then
      rsync_opts+=(--backup --suffix ".bak-${ts}")
    fi
    rsync "${rsync_opts[@]}" "${src}" "${dest}"
    return 0
  fi

  if [[ "${BACKUP}" -eq 1 && -e "${dest}" ]]; then
    mv "${dest}" "${dest}.bak-${ts}"
  fi

  cp -a "${src}" "${dest}"
}

install_common_tree() {
  local base_dir="$1"

  sync_dir "${SRC_DIR}/agents" "${base_dir}/agents"
  sync_dir "${SRC_DIR}/commands" "${base_dir}/commands"
  sync_dir "${SRC_DIR}/rules" "${base_dir}/rules"
  sync_dir "${SRC_DIR}/stack" "${base_dir}/stack"
  sync_dir "${SRC_DIR}/skills" "${base_dir}/skills"

  # Optional/empty in this repo right now, but create the expected location anyway.
  if [[ -d "${SRC_DIR}/hooks" ]]; then
    sync_dir "${SRC_DIR}/hooks" "${base_dir}/hooks"
  fi

  # MCP: put the main config at the tool root, and include the mcps/ folder as reference.
  if [[ -f "${SRC_DIR}/mcps/mcp.json" ]]; then
    sync_file "${SRC_DIR}/mcps/mcp.json" "${base_dir}/mcp.json"
  fi
  if [[ -d "${SRC_DIR}/mcps" ]]; then
    sync_dir "${SRC_DIR}/mcps" "${base_dir}/mcps"
  fi
}

install_cursor_project() {
  local project="$1"
  local base="${project}/.cursor"
  log "Installing Cursor configs to ${base}"
  install_common_tree "${base}"
}

install_cursor_global() {
  local base="${CURSOR_HOME}"
  log "Installing Cursor configs globally to ${base}"
  install_common_tree "${base}"
}

install_claude_project() {
  local project="$1"
  local base="${project}/.claude"
  log "Installing Claude Code configs to ${base}"
  install_common_tree "${base}"

  if [[ "${INSTALL_ROOT_FILES}" -eq 1 ]]; then
    [[ -f "${ROOT_DIR}/CLAUDE.md" ]] || die "Missing ${ROOT_DIR}/CLAUDE.md"
    [[ -f "${ROOT_DIR}/AGENTS.md" ]] || die "Missing ${ROOT_DIR}/AGENTS.md"
    log "Installing Claude Code root files to ${project}"
    sync_file "${ROOT_DIR}/CLAUDE.md" "${project}/CLAUDE.md"
    sync_file "${ROOT_DIR}/AGENTS.md" "${project}/AGENTS.md"
  fi
}

install_claude_global() {
  local base="${CLAUDE_HOME}"
  log "Installing Claude Code configs globally to ${base}"
  install_common_tree "${base}"

  if [[ "${INSTALL_ROOT_FILES}" -eq 1 ]]; then
    [[ -f "${ROOT_DIR}/CLAUDE.md" ]] || die "Missing ${ROOT_DIR}/CLAUDE.md"
    [[ -f "${ROOT_DIR}/AGENTS.md" ]] || die "Missing ${ROOT_DIR}/AGENTS.md"
    log "Installing Claude Code root files to ${base}"
    sync_file "${ROOT_DIR}/CLAUDE.md" "${base}/CLAUDE.md"
    sync_file "${ROOT_DIR}/AGENTS.md" "${base}/AGENTS.md"
  fi
}

install_opencode_project() {
  local project="$1"
  local base="${project}/.opencode"
  log "Installing OpenCode configs to ${base}"
  install_common_tree "${base}"
}

install_opencode_global() {
  local base="${OPENCODE_HOME}"
  log "Installing OpenCode configs globally to ${base}"
  install_common_tree "${base}"
}

prompt_tools() {
  echo "Select tools to install (space-separated numbers):"
  echo "  1) Claude Code"
  echo "  2) Cursor"
  echo "  3) OpenCode"
  echo "  4) All"
  read -r -p "> " choice

  for tok in ${choice}; do
    case "${tok}" in
      1) TOOLS_CLAUDE=1 ;;
      2) TOOLS_CURSOR=1 ;;
      3) TOOLS_OPENCODE=1 ;;
      4) TOOLS_CLAUDE=1; TOOLS_CURSOR=1; TOOLS_OPENCODE=1 ;;
      *) die "Unknown selection: ${tok}" ;;
    esac
  done
}

prompt_scope() {
  echo "Select install scope:"
  echo "  1) Project (current directory)"
  echo "  2) Global (home directory)"
  echo "  3) Both"
  read -r -p "> " choice
  case "${choice}" in
    1) SCOPE="project" ;;
    2) SCOPE="global" ;;
    3) SCOPE="both" ;;
    *) die "Unknown selection: ${choice}" ;;
  esac
}

prompt_project_dir() {
  echo "Project directory?"
  read -r -p "[${PROJECT_DIR}]> " input
  if [[ -n "${input}" ]]; then
    PROJECT_DIR="${input}"
  fi
}

prompt_claude_root_files() {
  echo "For Claude Code: install root files (CLAUDE.md + AGENTS.md)?"
  echo "  1) Yes (recommended)"
  echo "  2) No"
  read -r -p "> " choice
  case "${choice}" in
    1) INSTALL_ROOT_FILES=1 ;;
    2) INSTALL_ROOT_FILES=0 ;;
    *) die "Unknown selection: ${choice}" ;;
  esac
}

prompt_overwrite_behavior() {
  echo "Overwrite behavior:"
  echo "  1) Backup existing files (recommended)"
  echo "  2) Overwrite without backups (force)"
  read -r -p "> " choice
  case "${choice}" in
    1) BACKUP=1; FORCE=0 ;;
    2) BACKUP=0; FORCE=1 ;;
    *) die "Unknown selection: ${choice}" ;;
  esac
}

parse_tool_list() {
  local list="$1"
  local item
  IFS=',' read -r -a items <<< "${list}"
  for item in "${items[@]}"; do
    case "${item}" in
      claude) TOOLS_CLAUDE=1 ;;
      cursor) TOOLS_CURSOR=1 ;;
      opencode) TOOLS_OPENCODE=1 ;;
      all) TOOLS_CLAUDE=1; TOOLS_CURSOR=1; TOOLS_OPENCODE=1 ;;
      *) die "Unknown --tool entry: ${item}" ;;
    esac
  done
}

main() {
  [[ -d "${SRC_DIR}" ]] || die "Missing ${SRC_DIR}. Are you running this from the repo?"

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --tool)
        [[ $# -ge 2 ]] || die "--tool requires a value"
        parse_tool_list "$2"
        PROVIDED_TOOLS=1
        ANY_FLAGS=1
        shift 2
        ;;
      --scope)
        [[ $# -ge 2 ]] || die "--scope requires a value"
        SCOPE="$2"
        PROVIDED_SCOPE=1
        ANY_FLAGS=1
        shift 2
        ;;
      --project-dir)
        [[ $# -ge 2 ]] || die "--project-dir requires a value"
        PROJECT_DIR="$2"
        PROVIDED_PROJECT_DIR=1
        ANY_FLAGS=1
        shift 2
        ;;
      --claude-home)
        [[ $# -ge 2 ]] || die "--claude-home requires a value"
        CLAUDE_HOME="$2"
        ANY_FLAGS=1
        shift 2
        ;;
      --cursor-home)
        [[ $# -ge 2 ]] || die "--cursor-home requires a value"
        CURSOR_HOME="$2"
        ANY_FLAGS=1
        shift 2
        ;;
      --opencode-home)
        [[ $# -ge 2 ]] || die "--opencode-home requires a value"
        OPENCODE_HOME="$2"
        ANY_FLAGS=1
        shift 2
        ;;
      --skip-root-files)
        INSTALL_ROOT_FILES=0
        PROVIDED_INSTALL_ROOT_FILES=1
        ANY_FLAGS=1
        shift
        ;;
      --dry-run)
        DRY_RUN=1
        ANY_FLAGS=1
        shift
        ;;
      --no-backup)
        BACKUP=0
        PROVIDED_BACKUP_MODE=1
        ANY_FLAGS=1
        shift
        ;;
      --force)
        FORCE=1
        BACKUP=0
        PROVIDED_BACKUP_MODE=1
        ANY_FLAGS=1
        shift
        ;;
      -y|--yes)
        NON_INTERACTIVE=1
        ANY_FLAGS=1
        shift
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        die "Unknown argument: $1 (try --help)"
        ;;
    esac
  done

  if [[ "${NON_INTERACTIVE}" -eq 0 ]]; then
    if [[ "${TOOLS_CLAUDE}${TOOLS_CURSOR}${TOOLS_OPENCODE}" == "000" ]]; then
      prompt_tools
    fi
    if [[ -z "${SCOPE}" ]]; then
      prompt_scope
    fi
    if [[ ("${SCOPE}" == "project" || "${SCOPE}" == "both") && "${PROVIDED_PROJECT_DIR}" -eq 0 ]]; then
      prompt_project_dir
    fi
    if [[ "${TOOLS_CLAUDE}" -eq 1 && "${PROVIDED_INSTALL_ROOT_FILES}" -eq 0 ]]; then
      prompt_claude_root_files
    fi
    # Only ask about overwrite behavior in the fully interactive path (no CLI flags).
    if [[ "${ANY_FLAGS}" -eq 0 && "${DRY_RUN}" -eq 0 && "${FORCE}" -eq 0 && "${PROVIDED_BACKUP_MODE}" -eq 0 ]]; then
      prompt_overwrite_behavior
    fi
  else
    # Non-interactive defaults
    if [[ "${TOOLS_CLAUDE}${TOOLS_CURSOR}${TOOLS_OPENCODE}" == "000" ]]; then
      TOOLS_CLAUDE=1
      TOOLS_CURSOR=1
      TOOLS_OPENCODE=1
    fi
    if [[ -z "${SCOPE}" ]]; then
      SCOPE="project"
    fi
  fi

  case "${SCOPE}" in
    project|global|both) ;;
    *) die "Invalid --scope: ${SCOPE} (expected project|global|both)" ;;
  esac

  log "Source configs: ${SRC_DIR}"
  log "Selected tools: claude=${TOOLS_CLAUDE} cursor=${TOOLS_CURSOR} opencode=${TOOLS_OPENCODE}"
  log "Scope: ${SCOPE}"
  if [[ "${SCOPE}" == "project" || "${SCOPE}" == "both" ]]; then
    log "Project dir: ${PROJECT_DIR}"
  fi
  if [[ "${SCOPE}" == "global" || "${SCOPE}" == "both" ]]; then
    log "Global dirs: claude=${CLAUDE_HOME} cursor=${CURSOR_HOME} opencode=${OPENCODE_HOME}"
  fi
  if [[ "${TOOLS_CLAUDE}" -eq 1 ]]; then
    log "Claude root files: ${INSTALL_ROOT_FILES}"
  fi

  if [[ "${SCOPE}" == "project" || "${SCOPE}" == "both" ]]; then
    [[ -d "${PROJECT_DIR}" ]] || die "Project directory does not exist: ${PROJECT_DIR}"

    if [[ "${TOOLS_CURSOR}" -eq 1 ]]; then
      install_cursor_project "${PROJECT_DIR}"
    fi
    if [[ "${TOOLS_CLAUDE}" -eq 1 ]]; then
      install_claude_project "${PROJECT_DIR}"
    fi
    if [[ "${TOOLS_OPENCODE}" -eq 1 ]]; then
      install_opencode_project "${PROJECT_DIR}"
    fi
  fi

  if [[ "${SCOPE}" == "global" || "${SCOPE}" == "both" ]]; then
    if [[ "${TOOLS_CURSOR}" -eq 1 ]]; then
      install_cursor_global
    fi
    if [[ "${TOOLS_CLAUDE}" -eq 1 ]]; then
      install_claude_global
    fi
    if [[ "${TOOLS_OPENCODE}" -eq 1 ]]; then
      install_opencode_global
    fi
  fi

  log "Done."
}

main "$@"
