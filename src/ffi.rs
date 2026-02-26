//! C FFI bindings for vigo engine.
//!
//! This module exposes a C-compatible API for use in fcitx5 and other
//! input method frameworks.

use std::ffi::CString;
use std::os::raw::c_char;

use crate::syllable_engine::SyllableEngine;
use crate::action::InputMethod;

/// Opaque handle to a vigo engine instance.
pub struct VigoEngine {
    inner: SyllableEngine,
}

/// Creates a new vigo engine with Telex input method.
/// Returns a pointer that must be freed with `vigo_free`.
#[no_mangle]
pub extern "C" fn vigo_new_telex() -> *mut VigoEngine {
    Box::into_raw(Box::new(VigoEngine {
        inner: SyllableEngine::new(InputMethod::Telex),
    }))
}

/// Creates a new vigo engine with VNI input method.
/// Returns a pointer that must be freed with `vigo_free`.
#[no_mangle]
pub extern "C" fn vigo_new_vni() -> *mut VigoEngine {
    Box::into_raw(Box::new(VigoEngine {
        inner: SyllableEngine::new(InputMethod::Vni),
    }))
}

/// Frees a vigo engine instance.
#[no_mangle]
pub extern "C" fn vigo_free(engine: *mut VigoEngine) {
    if !engine.is_null() {
        unsafe {
            drop(Box::from_raw(engine));
        }
    }
}

/// Feeds a character into the engine.
/// Returns true if the character was processed.
#[no_mangle]
pub extern "C" fn vigo_feed(engine: *mut VigoEngine, ch: u32) -> bool {
    if engine.is_null() {
        return false;
    }
    let engine = unsafe { &mut *engine };
    if let Some(c) = char::from_u32(ch) {
        engine.inner.feed(c);
        true
    } else {
        false
    }
}

/// Gets the current output as a C string.
/// The returned string must be freed with `vigo_free_string`.
#[no_mangle]
pub extern "C" fn vigo_get_output(engine: *const VigoEngine) -> *mut c_char {
    if engine.is_null() {
        return std::ptr::null_mut();
    }
    let engine = unsafe { &*engine };
    let output = engine.inner.output();
    match CString::new(output) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Gets the raw input as a C string.
/// The returned string must be freed with `vigo_free_string`.
#[no_mangle]
pub extern "C" fn vigo_get_raw_input(engine: *const VigoEngine) -> *mut c_char {
    if engine.is_null() {
        return std::ptr::null_mut();
    }
    let engine = unsafe { &*engine };
    let raw = engine.inner.raw_input();
    match CString::new(raw) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Frees a string returned by vigo functions.
#[no_mangle]
pub extern "C" fn vigo_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            drop(CString::from_raw(s));
        }
    }
}

/// Processes a backspace.
/// Returns true if a character was removed.
#[no_mangle]
pub extern "C" fn vigo_backspace(engine: *mut VigoEngine) -> bool {
    if engine.is_null() {
        return false;
    }
    let engine = unsafe { &mut *engine };
    engine.inner.backspace().is_some()
}

/// Clears all input.
#[no_mangle]
pub extern "C" fn vigo_clear(engine: *mut VigoEngine) {
    if engine.is_null() {
        return;
    }
    let engine = unsafe { &mut *engine };
    engine.inner.clear();
}

/// Commits and returns the output, clearing the engine.
/// The returned string must be freed with `vigo_free_string`.
#[no_mangle]
pub extern "C" fn vigo_commit(engine: *mut VigoEngine) -> *mut c_char {
    if engine.is_null() {
        return std::ptr::null_mut();
    }
    let engine = unsafe { &mut *engine };
    let output = engine.inner.commit();
    match CString::new(output) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Returns true if the engine buffer is empty.
#[no_mangle]
pub extern "C" fn vigo_is_empty(engine: *const VigoEngine) -> bool {
    if engine.is_null() {
        return true;
    }
    let engine = unsafe { &*engine };
    engine.inner.is_empty()
}
