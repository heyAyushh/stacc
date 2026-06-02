#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CHECK_ROOT="${ROOT_DIR}/target/stacc-check"
CARGO_ROOT="${CHECK_ROOT}/cargo-root"
BUNDLE_ROOT="${CHECK_ROOT}/bundle"

cd "${ROOT_DIR}"
mkdir -p "${CHECK_ROOT}"

print_command() {
  printf '\n==>'
  for argument in "$@"; do
    printf ' %q' "${argument}"
  done
  printf '\n'
}

run() {
  print_command "$@"
  "$@"
}

run_if_available() {
  if command -v "$1" >/dev/null 2>&1; then
    run "$@"
    return
  fi

  printf '\n==> skipping %s; command not found\n' "$1"
}

run cargo fmt --all -- --check
run cargo test
run cargo clippy --workspace --all-targets --all-features -- -D warnings
run bash -n install.sh
run_if_available shellcheck -x install.sh
run jq empty configs/mcps/mcp.json configs/stacc-panel.json configs/metadata/skills.lock.json
run cargo install --path . --root "${CARGO_ROOT}" --locked --force

printf '\n==> installed binary smoke checks\n'
STACC_BUNDLE_ROOT="${BUNDLE_ROOT}" "${CARGO_ROOT}/bin/stacc" status --json >/dev/null
STACC_BUNDLE_ROOT="${BUNDLE_ROOT}" "${CARGO_ROOT}/bin/stacc" install \
  --editor codex \
  --scope global \
  --category rules \
  --category skills \
  --category mcps \
  --mcp-server github \
  --dry-run \
  --print-plan >/dev/null

printf '\nAll checks passed.\n'
