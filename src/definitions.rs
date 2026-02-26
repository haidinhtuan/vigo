//! Input method definitions for Telex and VNI.
//!
//! Each definition maps keystrokes to a list of actions. Actions are tried
//! in order until one succeeds.

use crate::action::Action;
use crate::syllable::{ToneMark, LetterModification};

/// A definition maps a character to a list of possible actions.
/// Actions are tried in order until one succeeds.
pub type Definition = &'static [(char, &'static [Action])];

/// Telex input method definition.
///
/// ## Tone marks
/// - `s` → sắc (acute)
/// - `f` → huyền (grave)
/// - `r` → hỏi (hook above)
/// - `x` → ngã (tilde)
/// - `j` → nặng (dot below)
/// - `z` → remove tone
///
/// ## Letter modifications
/// - `a` → â (circumflex on 'a' family)
/// - `e` → ê (circumflex on 'e' family)
/// - `o` → ô (circumflex on 'o' family)
/// - `w` → ư/ơ (horn) or ă (breve) or standalone ư
/// - `d` → đ (stroke)
///
/// ## Shortcuts
/// - `[` → ư
/// - `]` → ơ
pub static TELEX: Definition = &[
    // Tone marks
    ('s', &[Action::AddTone(ToneMark::Acute)]),
    ('f', &[Action::AddTone(ToneMark::Grave)]),
    ('r', &[Action::AddTone(ToneMark::HookAbove)]),
    ('x', &[Action::AddTone(ToneMark::Tilde)]),
    ('j', &[Action::AddTone(ToneMark::Underdot)]),
    ('z', &[Action::RemoveTone]),
    
    // Letter modifications (family-specific)
    ('a', &[Action::ModifyLetterOnFamily(LetterModification::Circumflex, 'a')]),
    ('e', &[Action::ModifyLetterOnFamily(LetterModification::Circumflex, 'e')]),
    ('o', &[Action::ModifyLetterOnFamily(LetterModification::Circumflex, 'o')]),
    
    // 'w' has multiple possible actions, tried in order
    ('w', &[
        Action::ResetInsertedU,                           // ưw → uw (undo)
        Action::ModifyLetter(LetterModification::Horn),   // u→ư, o→ơ
        Action::ModifyLetter(LetterModification::Breve),  // a→ă
        Action::InsertU,                                  // standalone → ư
    ]),
    
    // Stroke for đ
    ('d', &[Action::ModifyLetter(LetterModification::Stroke)]),
    
    // Bracket shortcuts
    ('[', &[Action::AppendChar('ư')]),
    (']', &[Action::AppendChar('ơ')]),
];

/// VNI input method definition.
///
/// ## Tone marks
/// - `1` → sắc (acute)
/// - `2` → huyền (grave)
/// - `3` → hỏi (hook above)
/// - `4` → ngã (tilde)
/// - `5` → nặng (dot below)
/// - `0` → remove tone
///
/// ## Letter modifications
/// - `6` → circumflex (â, ê, ô)
/// - `7` → horn (ơ, ư)
/// - `8` → breve (ă)
/// - `9` → stroke (đ)
pub static VNI: Definition = &[
    // Tone marks
    ('1', &[Action::AddTone(ToneMark::Acute)]),
    ('2', &[Action::AddTone(ToneMark::Grave)]),
    ('3', &[Action::AddTone(ToneMark::HookAbove)]),
    ('4', &[Action::AddTone(ToneMark::Tilde)]),
    ('5', &[Action::AddTone(ToneMark::Underdot)]),
    ('0', &[Action::RemoveTone]),
    
    // Letter modifications
    ('6', &[Action::ModifyLetter(LetterModification::Circumflex)]),
    ('7', &[Action::ModifyLetter(LetterModification::Horn)]),
    ('8', &[Action::ModifyLetter(LetterModification::Breve)]),
    ('9', &[Action::ModifyLetter(LetterModification::Stroke)]),
];

/// Looks up actions for a given key in a definition.
pub fn lookup_actions(definition: Definition, key: char) -> Option<&'static [Action]> {
    let key_lower = key.to_ascii_lowercase();
    for (k, actions) in definition {
        if *k == key_lower {
            return Some(actions);
        }
    }
    None
}

/// Returns the definition for a given input method.
pub fn get_definition(method: crate::action::InputMethod) -> Definition {
    match method {
        crate::action::InputMethod::Telex => TELEX,
        crate::action::InputMethod::Vni => VNI,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_telex_tone() {
        let actions = lookup_actions(TELEX, 's').unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::AddTone(ToneMark::Acute));
    }

    #[test]
    fn test_lookup_telex_w() {
        let actions = lookup_actions(TELEX, 'w').unwrap();
        assert_eq!(actions.len(), 4);
    }

    #[test]
    fn test_lookup_vni_tone() {
        let actions = lookup_actions(VNI, '1').unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0], Action::AddTone(ToneMark::Acute));
    }

    #[test]
    fn test_lookup_nonexistent() {
        assert!(lookup_actions(TELEX, 'q').is_none());
    }

    #[test]
    fn test_lookup_case_insensitive() {
        let actions = lookup_actions(TELEX, 'S').unwrap();
        assert_eq!(actions[0], Action::AddTone(ToneMark::Acute));
    }
}
