//! Action-based syllable transformation engine.
//!
//! This module provides `SyllableEngine`, which uses the action-based system
//! to transform Vietnamese input in real-time.

use crate::action::{Action, InputMethod, Transformation};
use crate::definitions::{get_definition, lookup_actions};
use crate::syllable::{AccentStyle, LetterModification, Syllable, ToneMark};
use crate::tone::{apply_modification, apply_tone, remove_modification, remove_tone};

/// History entry for undo support.
#[derive(Debug, Clone)]
struct HistoryEntry {
    /// The key that was pressed
    key: char,
    /// The transformation that was applied
    transformation: Transformation,
    /// Snapshot of syllable before the action
    syllable_before: Syllable,
}

/// Action-based Vietnamese input engine.
///
/// Uses the syllable-based architecture with action definitions
/// for clean separation of input method rules from transformation logic.
pub struct SyllableEngine {
    /// Current syllable being built
    syllable: Syllable,
    /// Raw input characters (for display/undo)
    raw_input: String,
    /// Current input method
    method: InputMethod,
    /// History for undo support
    history: Vec<HistoryEntry>,
    /// Whether the last action was a special insert (for ww -> uw undo)
    last_was_insert_u: bool,
    /// Consecutive same-key count for triple-key undo
    consecutive_count: u8,
    /// Last key pressed
    last_key: Option<char>,
    /// Whether bypass mode is active (no more transformations)
    bypass: bool,
}

impl Default for SyllableEngine {
    fn default() -> Self {
        Self::new(InputMethod::Telex)
    }
}

impl SyllableEngine {
    /// Creates a new engine with the specified input method.
    pub fn new(method: InputMethod) -> Self {
        Self {
            syllable: Syllable::new(),
            raw_input: String::new(),
            method,
            history: Vec::new(),
            last_was_insert_u: false,
            consecutive_count: 0,
            last_key: None,
            bypass: false,
        }
    }

    /// Creates a new engine with Telex input method.
    pub fn telex() -> Self {
        Self::new(InputMethod::Telex)
    }

    /// Creates a new engine with VNI input method.
    pub fn vni() -> Self {
        Self::new(InputMethod::Vni)
    }

    /// Sets the accent style for tone placement.
    pub fn set_accent_style(&mut self, style: AccentStyle) {
        self.syllable.accent_style = style;
    }

    /// Returns the current output string.
    pub fn output(&self) -> String {
        self.syllable.to_string()
    }

    /// Returns the raw input buffer.
    pub fn raw_input(&self) -> &str {
        &self.raw_input
    }

    /// Returns true if the engine is empty.
    pub fn is_empty(&self) -> bool {
        self.raw_input.is_empty()
    }

    /// Clears all state.
    pub fn clear(&mut self) {
        self.syllable.clear();
        self.raw_input.clear();
        self.history.clear();
        self.last_was_insert_u = false;
        self.consecutive_count = 0;
        self.last_key = None;
        self.bypass = false;
    }

    /// Commits the current output and clears state.
    pub fn commit(&mut self) -> String {
        let result = self.output();
        self.clear();
        result
    }

    /// Feeds a character into the engine.
    ///
    /// Returns the transformation result.
    pub fn feed(&mut self, ch: char) -> Transformation {
        self.raw_input.push(ch);
        
        // Track consecutive same keys
        let key_lower = ch.to_ascii_lowercase();
        if self.last_key == Some(key_lower) {
            self.consecutive_count += 1;
        } else {
            self.consecutive_count = 1;
        }
        self.last_key = Some(key_lower);

        // If bypass is active, just append character
        if self.bypass {
            self.syllable.push(ch);
            return Transformation::CharAppended;
        }

        // Check for triple-key undo (aaa -> aa, ddd -> dd)
        if self.consecutive_count >= 3 {
            return self.handle_triple_undo(ch);
        }

        // Look up actions for this key
        let definition = get_definition(self.method);
        if let Some(actions) = lookup_actions(definition, ch) {
            self.execute_actions(ch, actions)
        } else {
            // No special action - just append character
            self.syllable.push(ch);
            Transformation::CharAppended
        }
    }

    /// Handles triple-key undo (e.g., "aaa" -> "aa")
    fn handle_triple_undo(&mut self, ch: char) -> Transformation {
        // Undo the last transformation and enter bypass mode
        if let Some(entry) = self.history.pop() {
            self.syllable = entry.syllable_before;
        }
        // Add the literal character and enter bypass mode
        self.syllable.push(ch);
        self.bypass = true;
        Transformation::CharAppended
    }

    /// Executes a list of actions, returning on first success.
    fn execute_actions(&mut self, key: char, actions: &[Action]) -> Transformation {
        // Save state for potential undo
        let syllable_before = self.syllable.clone();

        for action in actions {
            let result = self.try_action(action);
            if result != Transformation::Ignored {
                // Check for double-tap undo
                if self.consecutive_count == 2 && self.should_undo_double_tap(&result) {
                    return self.handle_double_tap_undo(key, syllable_before);
                }

                // Record history for undo
                self.history.push(HistoryEntry {
                    key,
                    transformation: result,
                    syllable_before,
                });

                // Track InsertU for ww -> uw undo
                self.last_was_insert_u = matches!(action, Action::InsertU);

                return result;
            }
        }

        // No action applied - append character
        self.syllable.push(key);
        Transformation::CharAppended
    }

    /// Checks if this is a double-tap that should trigger undo.
    fn should_undo_double_tap(&self, result: &Transformation) -> bool {
        // Double-tap tone should undo (ass -> as)
        matches!(result, Transformation::ToneAdded | Transformation::ToneReplaced)
    }

    /// Handles double-tap undo (e.g., "ass" -> "as").
    fn handle_double_tap_undo(&mut self, key: char, syllable_before: Syllable) -> Transformation {
        // Restore previous state
        if let Some(entry) = self.history.pop() {
            self.syllable = entry.syllable_before;
        } else {
            self.syllable = syllable_before;
        }
        // Append the literal character and enter bypass
        self.syllable.push(key);
        self.bypass = true;
        Transformation::CharAppended
    }

    /// Tries to apply a single action.
    fn try_action(&mut self, action: &Action) -> Transformation {
        match action {
            Action::AddTone(tone) => self.try_add_tone(*tone),
            Action::ModifyLetter(modification) => self.try_modify_letter(*modification),
            Action::ModifyLetterOnFamily(modification, family) => {
                self.try_modify_letter_on_family(*modification, *family)
            }
            Action::InsertU => self.try_insert_u(),
            Action::ResetInsertedU => self.try_reset_inserted_u(),
            Action::RemoveTone => self.try_remove_tone(),
            Action::AppendChar(ch) => {
                self.syllable.push(*ch);
                Transformation::CharAppended
            }
        }
    }

    /// Tries to add a tone mark.
    fn try_add_tone(&mut self, tone: ToneMark) -> Transformation {
        if self.syllable.vowel.is_empty() {
            return Transformation::Ignored;
        }

        let had_tone = self.syllable.tone_mark.is_some();
        apply_tone(&mut self.syllable, tone);

        if had_tone {
            Transformation::ToneReplaced
        } else {
            Transformation::ToneAdded
        }
    }

    /// Tries to apply a letter modification.
    fn try_modify_letter(&mut self, modification: LetterModification) -> Transformation {
        if apply_modification(&mut self.syllable, modification) {
            Transformation::ModificationAdded
        } else {
            Transformation::Ignored
        }
    }

    /// Tries to apply a letter modification only if the family exists.
    fn try_modify_letter_on_family(
        &mut self,
        modification: LetterModification,
        family: char,
    ) -> Transformation {
        // Check if the family character exists in the syllable's vowel
        let family_lower = family.to_ascii_lowercase();
        let has_family = self.syllable.vowel.chars().any(|c| {
            let c_base = get_base_vowel(c);
            c_base == family_lower
        });

        if !has_family {
            return Transformation::Ignored;
        }

        // Apply modification to the family
        if apply_modification(&mut self.syllable, modification) {
            Transformation::ModificationAdded
        } else {
            Transformation::Ignored
        }
    }

    /// Tries to insert ư at the end.
    fn try_insert_u(&mut self) -> Transformation {
        // Check if syllable has content to attach to
        if self.syllable.is_empty() {
            // Insert standalone ư
            self.syllable.push('ư');
            self.last_was_insert_u = true;
            return Transformation::CharAppended;
        }

        // Try to convert last 'u' or 'o' to horn form first
        // This is handled by ModifyLetter(Horn), so InsertU only adds standalone ư
        self.syllable.push('ư');
        self.last_was_insert_u = true;
        Transformation::CharAppended
    }

    /// Tries to reset an inserted ư (for ww -> uw undo).
    fn try_reset_inserted_u(&mut self) -> Transformation {
        if !self.last_was_insert_u {
            return Transformation::Ignored;
        }

        // Check if syllable ends with ư
        let vowel_chars: Vec<char> = self.syllable.vowel.chars().collect();
        if let Some(&last) = vowel_chars.last() {
            let last_lower = last.to_lowercase().next().unwrap_or(last);
            if last_lower == 'ư' || is_horn_u(last_lower) {
                // Convert ư back to u
                if remove_modification(&mut self.syllable, LetterModification::Horn) {
                    self.last_was_insert_u = false;
                    self.bypass = true;
                    return Transformation::ModificationRemoved;
                }
            }
        }

        Transformation::Ignored
    }

    /// Tries to remove the tone mark.
    fn try_remove_tone(&mut self) -> Transformation {
        if self.syllable.tone_mark.is_none() {
            return Transformation::Ignored;
        }

        remove_tone(&mut self.syllable);
        Transformation::ToneRemoved
    }

    /// Removes the last character (backspace).
    pub fn backspace(&mut self) -> Option<char> {
        let ch = self.raw_input.pop();
        if ch.is_some() {
            // Rebuild syllable from raw input
            self.rebuild_from_raw();
        }
        ch
    }

    /// Rebuilds the syllable from raw input.
    fn rebuild_from_raw(&mut self) {
        self.syllable.clear();
        self.history.clear();
        self.last_was_insert_u = false;
        self.consecutive_count = 0;
        self.last_key = None;
        self.bypass = false;

        // Re-feed all characters
        let raw = self.raw_input.clone();
        self.raw_input.clear();
        for ch in raw.chars() {
            self.feed(ch);
        }
    }
}

/// Gets the base vowel (without diacritics) for family matching.
fn get_base_vowel(ch: char) -> char {
    let ch_lower = ch.to_lowercase().next().unwrap_or(ch);
    match ch_lower {
        'á' | 'à' | 'ả' | 'ã' | 'ạ' | 'ă' | 'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' | 
        'â' | 'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' => 'a',
        'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' | 'ê' | 'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' => 'e',
        'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị' => 'i',
        'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ' | 'ô' | 'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' |
        'ơ' | 'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' => 'o',
        'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ' | 'ư' | 'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' => 'u',
        'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ' => 'y',
        _ => ch_lower,
    }
}

/// Checks if a character is ư with any tone.
fn is_horn_u(ch: char) -> bool {
    matches!(ch, 'ư' | 'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn type_seq(engine: &mut SyllableEngine, s: &str) -> String {
        for ch in s.chars() {
            engine.feed(ch);
        }
        engine.output()
    }

    #[test]
    fn test_basic_telex_circumflex() {
        let mut e = SyllableEngine::telex();
        assert_eq!(type_seq(&mut e, "aa"), "â");

        let mut e = SyllableEngine::telex();
        assert_eq!(type_seq(&mut e, "ee"), "ê");

        let mut e = SyllableEngine::telex();
        assert_eq!(type_seq(&mut e, "oo"), "ô");
    }

    #[test]
    fn test_telex_breve() {
        let mut e = SyllableEngine::telex();
        assert_eq!(type_seq(&mut e, "aw"), "ă");
    }

    #[test]
    fn test_telex_horn() {
        let mut e = SyllableEngine::telex();
        assert_eq!(type_seq(&mut e, "ow"), "ơ");

        let mut e = SyllableEngine::telex();
        assert_eq!(type_seq(&mut e, "uw"), "ư");
    }

    #[test]
    fn test_telex_stroke() {
        let mut e = SyllableEngine::telex();
        assert_eq!(type_seq(&mut e, "dd"), "đ");
    }

    #[test]
    fn test_telex_tones() {
        let mut e = SyllableEngine::telex();
        assert_eq!(type_seq(&mut e, "as"), "á");

        let mut e = SyllableEngine::telex();
        assert_eq!(type_seq(&mut e, "af"), "à");

        let mut e = SyllableEngine::telex();
        assert_eq!(type_seq(&mut e, "ar"), "ả");

        let mut e = SyllableEngine::telex();
        assert_eq!(type_seq(&mut e, "ax"), "ã");

        let mut e = SyllableEngine::telex();
        assert_eq!(type_seq(&mut e, "aj"), "ạ");
    }

    #[test]
    fn test_telex_combined() {
        let mut e = SyllableEngine::telex();
        assert_eq!(type_seq(&mut e, "vieejt"), "việt");
    }

    #[test]
    fn test_triple_undo() {
        let mut e = SyllableEngine::telex();
        assert_eq!(type_seq(&mut e, "aaa"), "aa");

        let mut e = SyllableEngine::telex();
        assert_eq!(type_seq(&mut e, "ddd"), "dd");
    }

    #[test]
    fn test_double_tap_tone_undo() {
        let mut e = SyllableEngine::telex();
        assert_eq!(type_seq(&mut e, "ass"), "as");

        let mut e = SyllableEngine::telex();
        assert_eq!(type_seq(&mut e, "aff"), "af");
    }

    #[test]
    fn test_z_removes_tone() {
        let mut e = SyllableEngine::telex();
        type_seq(&mut e, "as");
        assert_eq!(e.output(), "á");
        e.feed('z');
        assert_eq!(e.output(), "a");
    }

    #[test]
    fn test_backspace() {
        let mut e = SyllableEngine::telex();
        type_seq(&mut e, "viee");
        assert_eq!(e.output(), "viê");
        e.backspace();
        // After removing one 'e', raw input is "vie" which gives "vie" (no circumflex)
        assert_eq!(e.output(), "vie");
        assert_eq!(e.raw_input(), "vie");
        e.backspace();
        assert_eq!(e.output(), "vi");
    }

    #[test]
    fn test_commit_clears() {
        let mut e = SyllableEngine::telex();
        type_seq(&mut e, "chaof");
        let result = e.commit();
        assert_eq!(result, "chào");
        assert!(e.is_empty());
    }

    #[test]
    fn test_vni_basic() {
        let mut e = SyllableEngine::vni();
        assert_eq!(type_seq(&mut e, "a6"), "â");

        let mut e = SyllableEngine::vni();
        assert_eq!(type_seq(&mut e, "a1"), "á");

        let mut e = SyllableEngine::vni();
        assert_eq!(type_seq(&mut e, "d9"), "đ");
    }

    #[test]
    fn test_thuong_with_horn() {
        // Test applying horn to "thuong" -> "thương"
        let mut e = SyllableEngine::telex();
        // Type th-u-o-n-g-w (w applies horn to o)
        type_seq(&mut e, "thuongw");
        assert!(e.output().contains('ơ')); // Should have ơ
    }

    #[test]
    fn test_bracket_shortcuts() {
        let mut e = SyllableEngine::telex();
        e.feed('[');
        assert_eq!(e.output(), "ư");

        let mut e = SyllableEngine::telex();
        e.feed(']');
        assert_eq!(e.output(), "ơ");
    }

    #[test]
    fn test_combined_modification_and_tone() {
        // Test "ân" with tone
        let mut e = SyllableEngine::telex();
        assert_eq!(type_seq(&mut e, "aasn"), "ấn");
    }

    #[test]
    fn test_horn_on_u() {
        let mut e = SyllableEngine::telex();
        assert_eq!(type_seq(&mut e, "tuw"), "tư");
    }

    #[test]
    fn test_horn_on_o() {
        let mut e = SyllableEngine::telex();
        assert_eq!(type_seq(&mut e, "tow"), "tơ");
    }
}
