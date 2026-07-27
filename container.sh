#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly ROOT_DIR
readonly IMAGE="${STACC_CONTAINER_IMAGE:-stacc-dev:latest}"
readonly CPUS="${STACC_CONTAINER_CPUS:-4}"
readonly MEMORY="${STACC_CONTAINER_MEMORY:-8g}"

usage() {
  cat <<'USAGE'
Usage: ./container.sh <command>

Commands:
  build            Build the STACC development image
  check            Run the full repository gate in the image
  docs             Build the documentation in the image
  shell            Open an interactive shell in the image
  run <command>    Run another command in the image
USAGE
}

require_runtime() {
  if ! command -v container >/dev/null 2>&1; then
    echo "Apple container is not installed: https://github.com/apple/container" >&2
    exit 1
  fi

  if ! container system status >/dev/null 2>&1; then
    echo "Apple container is not running. Start it with: container system start" >&2
    exit 1
  fi
}

build_image() {
  container build \
    --cpus "${CPUS}" \
    --memory "${MEMORY}" \
    --tag "${IMAGE}" \
    --file container/Dockerfile \
    container
}

run_image() {
  container run \
    --rm \
    --cpus "${CPUS}" \
    --memory "${MEMORY}" \
    --mount "type=bind,source=${ROOT_DIR},target=/source,readonly" \
    "${IMAGE}" \
    "$@"
}

main() {
  require_runtime
  cd "${ROOT_DIR}"

  case "${1:-}" in
    build)
      build_image
      ;;
    check)
      build_image
      run_image cargo run -- check
      ;;
    docs)
      build_image
      run_image npm run docs:build
      ;;
    shell)
      build_image
      container run \
        --rm \
        --cpus "${CPUS}" \
        --memory "${MEMORY}" \
        --tty \
        --interactive \
        --mount "type=bind,source=${ROOT_DIR},target=/source,readonly" \
        "${IMAGE}" \
        bash
      ;;
    run)
      shift
      if [[ "$#" -eq 0 ]]; then
        usage
        exit 1
      fi
      build_image
      run_image "$@"
      ;;
    *)
      usage
      exit 1
      ;;
  esac
}

main "$@"
