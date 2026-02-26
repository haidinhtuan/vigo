# Vietnamese Input Method Analysis

This document analyzes the best open-source Vietnamese input method projects to learn from their implementations.

## Projects Analyzed

| Project | Language | Stars | Description |
|---------|----------|-------|-------------|
| **ibus-bamboo** | Go | 1.4k | Most popular Linux Vietnamese IME |
| **OpenKey** | C++ | 890 | Cross-platform (macOS, Windows, Linux) |
| **vi-rs** | Rust | 157 | Rust library for Vietnamese input |
| **ibus-unikey** | C++ | 142 | Classic IBus Vietnamese engine |
| **v7** | Python | 101 | AI-powered Vietnamese input with prediction |
| **bogo-python** | Python | 15 | Reference implementation with clean architecture |

---

## Key Architectural Insights

### 1. Syllable Structure (CVC Pattern)

Vietnamese syllables follow a **Consonant-Vowel-Consonant (CVC)** pattern:

```
┌─────────────────────────────────────────────────┐
│  Initial Consonant │ Vowel │ Final Consonant   │
│    (optional)      │       │   (optional)      │
│   "th", "ng", etc  │ "ươ"  │   "ng", "ch"      │
└─────────────────────────────────────────────────┘
```

**Example:** `thương` → `th` + `ươ` + `ng`

This is implemented consistently across all projects:
- **vi-rs**: `Syllable { initial_consonant, vowel, final_consonant }`
- **bogo-python**: `separate(string)` → `[first_consonant, vowel, last_consonant]`
- **bamboo-core**: `extractCvcTrans()` → `(fc, vo, lc)`

### 2. Transformation System

All projects use a **transformation-based** approach:

```rust
// Pseudocode pattern
enum Action {
    AddTonemark(ToneMark),      // s, f, r, x, j (Telex)
    ModifyLetter(Modification), // aa→â, aw→ă, dd→đ
    RemoveToneMark,             // z key
    Undo,                       // double-tap undo
}
```

**Best Practice (from vi-rs):**
```rust
pub type Definition = Map<char, &'static [Action]>;

// Multiple possible actions per key
'w' => &[
    Action::ResetInsertedƯ,           // ưw → uw
    Action::ModifyLetter(Horn),        // u → ư, o → ơ
    Action::ModifyLetter(Breve),       // a → ă
    Action::InsertƯ,                   // standalone w → ư
],
```

### 3. Tone Mark Placement Rules

**New Style (Modern)** - Place tone on the vowel with diacritics:
- `hoà` (tone on `à`, not `ó`)
- Priority: modified vowel (ơ, ư, ê, ô, â, ă) > position rules

**Old Style (Traditional)** - Follow positional rules:
- `hòa` (tone on first vowel in open syllable)

**Algorithm (from vi-rs/bogo-python):**
```python
def find_tone_target(vowels, has_final_consonant):
    # 1. If single vowel → tone on that vowel
    # 2. If modified vowel exists (ơ, ư, ê, ô, â, ă) → tone on it
    # 3. If closed syllable (has final consonant) → tone on last vowel
    # 4. If open syllable → tone on second-to-last vowel
    # 5. Special: "oa", "oe", "uy" → tone on second vowel
```

### 4. Special Cases

#### qu/gi Clusters
- `qu` and `gi` are treated as **initial consonants**, not vowels
- `qua` → `qu` + `a` (not `q` + `ua`)
- `gia` → `gi` + `a` (not `g` + `ia`)

#### uơ/ươ Handling
Words like `huơ`, `thuở`, `quở` have irregular vowel `uơ` instead of `ươ`:
```python
# bogo-python
if vowel == "ươ" and final_consonant == "" and initial in ["", "h", "th", "kh"]:
    vowel = "uơ"  # Change ươ to uơ
```

#### Triple-Key Undo
- `aaa` → `aa` (undo the circumflex)
- `ddd` → `dd` (undo the đ)

#### Double-Tap Tone Undo
- `ass` → `as` (undo the tone, output literal 's')

### 5. Validation

**Spell Checking (from bamboo-core):**
```go
func isValidCVC(firstConsonant, vowel, lastConsonant string, complete bool) bool {
    // Validate consonant combinations
    // Validate vowel patterns
    // Validate tone + final consonant compatibility
}

// Tone restrictions:
// p, t, c, ch final consonants → only acute (sắc) or dot (nặng) tones
```

### 6. Performance Optimizations

**vi-rs:**
- Uses `SmallVec<[(usize, LetterModification); 2]>` - avoids heap allocation
- Caches syllable state to avoid re-parsing
- `IncrementalBuffer` for character-by-character processing

**uvie-rs (reference):**
- Fixed small buffers (cache-friendly)
- Table lookups instead of match/if ladders
- O(1) operations for core lookups
- Benchmarks: **15-22x faster** than vi-rs

### 7. State Machine Pattern (bamboo-core)

```go
type Transformation struct {
    Rule        Rule
    Target      *Transformation  // Points to the transformation being modified
    IsUpperCase bool
}

type BambooEngine struct {
    composition []*Transformation  // Stack of transformations
    inputMethod InputMethod
    flags       uint
}
```

This allows:
- Tracking transformation history
- Easy undo operations
- Backspace handling that removes both character and its effects

---

## Recommendations for vigo

### 1. Adopt Syllable Struct
```rust
pub struct Syllable {
    pub initial_consonant: String,
    pub vowel: String,  
    pub final_consonant: String,
    pub tone_mark: Option<ToneMark>,
    pub letter_modifications: SmallVec<[(usize, LetterModification); 2]>,
}
```

### 2. Use Action-Based Definition
```rust
pub static TELEX: Definition = phf_map! {
    's' => &[Action::AddTonemark(ToneMark::Acute)],
    'a' => &[Action::ModifyLetterOnFamily(Circumflex, 'a')],
    'w' => &[Action::ResetƯ, Action::Horn, Action::Breve, Action::InsertƯ],
    // ...
};
```

### 3. Implement Proper Tone Placement
- Support both Old and New accent styles
- Handle special vowel combinations (oa, oe, uy, uye)
- Respect qu/gi clusters

### 4. Add Validation
- Spell checking with Vietnamese syllable rules
- Tone + final consonant compatibility
- Valid consonant/vowel combinations

### 5. Performance
- Use lookup tables (like current `tables.rs`)
- Consider `SmallVec` for modifications
- Incremental buffer for real-time input

### 6. Missing Features in vigo
- [ ] Bracket shortcuts (`[` → `ư`, `]` → `ơ`) - mentioned but not working
- [ ] Heapless mode transformation (placeholder only)
- [ ] Proper syllable parsing (CVC separation)
- [ ] Spell validation
- [ ] Configurable accent style (Old/New)
- [ ] uơ/ươ special case handling

---

## Reference Code Locations

| Feature | vi-rs | bamboo-core | bogo-python |
|---------|-------|-------------|-------------|
| Syllable struct | `src/syllable.rs` | N/A (uses Transformation) | `utils.separate()` |
| Tone placement | `src/editing.rs:get_tone_mark_placement` | `bamboo_utils.go:findToneTarget` | `accent.py:add_accent` |
| Definition | `src/methods.rs:TELEX/VNI` | `input_method_def.go` | `core.py:get_telex_definition` |
| Validation | `src/validation.rs` | `spelling.go` | `validation.py` |
| CVC parsing | `src/parsing.rs` | `bamboo_utils.go:extractCvcTrans` | `utils.py:separate` |
