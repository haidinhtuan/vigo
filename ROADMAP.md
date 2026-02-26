# Vigo Implementation Roadmap

Based on the analysis of top Vietnamese input method projects, this roadmap outlines the implementation phases for vigo.

---

## Phase 1: Core Architecture Refactor
**Goal:** Establish a solid foundation with proper syllable parsing and action-based transformations.

### 1.1 Syllable Module (`src/syllable.rs`)
- [ ] Create `Syllable` struct with CVC components
  ```rust
  pub struct Syllable {
      pub initial_consonant: String,
      pub vowel: String,
      pub final_consonant: String,
      pub tone_mark: Option<ToneMark>,
      pub modifications: SmallVec<[(usize, LetterModification); 2]>,
      pub accent_style: AccentStyle,
  }
  ```
- [ ] Implement `parse_syllable(input: &str) -> Syllable`
- [ ] Handle `qu`/`gi` clusters as initial consonants
- [ ] Implement `Display` trait for syllable reconstruction
- [ ] Add unit tests for syllable parsing

### 1.2 Action System (`src/action.rs`)
- [ ] Define `Action` enum
  ```rust
  pub enum Action {
      AddTonemark(ToneMark),
      ModifyLetter(LetterModification),
      ModifyLetterOnFamily(LetterModification, char),
      InsertƯ,
      ResetInsertedƯ,
      RemoveToneMark,
  }
  ```
- [ ] Define `Transformation` result enum
- [ ] Create `Definition` type alias for input method maps

### 1.3 Input Method Definitions (`src/definitions.rs`)
- [ ] Define `TELEX` using action-based approach
- [ ] Define `VNI` using action-based approach
- [ ] Support customizable definitions

**Deliverable:** Clean separation of syllable parsing from transformation logic.

---

## Phase 2: Tone Placement & Modifications
**Goal:** Implement correct Vietnamese tone mark placement with both accent styles.

### 2.1 Tone Placement (`src/tone.rs`)
- [ ] Implement `AccentStyle` enum (Old, New)
- [ ] Implement `find_tone_position(syllable: &Syllable, style: AccentStyle) -> usize`
- [ ] Handle special vowel combinations:
  - [ ] `oa`, `oe` → tone on second vowel
  - [ ] `uy`, `uye` → tone on second/third vowel
  - [ ] Modified vowels (ơ, ư, ê, ô, â, ă) get priority
- [ ] Implement `apply_tone(syllable: &mut Syllable, tone: ToneMark)`
- [ ] Implement `remove_tone(syllable: &mut Syllable)`
- [ ] Add comprehensive tone placement tests

### 2.2 Letter Modifications (`src/modification.rs`)
- [ ] Implement `apply_modification(syllable: &mut Syllable, mod: LetterModification)`
- [ ] Handle modification replacement (e.g., `aw` then `aa` → circumflex replaces breve)
- [ ] Handle double horn (`ươ`)
- [ ] Implement `remove_modification(syllable: &mut Syllable, mod: LetterModification)`

### 2.3 Special Cases
- [ ] `uơ`/`ươ` handling for words like `huơ`, `thuở`, `quở`
- [ ] Automatic `uơ` → `ươ` conversion when final consonant added

**Deliverable:** Correct tone and modification placement matching native speakers' expectations.

---

## Phase 3: Transformation Engine
**Goal:** Implement the core transformation logic with undo support.

### 3.1 Engine Refactor (`src/engine.rs`)
- [ ] Refactor `Engine` to use `Syllable` internally
- [ ] Track transformation history for undo
- [ ] Implement action execution with fallback chain
  ```rust
  fn execute_actions(&mut self, actions: &[Action]) -> Transformation {
      for action in actions {
          if let result = self.try_action(action) {
              if result != Transformation::Ignored {
                  return result;
              }
          }
      }
      Transformation::Ignored
  }
  ```

### 3.2 Undo System
- [ ] Triple-key undo: `aaa` → `aa`, `ddd` → `dd`
- [ ] Double-tap tone undo: `ass` → `as`
- [ ] `ResetInsertedƯ` for `ưw` → `uw`
- [ ] `z` key removes tone

### 3.3 Transform Function (`src/transform.rs`)
- [ ] Refactor `transform_word()` to use new syllable system
- [ ] Support incremental transformation (character by character)
- [ ] Implement `IncrementalBuffer` for real-time input

**Deliverable:** Robust transformation engine with proper undo support.

---

## Phase 4: Validation & Spell Checking
**Goal:** Add Vietnamese syllable validation to prevent invalid combinations.

### 4.1 Validation Module (`src/validation.rs`)
- [ ] Valid initial consonants list
- [ ] Valid final consonants list  
- [ ] Valid vowel patterns
- [ ] Consonant + vowel compatibility rules
- [ ] Implement `is_valid_syllable(syllable: &Syllable) -> bool`

### 4.2 Tone Restrictions
- [ ] Implement tone + final consonant rules:
  - `p`, `t`, `c`, `ch` endings → only acute (sắc) or dot (nặng)
- [ ] Integrate validation into transformation (optional spell-check mode)

### 4.3 Auto-Correction (Optional)
- [ ] Suggest corrections for invalid syllables
- [ ] Flag for enabling/disabling validation

**Deliverable:** Optional spell-checking that prevents invalid Vietnamese syllables.

---

## Phase 5: Missing Features & Polish
**Goal:** Complete all documented features and improve usability.

### 5.1 Bracket Shortcuts
- [ ] `[` → `ư`
- [ ] `]` → `ơ`
- [ ] `{` → `Ư`
- [ ] `}` → `Ơ`

### 5.2 Heapless Mode
- [ ] Implement transformation for `heapless` feature
- [ ] Use fixed-size buffers
- [ ] Test on no-std environment

### 5.3 Configuration
- [ ] Accent style selection (Old/New)
- [ ] Enable/disable spell checking
- [ ] Custom key mappings

### 5.4 Documentation
- [ ] Update README with new features
- [ ] Add examples for common use cases
- [ ] Document all public APIs

**Deliverable:** Feature-complete Vietnamese input engine.

---

## Phase 6: Performance Optimization
**Goal:** Match or exceed uvie-rs performance benchmarks.

### 6.1 Profiling
- [ ] Run benchmarks against vi-rs and uvie-rs
- [ ] Identify bottlenecks with `cargo flamegraph`

### 6.2 Optimizations
- [ ] Replace `String` with fixed buffers where possible
- [ ] Use `SmallVec` for modifications list
- [ ] Optimize lookup tables
- [ ] Reduce allocations in hot paths
- [ ] Consider SIMD for batch processing

### 6.3 Benchmarks
- [ ] Add comprehensive benchmarks
- [ ] Compare with vi-rs, uvie-rs
- [ ] Document performance characteristics

**Deliverable:** High-performance engine competitive with fastest implementations.

---

## Phase 7: Platform Integration (Future)
**Goal:** Create platform-specific input method implementations.

### 7.1 Linux (IBus)
- [ ] IBus engine wrapper
- [ ] D-Bus integration
- [ ] System tray support

### 7.2 macOS (InputMethodKit)
- [ ] Swift/Rust bridge
- [ ] InputMethodKit integration

### 7.3 Windows
- [ ] TSF (Text Services Framework) integration
- [ ] System tray support

**Deliverable:** Cross-platform Vietnamese input method.

---

## Timeline Estimate

| Phase | Duration | Priority |
|-------|----------|----------|
| Phase 1: Core Architecture | 1-2 weeks | High |
| Phase 2: Tone & Modifications | 1 week | High |
| Phase 3: Transformation Engine | 1 week | High |
| Phase 4: Validation | 3-5 days | Medium |
| Phase 5: Missing Features | 3-5 days | Medium |
| Phase 6: Performance | 1 week | Medium |
| Phase 7: Platform Integration | 2-4 weeks | Low (Future) |

**Total Core Library:** ~4-6 weeks

---

## Success Criteria

1. **Correctness:** Pass all test cases from vi-rs and bogo-python
2. **Performance:** Within 2x of uvie-rs benchmarks
3. **Compatibility:** Support both Telex and VNI methods
4. **Usability:** Clean, well-documented API
5. **Flexibility:** Configurable accent style and validation
