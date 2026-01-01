#!/bin/bash
# Spacey Crates Publishing Script
#
# Publishes all Spacey crates to crates.io in the correct dependency order.
# Excludes spacey-browser which is distributed as a standalone application.
#
# Usage:
#   ./scripts/publish.sh          # Dry run (default)
#   ./scripts/publish.sh --dry-run # Dry run
#   ./scripts/publish.sh --publish # Actually publish
#
# Prerequisites:
#   - cargo login (must be authenticated with crates.io)
#   - All tests passing
#   - Version bumped in Cargo.toml

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

# Default to dry run
DRY_RUN=true
SKIP_TESTS=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --publish)
            DRY_RUN=false
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --dry-run     Simulate publishing (default)"
            echo "  --publish     Actually publish to crates.io"
            echo "  --skip-tests  Skip running tests before publishing"
            echo "  --help, -h    Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            exit 1
            ;;
    esac
done

# Header
echo -e "${CYAN}"
echo "╔════════════════════════════════════════════════════════════╗"
echo "║              🚀 Spacey Crates Publisher 🚀                ║"
echo "╠════════════════════════════════════════════════════════════╣"
if [ "$DRY_RUN" = true ]; then
echo "║                    🧪 DRY RUN MODE 🧪                      ║"
else
echo "║                 🔥 PUBLISHING TO CRATES.IO 🔥             ║"
fi
echo "╚════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

cd "$ROOT_DIR"

# Get version from workspace
VERSION=$(grep -E "^version = " Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo -e "${BLUE}📦 Version: ${VERSION}${NC}"
echo ""

# Crates to publish in dependency order
# Layer 1: No dependencies
LAYER_1=(
    "crates/spacey-macros"
)

# Layer 2: Depends on Layer 1 only or no internal deps
LAYER_2=(
    "crates/spacey-spidermonkey"
    "crates/spacey-npm"
)

# Layer 3: Depends on Layer 2
LAYER_3=(
    "crates/spacey-servo"
    "crates/spacey-node"
)

# Layer 4: Root crate
LAYER_4=(
    "."
)

# Excluded crates (not published)
EXCLUDED=(
    "crates/spacey-browser"
)

echo -e "${YELLOW}📋 Publishing order:${NC}"
echo -e "   Layer 1: ${LAYER_1[*]}"
echo -e "   Layer 2: ${LAYER_2[*]}"
echo -e "   Layer 3: ${LAYER_3[*]}"
echo -e "   Layer 4: ${LAYER_4[*]}"
echo -e "   ${RED}Excluded: ${EXCLUDED[*]}${NC}"
echo ""

# Function to publish a crate
publish_crate() {
    local crate_path=$1
    local crate_name=$(basename "$crate_path")
    
    # Handle root crate
    if [ "$crate_path" = "." ]; then
        crate_name="spacey"
    fi
    
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}📦 Publishing: ${crate_name}${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    cd "$ROOT_DIR/$crate_path"
    
    # Check if crate is publishable
    if grep -q 'publish = false' Cargo.toml 2>/dev/null; then
        echo -e "${YELLOW}⏭️  Skipping ${crate_name} (publish = false)${NC}"
        cd "$ROOT_DIR"
        return 0
    fi
    
    # Build check
    echo -e "${BLUE}🔨 Checking build...${NC}"
    cargo check --all-features
    
    if [ "$DRY_RUN" = true ]; then
        echo -e "${YELLOW}🧪 Dry run: cargo publish --dry-run${NC}"
        cargo publish --dry-run --allow-dirty 2>&1 || {
            echo -e "${YELLOW}⚠️  Dry run warning (may be normal for workspace deps)${NC}"
        }
    else
        echo -e "${GREEN}🚀 Publishing to crates.io...${NC}"
        cargo publish --allow-dirty
        
        # Wait for crates.io to index the package
        echo -e "${BLUE}⏳ Waiting for crates.io to index (30s)...${NC}"
        sleep 30
    fi
    
    echo -e "${GREEN}✅ ${crate_name} complete${NC}"
    echo ""
    
    cd "$ROOT_DIR"
}

# Run tests first
if [ "$SKIP_TESTS" = false ]; then
    echo -e "${BLUE}🧪 Running tests...${NC}"
    cargo test --workspace --exclude spacey-browser || {
        echo -e "${RED}❌ Tests failed! Fix tests before publishing.${NC}"
        exit 1
    }
    echo -e "${GREEN}✅ All tests passed${NC}"
    echo ""
fi

# Check cargo login
if [ "$DRY_RUN" = false ]; then
    echo -e "${BLUE}🔑 Checking crates.io authentication...${NC}"
    if ! cargo owner --list spacey-macros 2>/dev/null | grep -q "." ; then
        # This might be a new crate, that's okay
        echo -e "${YELLOW}⚠️  Could not verify ownership (may be a new crate)${NC}"
    fi
    echo ""
fi

# Publish Layer 1
echo -e "${CYAN}╭─────────────────────────────────────────╮${NC}"
echo -e "${CYAN}│           📦 Layer 1 (Base)             │${NC}"
echo -e "${CYAN}╰─────────────────────────────────────────╯${NC}"
for crate in "${LAYER_1[@]}"; do
    publish_crate "$crate"
done

# Publish Layer 2
echo -e "${CYAN}╭─────────────────────────────────────────╮${NC}"
echo -e "${CYAN}│         📦 Layer 2 (Core)               │${NC}"
echo -e "${CYAN}╰─────────────────────────────────────────╯${NC}"
for crate in "${LAYER_2[@]}"; do
    publish_crate "$crate"
done

# Publish Layer 3
echo -e "${CYAN}╭─────────────────────────────────────────╮${NC}"
echo -e "${CYAN}│       📦 Layer 3 (Integrations)         │${NC}"
echo -e "${CYAN}╰─────────────────────────────────────────╯${NC}"
for crate in "${LAYER_3[@]}"; do
    publish_crate "$crate"
done

# Publish Layer 4 (root)
echo -e "${CYAN}╭─────────────────────────────────────────╮${NC}"
echo -e "${CYAN}│          📦 Layer 4 (Root)              │${NC}"
echo -e "${CYAN}╰─────────────────────────────────────────╯${NC}"
for crate in "${LAYER_4[@]}"; do
    publish_crate "$crate"
done

# Summary
echo -e "${GREEN}"
echo "╔════════════════════════════════════════════════════════════╗"
if [ "$DRY_RUN" = true ]; then
echo "║              🧪 DRY RUN COMPLETE 🧪                        ║"
echo "║                                                            ║"
echo "║  Run with --publish to actually publish to crates.io      ║"
else
echo "║           🎉 PUBLISHING COMPLETE 🎉                        ║"
echo "║                                                            ║"
echo "║  All crates have been published to crates.io!             ║"
fi
echo "╚════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Show crates.io links
echo -e "${BLUE}📚 Crate links:${NC}"
echo "   https://crates.io/crates/spacey"
echo "   https://crates.io/crates/spacey-macros"
echo "   https://crates.io/crates/spacey-spidermonkey"
echo "   https://crates.io/crates/spacey-servo"
echo "   https://crates.io/crates/spacey-node"
echo "   https://crates.io/crates/spacey-npm"
echo ""
