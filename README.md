# Vigo - Vietnamese Input Method Engine

A fast, ergonomic Vietnamese input method engine supporting Telex and VNI input methods, written in Rust.

## Features

- **High Performance**: Optimized with lookup tables and minimal allocations
- **Multiple Input Methods**: Supports both Telex and VNI
- **No-std Compatible**: Can be built without std for embedded systems (`heapless` feature)
- **Ergonomic API**: Simple, intuitive interface
- **CLI Tool**: Interactive REPL and batch transformation modes
- **Well Documented**: Comprehensive documentation and examples

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
vigo = "0.1"
```

For embedded/no-std environments:

```toml
[dependencies]
vigo = { version = "0.1", default-features = false, features = ["heapless"] }
```

## Quick Start

### As a Library

```rust
use vigo::{Engine, InputMethod, transform_buffer};

// Simple transformation
let result = transform_buffer("vieetj");
assert_eq!(result, "việt");

// Stateful engine for real-time input
let mut engine = Engine::new(InputMethod::Telex);
for ch in "chaof".chars() {
    engine.feed(ch);
}
assert_eq!(engine.output(), "chào");
```

### CLI Usage

```bash
# Interactive REPL
vigo repl

# Transform text directly
vigo transform "xin chaof"

# Batch mode (stdin -> stdout)
echo "vieetj nam" | vigo batch

# Use VNI input method
vigo --vni transform "vie6t5"
```

## Telex Rules

### Vowel Diacritics

| Input | Output | Description |
|-------|--------|-------------|
| aa    | â      | a with circumflex |
| aw    | ă      | a with breve |
| ee    | ê      | e with circumflex |
| oo    | ô      | o with circumflex |
| ow    | ơ      | o with horn |
| uw    | ư      | u with horn |
| dd    | đ      | d with stroke |

### Tone Marks

| Key | Tone | Example |
|-----|------|---------|
| s   | sắc (acute) | as → á |
| f   | huyền (grave) | af → à |
| r   | hỏi (hook) | ar → ả |
| x   | ngã (tilde) | ax → ã |
| j   | nặng (dot) | aj → ạ |
| z   | remove tone | asz → a |

### Special Features

- **Triple-key undo**: `aaa` → `aa`, `ddd` → `dd`
- **Double-tap tone undo**: `ass` → `as`
- **W shortcut**: standalone `w` → `ư`
- **Bracket shortcuts**: `[` → `ư`, `]` → `ơ`

## VNI Rules

### Vowel Diacritics

| Input | Output |
|-------|--------|
| a6    | â      |
| a8    | ă      |
| e6    | ê      |
| o6    | ô      |
| o7    | ơ      |
| u7    | ư      |
| d9    | đ      |

### Tone Marks

| Key | Tone |
|-----|------|
| 1   | sắc  |
| 2   | huyền |
| 3   | hỏi  |
| 4   | ngã  |
| 5   | nặng |
| 0   | remove |

## Benchmarks

Run benchmarks:

```bash
cargo bench
```

## Building

```bash
# Standard build
cargo build --release

# No-std build
cargo build --release --no-default-features --features heapless

# Run tests
cargo test

# Run CLI
cargo run -- transform "vieetj"
```

## Comparison with Other Projects

Vigo combines the best features from existing Vietnamese input engines:

| Feature | Vigo | uvie-rs | xkey |
|---------|------|---------|------|
| Telex support | ✓ | ✓ | ✓ |
| VNI support | ✓ | ✓ | ✗ |
| No-std/heapless | ✓ | ✓ | ✗ |
| CLI REPL | ✓ | ✓ | ✓ |
| Clean API | ✓ | ○ | ○ |
| Documentation | ✓ | ○ | ✓ |

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
