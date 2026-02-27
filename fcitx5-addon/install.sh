#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== Vigo Fcitx5 Addon Installer ==="
echo ""

# Build vigo with FFI support
echo "[1/6] Building vigo library..."
cd "$PROJECT_DIR"
cargo build --release --features ffi

# Install Rust library
echo "[2/6] Installing vigo library..."
sudo install -Dm755 target/release/libvigo.so /usr/local/lib/libvigo.so
sudo ldconfig

# Build fcitx5 addon
echo "[3/6] Building fcitx5 addon..."
cd "$SCRIPT_DIR"
mkdir -p build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
make -j$(nproc)

# Install fcitx5 addon
echo "[4/6] Installing fcitx5 addon..."
sudo make install

# Fix addon configuration (critical!)
echo "[5/6] Fixing addon configuration..."
sudo tee /usr/local/share/fcitx5/addon/vigo.conf > /dev/null << 'EOF'
[Addon]
Name=Vigo
Category=InputMethod
Library=libvigo
Type=SharedLibrary
Version=5.1.7
OnDemand=False
Configurable=True
EOF

# Setup user profile
echo "[6/6] Setting up user profile..."
pkill fcitx5 2>/dev/null || true
sleep 1

mkdir -p "$HOME/.config/fcitx5"

# Write profile
cat > "$HOME/.config/fcitx5/profile" << 'EOF'
[Groups/0]
Name=Default
Default Layout=us
DefaultIM=keyboard-us

[Groups/0/Items/0]
Name=keyboard-us
Layout=

[Groups/0/Items/1]
Name=vigo-addon
Layout=

[GroupOrder]
0=Default
EOF

# Write config with Alt+Space hotkey
cat > "$HOME/.config/fcitx5/config" << 'EOF'
[Hotkey]
EnumerateWithTriggerKeys=True

[Hotkey/TriggerKeys]
0=Alt+space

[Hotkey/PrevPage]
0=Up

[Hotkey/NextPage]
0=Down

[Hotkey/PrevCandidate]
0=Shift+Tab

[Hotkey/NextCandidate]
0=Tab

[Behavior]
ActiveByDefault=False
ShareInputState=No
PreeditEnabledByDefault=True
ShowInputMethodInformation=True
DefaultPageSize=5
PreloadInputMethod=True
EOF

echo ""
echo "=== Installation complete! ==="
echo ""
echo "Start fcitx5:"
echo "  fcitx5 -r"
echo ""
echo "You should see:"
echo "  Loaded addon vigo"
echo "  Found 1 input method(s) in addon vigo"
echo ""
echo "Use Alt+Space to toggle between English and Vietnamese."
echo "When Vietnamese is active, type using Telex:"
echo "  xin chaof → xin chào"
echo "  Vieejt Nam → Việt Nam"
echo ""
