#!/bin/bash
set -e

# Build vigo with FFI support
echo "Building vigo library..."
cd "$(dirname "$0")/.."
cargo build --release --features ffi

# Generate header
echo "Generating C header..."
mkdir -p include
cat > include/vigo.h << 'EOF'
#ifndef VIGO_H
#define VIGO_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct vigo_engine_t vigo_engine_t;

vigo_engine_t* vigo_new_telex(void);
vigo_engine_t* vigo_new_vni(void);
void vigo_free(vigo_engine_t* engine);
bool vigo_feed(vigo_engine_t* engine, uint32_t ch);
char* vigo_get_output(const vigo_engine_t* engine);
char* vigo_get_raw_input(const vigo_engine_t* engine);
void vigo_free_string(char* s);
bool vigo_backspace(vigo_engine_t* engine);
void vigo_clear(vigo_engine_t* engine);
char* vigo_commit(vigo_engine_t* engine);
bool vigo_is_empty(const vigo_engine_t* engine);

#ifdef __cplusplus
}
#endif

#endif /* VIGO_H */
EOF

# Install library
echo "Installing vigo library..."
sudo install -Dm755 target/release/libvigo.so /usr/local/lib/libvigo.so
sudo ldconfig

# Build fcitx5 addon
echo "Building fcitx5 addon..."
cd fcitx5-addon
mkdir -p build && cd build
cmake .. -DCMAKE_BUILD_TYPE=Release
make -j$(nproc)

# Install fcitx5 addon
echo "Installing fcitx5 addon..."
sudo make install

echo ""
echo "Installation complete!"
echo ""
echo "To enable Vietnamese input:"
echo "  1. Run: fcitx5-configtool"
echo "  2. Add 'Vietnamese (Vigo Telex)' to your input methods"
echo "  3. Use Ctrl+Space to toggle"
echo ""
echo "Or restart fcitx5:"
echo "  fcitx5 -r"
