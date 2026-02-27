//! Zero-allocation Vietnamese input engine.
//!
//! `FastEngine` processes keystrokes using stack-only buffers and a fused
//! 2-pass render pipeline. No heap allocations on the hot path.

use crate::action::InputMethod;

/// Maximum raw input bytes (ASCII keystrokes).
const MAX_RAW: usize = 32;
/// Maximum UTF-8 output bytes.
const MAX_OUT: usize = 128;

/// Zero-allocation Vietnamese input engine.
///
/// All buffers are stack-allocated. Returns `&str` from internal buffer.
pub struct FastEngine {
    raw: [u8; MAX_RAW],
    raw_len: u8,
    out_utf8: [u8; MAX_OUT],
    out_utf8_len: u8,
    method: InputMethod,
}

impl FastEngine {
    /// Creates a new engine with the specified input method.
    pub fn new(method: InputMethod) -> Self {
        Self {
            raw: [0; MAX_RAW],
            raw_len: 0,
            out_utf8: [0; MAX_OUT],
            out_utf8_len: 0,
            method,
        }
    }

    /// Creates a new Telex engine.
    pub fn telex() -> Self {
        Self::new(InputMethod::Telex)
    }

    /// Creates a new VNI engine.
    pub fn vni() -> Self {
        Self::new(InputMethod::Vni)
    }

    /// Feeds a character and returns the current output.
    pub fn feed(&mut self, ch: char) -> &str {
        if self.raw_len < MAX_RAW as u8 && ch.is_ascii() {
            self.raw[self.raw_len as usize] = ch as u8;
            self.raw_len += 1;
        }
        self.render()
    }

    /// Removes the last keystroke and returns updated output.
    pub fn backspace(&mut self) -> &str {
        if self.raw_len > 0 {
            self.raw_len -= 1;
        }
        self.render()
    }

    /// Resets the engine for the next syllable.
    pub fn clear(&mut self) {
        self.raw_len = 0;
        self.out_utf8_len = 0;
    }

    /// Returns the current output as a borrowed string.
    pub fn output(&self) -> &str {
        core::str::from_utf8(&self.out_utf8[..self.out_utf8_len as usize])
            .unwrap_or("")
    }

    /// Returns the raw keystrokes as a borrowed string.
    pub fn raw_input(&self) -> &str {
        core::str::from_utf8(&self.raw[..self.raw_len as usize])
            .unwrap_or("")
    }

    /// Sets the input method.
    pub fn set_method(&mut self, method: InputMethod) {
        self.method = method;
    }

    /// Renders raw input to output. Passthrough for now (Task 1).
    fn render(&mut self) -> &str {
        self.out_utf8_len = 0;
        for i in 0..self.raw_len as usize {
            let ch = self.raw[i] as char;
            let len = ch.len_utf8();
            if (self.out_utf8_len as usize) + len <= MAX_OUT {
                ch.encode_utf8(&mut self.out_utf8[self.out_utf8_len as usize..]);
                self.out_utf8_len += len as u8;
            }
        }
        self.output()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::InputMethod;

    fn type_seq(engine: &mut FastEngine, s: &str) -> String {
        for ch in s.chars() {
            engine.feed(ch);
        }
        engine.output().to_string()
    }

    #[test]
    fn test_plain_ascii_passthrough() {
        let mut e = FastEngine::new(InputMethod::Telex);
        assert_eq!(type_seq(&mut e, "hello"), "hello");
    }

    #[test]
    fn test_clear_resets() {
        let mut e = FastEngine::new(InputMethod::Telex);
        type_seq(&mut e, "hello");
        e.clear();
        assert_eq!(e.output(), "");
        assert_eq!(e.raw_input(), "");
    }

    #[test]
    fn test_single_char() {
        let mut e = FastEngine::new(InputMethod::Telex);
        assert_eq!(type_seq(&mut e, "a"), "a");
        e.clear();
        assert_eq!(type_seq(&mut e, "b"), "b");
    }
}
