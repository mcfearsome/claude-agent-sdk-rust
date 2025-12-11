#!/bin/bash
# Install git hooks for Claude SDK development

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "🪝 Installing git hooks..."
echo ""

# Check if we're in a git repository
if [ ! -d .git ]; then
    echo "❌ Error: Not in a git repository root"
    echo "   Run this script from the project root directory"
    exit 1
fi

# Check if hooks directory exists in repo
if [ ! -d hooks ]; then
    echo "❌ Error: hooks/ directory not found"
    exit 1
fi

# Install pre-commit hook
if [ -f hooks/pre-commit ]; then
    cp hooks/pre-commit .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    echo -e "${GREEN}✓${NC} Installed pre-commit hook"
else
    echo -e "${YELLOW}⚠${NC}  pre-commit hook not found in hooks/"
fi

echo ""
echo -e "${GREEN}✅ Git hooks installed successfully!${NC}"
echo ""
echo "The pre-commit hook will run:"
echo "  • cargo test"
echo "  • cargo clippy"
echo "  • cargo fmt --check"
echo "  • cargo doc"
echo ""
echo "To skip the hook (not recommended), use: git commit --no-verify"
