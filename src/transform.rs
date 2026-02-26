//! Core transformation logic for Vietnamese input.
//!
//! This module handles the transformation of raw Telex/VNI input into Vietnamese text.

use crate::tables::{
    apply_tone, extract_tone, is_modified_vowel,
    is_vni_tone_key, is_vowel, TELEX_TONES, VNI_TONES,
};

/// Input method variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMethod {
    #[default]
    Telex,
    Vni,
}

/// Transforms a raw input buffer into Vietnamese text.
///
/// This is the main entry point for transformation. It processes the entire
/// buffer and handles multiple words separated by whitespace or punctuation.
///
/// # Examples
/// ```
/// use vigo::transform_buffer;
/// assert_eq!(transform_buffer("vieetj"), "việt");
/// assert_eq!(transform_buffer("xin chaof"), "xin chào");
/// ```
#[cfg(feature = "std")]
pub fn transform_buffer(buffer: &str) -> String {
    transform_buffer_with_method(buffer, InputMethod::Telex)
}

/// Transforms a buffer using a specific input method.
#[cfg(feature = "std")]
pub fn transform_buffer_with_method(buffer: &str, method: InputMethod) -> String {
    let mut result = String::new();
    let mut current_word = String::new();

    for ch in buffer.chars() {
        if ch.is_whitespace() || (ch.is_ascii_punctuation() && ch != '[' && ch != ']') {
            result.push_str(&transform_word(&current_word, method));
            result.push(ch);
            current_word.clear();
        } else {
            current_word.push(ch);
        }
    }
    
    result.push_str(&transform_word(&current_word, method));
    result
}

/// Transforms a single word according to Telex/VNI rules.
#[cfg(feature = "std")]
pub fn transform_word(word: &str, method: InputMethod) -> String {
    if word.is_empty() {
        return String::new();
    }

    let mut out_chars: Vec<char> = Vec::new();
    let mut out_upper: Vec<bool> = Vec::new();
    let mut tone: Option<u8> = None;
    let mut bypass = false;

    let chars: Vec<char> = word.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let current = chars[i];
        let current_lower = current.to_ascii_lowercase();
        
        if bypass {
            out_chars.push(current_lower);
            out_upper.push(current.is_uppercase());
            i += 1;
            continue;
        }

        let next = chars.get(i + 1).copied();
        let next_lower = next.map(|c| c.to_ascii_lowercase());
        let third_lower = chars.get(i + 2).map(|c| c.to_ascii_lowercase());

        match method {
            InputMethod::Telex => {
                match (current_lower, next_lower, third_lower) {
                    // Triple-key undo patterns
                    ('a', Some('a'), Some('a')) | ('e', Some('e'), Some('e')) | 
                    ('o', Some('o'), Some('o')) | ('d', Some('d'), Some('d')) => {
                        out_chars.push(current_lower);
                        out_upper.push(chars[i].is_uppercase());
                        out_chars.push(current_lower);
                        out_upper.push(chars[i + 1].is_uppercase());
                        bypass = true;
                        i += 3;
                    }
                    
                    // Circumflex vowels
                    ('a', Some('a'), _) => {
                        out_chars.push('â');
                        out_upper.push(current.is_uppercase());
                        i += 2;
                    }
                    ('e', Some('e'), _) => {
                        out_chars.push('ê');
                        out_upper.push(current.is_uppercase());
                        i += 2;
                    }
                    ('o', Some('o'), _) => {
                        out_chars.push('ô');
                        out_upper.push(current.is_uppercase());
                        i += 2;
                    }
                    
                    // Breve and horn with 'w'
                    ('a', Some('w'), _) => {
                        out_chars.push('ă');
                        out_upper.push(current.is_uppercase());
                        i += 2;
                    }
                    ('o', Some('w'), _) => {
                        out_chars.push('ơ');
                        out_upper.push(current.is_uppercase());
                        i += 2;
                    }
                    ('u', Some('w'), _) => {
                        out_chars.push('ư');
                        out_upper.push(current.is_uppercase());
                        i += 2;
                    }
                    
                    // Đ
                    ('d', Some('d'), _) => {
                        out_chars.push('đ');
                        out_upper.push(current.is_uppercase());
                        i += 2;
                    }
                    
                    // Double-w undo
                    ('w', Some('w'), _) => {
                        if let Some(&last) = out_chars.last() {
                            let last_lower = last.to_lowercase().next().unwrap_or(last);
                            if last_lower == 'ư' {
                                let was_upper = *out_upper.last().unwrap_or(&false);
                                out_chars.pop();
                                out_upper.pop();
                                out_chars.push('u');
                                out_upper.push(was_upper);
                                out_chars.push('w');
                                out_upper.push(chars[i].is_uppercase());
                                bypass = true;
                            } else if last_lower == 'ơ' {
                                let was_upper = *out_upper.last().unwrap_or(&false);
                                out_chars.pop();
                                out_upper.pop();
                                out_chars.push('o');
                                out_upper.push(was_upper);
                                out_chars.push('w');
                                out_upper.push(chars[i].is_uppercase());
                                bypass = true;
                            } else if last_lower == 'ă' {
                                let was_upper = *out_upper.last().unwrap_or(&false);
                                out_chars.pop();
                                out_upper.pop();
                                out_chars.push('a');
                                out_upper.push(was_upper);
                                out_chars.push('w');
                                out_upper.push(chars[i].is_uppercase());
                                bypass = true;
                            } else {
                                out_chars.push('w');
                                out_upper.push(chars[i].is_uppercase());
                                bypass = true;
                            }
                        } else {
                            out_chars.push('w');
                            out_upper.push(chars[i].is_uppercase());
                            bypass = true;
                        }
                        i += 2;
                    }
                    
                    // Standalone 'w'
                    ('w', _, _) => {
                        if let Some(&last) = out_chars.last() {
                            let last_lower = last.to_lowercase().next().unwrap_or(last);
                            if last_lower == 'u' {
                                out_chars.pop();
                                out_chars.push('ư');
                            } else if last_lower == 'o' {
                                out_chars.pop();
                                out_chars.push('ơ');
                            } else if last_lower == 'ă' {
                                let was_upper = *out_upper.last().unwrap_or(&false);
                                out_chars.pop();
                                out_upper.pop();
                                out_chars.push('a');
                                out_upper.push(was_upper);
                                out_chars.push('w');
                                out_upper.push(current.is_uppercase());
                                bypass = true;
                            } else {
                                out_chars.push('ư');
                                out_upper.push(current.is_uppercase());
                            }
                        } else {
                            out_chars.push('ư');
                            out_upper.push(current.is_uppercase());
                        }
                        i += 1;
                    }
                    
                    // Bracket shortcuts
                    ('[', _, _) => {
                        out_chars.push('ư');
                        out_upper.push(false);
                        i += 1;
                    }
                    (']', _, _) => {
                        out_chars.push('ơ');
                        out_upper.push(false);
                        i += 1;
                    }
                    
                    // Tone keys
                    ('s', _, _) | ('f', _, _) | ('r', _, _) | ('x', _, _) | ('j', _, _) => {
                        let has_vowel = out_chars.iter().any(|&c| is_vowel(c.to_lowercase().next().unwrap_or(c)));
                        
                        if has_vowel {
                            let new_tone = TELEX_TONES[current_lower as usize];
                            if tone == Some(new_tone) {
                                // Double-tap: undo and output literal
                                tone = None;
                                out_chars.push(current_lower);
                                out_upper.push(current.is_uppercase());
                                bypass = true;
                            } else {
                                tone = Some(new_tone);
                            }
                        } else {
                            out_chars.push(current);
                            out_upper.push(current.is_uppercase());
                        }
                        i += 1;
                    }
                    
                    // 'z' removes tone
                    ('z', _, _) => {
                        let has_vowel = out_chars.iter().any(|&c| is_vowel(c.to_lowercase().next().unwrap_or(c)));
                        
                        if has_vowel && tone.is_some() {
                            tone = None;
                            bypass = true;
                        } else {
                            out_chars.push(current);
                            out_upper.push(current.is_uppercase());
                        }
                        i += 1;
                    }
                    
                    // Regular character
                    (c, _, _) => {
                        out_chars.push(c);
                        out_upper.push(current.is_uppercase());
                        i += 1;
                    }
                }
            }
            
            InputMethod::Vni => {
                match (current_lower, next_lower) {
                    // Vowel modifications
                    ('a', Some('6')) => {
                        out_chars.push('â');
                        out_upper.push(current.is_uppercase());
                        i += 2;
                    }
                    ('a', Some('8')) => {
                        out_chars.push('ă');
                        out_upper.push(current.is_uppercase());
                        i += 2;
                    }
                    ('e', Some('6')) => {
                        out_chars.push('ê');
                        out_upper.push(current.is_uppercase());
                        i += 2;
                    }
                    ('o', Some('6')) => {
                        out_chars.push('ô');
                        out_upper.push(current.is_uppercase());
                        i += 2;
                    }
                    ('o', Some('7')) => {
                        out_chars.push('ơ');
                        out_upper.push(current.is_uppercase());
                        i += 2;
                    }
                    ('u', Some('7')) => {
                        out_chars.push('ư');
                        out_upper.push(current.is_uppercase());
                        i += 2;
                    }
                    ('d', Some('9')) => {
                        out_chars.push('đ');
                        out_upper.push(current.is_uppercase());
                        i += 2;
                    }
                    
                    // Tone keys
                    _ if is_vni_tone_key(current_lower) => {
                        let has_vowel = out_chars.iter().any(|&c| is_vowel(c.to_lowercase().next().unwrap_or(c)));
                        
                        if has_vowel {
                            let new_tone = VNI_TONES[current_lower as usize];
                            tone = Some(new_tone);
                        } else {
                            out_chars.push(current);
                            out_upper.push(current.is_uppercase());
                        }
                        i += 1;
                    }
                    
                    // Regular character
                    _ => {
                        out_chars.push(current_lower);
                        out_upper.push(current.is_uppercase());
                        i += 1;
                    }
                }
            }
        }
    }

    // Convert to string
    let mut out_str: String = out_chars.iter().collect();
    
    if bypass {
        // Restore case and return
        let mut result = String::new();
        for (i, ch) in out_str.chars().enumerate() {
            if *out_upper.get(i).unwrap_or(&false) {
                result.push(ch.to_uppercase().next().unwrap_or(ch));
            } else {
                result.push(ch);
            }
        }
        return result;
    }
    
    // Handle uơ -> ươ conversion
    if out_str.contains("uơ") {
        out_str = out_str.replace("uơ", "ươ");
    }
    
    // Apply tone to the appropriate vowel
    if let Some(t) = tone {
        out_str = apply_tone_to_word(&out_str, t);
    }
    
    // Restore case
    let mut result = String::new();
    for (i, ch) in out_str.chars().enumerate() {
        if *out_upper.get(i).unwrap_or(&false) {
            result.push(ch.to_uppercase().next().unwrap_or(ch));
        } else {
            result.push(ch);
        }
    }
    
    // Handle all-uppercase input
    let has_uppercase = word.chars().any(|c| c.is_uppercase());
    if has_uppercase && word.chars().all(|c| !c.is_alphabetic() || c.is_uppercase()) {
        result = result.to_uppercase();
    }
    
    // Relocate tone if needed
    relocate_tone(&result)
}

/// Applies a tone to the appropriate vowel in a word.
fn apply_tone_to_word(word: &str, tone: u8) -> String {
    let chars: Vec<char> = word.chars().collect();
    
    // Find all vowel positions
    let mut vowel_indices: Vec<usize> = Vec::new();
    for (i, &c) in chars.iter().enumerate() {
        if is_vowel(c.to_lowercase().next().unwrap_or(c)) {
            vowel_indices.push(i);
        }
    }
    
    if vowel_indices.is_empty() {
        return word.to_string();
    }
    
    // Handle qu/gi clusters
    if vowel_indices.len() > 1 && chars.len() >= 2 {
        let first = chars[0].to_ascii_lowercase();
        let second = chars[1].to_ascii_lowercase();
        if (first == 'q' && second == 'u') || (first == 'g' && second == 'i') {
            vowel_indices.remove(0);
        }
    }
    
    let last_char = chars.last().map(|c| c.to_lowercase().next().unwrap_or(*c)).unwrap_or(' ');
    let ends_with_consonant = !is_vowel(last_char);
    
    // Determine target vowel for tone placement
    let target_idx = if vowel_indices.len() == 1 {
        vowel_indices[0]
    } else {
        // Check for modified vowels first
        let modified_idx = vowel_indices.iter().rfind(|&&idx| {
            is_modified_vowel(chars[idx].to_lowercase().next().unwrap_or(chars[idx]))
        });
        
        if let Some(&idx) = modified_idx {
            idx
        } else if ends_with_consonant {
            // Closed syllable: tone on last vowel
            *vowel_indices.last().unwrap()
        } else {
            // Open syllable: tone on second-to-last vowel
            if vowel_indices.len() >= 2 {
                vowel_indices[vowel_indices.len() - 2]
            } else {
                vowel_indices[0]
            }
        }
    };
    
    // Build result with tone applied
    let mut result = String::new();
    for (i, &c) in chars.iter().enumerate() {
        if i == target_idx {
            result.push(apply_tone(c, tone));
        } else {
            result.push(c);
        }
    }
    
    result
}

/// Relocates a misplaced tone to the correct vowel.
fn relocate_tone(word: &str) -> String {
    // Find any existing tone
    let mut found_tone: Option<u8> = None;
    let mut chars_without_tone = String::new();
    
    for ch in word.chars() {
        let (base, tone) = extract_tone(ch);
        if tone != 0 && found_tone.is_none() {
            found_tone = Some(tone);
        }
        chars_without_tone.push(base);
    }
    
    if let Some(tone) = found_tone {
        apply_tone_to_word(&chars_without_tone, tone)
    } else {
        word.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_telex() {
        assert_eq!(transform_buffer("aa"), "â");
        assert_eq!(transform_buffer("aw"), "ă");
        assert_eq!(transform_buffer("ee"), "ê");
        assert_eq!(transform_buffer("oo"), "ô");
        assert_eq!(transform_buffer("ow"), "ơ");
        assert_eq!(transform_buffer("uw"), "ư");
        assert_eq!(transform_buffer("dd"), "đ");
    }

    #[test]
    fn test_tones() {
        assert_eq!(transform_buffer("as"), "á");
        assert_eq!(transform_buffer("af"), "à");
        assert_eq!(transform_buffer("ar"), "ả");
        assert_eq!(transform_buffer("ax"), "ã");
        assert_eq!(transform_buffer("aj"), "ạ");
    }

    #[test]
    fn test_combined() {
        assert_eq!(transform_buffer("vieetj"), "việt");
        assert_eq!(transform_buffer("chaof"), "chào");
        assert_eq!(transform_buffer("xin chaof"), "xin chào");
    }

    #[test]
    fn test_triple_undo() {
        assert_eq!(transform_buffer("aaa"), "aa");
        assert_eq!(transform_buffer("eee"), "ee");
        assert_eq!(transform_buffer("ooo"), "oo");
        assert_eq!(transform_buffer("ddd"), "dd");
    }

    #[test]
    fn test_double_tap_tone() {
        assert_eq!(transform_buffer("ass"), "as");
        assert_eq!(transform_buffer("aff"), "af");
    }

    #[test]
    fn test_z_removes_tone() {
        assert_eq!(transform_buffer("asz"), "a");
    }

    #[test]
    fn test_qu_gi() {
        assert_eq!(transform_buffer("quas"), "quá");
        assert_eq!(transform_buffer("giaf"), "già");
    }

    #[test]
    fn test_vni() {
        assert_eq!(transform_buffer_with_method("a6", InputMethod::Vni), "â");
        assert_eq!(transform_buffer_with_method("a1", InputMethod::Vni), "á");
        assert_eq!(transform_buffer_with_method("d9", InputMethod::Vni), "đ");
    }
}
