#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

REPO_URL="${STACC_REPO_URL:-https://github.com/heyAyushh/stacc.git}"
RELEASE_REPO="${STACC_RELEASE_REPO:-heyAyushh/stacc}"
RELEASE_TAG="${STACC_RELEASE_TAG:-latest}"
DEFAULT_CATEGORIES="commands,rules,agents,skills,stack,hooks,mcps,cursor-plugins,codex-skills"
DEFAULT_SCOPE="global"
INSTALL_MODE="755"
FIRST_MATCH_COUNT="1"
ROOT_DIR=""
TMP_ROOT=""
APPEND_CSV_RESULT=()
APPEND_CSV_COUNT=0
TRANSLATED_ARGS=()
TRANSLATED_DRY_RUN=0

# shellcheck disable=SC2329 # Invoked indirectly by trap on EXIT.
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
  ./install.sh bootstrap --dry-run
  ./install.sh install --editor codex --scope global --category rules --dry-run

Legacy install options translated to `stacc install`:
  --root PATH
  --cursor | --claude | --opencode | --codex | --ampcode | --both | --all
  --global | --project
  --categories LIST
  --stacks LIST
  --hooks LIST
  --mcp-servers LIST
  --conflict MODE
  --yes
  --dry-run

Examples:
  ./install.sh --yes
  ./install.sh --codex --global --categories rules,skills,mcps --mcp-servers github --yes
  ./install.sh --cursor --project --categories hooks --hooks continual-learning --dry-run
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

capture_root_arg() {
  while [ $# -gt 0 ]; do
    case "$1" in
      --root)
        [ $# -ge 2 ] || die "--root requires a path"
        ROOT_DIR="$2"
        return 0
        ;;
      --root=*)
        ROOT_DIR="${1#--root=}"
        [ -n "${ROOT_DIR}" ] || die "--root requires a path"
        return 0
        ;;
      *)
        shift
        ;;
    esac
  done
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

install_bin_dir() {
  local cargo_home="${CARGO_HOME:-${HOME}/.cargo}"
  printf '%s\n' "${cargo_home}/bin"
}

binary_name() {
  case "$(uname -s 2>/dev/null || printf unknown)" in
    MINGW*|MSYS*|CYGWIN*) printf 'stacc.exe\n' ;;
    *) printf 'stacc\n' ;;
  esac
}

local_built_stacc_bin() {
  local root="$1"
  local name
  name="$(binary_name)"
  if [ -x "${root}/target/release/${name}" ]; then
    printf '%s\n' "${root}/target/release/${name}"
    return 0
  fi
  if [ -x "${root}/target/debug/${name}" ]; then
    printf '%s\n' "${root}/target/debug/${name}"
    return 0
  fi
  return 1
}

args_have_root() {
  while [ $# -gt 0 ]; do
    case "$1" in
      --root)
        return 0
        ;;
      --root=*)
        return 0
        ;;
    esac
    shift
  done
  return 1
}

release_target() {
  local os
  local arch
  local target_os
  local target_arch

  os="$(uname -s 2>/dev/null || printf unknown)"
  arch="$(uname -m 2>/dev/null || printf unknown)"

  case "${os}" in
    Darwin) target_os="apple-darwin" ;;
    Linux) target_os="unknown-linux-gnu" ;;
    MINGW*|MSYS*|CYGWIN*) target_os="pc-windows-msvc" ;;
    *) return 1 ;;
  esac

  case "${arch}" in
    x86_64|amd64) target_arch="x86_64" ;;
    arm64|aarch64) target_arch="aarch64" ;;
    *) return 1 ;;
  esac

  if [ "${target_os}" = "unknown-linux-gnu" ] && [ "${target_arch}" != "x86_64" ]; then
    return 1
  fi
  if [ "${target_os}" = "pc-windows-msvc" ] && [ "${target_arch}" != "x86_64" ]; then
    return 1
  fi

  printf '%s-%s\n' "${target_arch}" "${target_os}"
}

release_asset_url() {
  local target="$1"
  local asset="stacc-${target}.tar.gz"
  if [ "${RELEASE_TAG}" = "latest" ]; then
    printf 'https://github.com/%s/releases/latest/download/%s\n' "${RELEASE_REPO}" "${asset}"
    return 0
  fi
  printf 'https://github.com/%s/releases/download/%s/%s\n' "${RELEASE_REPO}" "${RELEASE_TAG}" "${asset}"
}

download_file() {
  local url="$1"
  local output="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "${url}" -o "${output}"
    return $?
  fi
  if command -v wget >/dev/null 2>&1; then
    wget -qO "${output}" "${url}"
    return $?
  fi
  return 1
}

find_extracted_binary() {
  local directory="$1"
  local name
  name="$(binary_name)"
  find "${directory}" -type f -name "${name}" -print | head -n "${FIRST_MATCH_COUNT}"
}

download_release_binary() {
  local destination="$1"
  local target
  local url
  local archive
  local extract_dir
  local extracted

  target="$(release_target)" || return 1
  url="$(release_asset_url "${target}")"
  archive="${destination}/stacc-${target}.tar.gz"
  extract_dir="${destination}/extract"

  mkdir -p "${extract_dir}"
  printf '==> downloading %s\n' "${url}" >&2
  download_file "${url}" "${archive}" || return 1
  tar -xzf "${archive}" -C "${extract_dir}" || return 1
  extracted="$(find_extracted_binary "${extract_dir}")"
  [ -n "${extracted}" ] || return 1
  chmod "${INSTALL_MODE}" "${extracted}"
  printf '%s\n' "${extracted}"
}

install_release_binary() {
  local source_binary="$1"
  local bin_dir
  local target_binary
  bin_dir="$(install_bin_dir)"
  target_binary="${bin_dir}/$(binary_name)"
  mkdir -p "${bin_dir}"
  cp "${source_binary}" "${target_binary}"
  chmod "${INSTALL_MODE}" "${target_binary}"
  printf '%s\n' "${target_binary}"
}

has_interactive_terminal() {
  if [ -t 0 ] && [ -t 1 ]; then
    return 0
  fi
  [ -r /dev/tty ] && [ -t 1 ]
}

require_interactive_panel() {
  has_interactive_terminal && return 0
  die "no interactive terminal available; pass stacc args, e.g. ./install.sh install --editor codex --scope global --category rules --dry-run"
}

run_no_arg_local_stacc() {
  local root="$1"
  require_interactive_panel
  if [ ! -t 0 ] && [ -r /dev/tty ]; then
    exec cargo run --manifest-path "${root}/Cargo.toml" -- < /dev/tty
  fi
  exec cargo run --manifest-path "${root}/Cargo.toml" --
}

run_no_arg_binary_stacc() {
  local binary="$1"
  require_interactive_panel
  if [ ! -t 0 ] && [ -r /dev/tty ]; then
    "${binary}" < /dev/tty
    exit $?
  fi
  "${binary}"
  exit $?
}

run_local_binary_stacc() {
  local root="$1"
  local binary="$2"
  shift 2

  if [ $# -gt 0 ]; then
    if args_have_root "$@"; then
      exec "${binary}" "$@"
    fi
    exec "${binary}" --root "${root}" "$@"
  fi

  require_interactive_panel
  if [ ! -t 0 ] && [ -r /dev/tty ]; then
    "${binary}" --root "${root}" < /dev/tty
    exit $?
  fi
  "${binary}" --root "${root}"
  exit $?
}

run_stacc() {
  local dry_run="$1"
  shift
  local -a stacc_args=("$@")
  local root
  local binary

  if root="$(local_stacc_root "${ROOT_DIR}")"; then
    if binary="$(local_built_stacc_bin "${root}")"; then
      run_local_binary_stacc "${root}" "${binary}" "${stacc_args[@]}"
    fi
    command -v cargo >/dev/null 2>&1 || die "no built stacc binary found under ${root}/target; install Rust from https://rustup.rs or build once with cargo build --release"
    if [ "${#stacc_args[@]}" -gt 0 ]; then
      exec cargo run --manifest-path "${root}/Cargo.toml" -- "${stacc_args[@]}"
    fi
    run_no_arg_local_stacc "${root}"
  fi

  if [ "${dry_run}" -eq 1 ]; then
    TMP_ROOT="$(mktemp -d)"
    if binary="$(download_release_binary "${TMP_ROOT}/release")"; then
      :
    else
      command -v cargo >/dev/null 2>&1 || die "no prebuilt stacc binary found and cargo is unavailable; install Rust from https://rustup.rs"
      cargo install --git "${REPO_URL}" --root "${TMP_ROOT}/cargo" --locked --force
      binary="${TMP_ROOT}/cargo/bin/$(binary_name)"
    fi
    if [ "${#stacc_args[@]}" -gt 0 ]; then
      STACC_BUNDLE_ROOT="${TMP_ROOT}/bundle" "${binary}" "${stacc_args[@]}"
      exit $?
    fi
    STACC_BUNDLE_ROOT="${TMP_ROOT}/bundle"
    export STACC_BUNDLE_ROOT
    run_no_arg_binary_stacc "${binary}"
  fi

  TMP_ROOT="$(mktemp -d)"
  if binary="$(download_release_binary "${TMP_ROOT}/release")"; then
    binary="$(install_release_binary "${binary}")"
  else
    command -v cargo >/dev/null 2>&1 || die "no prebuilt stacc binary found and cargo is unavailable; install Rust from https://rustup.rs"
    cargo install --git "${REPO_URL}" --locked --force
    binary="$(installed_stacc_bin)"
  fi
  if [ "${#stacc_args[@]}" -gt 0 ]; then
    exec "${binary}" "${stacc_args[@]}"
  fi
  run_no_arg_binary_stacc "${binary}"
}

is_direct_stacc_invocation() {
  [ $# -gt 0 ] || return 0
  if [ "$1" = "--root" ]; then
    [ $# -ge 3 ] || return 0
    shift 2
  elif [ "${1#--root=}" != "$1" ]; then
    [ $# -ge 2 ] || return 0
    shift
  fi
  case "$1" in
    status|install|sync-metadata|bootstrap|check|--panel|--config|--help|-h|--version|-V)
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
  local hooks=""
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
      --hooks)
        [ $# -ge 2 ] || die "--hooks requires a list"
        hooks="$2"
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
  local -a unique_editors=()
  local existing
  local duplicate
  local editor
  for editor in "${editors[@]}"; do
    duplicate=0
    if [ "${#unique_editors[@]}" -gt 0 ]; then
      for existing in "${unique_editors[@]}"; do
        if [ "${existing}" = "${editor}" ]; then
          duplicate=1
          break
        fi
      done
    fi
    if [ "${duplicate}" -eq 0 ]; then
      unique_editors+=("${editor}")
    fi
  done
  editors=("${unique_editors[@]}")
  scope="${scope:-${DEFAULT_SCOPE}}"
  categories="${categories:-${DEFAULT_CATEGORIES}}"

  if [ -n "${ROOT_DIR}" ]; then
    TRANSLATED_ARGS+=("--root" "${ROOT_DIR}")
  fi
  TRANSLATED_ARGS+=("install")
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
  append_csv_flags "--hook" "${hooks}"
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

  capture_root_arg "$@"

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
