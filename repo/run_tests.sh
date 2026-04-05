#!/bin/bash

echo "============================================"
echo " CampusLearn Operations Suite - Test Runner"
echo "============================================"
echo ""

UNIT_PASS=0
UNIT_FAIL=0
API_PASS=0
API_FAIL=0

# --------------------------------------------------
# Unit Tests (no services required)
# --------------------------------------------------
echo ">>> Running Unit Tests..."
echo "--------------------------------------------"

UNIT_OUTPUT=$(python3 -m unittest discover -s unit_tests -p "test_*.py" -v 2>&1)
UNIT_EXIT=$?
echo "$UNIT_OUTPUT"

# Parse counts from unittest output
UNIT_RAN=$(echo "$UNIT_OUTPUT" | grep -oP 'Ran \K[0-9]+' | tail -1)
if [ "$UNIT_EXIT" -eq 0 ]; then
    UNIT_PASS=${UNIT_RAN:-0}
    UNIT_FAIL=0
    echo ""
    echo "PASS: Unit tests ($UNIT_PASS passed)"
else
    # Count failures from output
    UNIT_FAILURES=$(echo "$UNIT_OUTPUT" | grep -oP 'failures=\K[0-9]+' || echo "0")
    UNIT_ERRORS=$(echo "$UNIT_OUTPUT" | grep -oP 'errors=\K[0-9]+' || echo "0")
    UNIT_FAIL=$((${UNIT_FAILURES:-0} + ${UNIT_ERRORS:-0}))
    UNIT_PASS=$((${UNIT_RAN:-0} - UNIT_FAIL))
    echo ""
    echo "FAIL: Unit tests ($UNIT_PASS passed, $UNIT_FAIL failed)"
fi

echo ""

# --------------------------------------------------
# API Integration Tests (requires running backend)
# --------------------------------------------------
echo ">>> Running API Integration Tests..."
echo "--------------------------------------------"

API_BASE_URL="${API_BASE_URL:-http://localhost:8000}"
export API_BASE_URL

# Check if backend is reachable
if ! curl -sfk "$API_BASE_URL/health" > /dev/null 2>&1; then
    echo "WARNING: Backend not reachable at $API_BASE_URL"
    echo "SKIP: API tests skipped (start services with 'docker compose up' first)"
    API_PASS=0
    API_FAIL=0
    API_SKIPPED=true
else
    API_OUTPUT=$(python3 -m unittest discover -s API_tests -p "test_*.py" -v 2>&1)
    API_EXIT=$?
    echo "$API_OUTPUT"

    API_RAN=$(echo "$API_OUTPUT" | grep -oP 'Ran \K[0-9]+' | tail -1)
    if [ "$API_EXIT" -eq 0 ]; then
        API_PASS=${API_RAN:-0}
        API_FAIL=0
        echo ""
        echo "PASS: API tests ($API_PASS passed)"
    else
        API_FAILURES=$(echo "$API_OUTPUT" | grep -oP 'failures=\K[0-9]+' || echo "0")
        API_ERRORS=$(echo "$API_OUTPUT" | grep -oP 'errors=\K[0-9]+' || echo "0")
        API_FAIL=$((${API_FAILURES:-0} + ${API_ERRORS:-0}))
        API_PASS=$((${API_RAN:-0} - API_FAIL))
        echo ""
        echo "FAIL: API tests ($API_PASS passed, $API_FAIL failed)"
    fi
    API_SKIPPED=false
fi

echo ""
echo "============================================"
echo "              TEST SUMMARY"
echo "============================================"
TOTAL_PASS=$((UNIT_PASS + API_PASS))
TOTAL_FAIL=$((UNIT_FAIL + API_FAIL))
TOTAL=$((TOTAL_PASS + TOTAL_FAIL))

echo "  Unit Tests:  $UNIT_PASS passed, $UNIT_FAIL failed"
if [ "$API_SKIPPED" = true ]; then
    echo "  API Tests:   SKIPPED (backend not running)"
else
    echo "  API Tests:   $API_PASS passed, $API_FAIL failed"
fi
echo "  ─────────────────────────────────"
echo "  Total:       $TOTAL_PASS passed, $TOTAL_FAIL failed (of $TOTAL)"
echo "============================================"

if [ "$TOTAL_FAIL" -gt 0 ]; then
    echo "  RESULT: SOME TESTS FAILED"
    exit 1
else
    echo "  RESULT: ALL TESTS PASSED"
    exit 0
fi
