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

Benchmarked with Criterion on the phrase "Thủ tướng Chính phủ Việt Nam" (5 words, 28 keystrokes):

| Engine | Time | Description |
|--------|------|-------------|
| SyllableEngine | ~28 µs | Core engine used by fcitx5 addon |
| Legacy Engine | ~15 µs | First-generation table-lookup engine |
| SmartEngine | ~263 µs | Full pipeline with validation + prediction |

For comparison, a single keystroke at 60 WPM arrives every ~100,000 µs. The engine processes an entire sentence in a fraction of one keystroke interval.

## Architecture

```
src/
├── syllable.rs          # CVC syllable struct with tone/diacritic components
├── action.rs            # Action & Transformation types (declarative rules)
├── definitions.rs       # Telex and VNI rule tables
├── tone.rs              # Tone placement following Vietnamese grammar rules
├── syllable_engine.rs   # Main engine: feed/backspace/commit with undo history
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
