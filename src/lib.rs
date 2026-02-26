//! # Vigo - Vietnamese Input Method Engine
//!
//! A fast, ergonomic Vietnamese input method engine supporting Telex and VNI input methods.
//!
//! ## Features
//!
//! - **High Performance**: Optimized with lookup tables and minimal allocations
//! - **Multiple Input Methods**: Supports both Telex and VNI
//! - **No-std Compatible**: Can be built without std for embedded systems
//! - **Ergonomic API**: Simple, intuitive interface
//!
//! ## Quick Start
//!
//! ```rust
//! use vigo::{Engine, InputMethod};
//!
//! let mut engine = Engine::new(InputMethod::Telex);
//!
//! // Type "vieetj" to get "việt"
//! for ch in "vieetj".chars() {
//!     engine.feed(ch);
//! }
//! assert_eq!(engine.commit(), "việt");
//! ```
//!
//! ## Telex Rules
//!
//! ### Vowel Diacritics
//! - `aa` → `â`, `aw` → `ă`, `ee` → `ê`, `oo` → `ô`
//! - `ow` → `ơ`, `uw` → `ư`, `dd` → `đ`
//!
//! ### Tone Marks
//! - `s` → sắc (acute): á, é, í, ó, ú, ý
//! - `f` → huyền (grave): à, è, ì, ò, ù, ỳ
//! - `r` → hỏi (hook): ả, ẻ, ỉ, ỏ, ủ, ỷ
//! - `x` → ngã (tilde): ã, ẽ, ĩ, õ, ũ, ỹ
//! - `j` → nặng (dot): ạ, ẹ, ị, ọ, ụ, ỵ
//! - `z` → remove tone
//!
//! ## VNI Rules
//!
//! ### Vowel Diacritics
//! - `a6` → `â`, `a8` → `ă`, `e6` → `ê`, `o6` → `ô`
//! - `o7` → `ơ`, `u7` → `ư`, `d9` → `đ`
//!
//! ### Tone Marks
//! - `1` → sắc, `2` → huyền, `3` → hỏi, `4` → ngã, `5` → nặng, `0` → remove

#![cfg_attr(not(feature = "std"), no_std)]

mod buffer;
mod engine;
mod tables;
pub mod transform;

// New architecture modules (Phases 1-3)
pub mod syllable;
pub mod action;
pub mod definitions;
pub mod tone;
pub mod syllable_engine;

pub use engine::{Engine, InputMethod};
pub use transform::{transform_buffer, transform_buffer_with_method};

// Re-export new types
pub use syllable::{Syllable, ToneMark, LetterModification, AccentStyle};
pub use action::{Action, Transformation};
pub use definitions::{TELEX, VNI, lookup_actions};
pub use tone::{find_tone_position, apply_tone as apply_syllable_tone, remove_tone as remove_syllable_tone, apply_modification, remove_modification};
pub use syllable_engine::SyllableEngine;

#[cfg(feature = "std")]
pub mod repl;

#[cfg(feature = "tui")]
pub mod tui;

#[cfg(feature = "ffi")]
pub mod ffi;
