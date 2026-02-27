# Vigo - Vietnamese Input Method Engine

A Vietnamese input method engine written in Rust. Ships as a library, a C shared library, a CLI, and a ready-to-use fcitx5 addon for Linux desktops.

## Why Vigo

**Fast.** The `FastEngine` outperforms uvie-rs (the previous fastest Rust Vietnamese engine) in 4 out of 5 benchmarks — with zero heap allocations per keystroke.

**Correct.** CVC syllable parsing places tones and diacritics correctly even for tricky cases like `ướ`, `ượ`, `uơ`. Case is preserved through transformations: `Vieejt` → `Việt`, `VIEEJT` → `VIỆT`. Every action is fully reversible — triple-key undo (`aaa` → `aa`), tone undo (`ass` → `as`), and backspace all work through a history stack, not ad-hoc heuristics.

**Portable.** Builds as a Rust library, a C shared library (`libvigo.so`), or a `no_std` crate for embedded targets. The same engine powers a terminal UI, a CLI, and a native fcitx5 Linux input method — all from one codebase.

**Extensible.** Input rules are declarative definition tables, not hardcoded logic. Adding a new input method means adding a table, not rewriting transformation code. The engine tiers (FastEngine → SyllableEngine → SmartEngine) let you pick the right trade-off between speed and features.

## Performance

At 60 WPM, one keystroke arrives every ~100,000 µs. All engines finish well within that budget.

### Vigo FastEngine vs uvie-rs

Incremental feed — both engines use `feed(char)` with per-word commit:

| Input | Vigo FastEngine | uvie-rs | Winner |
|-------|----------------|---------|--------|
| simple word (`vieetj`) | **258 ns** | 274 ns | **Vigo 1.06x** |
| medium word (`thuwowngf`) | 578 ns | **521 ns** | uvie 1.11x |
| short sentence (2 words) | **209 ns** | 290 ns | **Vigo 1.39x** |
| medium sentence (8 words) | **1.46 µs** | 1.65 µs | **Vigo 1.13x** |
| long sentence (24 words) | **4.59 µs** | 5.21 µs | **Vigo 1.14x** |

FastEngine: zero heap allocations, 168 bytes of stack, O(1) byte-indexed action dispatch.

### Vigo vs vi-rs

Batch transform — full string in one call:

| Input | Vigo | vi-rs | Speedup |
|-------|------|-------|---------|
| simple word (`vieetj`) | 421 ns | 2,263 ns | **5.4x** |
| medium word (`thuwowngf`) | 613 ns | 4,020 ns | **6.6x** |
| short sentence (2 words) | 654 ns | 2,423 ns | **3.7x** |
| medium sentence (8 words) | 3.2 µs | 14.1 µs | **4.4x** |
| long sentence (24 words) | 9.6 µs | 44.1 µs | **4.6x** |

### Internal engine tiers

| Engine | Medium sentence | Heap allocs | Use case |
|--------|----------------|-------------|----------|
| FastEngine | ~1.5 µs | 0 | Embedded, hot loops, max throughput |
| SyllableEngine | ~29 µs | per commit | IME preedit display (fcitx5 addon) |
| SmartEngine | ~274 µs | per commit | Full pipeline: validation + prediction |

Run benchmarks: `cargo bench --bench vs_uvie_rs`, `cargo bench --bench vs_vi_rs`, or `cargo bench`

## Installation

### fcitx5 (Linux desktop)

```bash
# Prerequisites (Ubuntu/Debian)
sudo apt install fcitx5 libfcitx5core-dev cmake pkg-config

# Prerequisites (Arch)
sudo pacman -S fcitx5 cmake pkg-config

# Install
git clone https://github.com/haidinhtuan/vigo.git
cd vigo
./fcitx5-addon/install.sh
```

This builds everything, installs the addon, configures fcitx5 with Alt+Space toggle, sets up environment variables, and restarts fcitx5. Log out and back in for all applications to pick up the input method.

Toggle Vietnamese input with **Alt+Space**, then type using Telex:

```
xin chaof     → xin chào
Vieejt Nam    → Việt Nam
THUWF VIEEJN  → THƯ VIỆN
```

### As a Rust library

```toml
[dependencies]
vigo = { git = "https://github.com/haidinhtuan/vigo.git" }
```

```rust
use vigo::SyllableEngine;
use vigo::action::InputMethod;

let mut engine = SyllableEngine::new(InputMethod::Telex);
for ch in "Vieejt".chars() {
    engine.feed(ch);
}
assert_eq!(engine.output(), "Việt");
```

### As a C library

Build with `cargo build --release --features ffi` to produce `libvigo.so` and auto-generate `include/vigo.h`. The C API exposes an opaque `vigo_engine_t*` with `vigo_new`, `vigo_feed`, `vigo_output`, `vigo_free`. Callers own returned `char*` strings and must call `vigo_free_string()`.

## Engines

Vigo provides three engine tiers:

**FastEngine** — Zero-allocation, stack-only. 32-byte raw buffer, 128-byte UTF-8 output, 12-char max. Two-pass render pipeline: (1) scan raw bytes → resolve modifications → build char array, (2) apply tone → encode UTF-8. Best for embedded targets, hot loops, and benchmarks.

**SyllableEngine** — Action-based engine with CVC syllable parsing, full undo history, and bypass mode. Used by the fcitx5 addon for preedit display. Rebuilds output on every keystroke.

**SmartEngine** — Wraps SyllableEngine with Vietnamese syllable validation, word prediction (unigram/bigram), abbreviation expansion, and Vietnamese/English code-switching detection. All smart features are individually toggleable.

## Input Methods

### Telex

| Diacritics | | Tones | |
|------------|---|-------|---|
| `aa` → `â` | `aw` → `ă` | `s` → sắc (á) | `f` → huyền (à) |
| `ee` → `ê` | `oo` → `ô` | `r` → hỏi (ả) | `x` → ngã (ã) |
| `ow` → `ơ` | `uw` → `ư` | `j` → nặng (ạ) | `z` → remove tone |
| `dd` → `đ` | `w` → `ư` | | |

Shortcuts: `[` → `ư`, `]` → `ơ`

Undo: `aaa` → `aa`, `ddd` → `dd`, `ass` → `as`

### VNI

| Diacritics | | Tones | |
|------------|---|-------|---|
| `a6` → `â` | `a8` → `ă` | `1` → sắc | `2` → huyền |
| `e6` → `ê` | `o6` → `ô` | `3` → hỏi | `4` → ngã |
| `o7` → `ơ` | `u7` → `ư` | `5` → nặng | `0` → remove |
| `d9` → `đ` | | | |

## CLI

```bash
cargo run -- tui                   # Terminal UI with clipboard (default)
cargo run -- repl                  # Interactive REPL
cargo run -- transform "xin chaof" # Single transformation
echo "vieejt nam" | cargo run -- batch  # Pipe mode
```

Use `--vni` for VNI input method (default is Telex).

## Building

```bash
cargo build --release                                    # Library + CLI
cargo build --release --features ffi                     # C shared library + header
cargo build --release --no-default-features --features heapless  # no_std embedded
cargo test                                               # 137 tests
cargo bench                                              # Criterion benchmarks
```

### Feature flags

| Feature | Default | Purpose |
|---------|---------|---------|
| `std` | yes | Heap-allocated buffers, smart features, CLI |
| `heapless` | no | `no_std` with fixed-capacity buffers (64/128 bytes) |
| `tui` | yes | Terminal UI (crossterm + arboard clipboard) |
| `ffi` | no | C FFI exports, cbindgen header generation |

Exactly one of `std` or `heapless` must be enabled.

## Architecture

```
src/
├── action.rs            # Action enum and declarative transformation rules
├── definitions.rs       # Telex and VNI rule tables
├── syllable.rs          # CVC syllable struct with tone/diacritic components
├── tone.rs              # Tone placement following Vietnamese grammar rules
├── tables.rs            # Const O(1) lookup tables for vowel-to-tone mapping
├── buffer.rs            # Buffer abstraction (std Vec or heapless Vec)
├── fast_engine.rs       # Zero-allocation engine: stack-only, O(1) dispatch
├── syllable_engine.rs   # CVC engine with undo history and bypass mode
├── smart_engine.rs      # Full pipeline: validation + prediction + code-switching
├── validation.rs        # Vietnamese syllable validation
├── prediction.rs        # Next-word prediction (unigram/bigram)
├── abbreviation.rs      # Abbreviation expansion
├── codeswitching.rs     # Vietnamese/English detection
├── engine.rs            # Legacy engine (wraps transform.rs)
├── transform.rs         # Legacy stateless batch transformation
├── ffi.rs               # C FFI bindings (cbindgen-generated header)
├── repl.rs              # Interactive REPL and batch stdin processing
├── tui.rs               # Terminal UI with floating input box
└── main.rs              # CLI entry point

fcitx5-addon/
├── src/vigo.cpp         # Fcitx5 input method plugin (C++17)
├── data/vigo.conf       # Addon registration
├── CMakeLists.txt       # Build system
└── install.sh           # One-command installer
```

## License

MIT OR Apache-2.0
