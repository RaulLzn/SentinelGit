#!/bin/bash
set -e

echo "🚀 Installing SentinelGit..."

# Check dependencies
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo is not installed. Please install Rust: https://rustup.rs/"
    exit 1
fi

# Build
echo "🔨 Building Release..."
cargo build --release

# Installation
INSTALL_DIR="$HOME/.local/bin"
BINARY_NAME="sentinel-git"
SOURCE_BIN="./target/release/$BINARY_NAME"

mkdir -p "$INSTALL_DIR"

if [ -f "$SOURCE_BIN" ]; then
    echo "📦 Copying binary to $INSTALL_DIR..."
    cp "$SOURCE_BIN" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
    
    # Alias
    echo "🔗 Creating alias 'sg'..."
    # Shell config detection (simplified)
    SHELL_CONFIG=""
    if [ -f "$HOME/.bashrc" ]; then SHELL_CONFIG="$HOME/.bashrc"; fi
    if [ -f "$HOME/.zshrc" ]; then SHELL_CONFIG="$HOME/.zshrc"; fi
    
    if [ -n "$SHELL_CONFIG" ]; then
        if ! grep -q "alias sg=" "$SHELL_CONFIG"; then
            echo "" >> "$SHELL_CONFIG"
            echo "# SentinelGit Alias" >> "$SHELL_CONFIG"
            echo "alias sg='$BINARY_NAME'" >> "$SHELL_CONFIG"
            echo "✅ Alias added to $SHELL_CONFIG. Please restart your terminal or source it."
        else
            echo "ℹ️ Alias 'sg' already exists in $SHELL_CONFIG"
        fi
    else
        echo "⚠️  Could not detect shell config (bashrc/zshrc). Please add alias manually:"
        echo "   alias sg='$BINARY_NAME'"
    fi

    echo "✅ SentinelGit installed successfully!"
    echo "   Run '$BINARY_NAME' or 'sg' (after reloading shell) to start."
else
    echo "❌ Build failed. Binary not found."
    exit 1
fi
