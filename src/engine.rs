//! The main Engine API for Vietnamese input processing.
//!
//! This module provides a stateful engine that processes keystrokes one at a time,
//! transforming them into Vietnamese text in real-time.

use crate::buffer::{new_input_buffer, new_output_buffer, InputBuffer, OutputBuffer};

pub use crate::transform::InputMethod;

/// A stateful Vietnamese input method engine.
///
/// The engine maintains an internal buffer of raw keystrokes and provides
/// real-time transformation into Vietnamese text.
///
/// # Example
/// ```
/// use vigo::{Engine, InputMethod};
///
/// let mut engine = Engine::new(InputMethod::Telex);
///
/// // Feed characters one by one
/// for ch in "chaof".chars() {
///     engine.feed(ch);
/// }
///
/// // Get the transformed text
/// assert_eq!(engine.output(), "chào");
///
/// // Commit and clear
/// assert_eq!(engine.commit(), "chào");
/// assert!(engine.is_empty());
/// ```
pub struct Engine {
    /// Raw input buffer
    raw: InputBuffer,
    /// Transformed output buffer
    out: OutputBuffer,
    /// Current input method
    method: InputMethod,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(InputMethod::Telex)
    }
}

impl Engine {
    /// Creates a new engine with the specified input method.
    pub fn new(method: InputMethod) -> Self {
        Self {
            raw: new_input_buffer(),
            out: new_output_buffer(),
            method,
        }
    }

    /// Creates a new engine with Telex input method.
    pub fn telex() -> Self {
        Self::new(InputMethod::Telex)
    }

    /// Creates a new engine with VNI input method.
    pub fn vni() -> Self {
        Self::new(InputMethod::Vni)
    }

    /// Returns the current input method.
    pub fn input_method(&self) -> InputMethod {
        self.method
    }

    /// Sets the input method.
    pub fn set_input_method(&mut self, method: InputMethod) {
        self.method = method;
        self.refresh();
    }

    /// Feeds a character into the engine.
    ///
    /// If the character is whitespace, the current word is finalized and
    /// the whitespace is appended to the output.
    ///
    /// # Returns
    /// A reference to the current transformed output.
    pub fn feed(&mut self, ch: char) -> &str {
        if ch.is_whitespace() {
            // Finalize current word and append whitespace
            self.refresh();
            let _ = self.out.push(ch);
            self.raw.clear();
            return &self.out;
        }

        let _ = self.raw.push(ch.to_ascii_lowercase());
        self.refresh();
        &self.out
    }

    /// Feeds a string of characters into the engine.
    ///
    /// This is equivalent to calling `feed` for each character.
    pub fn feed_str(&mut self, s: &str) -> &str {
        for ch in s.chars() {
            self.feed(ch);
        }
        &self.out
    }

    /// Returns the current transformed output without consuming it.
    pub fn output(&self) -> &str {
        &self.out
    }

    /// Returns the raw input buffer.
    pub fn raw_input(&self) -> &str {
        &self.raw
    }

    /// Returns true if the input buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// Returns the length of the raw input buffer.
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    /// Commits the current output and clears the buffers.
    ///
    /// # Returns
    /// The final transformed text.
    #[cfg(feature = "std")]
    pub fn commit(&mut self) -> String {
        let result = self.out.clone();
        self.clear();
        result
    }

    /// Clears both input and output buffers.
    pub fn clear(&mut self) {
        self.raw.clear();
        self.out.clear();
    }

    /// Removes the last character from the input buffer.
    ///
    /// # Returns
    /// The removed character, or None if the buffer was empty.
    pub fn backspace(&mut self) -> Option<char> {
        let ch = self.raw.pop();
        if ch.is_some() {
            self.refresh();
        }
        ch
    }

    /// Refreshes the output buffer based on current input.
    fn refresh(&mut self) {
        self.out.clear();
        if self.raw.is_empty() {
            return;
        }

        #[cfg(feature = "std")]
        {
            let transformed = crate::transform::transform_buffer_with_method(&self.raw, self.method);
            let _ = self.out.push_str(&transformed);
        }

        #[cfg(all(feature = "heapless", not(feature = "std")))]
        {
            // For heapless mode, use a simplified transformation
            // This could be optimized further for embedded use cases
            for ch in self.raw.chars() {
                let _ = self.out.push(ch);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn type_seq(engine: &mut Engine, s: &str) -> String {
        let mut out = String::new();
        for ch in s.chars() {
            out = engine.feed(ch).to_string();
        }
        out
    }

    #[test]
    fn test_basic_telex() {
        let mut e = Engine::telex();
        assert_eq!(type_seq(&mut e, "aa"), "â");

        let mut e = Engine::telex();
        assert_eq!(type_seq(&mut e, "aw"), "ă");

        let mut e = Engine::telex();
        assert_eq!(type_seq(&mut e, "dd"), "đ");
    }

    #[test]
    fn test_tones() {
        let mut e = Engine::telex();
        assert_eq!(type_seq(&mut e, "as"), "á");

        let mut e = Engine::telex();
        assert_eq!(type_seq(&mut e, "vieetj"), "việt");
    }

    #[test]
    fn test_whitespace_commits() {
        let mut e = Engine::telex();
        assert_eq!(type_seq(&mut e, "vieetj"), "việt");
        assert_eq!(e.feed(' '), "việt ");
        assert_eq!(type_seq(&mut e, "nam"), "nam");
    }

    #[test]
    fn test_backspace() {
        let mut e = Engine::telex();
        type_seq(&mut e, "vieet");
        e.backspace();
        assert_eq!(e.output(), "viê");
    }

    #[test]
    fn test_clear() {
        let mut e = Engine::telex();
        type_seq(&mut e, "vieetj");
        e.clear();
        assert!(e.is_empty());
        assert_eq!(e.output(), "");
    }

    #[test]
    fn test_vni() {
        let mut e = Engine::vni();
        assert_eq!(type_seq(&mut e, "a6"), "â");

        let mut e = Engine::vni();
        assert_eq!(type_seq(&mut e, "a1"), "á");

        let mut e = Engine::vni();
        assert_eq!(type_seq(&mut e, "d9"), "đ");
    }

    #[test]
    fn test_commit() {
        let mut e = Engine::telex();
        type_seq(&mut e, "chaof");
        let result = e.commit();
        assert_eq!(result, "chào");
        assert!(e.is_empty());
    }
}
