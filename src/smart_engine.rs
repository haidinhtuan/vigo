//! Smart Vietnamese input engine with all advanced features.
//!
//! Combines:
//! - Core Vietnamese transformation (Telex/VNI)
//! - Spell checking and validation
//! - Word prediction
//! - Abbreviation expansion
//! - Code-switching (English detection)

use crate::abbreviation::Abbreviations;
use crate::action::InputMethod;
use crate::codeswitching::CodeSwitcher;
use crate::prediction::Predictor;
use crate::syllable_engine::SyllableEngine;
use crate::validation::{is_valid_vietnamese, suggest_corrections};

/// Configuration for smart engine features.
#[derive(Debug, Clone)]
pub struct SmartConfig {
    /// Enable spell checking.
    pub spell_check: bool,
    /// Enable word prediction.
    pub prediction: bool,
    /// Enable abbreviation expansion.
    pub abbreviations: bool,
    /// Enable code-switching (English detection).
    pub code_switching: bool,
    /// Maximum number of predictions to return.
    pub max_predictions: usize,
    /// Maximum number of spell suggestions.
    pub max_suggestions: usize,
}

impl Default for SmartConfig {
    fn default() -> Self {
        Self {
            spell_check: true,
            prediction: true,
            abbreviations: true,
            code_switching: true,
            max_predictions: 5,
            max_suggestions: 3,
        }
    }
}

/// Output from the smart engine with metadata.
#[derive(Debug, Clone)]
pub struct SmartOutput {
    /// The transformed/final text.
    pub text: String,
    /// Whether the word was detected as English.
    pub is_english: bool,
    /// Whether the word is a valid Vietnamese syllable.
    pub is_valid: bool,
    /// Whether an abbreviation was expanded.
    pub was_abbreviated: bool,
    /// Spell check suggestions (if invalid).
    pub suggestions: Vec<String>,
    /// Word predictions for next input.
    pub predictions: Vec<String>,
}

/// Smart Vietnamese input engine.
pub struct SmartEngine {
    /// Core syllable engine.
    engine: SyllableEngine,
    /// Abbreviation expander.
    abbreviations: Abbreviations,
    /// Word predictor.
    predictor: Predictor,
    /// Code-switcher for English detection.
    code_switcher: CodeSwitcher,
    /// Configuration.
    config: SmartConfig,
    /// Last committed word (for prediction context).
    last_word: String,
    /// Whether current word is being treated as English.
    current_is_english: bool,
    /// Raw input for English mode.
    english_buffer: String,
}

impl Default for SmartEngine {
    fn default() -> Self {
        Self::new(InputMethod::Telex, SmartConfig::default())
    }
}

impl SmartEngine {
    /// Creates a new smart engine with configuration.
    pub fn new(method: InputMethod, config: SmartConfig) -> Self {
        Self {
            engine: SyllableEngine::new(method),
            abbreviations: Abbreviations::new_with_defaults(),
            predictor: Predictor::new_with_defaults(),
            code_switcher: CodeSwitcher::new_with_defaults(),
            config,
            last_word: String::new(),
            current_is_english: false,
            english_buffer: String::new(),
        }
    }

    /// Creates a smart engine with Telex and default config.
    pub fn telex() -> Self {
        Self::new(InputMethod::Telex, SmartConfig::default())
    }

    /// Creates a smart engine with VNI and default config.
    pub fn vni() -> Self {
        Self::new(InputMethod::Vni, SmartConfig::default())
    }

    /// Returns mutable reference to abbreviations.
    pub fn abbreviations_mut(&mut self) -> &mut Abbreviations {
        &mut self.abbreviations
    }

    /// Returns mutable reference to predictor.
    pub fn predictor_mut(&mut self) -> &mut Predictor {
        &mut self.predictor
    }

    /// Returns mutable reference to code switcher.
    pub fn code_switcher_mut(&mut self) -> &mut CodeSwitcher {
        &mut self.code_switcher
    }

    /// Returns mutable reference to config.
    pub fn config_mut(&mut self) -> &mut SmartConfig {
        &mut self.config
    }

    /// Feeds a character and returns smart output.
    pub fn feed(&mut self, ch: char) -> SmartOutput {
        // Handle space - commit current word
        if ch == ' ' {
            let output = self.commit();
            // Reset for next word
            self.current_is_english = false;
            self.english_buffer.clear();
            return output;
        }

        // If in English mode, just buffer the character
        if self.current_is_english {
            self.english_buffer.push(ch);
            return self.build_output();
        }

        // Check if this could be starting an English word
        if self.config.code_switching && self.engine.is_empty() {
            // We'll re-evaluate after a few characters
        }

        // Feed to Vietnamese engine
        self.engine.feed(ch);

        // Check if current input looks like English
        if self.config.code_switching {
            let raw = self.engine.raw_input();
            if raw.len() >= 3 && self.code_switcher.is_english(raw) {
                // Switch to English mode
                self.current_is_english = true;
                self.english_buffer = raw.to_string();
                self.engine.clear();
            }
        }

        self.build_output()
    }

    /// Feeds a string of characters.
    pub fn feed_str(&mut self, s: &str) -> SmartOutput {
        let mut output = SmartOutput {
            text: String::new(),
            is_english: false,
            is_valid: true,
            was_abbreviated: false,
            suggestions: Vec::new(),
            predictions: Vec::new(),
        };
        
        for ch in s.chars() {
            output = self.feed(ch);
        }
        output
    }

    /// Returns current output without committing.
    pub fn output(&self) -> String {
        if self.current_is_english {
            self.english_buffer.clone()
        } else {
            self.engine.output()
        }
    }

    /// Returns the raw input buffer.
    pub fn raw_input(&self) -> &str {
        if self.current_is_english {
            &self.english_buffer
        } else {
            self.engine.raw_input()
        }
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.engine.is_empty() && self.english_buffer.is_empty()
    }

    /// Commits current word and returns smart output.
    pub fn commit(&mut self) -> SmartOutput {
        let mut output = self.build_output();
        
        // Check for abbreviation expansion
        if self.config.abbreviations {
            let text = output.text.clone();
            if let Some(expansion) = self.abbreviations.expand(&text) {
                output.text = expansion.to_string();
                output.was_abbreviated = true;
            }
        }

        // Update predictor with this word
        if !output.text.is_empty() {
            self.predictor.add_word(&output.text);
            if !self.last_word.is_empty() {
                self.predictor.add_bigram(&self.last_word, &output.text);
            }
            self.last_word = output.text.clone();
        }

        // Clear state
        self.engine.clear();
        self.english_buffer.clear();
        self.current_is_english = false;

        output
    }

    /// Clears all state.
    pub fn clear(&mut self) {
        self.engine.clear();
        self.english_buffer.clear();
        self.current_is_english = false;
    }

    /// Processes backspace.
    pub fn backspace(&mut self) -> SmartOutput {
        if self.current_is_english {
            self.english_buffer.pop();
            if self.english_buffer.is_empty() {
                self.current_is_english = false;
            }
        } else {
            self.engine.backspace();
        }
        self.build_output()
    }

    /// Builds the smart output with all metadata.
    fn build_output(&self) -> SmartOutput {
        let text = self.output();
        let is_english = self.current_is_english;
        
        // Spell check
        let (is_valid, suggestions) = if self.config.spell_check && !is_english && !text.is_empty() {
            let valid = is_valid_vietnamese(&text);
            let sugg = if valid {
                Vec::new()
            } else {
                suggest_corrections(&text, self.config.max_suggestions)
            };
            (valid, sugg)
        } else {
            (true, Vec::new())
        };

        // Word predictions
        let predictions = if self.config.prediction {
            self.predictor.predict(&self.last_word, self.config.max_predictions)
        } else {
            Vec::new()
        };

        SmartOutput {
            text,
            is_english,
            is_valid,
            was_abbreviated: false,
            suggestions,
            predictions,
        }
    }

    /// Gets predictions for next word.
    pub fn get_predictions(&self) -> Vec<String> {
        self.predictor.predict(&self.last_word, self.config.max_predictions)
    }

    /// Gets predictions with a prefix filter.
    pub fn get_predictions_with_prefix(&self, prefix: &str) -> Vec<String> {
        self.predictor.predict_with_prefix(&self.last_word, prefix, self.config.max_predictions)
    }

    /// Manually sets the last word (for context).
    pub fn set_context(&mut self, last_word: &str) {
        self.last_word = last_word.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_vietnamese() {
        let mut engine = SmartEngine::telex();
        engine.feed_str("vieetj");
        assert_eq!(engine.output(), "việt");
    }

    #[test]
    fn test_abbreviation_expansion() {
        let mut engine = SmartEngine::telex();
        engine.feed_str("vn");
        let output = engine.commit();
        assert_eq!(output.text, "Việt Nam");
        assert!(output.was_abbreviated);
    }

    #[test]
    fn test_english_detection() {
        let mut engine = SmartEngine::telex();
        engine.feed_str("email");
        let output = engine.build_output();
        assert!(output.is_english);
        assert_eq!(output.text, "email");
    }

    #[test]
    fn test_spell_check_valid() {
        let mut engine = SmartEngine::telex();
        engine.feed_str("xin");
        let output = engine.build_output();
        assert!(output.is_valid);
        assert!(output.suggestions.is_empty());
    }

    #[test]
    fn test_predictions() {
        let mut engine = SmartEngine::telex();
        engine.set_context("xin");
        let predictions = engine.get_predictions();
        assert!(predictions.contains(&"chào".to_string()));
    }

    #[test]
    fn test_commit_updates_predictor() {
        let mut engine = SmartEngine::telex();
        engine.feed_str("hello");
        engine.commit();
        engine.feed_str("world");
        engine.commit();
        
        // Now "world" should be predicted after "hello"
        engine.set_context("hello");
        let predictions = engine.get_predictions();
        assert!(predictions.contains(&"world".to_string()));
    }

    #[test]
    fn test_disable_features() {
        let config = SmartConfig {
            spell_check: false,
            prediction: false,
            abbreviations: false,
            code_switching: false,
            ..Default::default()
        };
        let mut engine = SmartEngine::new(InputMethod::Telex, config);
        
        // Abbreviation should not expand
        engine.feed_str("vn");
        let output = engine.commit();
        assert_eq!(output.text, "vn");
        assert!(!output.was_abbreviated);
    }
}
