//! Tone placement and modification logic for Vietnamese syllables.
//!
//! This module implements the rules for correct Vietnamese tone mark placement
//! according to both old and new accent styles.

use crate::syllable::{AccentStyle, LetterModification, Syllable, ToneMark};
use crate::tables::{apply_tone as apply_tone_char, extract_tone};

/// Finds the position (index into vowel string) where tone should be placed.
///
/// Vietnamese tone placement rules:
/// 1. If there's only one vowel, tone goes on it
/// 2. If vowel contains ơ, ư, â, ê, ô, ă: tone goes on the modified vowel
/// 3. For oa, oe, uy: tone on second vowel (new style) or first (old style)
/// 4. Closed syllables (ending in consonant): tone on last vowel
/// 5. Open syllables: tone on second-to-last vowel
///
/// # Arguments
/// * `syllable` - The syllable to analyze
/// * `style` - Accent placement style (Old or New)
///
/// # Returns
/// Index into the vowel string where tone should be placed
pub fn find_tone_position(syllable: &Syllable, style: AccentStyle) -> Option<usize> {
    let vowel = &syllable.vowel;
    if vowel.is_empty() {
        return None;
    }

    let vowel_chars: Vec<char> = vowel.chars().collect();
    let vowel_count = vowel_chars.len();

    // Single vowel - tone goes on it
    if vowel_count == 1 {
        return Some(0);
    }

    // Clean vowel for pattern matching
    let clean_vowel = syllable.clean_vowel();
    let has_final_consonant = !syllable.final_consonant.is_empty();

    // Check for modified vowels (ơ, ư, â, ê, ô, ă) - they take priority
    for (i, ch) in vowel_chars.iter().enumerate() {
        let ch_lower = ch.to_lowercase().next().unwrap_or(*ch);
        if is_modified_base(ch_lower) {
            return Some(i);
        }
    }

    // Handle special vowel patterns
    match clean_vowel.as_str() {
        // oa, oe, uy patterns - depends on style
        "oa" | "oe" => {
            match style {
                AccentStyle::New => Some(1), // hòa (new) vs hoà (old)
                AccentStyle::Old => Some(0),
            }
        }
        "uy" => {
            match style {
                AccentStyle::New => Some(1), // quý
                AccentStyle::Old => Some(0),
            }
        }
        "uye" | "uya" => {
            // uyên, uyết - tone on 'e' or 'a'
            Some(2)
        }
        "oai" | "oay" | "oao" => {
            // toại, xoáy - tone on second vowel
            Some(1)
        }
        "uoi" | "ươi" => {
            // người - tone on second vowel
            Some(1)
        }
        _ => {
            // General rules
            if has_final_consonant {
                // Closed syllable: tone on last vowel
                Some(vowel_count - 1)
            } else {
                // Open syllable: tone on second-to-last vowel
                if vowel_count >= 2 {
                    Some(vowel_count - 2)
                } else {
                    Some(0)
                }
            }
        }
    }
}

/// Checks if a character is a modified vowel base (ă, â, ê, ô, ơ, ư)
fn is_modified_base(ch: char) -> bool {
    matches!(ch, 
        'ă' | 'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' |
        'â' | 'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' |
        'ê' | 'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' |
        'ô' | 'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' |
        'ơ' | 'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' |
        'ư' | 'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự'
    )
}

/// Applies a tone mark to the syllable.
///
/// This replaces any existing tone mark.
pub fn apply_tone(syllable: &mut Syllable, tone: ToneMark) {
    // First remove existing tone
    remove_tone(syllable);
    
    // Find position to place tone
    let pos = match find_tone_position(syllable, syllable.accent_style) {
        Some(p) => p,
        None => return,
    };

    // Apply tone to the vowel at position
    let mut vowel_chars: Vec<char> = syllable.vowel.chars().collect();
    if pos < vowel_chars.len() {
        let tone_id = tone_mark_to_id(tone);
        vowel_chars[pos] = apply_tone_char(vowel_chars[pos], tone_id);
    }
    
    syllable.vowel = vowel_chars.into_iter().collect();
    syllable.tone_mark = Some(tone);
}

/// Removes the tone mark from the syllable.
pub fn remove_tone(syllable: &mut Syllable) {
    let mut vowel_chars: Vec<char> = syllable.vowel.chars().collect();
    
    for ch in vowel_chars.iter_mut() {
        let (base, _tone) = extract_tone(*ch);
        *ch = base;
    }
    
    syllable.vowel = vowel_chars.into_iter().collect();
    syllable.tone_mark = None;
}

/// Converts ToneMark enum to tone ID (1-5).
fn tone_mark_to_id(tone: ToneMark) -> u8 {
    match tone {
        ToneMark::Acute => 1,
        ToneMark::Grave => 2,
        ToneMark::HookAbove => 3,
        ToneMark::Tilde => 4,
        ToneMark::Underdot => 5,
    }
}

/// Applies a letter modification to the syllable.
///
/// Modifications include:
/// - Circumflex: a→â, e→ê, o→ô
/// - Breve: a→ă
/// - Horn: o→ơ, u→ư
/// - Stroke: d→đ
pub fn apply_modification(syllable: &mut Syllable, modification: LetterModification) -> bool {
    match modification {
        LetterModification::Stroke => apply_stroke(syllable),
        LetterModification::Circumflex => apply_circumflex(syllable),
        LetterModification::Breve => apply_breve(syllable),
        LetterModification::Horn => apply_horn(syllable),
    }
}

/// Applies the stroke modification to 'd' -> 'đ'.
fn apply_stroke(syllable: &mut Syllable) -> bool {
    let initial = syllable.initial_consonant.to_lowercase();
    if initial == "d" {
        let is_upper = syllable.initial_consonant.chars().next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false);
        syllable.initial_consonant = if is_upper { "Đ".to_string() } else { "đ".to_string() };
        syllable.modifications.push((0, LetterModification::Stroke));
        true
    } else {
        false
    }
}

/// Applies circumflex to a/e/o vowels.
fn apply_circumflex(syllable: &mut Syllable) -> bool {
    apply_vowel_mod(syllable, |ch| {
        let (base, tone) = extract_tone(ch);
        match base.to_lowercase().next().unwrap_or(base) {
            'a' => Some(apply_tone_char(if base.is_uppercase() { 'Â' } else { 'â' }, tone)),
            'e' => Some(apply_tone_char(if base.is_uppercase() { 'Ê' } else { 'ê' }, tone)),
            'o' => Some(apply_tone_char(if base.is_uppercase() { 'Ô' } else { 'ô' }, tone)),
            _ => None,
        }
    }, LetterModification::Circumflex)
}

/// Applies breve to 'a' vowel.
fn apply_breve(syllable: &mut Syllable) -> bool {
    apply_vowel_mod(syllable, |ch| {
        let (base, tone) = extract_tone(ch);
        match base.to_lowercase().next().unwrap_or(base) {
            'a' => Some(apply_tone_char(if base.is_uppercase() { 'Ă' } else { 'ă' }, tone)),
            _ => None,
        }
    }, LetterModification::Breve)
}

/// Applies horn to o/u vowels.
fn apply_horn(syllable: &mut Syllable) -> bool {
    apply_vowel_mod(syllable, |ch| {
        let (base, tone) = extract_tone(ch);
        match base.to_lowercase().next().unwrap_or(base) {
            'o' => Some(apply_tone_char(if base.is_uppercase() { 'Ơ' } else { 'ơ' }, tone)),
            'u' => Some(apply_tone_char(if base.is_uppercase() { 'Ư' } else { 'ư' }, tone)),
            _ => None,
        }
    }, LetterModification::Horn)
}

/// Helper to apply a vowel modification, searching from right to left.
fn apply_vowel_mod<F>(syllable: &mut Syllable, transform: F, mod_type: LetterModification) -> bool
where
    F: Fn(char) -> Option<char>,
{
    let mut vowel_chars: Vec<char> = syllable.vowel.chars().collect();
    
    // Search from right to left for applicable vowel
    for i in (0..vowel_chars.len()).rev() {
        if let Some(new_char) = transform(vowel_chars[i]) {
            vowel_chars[i] = new_char;
            syllable.vowel = vowel_chars.into_iter().collect();
            let offset = syllable.initial_consonant.chars().count();
            syllable.modifications.push((offset + i, mod_type));
            return true;
        }
    }
    
    false
}

/// Removes a letter modification from the syllable.
pub fn remove_modification(syllable: &mut Syllable, modification: LetterModification) -> bool {
    match modification {
        LetterModification::Stroke => remove_stroke(syllable),
        LetterModification::Circumflex => remove_circumflex(syllable),
        LetterModification::Breve => remove_breve(syllable),
        LetterModification::Horn => remove_horn(syllable),
    }
}

fn remove_stroke(syllable: &mut Syllable) -> bool {
    let initial = syllable.initial_consonant.to_lowercase();
    if initial == "đ" {
        let is_upper = syllable.initial_consonant.chars().next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false);
        syllable.initial_consonant = if is_upper { "D".to_string() } else { "d".to_string() };
        syllable.modifications.retain(|(_, m)| *m != LetterModification::Stroke);
        true
    } else {
        false
    }
}

fn remove_circumflex(syllable: &mut Syllable) -> bool {
    remove_vowel_mod(syllable, |ch| {
        let (base, tone) = extract_tone(ch);
        match base.to_lowercase().next().unwrap_or(base) {
            'â' => Some(apply_tone_char(if base.is_uppercase() { 'A' } else { 'a' }, tone)),
            'ê' => Some(apply_tone_char(if base.is_uppercase() { 'E' } else { 'e' }, tone)),
            'ô' => Some(apply_tone_char(if base.is_uppercase() { 'O' } else { 'o' }, tone)),
            _ => None,
        }
    }, LetterModification::Circumflex)
}

fn remove_breve(syllable: &mut Syllable) -> bool {
    remove_vowel_mod(syllable, |ch| {
        let (base, tone) = extract_tone(ch);
        match base.to_lowercase().next().unwrap_or(base) {
            'ă' => Some(apply_tone_char(if base.is_uppercase() { 'A' } else { 'a' }, tone)),
            _ => None,
        }
    }, LetterModification::Breve)
}

fn remove_horn(syllable: &mut Syllable) -> bool {
    remove_vowel_mod(syllable, |ch| {
        let (base, tone) = extract_tone(ch);
        match base.to_lowercase().next().unwrap_or(base) {
            'ơ' => Some(apply_tone_char(if base.is_uppercase() { 'O' } else { 'o' }, tone)),
            'ư' => Some(apply_tone_char(if base.is_uppercase() { 'U' } else { 'u' }, tone)),
            _ => None,
        }
    }, LetterModification::Horn)
}

fn remove_vowel_mod<F>(syllable: &mut Syllable, transform: F, mod_type: LetterModification) -> bool
where
    F: Fn(char) -> Option<char>,
{
    let mut vowel_chars: Vec<char> = syllable.vowel.chars().collect();
    let mut found = false;
    
    for i in 0..vowel_chars.len() {
        if let Some(new_char) = transform(vowel_chars[i]) {
            vowel_chars[i] = new_char;
            found = true;
        }
    }
    
    if found {
        syllable.vowel = vowel_chars.into_iter().collect();
        syllable.modifications.retain(|(_, m)| *m != mod_type);
    }
    
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_tone_position_single() {
        let s = Syllable::parse("ba");
        assert_eq!(find_tone_position(&s, AccentStyle::New), Some(0));
    }

    #[test]
    fn test_find_tone_position_oa() {
        let s = Syllable::parse("hoa");
        // New style: tone on 'a'
        assert_eq!(find_tone_position(&s, AccentStyle::New), Some(1));
        // Old style: tone on 'o'
        assert_eq!(find_tone_position(&s, AccentStyle::Old), Some(0));
    }

    #[test]
    fn test_find_tone_position_modified() {
        let s = Syllable::parse("thuong");
        // Find position - 'uo' has no modified vowel, so follow general rules
        // "thuong" vowel is "uo", closed syllable -> last vowel
        let pos = find_tone_position(&s, AccentStyle::New);
        assert!(pos.is_some());
    }

    #[test]
    fn test_apply_tone() {
        let mut s = Syllable::parse("ba");
        apply_tone(&mut s, ToneMark::Acute);
        assert_eq!(s.vowel, "á");
        assert_eq!(s.tone_mark, Some(ToneMark::Acute));
    }

    #[test]
    fn test_apply_tone_viet() {
        // "viet" has vowel "ie" (no circumflex), closed syllable -> tone on last vowel 'e'
        let mut s = Syllable::parse("viet");
        apply_tone(&mut s, ToneMark::Underdot);
        assert_eq!(s.vowel, "iẹ");
    }

    #[test]
    fn test_apply_tone_viet_with_circumflex() {
        // "việt" requires circumflex first, then tone
        let mut s = Syllable::parse("viet");
        apply_modification(&mut s, LetterModification::Circumflex);
        assert_eq!(s.vowel, "iê"); // e -> ê
        apply_tone(&mut s, ToneMark::Underdot);
        assert_eq!(s.vowel, "iệ"); // ê -> ệ
    }

    #[test]
    fn test_remove_tone() {
        let mut s = Syllable::parse("việt");
        remove_tone(&mut s);
        assert_eq!(s.vowel, "iê");
        assert_eq!(s.tone_mark, None);
    }

    #[test]
    fn test_apply_circumflex() {
        let mut s = Syllable::parse("an");
        assert!(apply_modification(&mut s, LetterModification::Circumflex));
        assert_eq!(s.vowel, "â");
    }

    #[test]
    fn test_apply_breve() {
        let mut s = Syllable::parse("an");
        assert!(apply_modification(&mut s, LetterModification::Breve));
        assert_eq!(s.vowel, "ă");
    }

    #[test]
    fn test_apply_horn() {
        let mut s = Syllable::parse("thuong");
        assert!(apply_modification(&mut s, LetterModification::Horn));
        // Should apply to rightmost applicable vowel (o -> ơ)
        assert!(s.vowel.contains('ơ') || s.vowel.contains('ư'));
    }

    #[test]
    fn test_apply_stroke() {
        let mut s = Syllable::parse("da");
        assert!(apply_modification(&mut s, LetterModification::Stroke));
        assert_eq!(s.initial_consonant, "đ");
    }

    #[test]
    fn test_remove_circumflex() {
        let mut s = Syllable::parse("ân");
        assert!(remove_modification(&mut s, LetterModification::Circumflex));
        assert_eq!(s.vowel, "a");
    }

    #[test]
    fn test_apply_tone_with_modification() {
        let mut s = Syllable::parse("an");
        apply_modification(&mut s, LetterModification::Circumflex);
        apply_tone(&mut s, ToneMark::Acute);
        assert_eq!(s.vowel, "ấ");
    }

    #[test]
    fn test_replace_tone() {
        let mut s = Syllable::parse("ba");
        apply_tone(&mut s, ToneMark::Acute);
        assert_eq!(s.vowel, "á");
        apply_tone(&mut s, ToneMark::Grave);
        assert_eq!(s.vowel, "à");
    }
}
