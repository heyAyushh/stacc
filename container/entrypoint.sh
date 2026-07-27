#!/usr/bin/env bash

set -euo pipefail

find /source \
  -mindepth 1 \
  -maxdepth 1 \
  ! -name .git \
  ! -name .next \
  ! -name docs \
  ! -name node_modules \
  ! -name target \
  -exec cp -R '{}' /workspace/ ';'

mkdir /workspace/docs
find /source/docs \
  -mindepth 1 \
  -maxdepth 1 \
  ! -name .next \
  ! -name node_modules \
  ! -name out \
  ! -name public \
  ! -name source \
  -exec cp -R '{}' /workspace/docs/ ';'

cd /workspace

case "${1:-}" in
  cargo | rustc | rustfmt | clippy-driver)
    cargo fetch --locked
    ;;
  node | npm | npx)
    npm ci
    ;;
  *)
    npm ci
    cargo fetch --locked
    ;;
esac

exec "$@"
