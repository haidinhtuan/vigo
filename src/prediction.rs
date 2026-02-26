//! Word prediction for Vietnamese input.
//!
//! Uses n-gram frequency data to predict the next word based on context.

use std::collections::HashMap;

/// Word predictor based on n-gram frequencies.
#[derive(Debug, Clone)]
pub struct Predictor {
    /// Unigram frequencies (word -> count).
    unigrams: HashMap<String, u32>,
    /// Bigram frequencies (prev_word -> (next_word -> count)).
    bigrams: HashMap<String, HashMap<String, u32>>,
}

impl Default for Predictor {
    fn default() -> Self {
        Self::new_with_defaults()
    }
}

impl Predictor {
    /// Creates an empty predictor.
    pub fn new() -> Self {
        Self {
            unigrams: HashMap::new(),
            bigrams: HashMap::new(),
        }
    }

    /// Creates a predictor with built-in Vietnamese data.
    pub fn new_with_defaults() -> Self {
        let mut pred = Self::new();
        pred.load_defaults();
        pred
    }

    /// Loads default Vietnamese word frequencies.
    fn load_defaults(&mut self) {
        // Most common Vietnamese words (unigrams)
        let common_words = [
            ("và", 1000), ("của", 950), ("có", 900), ("là", 880), ("được", 850),
            ("trong", 800), ("cho", 780), ("này", 750), ("với", 720), ("không", 700),
            ("các", 680), ("một", 660), ("những", 640), ("người", 620), ("đã", 600),
            ("như", 580), ("về", 560), ("để", 540), ("khi", 520), ("từ", 500),
            ("tôi", 480), ("anh", 460), ("em", 450), ("ông", 440), ("bà", 430),
            ("cô", 420), ("chú", 410), ("bạn", 400), ("họ", 390), ("chúng", 380),
            ("việt", 370), ("nam", 360), ("nhà", 350), ("nước", 340), ("năm", 330),
            ("thì", 320), ("mà", 310), ("nhưng", 300), ("vì", 290), ("nên", 280),
            ("nếu", 270), ("sẽ", 260), ("phải", 250), ("cần", 240), ("muốn", 230),
            ("biết", 220), ("làm", 210), ("đi", 200), ("đến", 190), ("ra", 180),
            ("rất", 170), ("quá", 160), ("lắm", 150), ("cũng", 140), ("vẫn", 130),
            ("còn", 120), ("đều", 110), ("chỉ", 100), ("toàn", 90), ("mới", 80),
            ("xin", 75), ("chào", 70), ("cảm", 65), ("ơn", 60), ("lỗi", 55),
        ];

        for (word, freq) in common_words {
            self.unigrams.insert(word.to_string(), freq);
        }

        // Common bigrams (word pairs)
        let common_bigrams = [
            // Greetings
            ("xin", &[("chào", 100), ("lỗi", 80), ("cảm", 60), ("mời", 40)][..]),
            ("cảm", &[("ơn", 100), ("xúc", 30), ("giác", 20)]),
            ("chúc", &[("mừng", 80), ("ngủ", 60), ("may", 40), ("sức", 30)]),
            
            // Common phrases
            ("việt", &[("nam", 100)]),
            ("hồ", &[("chí", 80)]),
            ("chí", &[("minh", 100)]),
            ("hà", &[("nội", 100)]),
            ("đà", &[("nẵng", 100)]),
            ("sài", &[("gòn", 100)]),
            
            // Verbs + objects
            ("đi", &[("làm", 50), ("học", 45), ("chơi", 40), ("ăn", 35), ("ngủ", 30)]),
            ("ăn", &[("cơm", 60), ("sáng", 40), ("trưa", 35), ("tối", 30), ("uống", 25)]),
            ("uống", &[("nước", 50), ("cà", 40), ("trà", 35), ("bia", 30)]),
            ("cà", &[("phê", 100)]),
            ("làm", &[("việc", 60), ("gì", 50), ("ơn", 40), ("sao", 30)]),
            
            // Question patterns
            ("là", &[("gì", 50), ("ai", 40), ("sao", 35), ("đâu", 30)]),
            ("như", &[("thế", 60), ("vậy", 50), ("nào", 40)]),
            ("thế", &[("nào", 70), ("nên", 30)]),
            ("bao", &[("nhiêu", 70), ("giờ", 50), ("lâu", 40)]),
            
            // Pronouns + verbs
            ("tôi", &[("là", 40), ("có", 35), ("muốn", 30), ("cần", 25), ("sẽ", 20)]),
            ("anh", &[("ấy", 40), ("có", 35), ("là", 30), ("đi", 25)]),
            ("em", &[("ấy", 40), ("có", 35), ("là", 30), ("yêu", 25)]),
            ("họ", &[("có", 40), ("là", 35), ("đã", 30), ("sẽ", 25)]),
            
            // Adjectives
            ("rất", &[("đẹp", 40), ("tốt", 35), ("hay", 30), ("vui", 25), ("buồn", 20)]),
            ("quá", &[("đẹp", 35), ("tốt", 30), ("hay", 25), ("trời", 20)]),
            
            // Time expressions
            ("hôm", &[("nay", 70), ("qua", 60)]),
            ("ngày", &[("mai", 50), ("hôm", 40), ("xưa", 30)]),
            ("bây", &[("giờ", 100)]),
            ("lúc", &[("này", 50), ("đó", 40), ("nào", 30)]),
            
            // Connectors
            ("và", &[("các", 30), ("những", 25), ("cũng", 20)]),
            ("nhưng", &[("mà", 40), ("vẫn", 35), ("không", 30)]),
            ("vì", &[("vậy", 50), ("sao", 40), ("thế", 30)]),
            ("nếu", &[("như", 50), ("không", 40), ("có", 30)]),
            
            // Numbers
            ("một", &[("người", 40), ("cái", 35), ("con", 30), ("ngày", 25)]),
            ("hai", &[("người", 35), ("cái", 30), ("con", 25)]),
            ("ba", &[("người", 35), ("cái", 30), ("con", 25)]),
        ];

        for (prev, nexts) in common_bigrams {
            let entry = self.bigrams.entry(prev.to_string()).or_insert_with(HashMap::new);
            for (next, freq) in nexts {
                entry.insert(next.to_string(), *freq);
            }
        }
    }

    /// Predicts the next word(s) given the previous word.
    pub fn predict(&self, prev_word: &str, max_results: usize) -> Vec<String> {
        let prev_lower = prev_word.to_lowercase();
        
        // First try bigrams
        if let Some(nexts) = self.bigrams.get(&prev_lower) {
            let mut predictions: Vec<_> = nexts.iter().collect();
            predictions.sort_by(|a, b| b.1.cmp(a.1));
            return predictions.into_iter()
                .take(max_results)
                .map(|(word, _)| word.clone())
                .collect();
        }

        // Fall back to unigrams
        let mut predictions: Vec<_> = self.unigrams.iter().collect();
        predictions.sort_by(|a, b| b.1.cmp(a.1));
        predictions.into_iter()
            .take(max_results)
            .map(|(word, _)| word.clone())
            .collect()
    }

    /// Predicts with prefix filtering (for autocomplete).
    pub fn predict_with_prefix(&self, prev_word: &str, prefix: &str, max_results: usize) -> Vec<String> {
        let prev_lower = prev_word.to_lowercase();
        let prefix_lower = prefix.to_lowercase();
        
        // First try bigrams
        if let Some(nexts) = self.bigrams.get(&prev_lower) {
            let mut predictions: Vec<_> = nexts.iter()
                .filter(|(word, _)| word.starts_with(&prefix_lower))
                .collect();
            predictions.sort_by(|a, b| b.1.cmp(a.1));
            if !predictions.is_empty() {
                return predictions.into_iter()
                    .take(max_results)
                    .map(|(word, _)| word.clone())
                    .collect();
            }
        }

        // Fall back to unigrams with prefix
        let mut predictions: Vec<_> = self.unigrams.iter()
            .filter(|(word, _)| word.starts_with(&prefix_lower))
            .collect();
        predictions.sort_by(|a, b| b.1.cmp(a.1));
        predictions.into_iter()
            .take(max_results)
            .map(|(word, _)| word.clone())
            .collect()
    }

    /// Adds a word to unigram frequencies.
    pub fn add_word(&mut self, word: &str) {
        *self.unigrams.entry(word.to_lowercase()).or_insert(0) += 1;
    }

    /// Adds a word pair to bigram frequencies.
    pub fn add_bigram(&mut self, prev: &str, next: &str) {
        let entry = self.bigrams.entry(prev.to_lowercase()).or_insert_with(HashMap::new);
        *entry.entry(next.to_lowercase()).or_insert(0) += 1;
    }

    /// Learns from a text (updates frequencies).
    pub fn learn(&mut self, text: &str) {
        let words: Vec<&str> = text.split_whitespace().collect();
        
        for word in &words {
            self.add_word(word);
        }

        for window in words.windows(2) {
            self.add_bigram(window[0], window[1]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predict_after_xin() {
        let pred = Predictor::new_with_defaults();
        let suggestions = pred.predict("xin", 5);
        assert!(suggestions.contains(&"chào".to_string()));
    }

    #[test]
    fn test_predict_after_viet() {
        let pred = Predictor::new_with_defaults();
        let suggestions = pred.predict("việt", 3);
        assert_eq!(suggestions.first(), Some(&"nam".to_string()));
    }

    #[test]
    fn test_predict_with_prefix() {
        let pred = Predictor::new_with_defaults();
        let suggestions = pred.predict_with_prefix("xin", "ch", 3);
        assert!(suggestions.contains(&"chào".to_string()));
    }

    #[test]
    fn test_learn() {
        let mut pred = Predictor::new();
        pred.learn("tôi yêu việt nam");
        
        let suggestions = pred.predict("yêu", 3);
        assert!(suggestions.contains(&"việt".to_string()));
    }

    #[test]
    fn test_fallback_to_unigrams() {
        let pred = Predictor::new_with_defaults();
        // "xyz" doesn't exist, should fall back to common unigrams
        let suggestions = pred.predict("xyz", 3);
        assert!(!suggestions.is_empty());
    }
}
