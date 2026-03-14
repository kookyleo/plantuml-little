#!/usr/bin/env bash
# Generate reference SVG reference files from the original Java PlantUML.
#
# Prerequisites:
#   - java on PATH
#   - plantuml.jar built (see PLANTUML_JAR below)
#   - dot (graphviz) on PATH (same version used by plantuml-little)
#
# Usage:
#   bash tests/generate_reference.sh [plantuml.jar]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
FIXTURES_DIR="$SCRIPT_DIR/fixtures"
GOLDEN_DIR="$SCRIPT_DIR/reference"

# Resolve plantuml.jar location
if [[ $# -ge 1 ]]; then
    PLANTUML_JAR="$1"
else
    # Default: look for the built jar in the sibling plantuml project
    PLANTUML_JAR="$(find /ext/plantuml/plantuml/build/libs -name 'plantuml-*.jar' \
        ! -name '*-sources*' ! -name '*-javadoc*' 2>/dev/null | head -1)"
    if [[ -z "$PLANTUML_JAR" ]]; then
        echo "ERROR: plantuml.jar not found. Build it first:" >&2
        echo "  cd /ext/plantuml/plantuml && ./gradlew shadowJar" >&2
        echo "Or pass the path as argument:" >&2
        echo "  bash tests/generate_reference.sh /path/to/plantuml.jar" >&2
        exit 1
    fi
fi

echo "Using plantuml.jar: $PLANTUML_JAR"
echo "Fixtures dir: $FIXTURES_DIR"
echo "Reference dir: $GOLDEN_DIR"

# Record environment versions
mkdir -p "$GOLDEN_DIR"
cat > "$GOLDEN_DIR/VERSION" <<EOF
plantuml_jar: $PLANTUML_JAR
plantuml_git: $(cd /ext/plantuml/plantuml && git rev-parse HEAD 2>/dev/null || echo "unknown")
java_version: $(java -version 2>&1 | head -1)
dot_version: $(dot -V 2>&1 || echo "unknown")
generated_at: $(date -u +%Y-%m-%dT%H:%M:%SZ)
EOF

echo "---"
cat "$GOLDEN_DIR/VERSION"
echo "---"

# Count for progress
total=0
success=0
failed=0
skipped=0

# Generate reference SVGs
find "$FIXTURES_DIR" -name '*.puml' -type f | sort | while read -r puml; do
    # Compute relative path: fixtures/class/foo.puml -> class/foo
    rel="${puml#$FIXTURES_DIR/}"
    category="$(dirname "$rel")"
    basename="$(basename "$rel" .puml)"

    # Create output directory
    out_dir="$GOLDEN_DIR/$category"
    mkdir -p "$out_dir"

    out_svg="$out_dir/$basename.svg"

    total=$((total + 1))

    # Run plantuml, capture output directly
    if java -jar "$PLANTUML_JAR" -tsvg -pipe < "$puml" > "$out_svg" 2>/dev/null; then
        # Verify it produced valid SVG
        if grep -q '<svg' "$out_svg"; then
            success=$((success + 1))
        else
            echo "WARN: $rel - no <svg> in output, removing"
            rm -f "$out_svg"
            skipped=$((skipped + 1))
        fi
    else
        echo "FAIL: $rel"
        rm -f "$out_svg"
        failed=$((failed + 1))
    fi
done

echo ""
echo "Done. Generated reference SVGs in: $GOLDEN_DIR"
echo "Check $GOLDEN_DIR/VERSION for environment details."
