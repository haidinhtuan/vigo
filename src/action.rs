//! Action-based transformation system for Vietnamese input.
//!
//! This module defines the actions that can be triggered by keystrokes
//! and the transformation results.

use crate::syllable::{ToneMark, LetterModification};

/// An action that can be triggered by a keystroke.
///
/// Actions are tried in order until one succeeds. This allows a single key
/// to have multiple possible effects depending on the current syllable state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Add a tone mark to the syllable.
    AddTone(ToneMark),
    
    /// Apply a letter modification (circumflex, breve, horn, stroke).
    ModifyLetter(LetterModification),
    
    /// Apply modification only if a character from the family exists.
    /// For example, Telex 'a' only adds circumflex if 'a' family exists.
    ModifyLetterOnFamily(LetterModification, char),
    
    /// Insert ư at the end of the syllable (standalone 'w' in Telex).
    InsertU,
    
    /// Reset an inserted ư (for 'ww' → 'w' undo in Telex).
    ResetInsertedU,
    
    /// Remove the current tone mark.
    RemoveTone,
    
    /// Append a literal character (no transformation).
    AppendChar(char),
}

/// Result of attempting to apply a transformation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transformation {
    /// Tone mark was added to a syllable without one.
    ToneAdded,
    /// Tone mark was replaced with a different one.
    ToneReplaced,
    /// Tone mark was removed.
    ToneRemoved,
    
    /// Letter modification was added.
    ModificationAdded,
    /// Letter modification was replaced.
    ModificationReplaced,
    /// Letter modification was removed.
    ModificationRemoved,
    
    /// Character was appended.
    CharAppended,
    
    /// The action was ignored (couldn't be applied).
    Ignored,
}

impl Transformation {
    /// Returns true if the transformation was successfully applied.
    pub fn is_applied(&self) -> bool {
        !matches!(self, Transformation::Ignored)
    }
}

/// Input method type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMethod {
    #[default]
    Telex,
    Vni,
}

impl From<crate::transform::InputMethod> for InputMethod {
    fn from(m: crate::transform::InputMethod) -> Self {
        match m {
            crate::transform::InputMethod::Telex => InputMethod::Telex,
            crate::transform::InputMethod::Vni => InputMethod::Vni,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transformation_is_applied() {
        assert!(Transformation::ToneAdded.is_applied());
        assert!(Transformation::ModificationAdded.is_applied());
        assert!(!Transformation::Ignored.is_applied());
    }
}
