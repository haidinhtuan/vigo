# Vigo - Vietnamese Input Method Engine

A modern Vietnamese input method engine written in Rust, with a native fcitx5 integration for Linux desktops including Wayland.

## What Makes Vigo Different

Most Vietnamese input engines (unikey, bamboo, OpenKey) are written in C/C++ or Go with tightly coupled architectures. Vigo takes a different approach:

- **Action-based architecture**: Input rules are declarative definitions, not hardcoded logic. Adding a new input method means adding a definition table, not rewriting transformation code.
- **Proper CVC syllable parsing**: Vietnamese syllables are parsed into consonant-vowel-consonant components, enabling correct tone and diacritic placement even for complex cases like `ướ` or `ượ`.
- **Case preservation**: Handles Shift+key capitalization correctly — `Vieejt` → `Việt`, `VIEEJT` → `VIỆT`. Many engines lose case information during transformation.
- **Full undo history**: Every action is reversible. Triple-key undo (`aaa` → `aa`), tone undo (`ass` → `as`), and backspace all work correctly through a history stack rather than ad-hoc heuristics.
- **Bypass mode**: When input doesn't match Vietnamese patterns, the engine transparently passes through raw text instead of producing garbled output.
- **C FFI**: The engine compiles to a shared library (`libvigo.so`) with a clean C API, making it embeddable in any input method framework.

## Performance

At 60 WPM, one keystroke arrives every ~100,000 µs. All engines below finish well within that budget.

### Vigo vs vi-rs (Rust, Criterion)

Batch transform — full string in one call:

| Input | Vigo | vi-rs | Speedup |
|-------|------|-------|---------|
| simple word (`vieetj`) | 421 ns | 2,263 ns | **5.4x** |
| medium word (`thuwowngf`) | 613 ns | 4,020 ns | **6.6x** |
| short sentence (2 words) | 654 ns | 2,423 ns | **3.7x** |
| medium sentence (8 words) | 3.2 µs | 14.1 µs | **4.4x** |
| long sentence (24 words) | 9.6 µs | 44.1 µs | **4.6x** |

Incremental — character-by-character engine:

| Input | Vigo SyllableEngine | vi-rs IncrementalBuffer |
|-------|-------------------|------------------------|
| simple word | **1.3 µs** | 2.2 µs |
| medium word | **2.0 µs** | 3.8 µs |
| medium sentence | 29 µs | **13.5 µs** |
| long sentence | 164 µs | **41.8 µs** |

> Vigo's `SyllableEngine` rebuilds the full output string on every keystroke (needed for IME preedit display), while vi-rs maintains an incremental cache. This accounts for the sentence-length gap; single-word performance is where the core algorithm comparison is fair.

### Vigo vs uvie-rs (Rust, Criterion)

Incremental feed — both engines use a `feed(char)` API with per-word commit, direct comparison:

| Input | Vigo FastEngine | uvie-rs | Winner |
|-------|----------------|---------|--------|
| simple word (`vieetj`) | **258 ns** | 274 ns | **Vigo 1.06x** |
| medium word (`thuwowngf`) | 578 ns | **521 ns** | uvie 1.11x |
| short sentence (2 words) | **209 ns** | 290 ns | **Vigo 1.39x** |
| medium sentence (8 words) | **1.46 µs** | 1.65 µs | **Vigo 1.13x** |
| long sentence (24 words) | **4.59 µs** | 5.21 µs | **Vigo 1.14x** |

FastEngine wins 4 out of 5 cases against uvie-rs. Both engines commit per word (clear on space). FastEngine uses O(1) byte-indexed action lookup and zero heap allocations per keystroke.

Batch transform (Vigo `transform_buffer` vs FastEngine vs uvie-rs feed loop):

| Input | Vigo batch | FastEngine | uvie-rs |
|-------|-----------|-----------|---------|
| simple word | 412 ns | **244 ns** | 261 ns |
| medium word | 609 ns | 570 ns | **512 ns** |
| complex word (`nghieeeng`) | **222 ns** | 324 ns | 435 ns |
| short sentence | 602 ns | 318 ns | **285 ns** |
| medium sentence | 2.84 µs | 9.59 µs | **1.62 µs** |
| long sentence | 8.59 µs | 50.1 µs | **5.05 µs** |

> `FastEngine` uses a zero-allocation, stack-only render pipeline (32-byte raw buffer, 128-byte UTF-8 output) with O(1) const action lookup tables. For single-word inputs, FastEngine matches or beats uvie-rs. For multi-word batch inputs (no space handling), FastEngine re-renders the entire raw buffer on every keystroke — use incremental mode with per-word commit for sentence-length inputs.

### Vigo vs Unikey (C++, 100k iterations)

Vigo is called through C FFI (`libvigo.so`); Unikey uses its native C++ API.

| Input | Vigo | Unikey | Ratio |
|-------|------|--------|-------|
| simple word (`vieetj`) | 1.18 µs | 0.16 µs | 7.4x |
| two words | 1.51 µs | 0.18 µs | 8.4x |
| medium (5 words) | 4.55 µs | 0.60 µs | 7.5x |
| long sentence (13 words) | 10.71 µs | 1.56 µs | 6.9x |

> Unikey's C++ engine is ~7x faster than Vigo through C FFI. This is expected — Unikey uses in-place buffer mutation with zero allocations, while Vigo's FFI path involves engine creation, string allocation, and UTF-8 encoding per commit. Both are far below the 100,000 µs keystroke budget.

### Internal engine comparison

| Engine | Medium sentence | Description |
|--------|----------------|-------------|
| Legacy Engine | ~15 µs | First-generation table-lookup engine |
| FastEngine | ~1.5 µs | Zero-allocation, stack-only CVC engine with O(1) lookup |
| SyllableEngine | ~32 µs | CVC-based engine used by fcitx5 addon |
| SmartEngine | ~295 µs | Full pipeline with validation + prediction |

Run benchmarks: `cargo bench --bench vs_vi_rs`, `cargo bench --bench vs_uvie_rs`, or `cargo bench --bench benchmark`

## Architecture

```
src/
├── syllable.rs          # CVC syllable struct with tone/diacritic components
├── action.rs            # Action & Transformation types (declarative rules)
├── definitions.rs       # Telex and VNI rule tables
├── tone.rs              # Tone placement following Vietnamese grammar rules
├── syllable_engine.rs   # Main engine: feed/backspace/commit with undo history
├── fast_engine.rs       # Zero-allocation engine: stack-only buffers, no std
├── transform.rs         # Stateless batch transformation
├── validation.rs        # Vietnamese syllable validation
├── prediction.rs        # Next-word prediction
├── codeswitching.rs     # Vietnamese/English detection
├── smart_engine.rs      # Full pipeline combining all features
├── ffi.rs               # C FFI bindings (cbindgen-generated header)
└── main.rs              # CLI: repl, transform, batch modes

fcitx5-addon/
├── src/vigo.cpp         # Fcitx5 input method plugin (C++17)
├── data/vigo.conf       # Addon registration
├── CMakeLists.txt       # Build system
└── install.sh           # One-command installer
```

## Installation

### Prerequisites

- Rust toolchain (1.70+)
- fcitx5 development headers (`fcitx5-dev` or `fcitx5-devel`)
- cmake, pkg-config
- A Linux desktop with fcitx5

On Ubuntu/Debian:
```bash
sudo apt install fcitx5 libfcitx5core-dev cmake pkg-config
```

On Arch:
```bash
sudo pacman -S fcitx5 cmake pkg-config
```

### One-Command Install

```bash
git clone https://github.com/haidinhtuan/vigo.git
cd vigo
./fcitx5-addon/install.sh
```

This builds everything, installs the addon, configures fcitx5 with Alt+Space toggle, sets up environment variables, and restarts fcitx5. You may need to log out and back in for all applications to pick up the input method.

### Using Vigo

Toggle Vietnamese input with **Alt+Space**, then type using Telex:

```
xin chaof     → xin chào
Vieejt Nam    → Việt Nam
THUWF VIEEJN  → THƯ VIỆN
```

## Telex Input Rules

### Vowel Diacritics

| Input | Output | Description |
|-------|--------|-------------|
| aa | â | circumflex |
| aw | ă | breve |
| ee | ê | circumflex |
| oo | ô | circumflex |
| ow | ơ | horn |
| uw | ư | horn |
| dd | đ | stroke |

### Tone Marks

| Key | Tone | Example |
|-----|------|---------|
| s | sắc (acute) | as → á |
| f | huyền (grave) | af → à |
| r | hỏi (hook) | ar → ả |
| x | ngã (tilde) | ax → ã |
| j | nặng (dot) | aj → ạ |
| z | remove tone | ász → a |

### Special

- Triple-key undo: `aaa` → `aa`, `ddd` → `dd`
- Tone undo: `ass` → `as`
- Standalone `w` → `ư`

## VNI Input Rules

| Input | Output | | Key | Tone |
|-------|--------|-|-----|------|
| a6 | â | | 1 | sắc |
| a8 | ă | | 2 | huyền |
| e6 | ê | | 3 | hỏi |
| o6 | ô | | 4 | ngã |
| o7 | ơ | | 5 | nặng |
| u7 | ư | | 0 | remove |
| d9 | đ | | | |

## As a Rust Library

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

## Building from Source

```bash
cargo build --release              # Library + CLI
cargo build --release --features ffi  # Shared library for FFI
cargo test                         # Run 112 tests
cargo bench                        # Run benchmarks
```

## CLI

```bash
cargo run -- repl                  # Interactive mode
cargo run -- transform "xin chaof" # Single transformation
echo "vieejt nam" | cargo run -- batch  # Pipe mode
```

## License

MIT OR Apache-2.0
