//! Code-switching support for mixed Vietnamese/English input.
//!
//! Detects English words and skips Vietnamese transformation for them.

use std::collections::HashSet;

/// Code-switch detector for Vietnamese/English mixed input.
#[derive(Debug, Clone)]
pub struct CodeSwitcher {
    /// Set of known English words (common ones).
    english_words: HashSet<String>,
    /// Minimum word length to consider for detection.
    min_length: usize,
}

impl Default for CodeSwitcher {
    fn default() -> Self {
        Self::new_with_defaults()
    }
}

impl CodeSwitcher {
    /// Creates an empty code switcher.
    pub fn new() -> Self {
        Self {
            english_words: HashSet::new(),
            min_length: 2,
        }
    }

    /// Creates a code switcher with common English words.
    pub fn new_with_defaults() -> Self {
        let mut cs = Self::new();
        cs.load_defaults();
        cs
    }

    /// Loads common English words used in Vietnamese contexts.
    fn load_defaults(&mut self) {
        // Common English words used in Vietnamese tech/business contexts
        let words = [
            // Tech
            "email", "mail", "inbox", "spam", "send", "file", "folder", "link",
            "click", "check", "download", "upload", "install", "update", "login", "logout",
            "password", "username", "account", "online", "offline", "wifi", "bluetooth",
            "laptop", "desktop", "server", "cloud", "database", "software", "hardware",
            "app", "application", "website", "web", "browser", "chrome", "firefox",
            "google", "facebook", "youtube", "instagram", "twitter", "tiktok", "zalo",
            "code", "debug", "bug", "fix", "error", "warning", "test", "deploy",
            "frontend", "backend", "fullstack", "api", "json", "html", "css", "javascript",
            "python", "java", "rust", "react", "vue", "angular", "node", "npm",
            "git", "github", "gitlab", "commit", "push", "pull", "merge", "branch",
            
            // Business
            "meeting", "deadline", "project", "team", "manager", "leader", "ceo", "cto",
            "report", "review", "feedback", "budget", "target", "goal", "plan",
            "marketing", "sales", "customer", "client", "partner", "investor",
            "startup", "company", "office", "remote", "hybrid", "freelance",
            "interview", "job", "career", "salary", "bonus", "promotion",
            
            // Common
            "ok", "okay", "yes", "no", "hi", "hello", "bye", "goodbye", "sorry",
            "thanks", "thank", "please", "welcome", "excuse", "pardon",
            "hot", "cool", "nice", "good", "bad", "great", "awesome", "amazing",
            "love", "like", "hate", "want", "need", "know", "think", "feel",
            "time", "day", "week", "month", "year", "today", "tomorrow", "yesterday",
            "morning", "afternoon", "evening", "night", "lunch", "dinner", "breakfast",
            "coffee", "tea", "beer", "wine", "food", "drink", "eat", "sleep",
            "work", "home", "school", "university", "hospital", "bank", "shop",
            "car", "bus", "taxi", "train", "plane", "bike", "walk", "run",
            "big", "small", "fast", "slow", "new", "old", "young", "happy", "sad",
            "easy", "hard", "simple", "complex", "free", "busy", "ready", "done",
            "start", "stop", "open", "close", "on", "off", "in", "out", "up", "down",
            "show", "hide", "add", "remove", "edit", "delete", "save", "cancel",
            "next", "back", "first", "last", "top", "bottom", "left", "right",
            "all", "some", "none", "any", "many", "few", "more", "less", "most",
            "and", "or", "but", "not", "with", "for", "from", "to", "at", "by",
            "the", "a", "an", "this", "that", "it", "its", "is", "are", "was", "were",
            "be", "been", "being", "have", "has", "had", "do", "does", "did",
            "will", "would", "could", "should", "may", "might", "can", "must",
            "if", "then", "else", "when", "where", "what", "which", "who", "why", "how",
            
            // Brands/Names often kept in English
            "apple", "samsung", "microsoft", "amazon", "netflix", "spotify",
            "uber", "grab", "shopee", "lazada", "tiki",
            
            // Acronyms
            "ai", "ml", "ux", "ui", "pm", "qa", "hr", "pr", "kpi", "roi",
            "asap", "fyi", "btw", "omg", "lol", "brb", "afk",
        ];

        for word in words {
            self.english_words.insert(word.to_string());
        }
    }

    /// Adds an English word to the dictionary.
    pub fn add_word(&mut self, word: &str) {
        self.english_words.insert(word.to_lowercase());
    }

    /// Removes a word from the dictionary.
    pub fn remove_word(&mut self, word: &str) {
        self.english_words.remove(&word.to_lowercase());
    }

    /// Checks if a word should be treated as English (skip Vietnamese transformation).
    pub fn is_english(&self, word: &str) -> bool {
        if word.len() < self.min_length {
            return false;
        }

        let lower = word.to_lowercase();

        // Direct dictionary lookup
        if self.english_words.contains(&lower) {
            return true;
        }

        // Heuristics for detecting English
        self.looks_like_english(word)
    }

    /// Heuristic check for English-looking words.
    fn looks_like_english(&self, word: &str) -> bool {
        let lower = word.to_lowercase();
        let chars: Vec<char> = lower.chars().collect();

        // Must be ASCII letters only (no Vietnamese diacritics)
        if !chars.iter().all(|c| c.is_ascii_alphabetic()) {
            return false;
        }

        // Common English endings
        let english_endings = [
            "ing", "tion", "ness", "ment", "able", "ible", "ful", "less",
            "ous", "ive", "ly", "er", "est", "ed", "es", "ity",
        ];
        for ending in english_endings {
            if lower.ends_with(ending) && lower.len() > ending.len() + 2 {
                return true;
            }
        }

        // Common English letter combinations rare in Vietnamese
        let _english_patterns = [
            "th", "wh", "sh", "ch", "ck", "ght", "tion", "qu",
            "wr", "kn", "gn", "ph", "gh",
        ];
        
        // Vietnamese doesn't use these letters
        if chars.iter().any(|c| matches!(c, 'w' | 'z' | 'j' | 'f')) {
            // But check it's not a Vietnamese word typed in Telex
            // (f is used for huyền, j for nặng, w for ư/ơ)
            // Only flag as English if the word looks complete
            if lower.len() >= 3 && !self.could_be_telex(&lower) {
                return true;
            }
        }

        false
    }

    /// Checks if a word could be Vietnamese typed in Telex.
    fn could_be_telex(&self, word: &str) -> bool {
        // Common Telex patterns
        let telex_indicators = ["aw", "ow", "uw", "aa", "ee", "oo", "dd"];
        for pattern in telex_indicators {
            if word.contains(pattern) {
                return true;
            }
        }

        // Check for tone markers at the end
        let last_char = word.chars().last().unwrap_or(' ');
        if matches!(last_char, 's' | 'f' | 'r' | 'x' | 'j' | 'z') {
            // Could be a tone marker
            let without_tone = &word[..word.len()-1];
            if !without_tone.is_empty() {
                return true;
            }
        }

        false
    }

    /// Processes text and returns segments marked as Vietnamese or English.
    pub fn segment(&self, text: &str) -> Vec<Segment> {
        let mut segments = Vec::new();
        
        for word in text.split_whitespace() {
            let is_eng = self.is_english(word);
            segments.push(Segment {
                text: word.to_string(),
                is_english: is_eng,
            });
        }

        segments
    }
}

/// A text segment with language annotation.
#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub text: String,
    pub is_english: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_english_words() {
        let cs = CodeSwitcher::new_with_defaults();
        assert!(cs.is_english("email"));
        assert!(cs.is_english("meeting"));
        assert!(cs.is_english("download"));
    }

    #[test]
    fn test_not_english() {
        let cs = CodeSwitcher::new_with_defaults();
        // These should not be detected as English
        assert!(!cs.is_english("xin"));
        assert!(!cs.is_english("chào"));
        assert!(!cs.is_english("việt"));
    }

    #[test]
    fn test_english_by_heuristics() {
        let cs = CodeSwitcher::new_with_defaults();
        // Words with English patterns
        assert!(cs.is_english("something"));
        assert!(cs.is_english("working"));
        assert!(cs.is_english("beautiful"));
    }

    #[test]
    fn test_telex_not_english() {
        let cs = CodeSwitcher::new_with_defaults();
        // These look like Telex input, not English
        assert!(!cs.is_english("vieetj")); // việt in Telex
        assert!(!cs.is_english("chaof")); // chào in Telex
    }

    #[test]
    fn test_segment() {
        let cs = CodeSwitcher::new_with_defaults();
        let segments = cs.segment("tôi check email nhé");
        
        assert!(!segments[0].is_english); // tôi
        assert!(segments[1].is_english);  // check
        assert!(segments[2].is_english);  // email
        assert!(!segments[3].is_english); // nhé
    }

    #[test]
    fn test_case_insensitive() {
        let cs = CodeSwitcher::new_with_defaults();
        assert!(cs.is_english("Email"));
        assert!(cs.is_english("EMAIL"));
        assert!(cs.is_english("eMaIl"));
    }
}
