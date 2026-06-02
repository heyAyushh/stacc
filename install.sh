#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

REPO_URL="${STACC_REPO_URL:-https://github.com/heyAyushh/stacc.git}"
DEFAULT_CATEGORIES="commands,rules,agents,skills,stack,hooks,mcps,cursor-plugins,codex-skills"
DEFAULT_SCOPE="global"
ROOT_DIR=""
TMP_ROOT=""
APPEND_CSV_RESULT=()
APPEND_CSV_COUNT=0
TRANSLATED_ARGS=()
TRANSLATED_DRY_RUN=0

cleanup() {
  if [ -n "${TMP_ROOT}" ] && [ -d "${TMP_ROOT}" ]; then
    rm -rf "${TMP_ROOT}"
  fi
}
trap cleanup EXIT

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

usage() {
  cat <<'EOF'
stacc bootstrap

Usage:
  ./install.sh [stacc args]
  ./install.sh [legacy install options]

Direct stacc examples:
  ./install.sh
  ./install.sh check
  ./install.sh install --editor codex --scope global --category rules --dry-run

Legacy install options translated to `stacc install`:
  --root PATH
  --cursor | --claude | --opencode | --codex | --ampcode | --both | --all
  --global | --project
  --categories LIST
  --stacks LIST
  --mcp-servers LIST
  --conflict MODE
  --yes
  --dry-run

Examples:
  ./install.sh --yes
  ./install.sh --codex --global --categories rules,skills,mcps --mcp-servers github --yes
  curl -fsSL https://raw.githubusercontent.com/heyAyushh/stacc/main/install.sh | bash
EOF
}

append_csv_flags() {
  local flag="$1"
  local values="$2"
  local old_ifs item

  APPEND_CSV_RESULT=()
  APPEND_CSV_COUNT=0
  values="${values// /}"
  [ -n "${values}" ] || return 0

  old_ifs="${IFS}"
  IFS=','
  for item in ${values}; do
    [ -n "${item}" ] || continue
    APPEND_CSV_RESULT+=("${flag}" "${item}")
    APPEND_CSV_COUNT=$((APPEND_CSV_COUNT + 2))
  done
  IFS="${old_ifs}"
}

args_include_dry_run() {
  local arg
  for arg in "$@"; do
    if [ "${arg}" = "--dry-run" ]; then
      return 0
    fi
  done
  return 1
}

local_stacc_root() {
  local candidate="${1:-}"
  if [ -n "${candidate}" ] && [ -f "${candidate}/Cargo.toml" ] && [ -d "${candidate}/configs" ]; then
    printf '%s\n' "${candidate}"
    return 0
  fi
  if [ -f "./Cargo.toml" ] && [ -d "./configs" ]; then
    pwd
    return 0
  fi
  return 1
}

installed_stacc_bin() {
  if command -v stacc >/dev/null 2>&1; then
    command -v stacc
    return 0
  fi

  local cargo_home="${CARGO_HOME:-${HOME}/.cargo}"
  printf '%s\n' "${cargo_home}/bin/stacc"
}

run_stacc() {
  local dry_run="$1"
  shift
  local -a stacc_args=("$@")
  local root

  command -v cargo >/dev/null 2>&1 || die "cargo is required; install Rust from https://rustup.rs"

  if root="$(local_stacc_root "${ROOT_DIR}")"; then
    if [ "${#stacc_args[@]}" -gt 0 ]; then
      exec cargo run --manifest-path "${root}/Cargo.toml" -- "${stacc_args[@]}"
    fi
    exec cargo run --manifest-path "${root}/Cargo.toml" --
  fi

  if [ "${dry_run}" -eq 1 ]; then
    TMP_ROOT="$(mktemp -d)"
    cargo install --git "${REPO_URL}" --root "${TMP_ROOT}/cargo" --locked --force
    if [ "${#stacc_args[@]}" -gt 0 ]; then
      "${TMP_ROOT}/cargo/bin/stacc" "${stacc_args[@]}"
      exit $?
    fi
    "${TMP_ROOT}/cargo/bin/stacc"
    exit $?
  fi

  cargo install --git "${REPO_URL}" --locked --force
  if [ "${#stacc_args[@]}" -gt 0 ]; then
    exec "$(installed_stacc_bin)" "${stacc_args[@]}"
  fi
  exec "$(installed_stacc_bin)"
}

is_direct_stacc_invocation() {
  [ $# -gt 0 ] || return 0
  if [ "$1" = "--root" ]; then
    [ $# -ge 3 ] || return 0
    shift 2
  fi
  case "$1" in
    status|install|sync-metadata|check|--panel|--config|--version|-V)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

translate_legacy_args() {
  local -a editors=()
  local categories=""
  local conflict=""
  local mcp_servers=""
  local scope=""
  local stacks=""
  local yes=0
  local dry_run=0
  local legacy_mode=0

  TRANSLATED_ARGS=()
  TRANSLATED_DRY_RUN=0

  while [ $# -gt 0 ]; do
    case "$1" in
      --root)
        [ $# -ge 2 ] || die "--root requires a path"
        ROOT_DIR="$2"
        shift 2
        ;;
      --cursor)
        editors+=("cursor")
        legacy_mode=1
        shift
        ;;
      --claude)
        editors+=("claude")
        legacy_mode=1
        shift
        ;;
      --opencode)
        editors+=("opencode")
        legacy_mode=1
        shift
        ;;
      --codex)
        editors+=("codex")
        legacy_mode=1
        shift
        ;;
      --ampcode)
        editors+=("ampcode")
        legacy_mode=1
        shift
        ;;
      --both)
        editors+=("cursor" "claude")
        legacy_mode=1
        shift
        ;;
      --all)
        editors+=("cursor" "claude" "opencode" "codex" "ampcode")
        legacy_mode=1
        shift
        ;;
      --global)
        scope="global"
        legacy_mode=1
        shift
        ;;
      --project)
        scope="project"
        legacy_mode=1
        shift
        ;;
      --categories)
        [ $# -ge 2 ] || die "--categories requires a list"
        categories="$2"
        legacy_mode=1
        shift 2
        ;;
      --stacks)
        [ $# -ge 2 ] || die "--stacks requires a list"
        stacks="$2"
        legacy_mode=1
        shift 2
        ;;
      --mcp-servers)
        [ $# -ge 2 ] || die "--mcp-servers requires a list"
        mcp_servers="$2"
        legacy_mode=1
        shift 2
        ;;
      --conflict)
        [ $# -ge 2 ] || die "--conflict requires a mode"
        conflict="$2"
        legacy_mode=1
        shift 2
        ;;
      --yes)
        yes=1
        legacy_mode=1
        shift
        ;;
      --dry-run)
        dry_run=1
        legacy_mode=1
        shift
        ;;
      --verbose)
        legacy_mode=1
        shift
        ;;
      --help|-h)
        usage
        exit 0
        ;;
      *)
        die "unknown legacy option: $1"
        ;;
    esac
  done

  if [ "${legacy_mode}" -eq 0 ]; then
    if [ -n "${ROOT_DIR}" ]; then
      TRANSLATED_ARGS+=("--root" "${ROOT_DIR}")
    fi
    TRANSLATED_DRY_RUN="${dry_run}"
    return 0
  fi

  if [ "${#editors[@]}" -eq 0 ]; then
    editors=("cursor" "claude" "opencode" "codex" "ampcode")
  fi
  scope="${scope:-${DEFAULT_SCOPE}}"
  categories="${categories:-${DEFAULT_CATEGORIES}}"

  if [ -n "${ROOT_DIR}" ]; then
    TRANSLATED_ARGS+=("--root" "${ROOT_DIR}")
  fi
  TRANSLATED_ARGS+=("install")
  local editor
  for editor in "${editors[@]}"; do
    TRANSLATED_ARGS+=("--editor" "${editor}")
  done
  TRANSLATED_ARGS+=("--scope" "${scope}")
  append_csv_flags "--category" "${categories}"
  if [ "${APPEND_CSV_COUNT}" -gt 0 ]; then
    TRANSLATED_ARGS+=("${APPEND_CSV_RESULT[@]}")
  fi
  append_csv_flags "--stack" "${stacks}"
  if [ "${APPEND_CSV_COUNT}" -gt 0 ]; then
    TRANSLATED_ARGS+=("${APPEND_CSV_RESULT[@]}")
  fi
  append_csv_flags "--mcp-server" "${mcp_servers}"
  if [ "${APPEND_CSV_COUNT}" -gt 0 ]; then
    TRANSLATED_ARGS+=("${APPEND_CSV_RESULT[@]}")
  fi
  if [ -n "${conflict}" ]; then
    TRANSLATED_ARGS+=("--conflict" "${conflict}")
  fi
  if [ "${yes}" -eq 1 ]; then
    TRANSLATED_ARGS+=("--yes")
  fi
  if [ "${dry_run}" -eq 1 ]; then
    TRANSLATED_ARGS+=("--dry-run" "--print-plan")
  fi

  TRANSLATED_DRY_RUN="${dry_run}"
}

main() {
  if [ $# -gt 0 ] && { [ "$1" = "--help" ] || [ "$1" = "-h" ]; }; then
    usage
    exit 0
  fi

  if is_direct_stacc_invocation "$@"; then
    if args_include_dry_run "$@"; then
      run_stacc 1 "$@"
    fi
    run_stacc 0 "$@"
  fi

  translate_legacy_args "$@"

  if [ "${#TRANSLATED_ARGS[@]}" -gt 0 ]; then
    run_stacc "${TRANSLATED_DRY_RUN}" "${TRANSLATED_ARGS[@]}"
  fi
  run_stacc "${TRANSLATED_DRY_RUN}"
}

main "$@"
