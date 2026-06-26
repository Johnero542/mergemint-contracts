#!/usr/bin/env bash
set -euo pipefail

HOOK_DIR="$(git rev-parse --git-dir)/hooks"
PRE_COMMIT="$HOOK_DIR/pre-commit"

cat > "$PRE_COMMIT" << 'EOF'
#!/usr/bin/env bash
set -euo pipefail

echo "Running cargo fmt --check..."
if ! cargo fmt --check; then
  echo ""
  echo "ERROR: Formatting check failed. Run 'cargo fmt' to fix and re-stage your changes."
  exit 1
fi

echo "Running cargo clippy..."
if ! cargo clippy -- -D warnings; then
  echo ""
  echo "ERROR: Clippy found warnings (treated as errors). Fix them and re-stage your changes."
  exit 1
fi

echo "Pre-commit checks passed."
EOF

chmod +x "$PRE_COMMIT"
echo "Pre-commit hook installed at $PRE_COMMIT"
