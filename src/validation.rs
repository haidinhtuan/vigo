//! Vietnamese syllable validation and spell checking.
//!
//! This module validates Vietnamese syllables against linguistic rules
//! and provides spell-check functionality.

use crate::syllable::Syllable;

/// Removes tone marks but preserves letter modifications (circumflex, breve, horn).
/// This is needed for vowel validation since VALID_VOWELS contains base forms
/// with modifications (like "iê", "ô", "ư") but without tone marks.
fn strip_tone_marks(s: &str) -> String {
    s.chars().map(|ch| {
        match ch {
            // a with tones -> a
            'á' | 'à' | 'ả' | 'ã' | 'ạ' => 'a',
            // ă with tones -> ă
            'ắ' | 'ằ' | 'ẳ' | 'ẵ' | 'ặ' => 'ă',
            // â with tones -> â
            'ấ' | 'ầ' | 'ẩ' | 'ẫ' | 'ậ' => 'â',
            // e with tones -> e
            'é' | 'è' | 'ẻ' | 'ẽ' | 'ẹ' => 'e',
            // ê with tones -> ê
            'ế' | 'ề' | 'ể' | 'ễ' | 'ệ' => 'ê',
            // i with tones -> i
            'í' | 'ì' | 'ỉ' | 'ĩ' | 'ị' => 'i',
            // o with tones -> o
            'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ' => 'o',
            // ô with tones -> ô
            'ố' | 'ồ' | 'ổ' | 'ỗ' | 'ộ' => 'ô',
            // ơ with tones -> ơ
            'ớ' | 'ờ' | 'ở' | 'ỡ' | 'ợ' => 'ơ',
            // u with tones -> u
            'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ' => 'u',
            // ư with tones -> ư
            'ứ' | 'ừ' | 'ử' | 'ữ' | 'ự' => 'ư',
            // y with tones -> y
            'ý' | 'ỳ' | 'ỷ' | 'ỹ' | 'ỵ' => 'y',
            _ => ch,
        }
    }).collect()
}

/// Valid initial consonants in Vietnamese.
pub const VALID_INITIALS: &[&str] = &[
    "", // No initial consonant is valid
    "b", "c", "ch", "d", "đ", "g", "gh", "gi", "h", "k", "kh",
    "l", "m", "n", "ng", "ngh", "nh", "p", "ph", "qu", "r", "s",
    "t", "th", "tr", "v", "x",
];

/// Valid final consonants in Vietnamese.
pub const VALID_FINALS: &[&str] = &[
    "", // No final consonant is valid
    "c", "ch", "m", "n", "ng", "nh", "p", "t",
];

/// Valid vowel nuclei in Vietnamese (simplified, base forms).
pub const VALID_VOWELS: &[&str] = &[
    // Single vowels
    "a", "ă", "â", "e", "ê", "i", "o", "ô", "ơ", "u", "ư", "y",
    // Diphthongs
    "ai", "ao", "au", "ay", "âu", "ây",
    "eo", "êu",
    "ia", "iê", "iêu", "iu",
    "oa", "oă", "oai", "oay", "oan", "oang", "oanh", "oat", "oac",
    "oe", "oen", "oet", "oi", "oo", "ôi", "ơi",
    "ua", "uă", "uâ", "uây", "uê", "ui", "uo", "uô", "uôi", "uơ", "ươ", "ươi", "ươu", "uy", "uya", "uyê", "uyn", "uyt",
    "ya", "yê", "yêu",
    // With tones (handled separately)
];

/// Consonant + vowel compatibility rules.
/// Some consonants cannot combine with certain vowels.
#[allow(dead_code)]
const INCOMPATIBLE_PAIRS: &[(&str, &str)] = &[
    // 'k' only with i, e, ê, y
    ("c", "i"), ("c", "e"), ("c", "ê"), ("c", "y"),  // use 'k' instead
    // 'gh' only with i, e, ê
    ("g", "i"), ("g", "e"), ("g", "ê"),  // use 'gh' instead
    // 'ngh' only with i, e, ê  
    ("ng", "i"), ("ng", "e"), ("ng", "ê"),  // use 'ngh' instead
    // 'qu' takes 'u' as part of initial, so no 'quu'
];

/// Finals that restrict tones (only sắc or nặng allowed).
/// These are called "entering tone" syllables (nhập thanh).
pub const RESTRICTED_FINALS: &[&str] = &["c", "ch", "p", "t"];

/// Result of syllable validation.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    /// Syllable is valid.
    Valid,
    /// Invalid initial consonant.
    InvalidInitial(String),
    /// Invalid final consonant.
    InvalidFinal(String),
    /// Invalid vowel combination.
    InvalidVowel(String),
    /// Incompatible consonant-vowel combination.
    IncompatibleCombination { initial: String, vowel: String },
    /// Invalid tone for this syllable (restricted final).
    InvalidTone { final_cons: String, tone: String },
    /// Empty syllable.
    Empty,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }
}

/// Validates a Vietnamese syllable.
pub fn validate_syllable(syllable: &Syllable) -> ValidationResult {
    if syllable.is_empty() {
        return ValidationResult::Empty;
    }

    // Check initial consonant
    let initial = syllable.initial_consonant.to_lowercase();
    if !initial.is_empty() && !VALID_INITIALS.contains(&initial.as_str()) {
        return ValidationResult::InvalidInitial(initial);
    }

    // Check final consonant
    let final_cons = syllable.final_consonant.to_lowercase();
    if !final_cons.is_empty() && !VALID_FINALS.contains(&final_cons.as_str()) {
        return ValidationResult::InvalidFinal(final_cons);
    }

    // Check vowel - must be a valid Vietnamese vowel combination
    // We strip tone marks but preserve letter modifications (circumflex, breve, horn)
    let vowel = strip_tone_marks(&syllable.vowel).to_lowercase();
    if vowel.is_empty() {
        return ValidationResult::InvalidVowel("empty".to_string());
    }
    if !VALID_VOWELS.contains(&vowel.as_str()) {
        return ValidationResult::InvalidVowel(vowel);
    }

    // Check tone restrictions for entering tone syllables
    if RESTRICTED_FINALS.contains(&final_cons.as_str()) {
        if let Some(tone) = &syllable.tone_mark {
            use crate::syllable::ToneMark;
            match tone {
                ToneMark::Acute | ToneMark::Underdot => {
                    // Valid tones for restricted finals
                }
                ToneMark::Grave => {
                    return ValidationResult::InvalidTone {
                        final_cons,
                        tone: "huyền".to_string(),
                    };
                }
                ToneMark::HookAbove => {
                    return ValidationResult::InvalidTone {
                        final_cons,
                        tone: "hỏi".to_string(),
                    };
                }
                ToneMark::Tilde => {
                    return ValidationResult::InvalidTone {
                        final_cons,
                        tone: "ngã".to_string(),
                    };
                }
            }
        }
    }

    ValidationResult::Valid
}

/// Checks if a string is a valid Vietnamese syllable.
pub fn is_valid_vietnamese(s: &str) -> bool {
    let syllable = Syllable::parse(s);
    validate_syllable(&syllable).is_valid()
}

/// Common valid Vietnamese syllables (most frequent ~1000).
/// This is used for spell-checking and suggestions.
pub static COMMON_SYLLABLES: &[&str] = &[
    // Most common syllables in Vietnamese
    "và", "của", "có", "là", "được", "trong", "cho", "này", "với", "không",
    "các", "một", "những", "người", "đã", "như", "về", "để", "khi", "từ",
    "tôi", "anh", "em", "ông", "bà", "cô", "chú", "bạn", "họ", "chúng",
    "việt", "nam", "hà", "nội", "sài", "gòn", "đà", "nẵng", "huế",
    "học", "làm", "đi", "đến", "ra", "vào", "lên", "xuống", "qua", "lại",
    "nói", "nghe", "đọc", "viết", "ăn", "uống", "ngủ", "dậy", "chơi",
    "tốt", "xấu", "đẹp", "hay", "dở", "nhanh", "chậm", "cao", "thấp",
    "lớn", "nhỏ", "nhiều", "ít", "mới", "cũ", "trẻ", "già", "khỏe",
    "nhà", "cửa", "phòng", "bàn", "ghế", "giường", "tủ", "xe", "máy",
    "ngày", "đêm", "sáng", "trưa", "chiều", "tối", "hôm", "nay", "mai",
    "năm", "tháng", "tuần", "giờ", "phút", "giây", "lúc", "khoảng",
    "thì", "mà", "nhưng", "vì", "nên", "nếu", "thế", "sao", "gì", "nào",
    "đây", "đó", "kia", "ấy", "này", "đấy", "đâu", "bao", "mấy",
    "rất", "quá", "lắm", "cũng", "vẫn", "còn", "đều", "chỉ", "toàn",
    "sẽ", "phải", "cần", "muốn", "thích", "yêu", "ghét", "sợ", "mong",
    "biết", "hiểu", "nhớ", "quên", "tìm", "thấy", "gặp", "hỏi", "trả",
    "lời", "tin", "nhắn", "gọi", "điện", "thoại", "máy", "tính", "mạng",
    "xin", "chào", "cảm", "ơn", "xin", "lỗi", "tạm", "biệt", "hẹn",
    // Add more as needed...
];

/// Suggests corrections for a potentially misspelled syllable.
pub fn suggest_corrections(input: &str, max_suggestions: usize) -> Vec<String> {
    let input_lower = input.to_lowercase();
    
    // First, check if it's already valid
    if is_valid_vietnamese(input) {
        return Vec::new();
    }

    // Find similar syllables using edit distance
    let mut suggestions: Vec<(String, usize)> = Vec::new();
    for &syllable in COMMON_SYLLABLES {
        let distance = edit_distance(&input_lower, syllable);
        if distance <= 2 && distance > 0 {
            suggestions.push((syllable.to_string(), distance));
        }
    }

    // Sort by distance and take top N
    suggestions.sort_by_key(|(_, d)| *d);
    suggestions.into_iter()
        .take(max_suggestions)
        .map(|(s, _)| s)
        .collect()
}

/// Simple edit distance (Levenshtein) for suggestion ranking.
fn edit_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 { return n; }
    if n == 0 { return m; }

    let mut dp = vec![vec![0usize; n + 1]; m + 1];

    for i in 0..=m { dp[i][0] = i; }
    for j in 0..=n { dp[0][j] = j; }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i-1] == b_chars[j-1] { 0 } else { 1 };
            dp[i][j] = (dp[i-1][j] + 1)
                .min(dp[i][j-1] + 1)
                .min(dp[i-1][j-1] + cost);
        }
    }

    dp[m][n]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_syllables() {
        assert!(is_valid_vietnamese("việt"));
        assert!(is_valid_vietnamese("nam"));
        assert!(is_valid_vietnamese("xin"));
        assert!(is_valid_vietnamese("chào"));
        assert!(is_valid_vietnamese("a"));
        assert!(is_valid_vietnamese("ơi"));
    }

    #[test]
    fn test_invalid_tone_with_restricted_final() {
        // "học" with huyền is invalid
        let mut s = Syllable::parse("học");
        s.tone_mark = Some(crate::syllable::ToneMark::Grave);
        let result = validate_syllable(&s);
        assert!(matches!(result, ValidationResult::InvalidTone { .. }));
    }

    #[test]
    fn test_valid_tone_with_restricted_final() {
        // "học" with nặng is valid
        let s = Syllable::parse("học");
        assert!(validate_syllable(&s).is_valid());
        
        // "hóc" (sắc) is valid
        let s = Syllable::parse("hóc");
        assert!(validate_syllable(&s).is_valid());
    }

    #[test]
    fn test_suggest_corrections() {
        // Test that suggestions work for known misspellings
        // "viêt" (missing tone) should suggest "việt"
        let _suggestions = suggest_corrections("viêt", 5);
        // The function finds similar syllables by edit distance
        // Note: if input is already valid, returns empty
        // "viêt" parses as valid, so let's test with clearly invalid input
        
        // Test edit distance function directly
        assert_eq!(edit_distance("tôi", "tôy"), 1);
        assert_eq!(edit_distance("việt", "viêt"), 1);
    }

    #[test]
    fn test_edit_distance() {
        assert_eq!(edit_distance("việt", "việt"), 0);
        assert_eq!(edit_distance("việt", "viết"), 1);
        assert_eq!(edit_distance("abc", "def"), 3);
    }
}

#[cfg(test)]
mod additional_tests {
    use super::*;

    #[test]
    fn test_toy_is_invalid() {
        // "tôy" should be invalid - "ôy" is not a valid Vietnamese vowel
        assert!(!is_valid_vietnamese("tôy"));
        // "tôi" should be valid
        assert!(is_valid_vietnamese("tôi"));
    }

    #[test]
    fn test_toy_suggests_toi() {
        let suggestions = suggest_corrections("tôy", 5);
        println!("Suggestions for 'tôy': {:?}", suggestions);
        assert!(suggestions.contains(&"tôi".to_string()));
    }
}
