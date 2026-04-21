#!/usr/bin/env bash
# Run the reference test suite and require the failing set to be exactly
# what tests/known_failures.txt pins.
#
# Exits 0 iff the set of failing tests equals the baseline.
# Exits 1 if any test regresses (fails that wasn't known-failing) OR if a
# previously-failing test now passes (the baseline must be updated before
# the CI can stay green — forcing us to notice fixture drift both ways).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BASELINE="${REPO_ROOT}/tests/known_failures.txt"

if [[ ! -f "${BASELINE}" ]]; then
    echo "::error::Baseline file missing: ${BASELINE}"
    exit 1
fi

# Ensure the native graphviz-anywhere lib is on the dynamic linker path.
# On macOS, DYLD_LIBRARY_PATH is stripped by SIP when bash nests; set it
# inside the script based on GRAPHVIZ_ANYWHERE_DIR so locals and CI alike
# get a consistent environment.
if [[ -n "${GRAPHVIZ_ANYWHERE_DIR:-}" ]]; then
    case "$(uname -s)" in
        Darwin) export DYLD_LIBRARY_PATH="${GRAPHVIZ_ANYWHERE_DIR}/lib${DYLD_LIBRARY_PATH:+:${DYLD_LIBRARY_PATH}}" ;;
        Linux)  export LD_LIBRARY_PATH="${GRAPHVIZ_ANYWHERE_DIR}/lib${LD_LIBRARY_PATH:+:${LD_LIBRARY_PATH}}" ;;
    esac
fi

results=$(mktemp)
trap 'rm -f "${results}"' EXIT

# Run cargo test, capturing both stdout and stderr. We MUST NOT
# short-circuit on cargo's non-zero exit — we need to inspect the
# failure set regardless.
set +e
cargo test --test reference_tests >"${results}" 2>&1
cargo_exit=$?
set -e
cat "${results}"

# Extract the failure set from cargo test output. Failing tests are listed
# under "failures:" as indented lines.
actual=$(mktemp)
trap 'rm -f "${results}" "${actual}"' EXIT
awk '/^failures:$/ {flag=1; next} flag && /^    [a-zA-Z_0-9]+$/ {print $1} /^test result:/ {flag=0}' \
    "${results}" | sort -u > "${actual}"

# Strip comments + blank lines from baseline for the comparison.
expected=$(mktemp)
trap 'rm -f "${results}" "${actual}" "${expected}"' EXIT
grep -vE '^\s*(#|$)' "${BASELINE}" | sort -u > "${expected}"

echo ""
echo "=== Reference test failure check ==="
echo "Baseline (${BASELINE}): $(wc -l < "${expected}" | tr -d ' ') expected failures"
echo "Actual run: $(wc -l < "${actual}" | tr -d ' ') failures"

new_regressions=$(comm -23 "${actual}" "${expected}")
newly_passing=$(comm -13 "${actual}" "${expected}")

status=0
if [[ -n "${new_regressions}" ]]; then
    echo ""
    echo "::error::NEW REGRESSIONS (tests failing that weren't known):"
    echo "${new_regressions}" | sed 's/^/  - /'
    status=1
fi

if [[ -n "${newly_passing}" ]]; then
    echo ""
    echo "::error::UNEXPECTEDLY PASSING (tests in baseline that now pass — update baseline):"
    echo "${newly_passing}" | sed 's/^/  - /'
    status=1
fi

if [[ ${status} -eq 0 ]]; then
    echo ""
    echo "OK: failing set matches baseline exactly (cargo exit was ${cargo_exit})."
fi

exit ${status}
