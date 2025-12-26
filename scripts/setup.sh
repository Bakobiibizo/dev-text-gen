#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

NO_CACHE=0
for arg in "$@"; do
  case "$arg" in
    --no-cache) NO_CACHE=1 ;;
    -h|--help)
      echo "Usage: $0 [--no-cache]" >&2
      exit 0
      ;;
    *)
      echo "[setup] error: unknown argument: $arg" >&2
      exit 2
      ;;
  esac
done

log() { echo "[setup] $*"; }
die() { echo "[setup] error: $*" >&2; exit 1; }

command -v docker >/dev/null 2>&1 || die "docker is required"
docker compose version >/dev/null 2>&1 || die "docker compose plugin is required (docker compose ...)"

log "Building container (dev-text-gen)..."
(
  cd "$ROOT_DIR"
  if [[ "$NO_CACHE" == "1" ]]; then
    docker compose build --no-cache dev-text-gen
  else
    docker compose build dev-text-gen
  fi
)

log "Starting container (dev-text-gen)..."
(
  cd "$ROOT_DIR"
  docker compose up -d dev-text-gen
)

# Install remembered missing deps (persisted from add-missing-dep.sh)
MISSING_REQ="$ROOT_DIR/scripts/missing-deps.txt"
if [[ -f "$MISSING_REQ" ]]; then
  log "Installing remembered missing deps from scripts/missing-deps.txt..."
  (
    cd "$ROOT_DIR"
    docker compose run --rm dev-text-gen bash -lc "pip install -r /workspace/scripts/missing-deps.txt"
  )
fi
