#!/bin/bash
# Install git hooks for Scrybe project

HOOKS_DIR="$(cd "$(dirname "$0")" && pwd)"
GIT_DIR="$(git rev-parse --git-dir 2>/dev/null)"

if [ -z "$GIT_DIR" ]; then
    echo "Error: Not in a git repository"
    exit 1
fi

echo "Installing Scrybe git hooks..."

# Create hooks directory if it doesn't exist
mkdir -p "$GIT_DIR/hooks"

# Install pre-commit hook
if [ -f "$HOOKS_DIR/pre-commit" ]; then
    cp "$HOOKS_DIR/pre-commit" "$GIT_DIR/hooks/pre-commit"
    chmod +x "$GIT_DIR/hooks/pre-commit"
    echo "✓ Installed pre-commit hook"
else
    echo "✗ pre-commit hook not found"
    exit 1
fi

echo ""
echo "Git hooks installed successfully!"
echo ""
echo "To bypass hooks (not recommended):"
echo "  git commit --no-verify"
echo ""
echo "To uninstall hooks:"
echo "  rm $GIT_DIR/hooks/pre-commit"
