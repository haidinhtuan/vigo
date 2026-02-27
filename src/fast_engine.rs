//! Zero-allocation Vietnamese input engine.
//!
//! `FastEngine` processes keystrokes using stack-only buffers and a fused
//! 2-pass render pipeline. No heap allocations on the hot path.

use crate::action::{Action, InputMethod};
use crate::syllable::{LetterModification, ToneMark};
use crate::tables::{vowel_to_id, apply_tone as tables_apply_tone, extract_tone};
use crate::definitions::{get_definition, lookup_actions};

/// Maximum raw input bytes (ASCII keystrokes).
const MAX_RAW: usize = 32;
/// Maximum UTF-8 output bytes.
const MAX_OUT: usize = 128;
/// Maximum output chars.
const MAX_CHARS: usize = 12;

/// Apply circumflex to a base vowel: a→â, e→ê, o→ô
#[inline]
const fn apply_circumflex(c: char) -> Option<char> {
    match c {
        'a' => Some('â'),
        'e' => Some('ê'),
        'o' => Some('ô'),
        _ => None,
    }
}

/// Apply breve: a→ă
#[inline]
const fn apply_breve(c: char) -> Option<char> {
    match c {
        'a' => Some('ă'),
        _ => None,
    }
}

/// Apply horn: o→ơ, u→ư
#[inline]
const fn apply_horn(c: char) -> Option<char> {
    match c {
        'o' => Some('ơ'),
        'u' => Some('ư'),
        _ => None,
    }
}

/// Apply stroke: d→đ
#[inline]
const fn apply_stroke(c: char) -> Option<char> {
    match c {
        'd' => Some('đ'),
        _ => None,
    }
}

/// Apply a letter modification to a base char.
#[inline]
fn apply_mod_to_base(c: char, mod_: LetterModification) -> Option<char> {
    match mod_ {
        LetterModification::Circumflex => apply_circumflex(c),
        LetterModification::Breve => apply_breve(c),
        LetterModification::Horn => apply_horn(c),
        LetterModification::Stroke => apply_stroke(c),
    }
}

/// Find the last position in chars[0..len] whose vowel family matches `family_char`.
/// The family_char is the BASE vowel char (e.g., 'a', 'e', 'o').
/// Only matches chars in the PLAIN family (vowel_id of the family char), not modified variants.
fn find_family_target(chars: &[char; MAX_CHARS], len: usize, family_char: char) -> Option<usize> {
    let family_lower = family_char.to_ascii_lowercase();
    let family_id = vowel_to_id(family_lower)?;
    for i in (0..len).rev() {
        let c_lower = chars[i].to_lowercase().next().unwrap_or(chars[i]);
        if let Some(id) = vowel_to_id(c_lower) {
            if id == family_id {
                return Some(i);
            }
        }
    }
    None
}

/// Find the last position in chars[0..len] that can accept `mod_`.
fn find_mod_target(chars: &[char; MAX_CHARS], len: usize, mod_: LetterModification) -> Option<usize> {
    match mod_ {
        LetterModification::Horn => {
            // Look for 'o' (vowel_id 6) or 'u' (vowel_id 9) family — base form only
            for i in (0..len).rev() {
                let c_lower = chars[i].to_lowercase().next().unwrap_or(chars[i]);
                if let Some(id) = vowel_to_id(c_lower) {
                    if id == 6 || id == 9 {
                        return Some(i);
                    }
                }
            }
            None
        }
        LetterModification::Breve => {
            // Look for 'a' family (vowel_id 0)
            for i in (0..len).rev() {
                let c_lower = chars[i].to_lowercase().next().unwrap_or(chars[i]);
                if let Some(id) = vowel_to_id(c_lower) {
                    if id == 0 {
                        return Some(i);
                    }
                }
            }
            None
        }
        LetterModification::Stroke => {
            // Look for 'd'
            for i in (0..len).rev() {
                if chars[i].to_ascii_lowercase() == 'd' {
                    return Some(i);
                }
            }
            None
        }
        LetterModification::Circumflex => {
            // Look for 'a' (0), 'e' (3), or 'o' (6) family — base form only
            for i in (0..len).rev() {
                let c_lower = chars[i].to_lowercase().next().unwrap_or(chars[i]);
                if let Some(id) = vowel_to_id(c_lower) {
                    if id == 0 || id == 3 || id == 6 {
                        return Some(i);
                    }
                }
            }
            None
        }
    }
}

/// Apply a modification to chars[pos], preserving case and existing tone.
fn apply_modification_at(chars: &mut [char; MAX_CHARS], pos: usize, mod_: LetterModification) -> bool {
    let orig = chars[pos];
    let is_upper = orig.is_uppercase();
    // Unicode-aware lowercase for non-ASCII chars like 'Ê'
    let c_lower = orig.to_lowercase().next().unwrap_or(orig);

    // For consonant-like chars ('d'), extract_tone returns (c, 0)
    let (base, tone) = extract_tone(c_lower);

    if let Some(new_base) = apply_mod_to_base(base, mod_) {
        let toned = tables_apply_tone(new_base, tone);
        chars[pos] = if is_upper {
            toned.to_uppercase().next().unwrap_or(toned)
        } else {
            toned
        };
        true
    } else {
        false
    }
}

/// Convert ToneMark enum to tone_id (0-5).
#[inline]
const fn tone_mark_to_id(tm: ToneMark) -> u8 {
    tm as u8
}

/// Find the best vowel position in chars[initial_end..vowel_end] for tone placement.
/// Returns index into `chars` array (not relative to vowel start).
fn find_tone_target(
    chars: &[char; MAX_CHARS],
    len: usize,
    vowel_start: usize,
    vowel_end: usize,
) -> Option<usize> {
    if vowel_start >= vowel_end {
        return None;
    }

    let vowel_count = vowel_end - vowel_start;

    if vowel_count == 1 {
        return Some(vowel_start);
    }

    // Priority 1: modified vowel (â, ă, ê, ô, ơ, ư) takes precedence
    for i in vowel_start..vowel_end {
        let c_lower = chars[i].to_lowercase().next().unwrap_or(chars[i]);
        if is_modified_base(c_lower) {
            return Some(i);
        }
    }

    let has_final = vowel_end < len;

    // Build clean vowel string for pattern matching (lowercase, tone-stripped)
    let mut clean_buf = ['\0'; MAX_CHARS];
    let mut clean_len = 0usize;
    for i in vowel_start..vowel_end {
        let c_lower = chars[i].to_lowercase().next().unwrap_or(chars[i]);
        let (base, _) = extract_tone(c_lower);
        clean_buf[clean_len] = base;
        clean_len += 1;
    }

    // Special patterns: oa, oe → second vowel; uy → second vowel
    if clean_len == 2 {
        let (c0, c1) = (clean_buf[0], clean_buf[1]);
        if (c0 == 'o' && (c1 == 'a' || c1 == 'e')) || (c0 == 'u' && c1 == 'y') {
            return Some(vowel_start + 1);
        }
    }

    if has_final {
        // Closed syllable: last vowel
        Some(vowel_end - 1)
    } else {
        // Open syllable: second-to-last vowel
        if vowel_count >= 2 {
            Some(vowel_end - 2)
        } else {
            Some(vowel_start)
        }
    }
}

/// Returns true if `c` is a modified base vowel (has diacritic shape).
#[inline]
const fn is_modified_base(c: char) -> bool {
    matches!(c, 'â' | 'ă' | 'ê' | 'ô' | 'ơ' | 'ư')
}

/// Returns true if a char is a vowel (any Vietnamese vowel, any tone).
#[inline]
fn is_vowel_char(c: char) -> bool {
    // Use Unicode-aware lowercase to handle uppercase non-ASCII chars like 'Ê', 'Ô'
    let c_lower = c.to_lowercase().next().unwrap_or(c);
    vowel_to_id(c_lower).is_some()
}

/// Zero-allocation Vietnamese input engine.
///
/// All buffers are stack-allocated. Returns `&str` from internal buffer.
pub struct FastEngine {
    raw: [u8; MAX_RAW],
    raw_len: u8,
    out_utf8: [u8; MAX_OUT],
    out_utf8_len: u8,
    method: InputMethod,
}

impl FastEngine {
    /// Creates a new engine with the specified input method.
    pub fn new(method: InputMethod) -> Self {
        Self {
            raw: [0; MAX_RAW],
            raw_len: 0,
            out_utf8: [0; MAX_OUT],
            out_utf8_len: 0,
            method,
        }
    }

    /// Creates a new Telex engine.
    pub fn telex() -> Self {
        Self::new(InputMethod::Telex)
    }

    /// Creates a new VNI engine.
    pub fn vni() -> Self {
        Self::new(InputMethod::Vni)
    }

    /// Feeds a character and returns the current output.
    pub fn feed(&mut self, ch: char) -> &str {
        if self.raw_len < MAX_RAW as u8 && ch.is_ascii() {
            self.raw[self.raw_len as usize] = ch as u8;
            self.raw_len += 1;
        }
        self.render()
    }

    /// Removes the last keystroke and returns updated output.
    pub fn backspace(&mut self) -> &str {
        if self.raw_len > 0 {
            self.raw_len -= 1;
        }
        self.render()
    }

    /// Resets the engine for the next syllable.
    pub fn clear(&mut self) {
        self.raw_len = 0;
        self.out_utf8_len = 0;
    }

    /// Returns the current output as a borrowed string.
    pub fn output(&self) -> &str {
        core::str::from_utf8(&self.out_utf8[..self.out_utf8_len as usize])
            .unwrap_or("")
    }

    /// Returns the raw keystrokes as a borrowed string.
    pub fn raw_input(&self) -> &str {
        core::str::from_utf8(&self.raw[..self.raw_len as usize])
            .unwrap_or("")
    }

    /// Sets the input method.
    pub fn set_method(&mut self, method: InputMethod) {
        self.method = method;
    }

    /// Renders raw input to UTF-8 output using a 2-pass pipeline.
    ///
    /// Pass 1: scan raw bytes → resolve modifications → build chars[] + track tone + CVC bounds
    /// Pass 2: apply tone to the correct vowel → encode UTF-8
    fn render(&mut self) -> &str {
        let definition = get_definition(self.method);

        // ── Pass 1: Build chars[] with modifications applied ──────────────────
        let mut chars = ['\0'; MAX_CHARS];
        let mut n: usize = 0;
        let mut tone: u8 = 0;              // pending tone (0 = none)
        let mut inserted_u_pos: Option<usize> = None; // position of InsertU'd ư

        // Toggle-detection tracking
        let mut last_mod_key: u8 = 0;       // key that last applied a modification
        let mut mod_applied_pos: usize = 0; // chars[] index of that modification
        let mut mod_original: char = '\0';  // original char before modification
        let mut last_tone_key: u8 = 0;      // key that last applied a tone

        for i in 0..self.raw_len as usize {
            let raw_byte = self.raw[i];
            let ch = raw_byte as char;
            let ch_lower = ch.to_ascii_lowercase() as u8;
            let prev_lower = if i > 0 { self.raw[i-1].to_ascii_lowercase() } else { 0 };

            // ── Toggle: same mod key pressed consecutively ──────────────────
            if ch_lower == last_mod_key && prev_lower == last_mod_key && last_mod_key != 0 {
                // Undo the modification
                chars[mod_applied_pos] = mod_original;
                // Append the current raw char as literal
                if n < MAX_CHARS { chars[n] = ch; n += 1; }
                last_mod_key = 0;
                continue;
            }

            // ── Toggle: same tone key pressed consecutively ─────────────────
            if ch_lower == last_tone_key && prev_lower == last_tone_key && last_tone_key != 0 {
                // Undo the tone
                tone = 0;
                // Append the current raw char as literal
                if n < MAX_CHARS { chars[n] = ch; n += 1; }
                last_tone_key = 0;
                continue;
            }

            let ch_char = ch_lower as char;
            let actions = lookup_actions(definition, ch_char);
            let mut applied = false;

            if let Some(actions) = actions {
                'action_loop: for action in actions.iter() {
                    match action {
                        Action::ModifyLetterOnFamily(mod_, family) => {
                            if let Some(pos) = find_family_target(&chars, n, *family) {
                                let orig = chars[pos];
                                if apply_modification_at(&mut chars, pos, *mod_) {
                                    last_mod_key = ch_lower;
                                    mod_applied_pos = pos;
                                    mod_original = orig;
                                    last_tone_key = 0;
                                    applied = true;
                                    break 'action_loop;
                                }
                            }
                        }
                        Action::ModifyLetter(mod_) => {
                            if let Some(pos) = find_mod_target(&chars, n, *mod_) {
                                let orig = chars[pos];
                                if apply_modification_at(&mut chars, pos, *mod_) {
                                    last_mod_key = ch_lower;
                                    mod_applied_pos = pos;
                                    mod_original = orig;
                                    last_tone_key = 0;
                                    applied = true;
                                    break 'action_loop;
                                }
                            }
                        }
                        Action::AddTone(tm) => {
                            // Only apply if there's at least one vowel in chars
                            if n > 0 && chars[0..n].iter().any(|c| is_vowel_char(*c)) {
                                tone = tone_mark_to_id(*tm);
                                last_tone_key = ch_lower;
                                last_mod_key = 0;
                                applied = true;
                                break 'action_loop;
                            }
                        }
                        Action::RemoveTone => {
                            if tone > 0 {
                                tone = 0;
                                last_tone_key = 0;
                                applied = true;
                                break 'action_loop;
                            }
                        }
                        Action::InsertU => {
                            if n < MAX_CHARS {
                                chars[n] = 'ư';
                                inserted_u_pos = Some(n);
                                // Don't track in last_mod_key — ResetInsertedU handles undo
                                last_tone_key = 0;
                                n += 1;
                                applied = true;
                                break 'action_loop;
                            }
                        }
                        Action::ResetInsertedU => {
                            if let Some(pos) = inserted_u_pos {
                                // Remove the inserted ư, shifting remaining chars left
                                for j in pos..n.saturating_sub(1) {
                                    chars[j] = chars[j + 1];
                                }
                                if n > 0 { n -= 1; }
                                inserted_u_pos = None;
                                last_mod_key = 0;
                                // Don't set applied=true — let the raw 'w' be appended as literal
                                break 'action_loop;
                            }
                        }
                        Action::AppendChar(c) => {
                            if n < MAX_CHARS {
                                chars[n] = *c;
                                n += 1;
                                applied = true;
                                break 'action_loop;
                            }
                        }
                    }
                }
            }

            if !applied {
                // Append the raw char as-is (preserve original case)
                if n < MAX_CHARS {
                    chars[n] = ch;
                    n += 1;
                }
                // Clear toggle tracking when raw char is appended
                last_mod_key = 0;
                last_tone_key = 0;
            }
        }

        // ── Pass 2: Apply tone to the correct vowel + encode UTF-8 ───────────
        if tone > 0 {
            // Determine CVC bounds for tone placement
            let vowel_start = find_vowel_start(&chars, n);
            let vowel_end = find_vowel_end(&chars, n, vowel_start);

            let tone_pos = find_tone_target(&chars, n, vowel_start, vowel_end);

            if let Some(pos) = tone_pos {
                // Apply tone, preserving case
                let orig = chars[pos];
                let is_upper = orig.is_uppercase();
                let c_lower = orig.to_ascii_lowercase();
                let result = tables_apply_tone(c_lower, tone);
                chars[pos] = if is_upper {
                    result.to_uppercase().next().unwrap_or(result)
                } else {
                    result
                };
            }
        }

        // Encode chars[0..n] to out_utf8
        self.out_utf8_len = 0;
        for i in 0..n {
            let c = chars[i];
            let len = c.len_utf8();
            if (self.out_utf8_len as usize) + len <= MAX_OUT {
                c.encode_utf8(&mut self.out_utf8[self.out_utf8_len as usize..]);
                self.out_utf8_len += len as u8;
            }
        }

        self.output()
    }
}

/// Find the start index of the vowel region in chars[0..len].
fn find_vowel_start(chars: &[char; MAX_CHARS], len: usize) -> usize {
    for i in 0..len {
        if is_vowel_char(chars[i]) {
            return i;
        }
    }
    len // no vowels
}

/// Find the end index (exclusive) of the vowel region starting at vowel_start.
fn find_vowel_end(chars: &[char; MAX_CHARS], len: usize, vowel_start: usize) -> usize {
    if vowel_start >= len {
        return len;
    }
    let mut i = vowel_start;
    while i < len && is_vowel_char(chars[i]) {
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::InputMethod;

    fn type_seq(engine: &mut FastEngine, s: &str) -> String {
        for ch in s.chars() {
            engine.feed(ch);
        }
        engine.output().to_string()
    }

    // ── Task 1 tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_plain_ascii_passthrough() {
        let mut e = FastEngine::new(InputMethod::Telex);
        assert_eq!(type_seq(&mut e, "hello"), "hello");
    }

    #[test]
    fn test_clear_resets() {
        let mut e = FastEngine::new(InputMethod::Telex);
        type_seq(&mut e, "hello");
        e.clear();
        assert_eq!(e.output(), "");
        assert_eq!(e.raw_input(), "");
    }

    #[test]
    fn test_single_char() {
        let mut e = FastEngine::new(InputMethod::Telex);
        assert_eq!(type_seq(&mut e, "a"), "a");
        e.clear();
        assert_eq!(type_seq(&mut e, "b"), "b");
    }

    // ── Task 2 tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_telex_circumflex() {
        let mut e = FastEngine::telex();
        assert_eq!(type_seq(&mut e, "aa"), "â");
        e.clear();
        assert_eq!(type_seq(&mut e, "ee"), "ê");
        e.clear();
        assert_eq!(type_seq(&mut e, "oo"), "ô");
    }

    #[test]
    fn test_telex_breve() {
        let mut e = FastEngine::telex();
        assert_eq!(type_seq(&mut e, "aw"), "ă");
    }

    #[test]
    fn test_telex_horn() {
        let mut e = FastEngine::telex();
        assert_eq!(type_seq(&mut e, "ow"), "ơ");
        e.clear();
        assert_eq!(type_seq(&mut e, "uw"), "ư");
    }

    #[test]
    fn test_telex_stroke() {
        let mut e = FastEngine::telex();
        assert_eq!(type_seq(&mut e, "dd"), "đ");
    }

    #[test]
    fn test_word_with_modification() {
        let mut e = FastEngine::telex();
        assert_eq!(type_seq(&mut e, "vieet"), "viêt");
    }

    // ── Task 3 tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_telex_tones() {
        let mut e = FastEngine::telex();
        assert_eq!(type_seq(&mut e, "as"), "á");
        e.clear();
        assert_eq!(type_seq(&mut e, "af"), "à");
        e.clear();
        assert_eq!(type_seq(&mut e, "ar"), "ả");
        e.clear();
        assert_eq!(type_seq(&mut e, "ax"), "ã");
        e.clear();
        assert_eq!(type_seq(&mut e, "aj"), "ạ");
    }

    #[test]
    fn test_tone_on_multi_vowel() {
        // "vieetj" -> "việt": tone on ê (the modified vowel)
        let mut e = FastEngine::telex();
        assert_eq!(type_seq(&mut e, "vieetj"), "việt");
    }

    #[test]
    fn test_tone_placement_closed_syllable() {
        // "toois" -> "tối": circumflex on o, tone on ô
        let mut e = FastEngine::telex();
        assert_eq!(type_seq(&mut e, "toois"), "tối");
    }

    // ── Task 4 tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_triple_undo() {
        let mut e = FastEngine::telex();
        assert_eq!(type_seq(&mut e, "aaa"), "aa");
        e.clear();
        assert_eq!(type_seq(&mut e, "ddd"), "dd");
        e.clear();
        assert_eq!(type_seq(&mut e, "eee"), "ee");
        e.clear();
        assert_eq!(type_seq(&mut e, "ooo"), "oo");
    }

    #[test]
    fn test_tone_undo() {
        let mut e = FastEngine::telex();
        assert_eq!(type_seq(&mut e, "ass"), "as");
        e.clear();
        assert_eq!(type_seq(&mut e, "aff"), "af");
    }

    #[test]
    fn test_z_removes_tone() {
        let mut e = FastEngine::telex();
        assert_eq!(type_seq(&mut e, "asz"), "a");
    }

    // ── Task 5 tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_case_first_upper() {
        let mut e = FastEngine::telex();
        assert_eq!(type_seq(&mut e, "Vieetj"), "Việt");
    }

    #[test]
    fn test_case_all_upper() {
        let mut e = FastEngine::telex();
        assert_eq!(type_seq(&mut e, "VIEETJ"), "VIỆT");
    }

    #[test]
    fn test_standalone_w() {
        let mut e = FastEngine::telex();
        assert_eq!(type_seq(&mut e, "w"), "ư");
    }

    #[test]
    fn test_ww_undo() {
        let mut e = FastEngine::telex();
        assert_eq!(type_seq(&mut e, "ww"), "w");
    }

    #[test]
    fn test_bracket_shortcuts() {
        let mut e = FastEngine::telex();
        e.feed('[');
        assert_eq!(e.output(), "ư");
        e.clear();
        e.feed(']');
        assert_eq!(e.output(), "ơ");
    }

    // ── Task 6 tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_vni_basic() {
        let mut e = FastEngine::vni();
        assert_eq!(type_seq(&mut e, "a6"), "â");
        e.clear();
        assert_eq!(type_seq(&mut e, "a1"), "á");
        e.clear();
        assert_eq!(type_seq(&mut e, "d9"), "đ");
    }

    #[test]
    fn test_vni_combined() {
        let mut e = FastEngine::vni();
        assert_eq!(type_seq(&mut e, "vie6t5"), "việt");
    }

    // ── Task 7 tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_backspace() {
        let mut e = FastEngine::telex();
        type_seq(&mut e, "viee");
        assert_eq!(e.output(), "viê");
        e.backspace();
        assert_eq!(e.output(), "vie");
        assert_eq!(e.raw_input(), "vie");
        e.backspace();
        assert_eq!(e.output(), "vi");
    }

    #[test]
    fn test_backspace_empty() {
        let mut e = FastEngine::telex();
        e.backspace();
        assert_eq!(e.output(), "");
    }
}
