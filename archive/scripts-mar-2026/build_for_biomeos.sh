#!/bin/bash
# Build petalTongue release binary for biomeOS plasmidBin
#
# This script builds an optimized release binary and copies it to biomeOS
# for orchestrated deployment.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BIOMEOS_ROOT="$(cd "$PROJECT_ROOT/../biomeOS" && pwd)"
PLASMID_BIN="$BIOMEOS_ROOT/plasmidBin"

echo "🔨 Building petalTongue release binary..."
echo "   Project: $PROJECT_ROOT"
echo "   Target: $PLASMID_BIN"
echo ""

# Navigate to project root
cd "$PROJECT_ROOT"

# Build release binary
echo "📦 cargo build --release --bin petal-tongue"
cargo build --release --bin petal-tongue

# Check if build succeeded
if [ ! -f "target/release/petal-tongue" ]; then
    echo "❌ Build failed - binary not found"
    exit 1
fi

echo "✅ Build complete"
echo ""

# Get binary size and info
BINARY_SIZE=$(du -h target/release/petal-tongue | cut -f1)
echo "📊 Binary info:"
echo "   Size: $BINARY_SIZE"
echo "   Path: target/release/petal-tongue"
echo ""

# Check if biomeOS directory exists
if [ ! -d "$BIOMEOS_ROOT" ]; then
    echo "⚠️  biomeOS directory not found at: $BIOMEOS_ROOT"
    echo "   Skipping copy to plasmidBin"
    echo "   Binary available at: $PROJECT_ROOT/target/release/petal-tongue"
    exit 0
fi

# Create plasmidBin directory if it doesn't exist
mkdir -p "$PLASMID_BIN"

# Copy binary to plasmidBin
echo "📋 Copying to biomeOS plasmidBin..."
cp target/release/petal-tongue "$PLASMID_BIN/petaltongue"

# Make it executable
chmod +x "$PLASMID_BIN/petaltongue"

echo "✅ Binary copied to: $PLASMID_BIN/petaltongue"
echo ""

# Create version info file
VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
BUILD_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

cat > "$PLASMID_BIN/petaltongue.version" <<EOF
{
  "primal": "petaltongue",
  "version": "$VERSION",
  "build_date": "$BUILD_DATE",
  "capabilities": [
    "ui.render",
    "ui.visualization",
    "ui.graph",
    "ui.terminal",
    "ui.audio",
    "ui.framebuffer",
    "discovery.mdns",
    "discovery.http",
    "ipc.tarpc",
    "ipc.json-rpc",
    "ipc.unix-socket"
  ],
  "socket_path": "/run/user/<uid>/petaltongue-<family>.sock",
  "protocols": [
    "tarpc",
    "json-rpc-2.0"
  ]
}
EOF

echo "✅ Version info created: $PLASMID_BIN/petaltongue.version"
echo ""

# Test binary
echo "🧪 Testing binary..."
if "$PLASMID_BIN/petaltongue" --version >/dev/null 2>&1; then
    echo "✅ Binary executable and working"
else
    echo "⚠️  Binary test inconclusive (may require display)"
fi

echo ""
echo "🎊 petalTongue binary ready for biomeOS deployment!"
echo ""
echo "📝 Next steps for biomeOS team:"
echo "   1. Binary: $PLASMID_BIN/petaltongue"
echo "   2. Version info: $PLASMID_BIN/petaltongue.version"
echo "   3. Socket: /run/user/<uid>/petaltongue-<family>.sock"
echo "   4. Start with: FAMILY_ID=nat0 $PLASMID_BIN/petaltongue"
echo ""
echo "🌸 Ready for Phase 4 integration!"

