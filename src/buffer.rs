//! Buffer types for input storage.
//!
//! Provides both heap-allocated (std) and fixed-capacity (heapless) buffer implementations.

/// Maximum buffer size for heapless mode.
pub const MAX_BUFFER_SIZE: usize = 64;

/// Maximum output size for heapless mode.
pub const MAX_OUTPUT_SIZE: usize = 128;

#[cfg(feature = "std")]
mod std_impl {
    use super::*;
    
    /// Input buffer type (heap-allocated String).
    pub type InputBuffer = String;
    
    /// Output buffer type (heap-allocated String).
    pub type OutputBuffer = String;
    
    /// Creates a new empty input buffer.
    #[inline]
    pub fn new_input_buffer() -> InputBuffer {
        String::with_capacity(MAX_BUFFER_SIZE)
    }
    
    /// Creates a new empty output buffer.
    #[inline]
    pub fn new_output_buffer() -> OutputBuffer {
        String::with_capacity(MAX_OUTPUT_SIZE)
    }
}

#[cfg(feature = "heapless")]
mod heapless_impl {
    use super::*;
    use heapless::String as HeaplessString;
    
    /// Input buffer type (fixed-capacity heapless String).
    pub type InputBuffer = HeaplessString<MAX_BUFFER_SIZE>;
    
    /// Output buffer type (fixed-capacity heapless String).
    pub type OutputBuffer = HeaplessString<MAX_OUTPUT_SIZE>;
    
    /// Creates a new empty input buffer.
    #[inline]
    pub fn new_input_buffer() -> InputBuffer {
        HeaplessString::new()
    }
    
    /// Creates a new empty output buffer.
    #[inline]
    pub fn new_output_buffer() -> OutputBuffer {
        HeaplessString::new()
    }
}

#[cfg(all(feature = "std", not(feature = "heapless")))]
pub use std_impl::*;

#[cfg(feature = "heapless")]
pub use heapless_impl::*;

#[cfg(all(not(feature = "std"), not(feature = "heapless")))]
compile_error!("Either 'std' or 'heapless' feature must be enabled");
