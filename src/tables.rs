//! Lookup tables for Vietnamese character transformations.
//!
//! These tables provide O(1) lookups for vowel/tone combinations,
//! character classification, and validation.

/// All 12 Vietnamese vowel bases with their 6 tone variants each.
/// Index: [vowel_id][tone_id] where tone_id: 0=none, 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
pub const TONE_VOWELS: [[char; 6]; 12] = [
    ['a', 'á', 'à', 'ả', 'ã', 'ạ'], // 0: a
    ['ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ'], // 1: ă
    ['â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ'], // 2: â
    ['e', 'é', 'è', 'ẻ', 'ẽ', 'ẹ'], // 3: e
    ['ê', 'ế', 'ề', 'ể', 'ễ', 'ệ'], // 4: ê
    ['i', 'í', 'ì', 'ỉ', 'ĩ', 'ị'], // 5: i
    ['o', 'ó', 'ò', 'ỏ', 'õ', 'ọ'], // 6: o
    ['ô', 'ố', 'ồ', 'ổ', 'ỗ', 'ộ'], // 7: ô
    ['ơ', 'ớ', 'ờ', 'ở', 'ỡ', 'ợ'], // 8: ơ
    ['u', 'ú', 'ù', 'ủ', 'ũ', 'ụ'], // 9: u
    ['ư', 'ứ', 'ừ', 'ử', 'ữ', 'ự'], // 10: ư
    ['y', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ'], // 11: y
];

/// Maps a vowel character to its base ID (0-11) or None if not a vowel.
#[inline]
pub const fn vowel_to_id(c: char) -> Option<usize> {
    match c {
        'a' | 'á' | 'à' | 'ả' | 'ã' | 'ạ' => Some(0),
        'ă' | 'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' => Some(1),
        'â' | 'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' => Some(2),
        'e' | 'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' => Some(3),
        'ê' | 'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' => Some(4),
        'i' | 'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị' => Some(5),
        'o' | 'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ' => Some(6),
        'ô' | 'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' => Some(7),
        'ơ' | 'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' => Some(8),
        'u' | 'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ' => Some(9),
        'ư' | 'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' => Some(10),
        'y' | 'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ' => Some(11),
        _ => None,
    }
}

/// Checks if a character is a Vietnamese vowel (any tone variant).
#[inline]
pub const fn is_vowel(c: char) -> bool {
    vowel_to_id(c).is_some()
}

/// Checks if a character is a modified vowel (has diacritics like â, ă, ê, ô, ơ, ư).
#[inline]
pub const fn is_modified_vowel(c: char) -> bool {
    matches!(c, 'â' | 'ă' | 'ê' | 'ô' | 'ơ' | 'ư' |
             'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' |
             'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' |
             'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' |
             'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' |
             'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' |
             'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự')
}

/// Applies a tone to a vowel character.
/// 
/// # Arguments
/// * `c` - The vowel character (may already have a tone)
/// * `tone` - Tone ID: 0=none, 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
/// 
/// # Returns
/// The vowel with the new tone applied, or the original char if not a vowel.
#[inline]
pub fn apply_tone(c: char, tone: u8) -> char {
    let is_upper = c.is_uppercase();
    let c_lower = c.to_lowercase().next().unwrap_or(c);
    
    if let Some(id) = vowel_to_id(c_lower) {
        let tone_idx = (tone as usize).min(5);
        let result = TONE_VOWELS[id][tone_idx];
        if is_upper {
            result.to_uppercase().next().unwrap_or(result)
        } else {
            result
        }
    } else {
        c
    }
}

/// Extracts the tone from a vowel character.
/// 
/// # Returns
/// (base_vowel, tone_id) where tone_id is 0-5 or 0 if not a vowel.
#[inline]
pub fn extract_tone(c: char) -> (char, u8) {
    let is_upper = c.is_uppercase();
    let c_lower = c.to_lowercase().next().unwrap_or(c);
    
    if let Some(vowel_id) = vowel_to_id(c_lower) {
        // Find which tone slot this character is in
        for tone in 0..6 {
            if TONE_VOWELS[vowel_id][tone] == c_lower {
                let base = TONE_VOWELS[vowel_id][0];
                let result = if is_upper {
                    base.to_uppercase().next().unwrap_or(base)
                } else {
                    base
                };
                return (result, tone as u8);
            }
        }
    }
    (c, 0)
}

/// Telex tone key mappings: character -> tone_id
pub const TELEX_TONES: [u8; 128] = {
    let mut t = [0u8; 128];
    t[b's' as usize] = 1; // sắc
    t[b'f' as usize] = 2; // huyền
    t[b'r' as usize] = 3; // hỏi
    t[b'x' as usize] = 4; // ngã
    t[b'j' as usize] = 5; // nặng
    t[b'z' as usize] = 0; // remove tone
    t
};

/// VNI tone key mappings: character -> tone_id
pub const VNI_TONES: [u8; 128] = {
    let mut t = [0u8; 128];
    t[b'1' as usize] = 1; // sắc
    t[b'2' as usize] = 2; // huyền
    t[b'3' as usize] = 3; // hỏi
    t[b'4' as usize] = 4; // ngã
    t[b'5' as usize] = 5; // nặng
    t[b'0' as usize] = 0; // remove tone
    t
};

/// Checks if a character is a VNI tone key.
#[inline]
pub const fn is_vni_tone_key(c: char) -> bool {
    matches!(c, '0' | '1' | '2' | '3' | '4' | '5')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vowel_to_id() {
        assert_eq!(vowel_to_id('a'), Some(0));
        assert_eq!(vowel_to_id('á'), Some(0));
        assert_eq!(vowel_to_id('ă'), Some(1));
        assert_eq!(vowel_to_id('â'), Some(2));
        assert_eq!(vowel_to_id('ư'), Some(10));
        assert_eq!(vowel_to_id('b'), None);
    }

    #[test]
    fn test_apply_tone() {
        assert_eq!(apply_tone('a', 1), 'á');
        assert_eq!(apply_tone('a', 2), 'à');
        assert_eq!(apply_tone('ă', 1), 'ắ');
        assert_eq!(apply_tone('ư', 5), 'ự');
        assert_eq!(apply_tone('A', 1), 'Á');
        assert_eq!(apply_tone('b', 1), 'b'); // non-vowel unchanged
    }

    #[test]
    fn test_extract_tone() {
        assert_eq!(extract_tone('á'), ('a', 1));
        assert_eq!(extract_tone('à'), ('a', 2));
        assert_eq!(extract_tone('ắ'), ('ă', 1));
        assert_eq!(extract_tone('a'), ('a', 0));
        assert_eq!(extract_tone('Á'), ('A', 1));
    }

}
