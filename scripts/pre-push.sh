#!/usr/bin/env bash
# Pre-push hook for Apollo
# Install: cp scripts/pre-push.sh .git/hooks/pre-push && chmod +x .git/hooks/pre-push

set -e

echo "Running pre-push checks..."

# Run all pre-commit checks first
./scripts/pre-commit.sh

# Run tests
echo "Running tests..."
cargo test --workspace
if [ $? -ne 0 ]; then
    echo "Error: Tests failed."
    exit 1
fi

echo "Pre-push checks passed!"
