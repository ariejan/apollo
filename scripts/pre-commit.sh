#!/usr/bin/env bash
# Pre-commit hook for Apollo
# Install: cp scripts/pre-commit.sh .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit
# Or use: pre-commit install

set -e

echo "Running pre-commit checks..."

# Format check
echo "Checking formatting..."
cargo fmt --all --check
if [ $? -ne 0 ]; then
    echo "Error: Code is not formatted. Run 'cargo fmt' to fix."
    exit 1
fi

# Clippy
echo "Running clippy..."
cargo clippy --workspace --all-targets -- -D warnings
if [ $? -ne 0 ]; then
    echo "Error: Clippy found issues."
    exit 1
fi

echo "Pre-commit checks passed!"
