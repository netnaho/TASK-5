#!/usr/bin/env bash
# =============================================================================
# CampusLearn Operations Suite — Docker-contained test runner.
#
# Strict requirements honored by this script:
#   1. No host runtime deps: python3, cargo, trunk, node, etc. ARE NOT USED.
#      All commands execute inside containers we launch via `docker` / `docker
#      compose`. Only the host `docker` CLI and `docker compose` plugin are
#      required on the developer machine.
#   2. `set -Eeuo pipefail` — abort on any failure.
#   3. Trap errors, dump useful diagnostics, and reliably clean up.
#   4. Enforces API endpoint coverage strictly greater than 95.00%.
# =============================================================================
set -Eeuo pipefail

# -----------------------------------------------------------------------------
# Configuration (can be overridden by env)
# -----------------------------------------------------------------------------
COVERAGE_MIN="${COVERAGE_MIN:-95.00}"
RUST_IMAGE="${RUST_IMAGE:-rust:1.85-slim}"
TESTER_IMAGE="${TESTER_IMAGE:-campuslearn-test-runner:local}"
TESTER_DOCKERFILE="scripts/test-runner.Dockerfile"
TESTER_CONTEXT="scripts"
COMPOSE_PROJECT_NAME="${COMPOSE_PROJECT_NAME:-campuslearn}"
export COMPOSE_PROJECT_NAME

KEEP_SERVICES="${KEEP_SERVICES:-1}"      # 1: leave compose up on exit; 0: tear down
BACKEND_URL_IN_CONTAINER="http://backend:8000"

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
cd "$SCRIPT_DIR"

# -----------------------------------------------------------------------------
# Pretty printing & failure reporting
# -----------------------------------------------------------------------------
log()   { printf "[run_tests] %s\n" "$*"; }
warn()  { printf "[run_tests][WARN] %s\n" "$*" >&2; }
fatal() { printf "[run_tests][FATAL] %s\n" "$*" >&2; }

UNIT_EXIT=0
FRONTEND_RS_EXIT=0
API_EXIT=0
COVERAGE_EXIT=0

_dump_diag() {
    local code=$?
    echo ""
    echo "============================================================"
    echo "  FAILURE — collecting diagnostics (exit=$code)"
    echo "============================================================"
    if docker compose -p "$COMPOSE_PROJECT_NAME" ps >/dev/null 2>&1; then
        echo "--- docker compose ps ---"
        docker compose -p "$COMPOSE_PROJECT_NAME" ps || true
        echo "--- backend logs (tail 100) ---"
        docker compose -p "$COMPOSE_PROJECT_NAME" logs --tail=100 backend 2>&1 || true
        echo "--- mysql logs (tail 40) ---"
        docker compose -p "$COMPOSE_PROJECT_NAME" logs --tail=40 mysql 2>&1 || true
    fi
    return $code
}

_cleanup() {
    local code=$?
    if [[ "$KEEP_SERVICES" != "1" ]]; then
        log "Tearing down compose project '$COMPOSE_PROJECT_NAME' (KEEP_SERVICES=0)..."
        docker compose -p "$COMPOSE_PROJECT_NAME" down --remove-orphans >/dev/null 2>&1 || true
    else
        log "Leaving compose project running (KEEP_SERVICES=1)."
    fi
    return $code
}

trap '_dump_diag' ERR
trap '_cleanup' EXIT

# -----------------------------------------------------------------------------
# Preconditions
# -----------------------------------------------------------------------------
command -v docker >/dev/null 2>&1 || {
    fatal "Docker CLI not found. Install Docker Desktop or docker-ce."
    exit 2
}
docker compose version >/dev/null 2>&1 || {
    fatal "'docker compose' plugin not available. Install Docker Compose v2."
    exit 2
}

# -----------------------------------------------------------------------------
# 1. Build test-runner image (python + docker CLI)
# -----------------------------------------------------------------------------
log "Building test-runner image ($TESTER_IMAGE) …"
docker build \
    --quiet \
    -t "$TESTER_IMAGE" \
    -f "$TESTER_DOCKERFILE" \
    "$TESTER_CONTEXT" \
    >/dev/null

# -----------------------------------------------------------------------------
# 2. Bring up stack (mysql, backend, frontend) and wait for health
# -----------------------------------------------------------------------------
log "Starting docker compose services …"
docker compose -p "$COMPOSE_PROJECT_NAME" up -d --wait --wait-timeout 240

NETWORK_NAME="${COMPOSE_PROJECT_NAME}_campuslearn"
if ! docker network inspect "$NETWORK_NAME" >/dev/null 2>&1; then
    fatal "Expected compose network '$NETWORK_NAME' not found."
    exit 2
fi

# -----------------------------------------------------------------------------
# 3. Unit tests (Python unittest inside tester container)
#    — excludes the frontend_rs crate, which runs in a Rust container below.
# -----------------------------------------------------------------------------
log "Running unit tests …"
set +e
docker run --rm \
    --network "$NETWORK_NAME" \
    -v "$SCRIPT_DIR:/work" -w /work \
    -v /var/run/docker.sock:/var/run/docker.sock \
    -e PYTHONDONTWRITEBYTECODE=1 \
    "$TESTER_IMAGE" \
    python -m unittest discover -s unit_tests/backend -p "test_*.py" -v
UNIT_BACKEND_EXIT=$?

docker run --rm \
    --network "$NETWORK_NAME" \
    -v "$SCRIPT_DIR:/work" -w /work \
    -e PYTHONDONTWRITEBYTECODE=1 \
    "$TESTER_IMAGE" \
    python -m unittest discover -s unit_tests/frontend -p "test_*.py" -v
UNIT_FRONTEND_EXIT=$?
set -e
UNIT_EXIT=$(( UNIT_BACKEND_EXIT | UNIT_FRONTEND_EXIT ))

# -----------------------------------------------------------------------------
# 4. Frontend Rust unit tests (imports real frontend/src/* via #[path])
# -----------------------------------------------------------------------------
log "Running frontend Rust tests (cargo test in $RUST_IMAGE) …"
mkdir -p .cargo-cache-frontend-tests .cargo-target-frontend-tests
set +e
docker run --rm \
    -v "$SCRIPT_DIR:/work" -w /work/unit_tests/frontend_rs \
    -v "$SCRIPT_DIR/.cargo-cache-frontend-tests:/usr/local/cargo/registry" \
    -v "$SCRIPT_DIR/.cargo-target-frontend-tests:/target" \
    -e CARGO_TARGET_DIR=/target \
    "$RUST_IMAGE" \
    cargo test --manifest-path Cargo.toml --color never
FRONTEND_RS_EXIT=$?
set -e

# -----------------------------------------------------------------------------
# 5. API integration tests against running backend
# -----------------------------------------------------------------------------
log "Running API integration tests …"
set +e
docker run --rm \
    --network "$NETWORK_NAME" \
    -v "$SCRIPT_DIR:/work" -w /work \
    -v /var/run/docker.sock:/var/run/docker.sock \
    -e API_BASE_URL="$BACKEND_URL_IN_CONTAINER" \
    -e PYTHONDONTWRITEBYTECODE=1 \
    "$TESTER_IMAGE" \
    python -m unittest discover -s API_tests -p "test_*.py" -v
API_EXIT=$?
set -e

# -----------------------------------------------------------------------------
# 6. Coverage gate (>95%)
# -----------------------------------------------------------------------------
log "Enforcing API endpoint coverage gate (> ${COVERAGE_MIN}%) …"
set +e
docker run --rm \
    -v "$SCRIPT_DIR:/work" -w /work \
    "$TESTER_IMAGE" \
    python scripts/coverage_gate.py --min "$COVERAGE_MIN" --verbose
COVERAGE_EXIT=$?
set -e

# -----------------------------------------------------------------------------
# 7. Summary
# -----------------------------------------------------------------------------
echo ""
echo "============================================================"
echo "                      TEST SUMMARY"
echo "============================================================"
printf "  Unit tests (Python)          : %s\n" "$([[ $UNIT_EXIT -eq 0 ]] && echo PASS || echo FAIL)"
printf "  Frontend Rust tests          : %s\n" "$([[ $FRONTEND_RS_EXIT -eq 0 ]] && echo PASS || echo FAIL)"
printf "  API integration tests        : %s\n" "$([[ $API_EXIT -eq 0 ]] && echo PASS || echo FAIL)"
printf "  Coverage gate (> %s%%)      : %s\n" "$COVERAGE_MIN" "$([[ $COVERAGE_EXIT -eq 0 ]] && echo PASS || echo FAIL)"
echo "============================================================"

TOTAL_FAIL=$(( UNIT_EXIT != 0 ? 1 : 0 ))
TOTAL_FAIL=$(( TOTAL_FAIL + (FRONTEND_RS_EXIT != 0 ? 1 : 0) ))
TOTAL_FAIL=$(( TOTAL_FAIL + (API_EXIT != 0 ? 1 : 0) ))
TOTAL_FAIL=$(( TOTAL_FAIL + (COVERAGE_EXIT != 0 ? 1 : 0) ))

if [[ $TOTAL_FAIL -ne 0 ]]; then
    fatal "$TOTAL_FAIL test stage(s) failed."
    exit 1
fi
log "ALL TESTS PASSED"
exit 0
