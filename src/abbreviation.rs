//! Abbreviation expansion for Vietnamese input.
//!
//! Supports built-in common abbreviations and user-defined custom ones.

use std::collections::HashMap;

/// Abbreviation expander.
#[derive(Debug, Clone)]
pub struct Abbreviations {
    /// Mapping from abbreviation to expansion.
    map: HashMap<String, String>,
}

impl Default for Abbreviations {
    fn default() -> Self {
        Self::new_with_defaults()
    }
}

impl Abbreviations {
    /// Creates an empty abbreviation set.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Creates abbreviations with built-in defaults.
    pub fn new_with_defaults() -> Self {
        let mut abbr = Self::new();
        abbr.load_defaults();
        abbr
    }

    /// Loads built-in default abbreviations.
    pub fn load_defaults(&mut self) {
        // Common Vietnamese abbreviations
        let defaults = [
            // Places
            ("vn", "Việt Nam"),
            ("hcm", "Hồ Chí Minh"),
            ("hn", "Hà Nội"),
            ("dn", "Đà Nẵng"),
            ("sg", "Sài Gòn"),
            ("hp", "Hải Phòng"),
            ("ct", "Cần Thơ"),
            
            // Common texting shortcuts
            ("k", "không"),
            ("ko", "không"),
            ("dc", "được"),
            ("dk", "được"),
            ("đc", "được"),
            ("vs", "với"),
            ("v", "với"),
            ("ng", "người"),
            ("ngta", "người ta"),
            ("ns", "nói"),
            ("nt", "nhắn tin"),
            ("nc", "nói chuyện"),
            ("mk", "mình"),
            ("mn", "mọi người"),
            ("bt", "bình thường"),
            ("bth", "bình thường"),
            ("cx", "cũng"),
            ("cg", "cũng"),
            ("r", "rồi"),
            ("ck", "chồng"),
            ("vk", "vợ"),
            ("gf", "bạn gái"),
            ("bf", "bạn trai"),
            ("ak", "à"),
            ("uk", "ừ"),
            ("hk", "ha"),
            ("oke", "được"),
            ("ok", "được"),
            ("tks", "cảm ơn"),
            ("thanks", "cảm ơn"),
            ("ty", "cảm ơn"),
            ("sr", "xin lỗi"),
            ("sorry", "xin lỗi"),
            ("plz", "làm ơn"),
            ("pls", "làm ơn"),
            
            // Time
            ("hqua", "hôm qua"),
            ("hnay", "hôm nay"),
            ("hom nay", "hôm nay"),
            ("hom qua", "hôm qua"),
            ("ngay mai", "ngày mai"),
            
            // Phrases
            ("chuc ngu ngon", "chúc ngủ ngon"),
            ("cnn", "chúc ngủ ngon"),
            ("gm", "good morning"),
            ("gn", "good night"),
            
            // Internet slang
            ("wtf", "what the fuck"),
            ("lol", "cười lớn"),
            ("brb", "quay lại ngay"),
            ("btw", "nhân tiện"),
            ("fyi", "để bạn biết"),
            ("omg", "trời ơi"),
            ("idk", "không biết"),
            ("imo", "theo ý mình"),
            ("tbh", "thật ra thì"),
            ("asap", "càng sớm càng tốt"),
        ];

        for (abbr, expansion) in defaults {
            self.add(abbr, expansion);
        }
    }

    /// Adds an abbreviation.
    pub fn add(&mut self, abbr: &str, expansion: &str) {
        self.map.insert(abbr.to_lowercase(), expansion.to_string());
    }

    /// Removes an abbreviation.
    pub fn remove(&mut self, abbr: &str) -> Option<String> {
        self.map.remove(&abbr.to_lowercase())
    }

    /// Looks up an abbreviation and returns its expansion.
    pub fn expand(&self, abbr: &str) -> Option<&str> {
        self.map.get(&abbr.to_lowercase()).map(|s| s.as_str())
    }

    /// Checks if an abbreviation exists.
    pub fn contains(&self, abbr: &str) -> bool {
        self.map.contains_key(&abbr.to_lowercase())
    }

    /// Returns the number of abbreviations.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Iterates over all abbreviations.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.map.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Clears all abbreviations.
    pub fn clear(&mut self) {
        self.map.clear();
    }

    /// Loads abbreviations from a string (one per line, format: "abbr=expansion").
    pub fn load_from_str(&mut self, content: &str) {
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((abbr, expansion)) = line.split_once('=') {
                self.add(abbr.trim(), expansion.trim());
            }
        }
    }

    /// Exports abbreviations to a string.
    pub fn export_to_str(&self) -> String {
        let mut lines: Vec<String> = self.map
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        lines.sort();
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_abbreviations() {
        let abbr = Abbreviations::new_with_defaults();
        assert_eq!(abbr.expand("vn"), Some("Việt Nam"));
        assert_eq!(abbr.expand("hcm"), Some("Hồ Chí Minh"));
        assert_eq!(abbr.expand("k"), Some("không"));
    }

    #[test]
    fn test_case_insensitive() {
        let abbr = Abbreviations::new_with_defaults();
        assert_eq!(abbr.expand("VN"), Some("Việt Nam"));
        assert_eq!(abbr.expand("Vn"), Some("Việt Nam"));
    }

    #[test]
    fn test_custom_abbreviation() {
        let mut abbr = Abbreviations::new();
        abbr.add("myabbr", "my expansion");
        assert_eq!(abbr.expand("myabbr"), Some("my expansion"));
    }

    #[test]
    fn test_remove_abbreviation() {
        let mut abbr = Abbreviations::new_with_defaults();
        assert!(abbr.contains("vn"));
        abbr.remove("vn");
        assert!(!abbr.contains("vn"));
    }

    #[test]
    fn test_load_from_str() {
        let mut abbr = Abbreviations::new();
        abbr.load_from_str("
            # Comment line
            test=testing
            foo=bar baz
        ");
        assert_eq!(abbr.expand("test"), Some("testing"));
        assert_eq!(abbr.expand("foo"), Some("bar baz"));
    }

    #[test]
    fn test_export_to_str() {
        let mut abbr = Abbreviations::new();
        abbr.add("b", "beta");
        abbr.add("a", "alpha");
        let exported = abbr.export_to_str();
        assert!(exported.contains("a=alpha"));
        assert!(exported.contains("b=beta"));
    }
}
