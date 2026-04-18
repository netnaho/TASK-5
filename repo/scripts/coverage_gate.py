#!/usr/bin/env python3
"""Enforce API endpoint test coverage gate.

Parses backend Rocket route macros from ``backend/src/routes/*.rs`` and HTTP
calls from ``API_tests/*.py``. Produces a normalized set of endpoints for
each and prints the coverage percentage. Fails (non-zero exit) if below
the --min threshold.

Endpoint normalization:
  - Rocket path params like `<uuid>` / `<approval_uuid>` are replaced with
    the placeholder `:param`.
  - Python test paths use f-strings or concrete values; each segment that
    looks like a UUID or a numeric id is normalized to `:param` too.

Usage:
    python scripts/coverage_gate.py --min 95.00
"""
from __future__ import annotations

import argparse
import os
import re
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
ROUTES_DIR = REPO_ROOT / "backend" / "src" / "routes"
API_TESTS_DIR = REPO_ROOT / "API_tests"
MAIN_RS = REPO_ROOT / "backend" / "src" / "main.rs"

HTTP_METHODS = ("GET", "POST", "PUT", "DELETE", "PATCH")

UUID_RE = re.compile(r"^[0-9a-fA-F-]{8,}$")


def _normalize_path(path: str) -> str:
    # Drop any query string — we only compare method + path shape
    if "?" in path:
        path = path.split("?", 1)[0]
    out = []
    for seg in path.split("/"):
        if not seg:
            out.append(seg)
            continue
        # Rocket-style <x> -> :param
        if seg.startswith("<") and seg.endswith(">"):
            out.append(":param")
            continue
        # uuid-shaped segment
        if UUID_RE.match(seg) and "-" in seg and len(seg) >= 20:
            out.append(":param")
            continue
        # numeric segment
        if seg.isdigit():
            out.append(":param")
            continue
        # python f-string placeholder {self.uuid}
        if seg.startswith("{") and seg.endswith("}"):
            out.append(":param")
            continue
        out.append(seg)
    norm = "/".join(out)
    if not norm.startswith("/"):
        norm = "/" + norm
    return norm.rstrip("/") or "/"


def _parse_mount_points() -> dict[str, str]:
    """Return {module_name: mount_prefix} from backend/src/main.rs."""
    mounts = {}
    # Special-case: health is mounted at "/"
    txt = MAIN_RS.read_text()
    for m in re.finditer(
        r'\.mount\("([^"]+)",\s*routes::(\w+)::routes\(\)\)', txt
    ):
        prefix, module = m.group(1), m.group(2)
        mounts[module] = prefix
    return mounts


_ROUTE_MACRO_RE = re.compile(
    r'#\[(get|post|put|delete|patch)\(\s*"([^"]+)"',
    re.IGNORECASE,
)


def collect_backend_endpoints() -> set[tuple[str, str]]:
    mounts = _parse_mount_points()
    endpoints: set[tuple[str, str]] = set()
    for rs_file in sorted(ROUTES_DIR.glob("*.rs")):
        module = rs_file.stem
        prefix = mounts.get(module, "")
        text = rs_file.read_text()
        for m in _ROUTE_MACRO_RE.finditer(text):
            method = m.group(1).upper()
            route_path = m.group(2)
            full = prefix.rstrip("/") + "/" + route_path.lstrip("/")
            endpoints.add((method, _normalize_path(full)))
    return endpoints


_API_CALL_REGEXES = [
    # api_request("METHOD", "/path", ...)
    re.compile(
        r'api_request\(\s*["\'](GET|POST|PUT|DELETE|PATCH)["\']\s*,\s*f?["\']([^"\']+)["\']',
        re.IGNORECASE,
    ),
    # api("METHOD", "/path", ...)
    re.compile(
        r'\bapi\(\s*["\'](GET|POST|PUT|DELETE|PATCH)["\']\s*,\s*f?["\']([^"\']+)["\']',
        re.IGNORECASE,
    ),
    # api_request("METHOD", PATH, ...) — PATH constant captured separately below
    re.compile(
        r'api_request\(\s*["\'](GET|POST|PUT|DELETE|PATCH)["\']\s*,\s*([A-Z_][A-Z0-9_]*)\b',
    ),
    # _try_paths("METHOD", ...) - tag-specific helper; infer path from TAG_PATHS constant ("/api/v1/tags/")
    re.compile(
        r'_try_paths\(\s*["\'](GET|POST|PUT|DELETE|PATCH)["\']',
        re.IGNORECASE,
    ),
    # f"{BASE_URL}/path"  (with optional method= kw discovered in nearby lines)
    re.compile(
        r'f["\']\{BASE_URL\}(/[A-Za-z0-9_/\-{}:]+)["\']',
    ),
]


def _collect_url_constants(text: str) -> dict[str, str]:
    """Capture simple module-level constants like PATH = "/api/v1/foo"."""
    out: dict[str, str] = {}
    for m in re.finditer(
        r'^([A-Z_][A-Z0-9_]*)\s*=\s*["\'](/[^"\']+)["\']',
        text,
        re.MULTILINE,
    ):
        out[m.group(1)] = m.group(2)
    return out


_REQUEST_METHOD_RE = re.compile(
    r'method\s*=\s*["\'](GET|POST|PUT|DELETE|PATCH)["\']',
    re.IGNORECASE,
)


def collect_tested_endpoints() -> set[tuple[str, str]]:
    tested: set[tuple[str, str]] = set()
    for py in sorted(API_TESTS_DIR.glob("test_*.py")):
        text = py.read_text()
        const_paths = _collect_url_constants(text)

        # Pattern 1 & 2 — explicit path in call
        for rx in _API_CALL_REGEXES[:2]:
            for m in rx.finditer(text):
                method = m.group(1).upper()
                path = m.group(2)
                if not path.startswith("/"):
                    continue
                tested.add((method, _normalize_path(path)))

        # Pattern 3 — api_request("METHOD", CONSTANT_NAME, ...)
        for m in _API_CALL_REGEXES[2].finditer(text):
            method = m.group(1).upper()
            const_name = m.group(2)
            path = const_paths.get(const_name)
            if path:
                tested.add((method, _normalize_path(path)))

        # Pattern 4 — tag-specific helper uses /api/v1/tags
        if "_try_paths(" in text and "TAG_PATHS" in text:
            for m in _API_CALL_REGEXES[3].finditer(text):
                method = m.group(1).upper()
                tested.add((method, "/api/v1/tags"))

        # Pattern 5 — raw f-string URL plus sibling method= kw
        f_urls = list(_API_CALL_REGEXES[4].finditer(text))
        for m in f_urls:
            path = m.group(1)
            # Find a method= kw within 4 lines after the URL
            start = m.end()
            window = text[start : start + 400]
            mm = _REQUEST_METHOD_RE.search(window)
            method = (mm.group(1).upper() if mm else "GET")
            tested.add((method, _normalize_path(path)))
    return tested


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--min",
        type=float,
        default=95.0,
        help="Minimum required coverage percentage (strictly greater than).",
    )
    parser.add_argument("--verbose", action="store_true")
    args = parser.parse_args()

    backend = collect_backend_endpoints()
    tested = collect_tested_endpoints()

    covered = {ep for ep in backend if ep in tested}
    uncovered = sorted(backend - tested)

    total = len(backend)
    if total == 0:
        print("ERROR: zero backend endpoints discovered — parsing failure", file=sys.stderr)
        return 2

    pct = (len(covered) / total) * 100.0

    print("=" * 60)
    print(" API Endpoint Coverage Report")
    print("=" * 60)
    print(f"  Backend endpoints discovered : {total}")
    print(f"  Endpoints with API tests     : {len(covered)}")
    print(f"  Uncovered endpoints          : {len(uncovered)}")
    print(f"  Coverage percentage          : {pct:.2f}%")
    print(f"  Required threshold (>)        : {args.min:.2f}%")
    print("=" * 60)

    if args.verbose or uncovered:
        if uncovered:
            print("Uncovered endpoints:")
            for method, path in uncovered:
                print(f"  - {method} {path}")

    if pct > args.min:
        print(f"PASS: coverage {pct:.2f}% > {args.min:.2f}%")
        return 0
    else:
        print(f"FAIL: coverage {pct:.2f}% not strictly greater than {args.min:.2f}%", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
