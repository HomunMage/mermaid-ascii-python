#!/usr/bin/env bash
# Generate output for all examples and verify against .expect.txt files.
# Usage: bash examples/gen.sh          # generate only
#        bash examples/gen.sh --check  # generate + verify against .expect.txt

set -euo pipefail
cd "$(dirname "$0")/.."

CHECK=false
if [[ "${1:-}" == "--check" ]]; then
    CHECK=true
fi

# Build Rust binary (release)
echo "Building Rust binary..."
cargo build --release
BINARY="./target/release/mermaid-ascii"

# Remove old generated files
find examples -name '*.out.txt' -delete 2>/dev/null || true

# Process all .mm.md files recursively
find examples -name '*.mm.md' | sort | while read -r src; do
    out="${src%.mm.md}.out.txt"
    echo "  $src -> $out"
    "$BINARY" "$src" -o "$out"
done

echo ""
echo "Done. Generated output:"
find examples -name '*.out.txt' | sort | while read -r f; do
    echo "  $f"
done

# Verify against .expect.txt files
if $CHECK; then
    echo ""
    echo "Checking against .expect.txt files..."
    FAIL=0
    for expect in examples/*.expect.txt; do
        base="${expect%.expect.txt}"
        out="${base}.out.txt"
        if [[ ! -f "$out" ]]; then
            echo "  MISSING: $out (no output generated for $expect)"
            FAIL=1
            continue
        fi
        if diff -q "$expect" "$out" > /dev/null 2>&1; then
            echo "  OK: $(basename "$base")"
        else
            echo "  FAIL: $(basename "$base")"
            diff --color=auto "$expect" "$out" || true
            FAIL=1
        fi
    done
    if [[ $FAIL -ne 0 ]]; then
        echo ""
        echo "FAILED: some outputs differ from .expect.txt files"
        exit 1
    fi
    echo ""
    echo "All outputs match .expect.txt files."
fi
