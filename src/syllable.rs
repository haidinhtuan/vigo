//! Syllable parsing and representation for Vietnamese text.
//!
//! Vietnamese syllables follow a Consonant-Vowel-Consonant (CVC) pattern:
//! - Initial consonant (optional): th, ng, tr, ch, etc.
//! - Vowel (required): a, ă, â, e, ê, i, o, ô, ơ, u, ư, y, and combinations
//! - Final consonant (optional): c, ch, m, n, ng, nh, p, t
//!
//! Example: "thương" → "th" + "ương" + ""

use smallvec::SmallVec;
use crate::tables::is_vowel;

/// Vietnamese tone marks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToneMark {
    /// Dấu sắc (acute accent) - rising tone
    Acute = 1,
    /// Dấu huyền (grave accent) - falling tone  
    Grave = 2,
    /// Dấu hỏi (hook above) - dipping tone
    HookAbove = 3,
    /// Dấu ngã (tilde) - creaky rising tone
    Tilde = 4,
    /// Dấu nặng (dot below) - low broken tone
    Underdot = 5,
}

/// Letter modifications (diacritics on base letters).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LetterModification {
    /// Circumflex (^): a→â, e→ê, o→ô
    Circumflex,
    /// Breve (˘): a→ă
    Breve,
    /// Horn: o→ơ, u→ư
    Horn,
    /// Stroke: d→đ
    Stroke,
}

/// Accent placement style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AccentStyle {
    /// Old style: tone on first vowel in certain cases (hòa)
    Old,
    /// New style: tone on the vowel with modification (hoà)
    #[default]
    New,
}

/// A parsed Vietnamese syllable.
///
/// Stores the components separately for efficient manipulation during
/// transformation. Use `Display` to reconstruct the final string.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Syllable {
    /// Initial consonant (e.g., "th", "ng", "tr")
    pub initial_consonant: String,
    /// Vowel part (e.g., "a", "ươ", "iê")
    pub vowel: String,
    /// Final consonant (e.g., "ng", "ch", "m")
    pub final_consonant: String,
    /// Current tone mark, if any
    pub tone_mark: Option<ToneMark>,
    /// Letter modifications with their positions (relative to full syllable)
    pub modifications: SmallVec<[(usize, LetterModification); 2]>,
    /// Accent placement style
    pub accent_style: AccentStyle,
}

/// Valid initial consonants in Vietnamese.
const INITIAL_CONSONANTS: &[&str] = &[
    "ngh", "nh", "ng", "gh", "gi", "kh", "ph", "th", "tr", "qu",
    "ch", "b", "c", "d", "đ", "g", "h", "k", "l", "m", 
    "n", "p", "r", "s", "t", "v", "x",
];

/// Valid final consonants in Vietnamese.
const FINAL_CONSONANTS: &[&str] = &[
    "ng", "nh", "ch", "c", "m", "n", "p", "t",
];

impl Syllable {
    /// Creates a new empty syllable.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a syllable with a specific accent style.
    pub fn with_style(accent_style: AccentStyle) -> Self {
        Self {
            accent_style,
            ..Default::default()
        }
    }

    /// Parses a string into a syllable.
    ///
    /// # Example
    /// ```
    /// use vigo::syllable::Syllable;
    /// 
    /// let s = Syllable::parse("thang");
    /// assert_eq!(s.initial_consonant, "th");
    /// assert_eq!(s.vowel, "a");
    /// assert_eq!(s.final_consonant, "ng");
    /// ```
    pub fn parse(input: &str) -> Self {
        let mut syllable = Self::new();
        syllable.set(input);
        syllable
    }

    /// Sets the syllable from a raw string, parsing it into components.
    pub fn set(&mut self, input: &str) {
        let input_lower = input.to_lowercase();
        let chars: Vec<char> = input_lower.chars().collect();
        
        if chars.is_empty() {
            self.clear();
            return;
        }

        // Extract initial consonant
        let (initial, rest_start) = self.extract_initial_consonant(&input_lower);
        self.initial_consonant = initial;

        // Extract final consonant (from the end)
        let rest = &input_lower[rest_start..];
        let (vowel, final_cons) = self.extract_vowel_and_final(rest);
        self.vowel = vowel;
        self.final_consonant = final_cons;

        // Extract tone mark from vowels
        self.tone_mark = self.extract_tone_from_vowel();
        
        // Extract letter modifications
        self.modifications = self.extract_modifications(input);
    }

    /// Extracts the initial consonant from the input.
    fn extract_initial_consonant(&self, input: &str) -> (String, usize) {
        // Check for longer consonants first
        for consonant in INITIAL_CONSONANTS {
            if input.starts_with(consonant) {
                return (consonant.to_string(), consonant.len());
            }
        }
        
        // Check if first char is a consonant
        if let Some(first) = input.chars().next() {
            if !is_vowel(first) && first.is_alphabetic() {
                return (first.to_string(), first.len_utf8());
            }
        }
        
        (String::new(), 0)
    }

    /// Extracts vowel and final consonant from the remaining string.
    fn extract_vowel_and_final(&self, rest: &str) -> (String, String) {
        if rest.is_empty() {
            return (String::new(), String::new());
        }

        let chars: Vec<char> = rest.chars().collect();
        
        // Find where vowels end (last vowel position)
        let mut last_vowel_idx = None;
        for (i, &ch) in chars.iter().enumerate() {
            if is_vowel(ch) {
                last_vowel_idx = Some(i);
            }
        }

        // If no vowel found, treat everything as vowel (edge case)
        let Some(last_vowel) = last_vowel_idx else {
            return (rest.to_string(), String::new());
        };

        // Check if there are consonants after the last vowel
        if last_vowel == chars.len() - 1 {
            // No final consonant - all vowels
            return (rest.to_string(), String::new());
        }

        // Extract potential final consonant
        let vowel_part: String = chars[..=last_vowel].iter().collect();
        let final_part: String = chars[last_vowel + 1..].iter().collect();

        // Special case: vowel clusters ending with ơ/ư + ng are pure vowels
        // e.g., "ương" in "thương" is a vowel cluster, not "ươ" + "ng"
        let last_vowel_char = chars[last_vowel];
        let last_vowel_lower = last_vowel_char.to_lowercase().next().unwrap_or(last_vowel_char);
        if (last_vowel_lower == 'ơ' || last_vowel_lower == 'ư' || 
            is_horn_vowel(last_vowel_lower)) && final_part == "ng" {
            // This is a vowel cluster like ương, not vowel + final consonant
            return (rest.to_string(), String::new());
        }

        // Validate the final consonant
        if FINAL_CONSONANTS.contains(&final_part.as_str()) {
            return (vowel_part, final_part);
        }

        // If not a valid final consonant, treat everything as vowel
        (rest.to_string(), String::new())
    }

    /// Extracts tone mark from the vowel part.
    fn extract_tone_from_vowel(&self) -> Option<ToneMark> {
        for ch in self.vowel.chars() {
            if let Some(tone) = char_to_tone(ch) {
                return Some(tone);
            }
        }
        None
    }

    /// Extracts letter modifications from the input.
    fn extract_modifications(&self, _input: &str) -> SmallVec<[(usize, LetterModification); 2]> {
        let mut mods = SmallVec::new();
        let full = format!("{}{}{}", self.initial_consonant, self.vowel, self.final_consonant);
        
        for (i, ch) in full.chars().enumerate() {
            if let Some(modification) = char_to_modification(ch) {
                mods.push((i, modification));
            }
        }
        
        mods
    }

    /// Pushes a character to the syllable, re-parsing the result.
    pub fn push(&mut self, ch: char) {
        let current = self.to_string();
        let new_input = format!("{}{}", current, ch);
        self.set(&new_input);
    }

    /// Returns true if the syllable is empty.
    pub fn is_empty(&self) -> bool {
        self.initial_consonant.is_empty() 
            && self.vowel.is_empty() 
            && self.final_consonant.is_empty()
    }

    /// Returns the total character count.
    pub fn len(&self) -> usize {
        self.initial_consonant.chars().count()
            + self.vowel.chars().count()
            + self.final_consonant.chars().count()
    }

    /// Clears all components.
    pub fn clear(&mut self) {
        self.initial_consonant.clear();
        self.vowel.clear();
        self.final_consonant.clear();
        self.tone_mark = None;
        self.modifications.clear();
    }

    /// Returns true if the syllable contains a specific modification.
    pub fn has_modification(&self, modification: LetterModification) -> bool {
        self.modifications.iter().any(|(_, m)| *m == modification)
    }

    /// Returns the clean (no tone, no modification) version of the vowel.
    pub fn clean_vowel(&self) -> String {
        self.vowel.chars().map(clean_char).collect()
    }
}

impl std::fmt::Display for Syllable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // For now, just concatenate. Later we'll apply tone and modifications properly.
        write!(f, "{}{}{}", self.initial_consonant, self.vowel, self.final_consonant)
    }
}

/// Converts a character to its tone mark, if any.
fn char_to_tone(ch: char) -> Option<ToneMark> {
    match ch {
        'á' | 'ắ' | 'ấ' | 'é' | 'ế' | 'í' | 'ó' | 'ố' | 'ớ' | 'ú' | 'ứ' | 'ý' => Some(ToneMark::Acute),
        'à' | 'ằ' | 'ầ' | 'è' | 'ề' | 'ì' | 'ò' | 'ồ' | 'ờ' | 'ù' | 'ừ' | 'ỳ' => Some(ToneMark::Grave),
        'ả' | 'ẳ' | 'ẩ' | 'ẻ' | 'ể' | 'ỉ' | 'ỏ' | 'ổ' | 'ở' | 'ủ' | 'ử' | 'ỷ' => Some(ToneMark::HookAbove),
        'ã' | 'ẵ' | 'ẫ' | 'ẽ' | 'ễ' | 'ĩ' | 'õ' | 'ỗ' | 'ỡ' | 'ũ' | 'ữ' | 'ỹ' => Some(ToneMark::Tilde),
        'ạ' | 'ặ' | 'ậ' | 'ẹ' | 'ệ' | 'ị' | 'ọ' | 'ộ' | 'ợ' | 'ụ' | 'ự' | 'ỵ' => Some(ToneMark::Underdot),
        _ => None,
    }
}

/// Converts a character to its letter modification, if any.
fn char_to_modification(ch: char) -> Option<LetterModification> {
    match ch {
        'â' | 'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' => Some(LetterModification::Circumflex),
        'ê' | 'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' => Some(LetterModification::Circumflex),
        'ô' | 'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' => Some(LetterModification::Circumflex),
        'ă' | 'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' => Some(LetterModification::Breve),
        'ơ' | 'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' => Some(LetterModification::Horn),
        'ư' | 'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' => Some(LetterModification::Horn),
        'đ' => Some(LetterModification::Stroke),
        _ => None,
    }
}

/// Checks if a character is a horn vowel (ơ or ư with any tone).
fn is_horn_vowel(ch: char) -> bool {
    matches!(ch, 'ơ' | 'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' |
                 'ư' | 'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự')
}

/// Removes tone and modification from a character, returning the base form.
fn clean_char(ch: char) -> char {
    match ch {
        'á' | 'à' | 'ả' | 'ã' | 'ạ' => 'a',
        'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' | 'ă' => 'a',
        'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' | 'â' => 'a',
        'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' => 'e',
        'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' | 'ê' => 'e',
        'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị' => 'i',
        'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ' => 'o',
        'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' | 'ô' => 'o',
        'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' | 'ơ' => 'o',
        'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ' => 'u',
        'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' | 'ư' => 'u',
        'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ' => 'y',
        'đ' => 'd',
        _ => ch,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let s = Syllable::parse("an");
        assert_eq!(s.initial_consonant, "");
        assert_eq!(s.vowel, "a");
        assert_eq!(s.final_consonant, "n");
    }

    #[test]
    fn test_parse_with_initial() {
        let s = Syllable::parse("ban");
        assert_eq!(s.initial_consonant, "b");
        assert_eq!(s.vowel, "a");
        assert_eq!(s.final_consonant, "n");
    }

    #[test]
    fn test_parse_complex_initial() {
        let s = Syllable::parse("thang");
        assert_eq!(s.initial_consonant, "th");
        assert_eq!(s.vowel, "a");
        assert_eq!(s.final_consonant, "ng");
    }

    #[test]
    fn test_parse_qu_cluster() {
        let s = Syllable::parse("quan");
        assert_eq!(s.initial_consonant, "qu");
        assert_eq!(s.vowel, "a");
        assert_eq!(s.final_consonant, "n");
    }

    #[test]
    fn test_parse_gi_cluster() {
        let s = Syllable::parse("gia");
        assert_eq!(s.initial_consonant, "gi");
        assert_eq!(s.vowel, "a");
        assert_eq!(s.final_consonant, "");
    }

    #[test]
    fn test_parse_vowel_only() {
        let s = Syllable::parse("a");
        assert_eq!(s.initial_consonant, "");
        assert_eq!(s.vowel, "a");
        assert_eq!(s.final_consonant, "");
    }

    #[test]
    fn test_parse_complex_vowel() {
        let s = Syllable::parse("tuyen");
        assert_eq!(s.initial_consonant, "t");
        assert_eq!(s.vowel, "uye");
        assert_eq!(s.final_consonant, "n");
    }

    #[test]
    fn test_parse_with_tone() {
        let s = Syllable::parse("việt");
        assert_eq!(s.initial_consonant, "v");
        assert_eq!(s.vowel, "iệ");
        assert_eq!(s.final_consonant, "t");
        assert_eq!(s.tone_mark, Some(ToneMark::Underdot));
    }

    #[test]
    fn test_parse_ngh() {
        let s = Syllable::parse("nghieng");
        assert_eq!(s.initial_consonant, "ngh");
        assert_eq!(s.vowel, "ie");
        assert_eq!(s.final_consonant, "ng");
    }

    #[test]
    fn test_is_empty() {
        let s = Syllable::new();
        assert!(s.is_empty());
        
        let s = Syllable::parse("a");
        assert!(!s.is_empty());
    }

    #[test]
    fn test_push() {
        let mut s = Syllable::new();
        s.push('v');
        s.push('i');
        s.push('e');
        s.push('t');
        assert_eq!(s.initial_consonant, "v");
        assert_eq!(s.vowel, "ie");
        assert_eq!(s.final_consonant, "t");
    }

    // Additional tests for special cases

    #[test]
    fn test_parse_thuong() {
        // "thương" - no final consonant since ương is the vowel
        let s = Syllable::parse("thương");
        assert_eq!(s.initial_consonant, "th");
        assert_eq!(s.vowel, "ương"); // ương is treated as vowel since ư and ơ are vowels
        assert_eq!(s.final_consonant, "");
    }

    #[test]
    fn test_parse_chuong() {
        let s = Syllable::parse("chuong");
        assert_eq!(s.initial_consonant, "ch");
        assert_eq!(s.vowel, "uo");
        assert_eq!(s.final_consonant, "ng");
    }

    #[test]
    fn test_parse_oa() {
        let s = Syllable::parse("hoa");
        assert_eq!(s.initial_consonant, "h");
        assert_eq!(s.vowel, "oa");
        assert_eq!(s.final_consonant, "");
    }

    #[test]
    fn test_parse_uy() {
        let s = Syllable::parse("quy");
        assert_eq!(s.initial_consonant, "qu");
        assert_eq!(s.vowel, "y");
        assert_eq!(s.final_consonant, "");
    }

    #[test]
    fn test_parse_with_modification() {
        let s = Syllable::parse("đà");
        assert_eq!(s.initial_consonant, "đ");
        assert_eq!(s.vowel, "à");
        assert!(s.has_modification(LetterModification::Stroke));
        assert_eq!(s.tone_mark, Some(ToneMark::Grave));
    }

    #[test]
    fn test_parse_uyen() {
        let s = Syllable::parse("tuyen");
        assert_eq!(s.initial_consonant, "t");
        assert_eq!(s.vowel, "uye");
        assert_eq!(s.final_consonant, "n");
    }

    #[test]
    fn test_parse_oai() {
        let s = Syllable::parse("toai");
        assert_eq!(s.initial_consonant, "t");
        assert_eq!(s.vowel, "oai");
        assert_eq!(s.final_consonant, "");
    }

    #[test]
    fn test_clean_vowel() {
        let s = Syllable::parse("việt");
        assert_eq!(s.clean_vowel(), "ie");
    }

    #[test]
    fn test_len() {
        let s = Syllable::parse("việt");
        assert_eq!(s.len(), 4);
        
        let s = Syllable::parse("a");
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn test_display() {
        let s = Syllable::parse("việt");
        assert_eq!(s.to_string(), "việt");
    }

    #[test]
    fn test_clear() {
        let mut s = Syllable::parse("việt");
        s.clear();
        assert!(s.is_empty());
        assert!(s.tone_mark.is_none());
        assert!(s.modifications.is_empty());
    }
}
