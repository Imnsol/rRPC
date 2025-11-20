//! # rRPC Core
//!
//! Type-safe schema-driven FFI runtime for .NET ↔ Rust communication.
//!
//! ## Example
//!
//! ```rust
//! use rrpc_core::{Registry, RpcError};
//!
//! fn echo(input: &[u8]) -> Result<Vec<u8>, RpcError> {
//!     Ok(input.to_vec())
//! }
//!
//! let mut registry = Registry::new();
//! registry.register("echo", echo);
//! ```

use parking_lot::Mutex;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::sync::OnceLock;

pub mod error;
pub mod registry;

pub use error::RpcError;
pub use registry::Registry;

/// Global registry instance
static GLOBAL_REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();

/// Error codes returned to FFI callers
pub const ERR_SUCCESS: c_int = 0;
pub const ERR_NOT_INITIALIZED: c_int = 1;
pub const ERR_UNKNOWN_METHOD: c_int = 2;
pub const ERR_PARSE_ERROR: c_int = 3;
pub const ERR_NOT_FOUND: c_int = 4;
pub const ERR_SERIALIZATION: c_int = 5;
pub const ERR_INTERNAL: c_int = 99;

/// Initialize the rRPC runtime
///
/// Must be called once before any `rrpc_call` invocations.
///
/// # Safety
/// Safe to call multiple times (idempotent).
#[no_mangle]
pub unsafe extern "C" fn rrpc_init() -> c_int {
    GLOBAL_REGISTRY.get_or_init(|| Mutex::new(Registry::new()));
    ERR_SUCCESS
}

/// Call an RPC method by name
///
/// # Arguments
/// * `method_ptr` - Null-terminated UTF-8 method name
/// * `in_ptr` - Input data buffer
/// * `in_len` - Input buffer length in bytes
/// * `out_ptr` - Output buffer pointer (allocated by this function)
/// * `out_len` - Output buffer length (written by this function)
///
/// # Returns
/// * `ERR_SUCCESS` (0) on success
/// * Error code (>0) on failure
///
/// # Safety
/// Caller must:
/// - Ensure `method_ptr` is valid null-terminated UTF-8
/// - Ensure `in_ptr` points to at least `in_len` bytes
/// - Call `rrpc_free()` on `*out_ptr` when done
#[no_mangle]
pub unsafe extern "C" fn rrpc_call(
    method_ptr: *const c_char,
    in_ptr: *const u8,
    in_len: usize,
    out_ptr: *mut *mut u8,
    out_len: *mut usize,
) -> c_int {
    // Validate registry initialized
    let Some(registry) = GLOBAL_REGISTRY.get() else {
        return ERR_NOT_INITIALIZED;
    };

    // Parse method name
    let method = match CStr::from_ptr(method_ptr).to_str() {
        Ok(s) => s,
        Err(_) => return ERR_PARSE_ERROR,
    };

    // Get input slice
    let input = std::slice::from_raw_parts(in_ptr, in_len);

    // Call handler
    let registry = registry.lock();
    let result = match registry.call(method, input) {
        Ok(data) => data,
        Err(RpcError::UnknownMethod(_)) => return ERR_UNKNOWN_METHOD,
        Err(RpcError::NotFound(_)) => return ERR_NOT_FOUND,
        Err(RpcError::ParseError(_)) => return ERR_PARSE_ERROR,
        Err(RpcError::SerializationError(_)) => return ERR_SERIALIZATION,
        Err(RpcError::Internal(_)) => return ERR_INTERNAL,
    };

    // Allocate output buffer
    let len = result.len();
    let ptr = libc::malloc(len) as *mut u8;
    if ptr.is_null() {
        return ERR_INTERNAL;
    }

    std::ptr::copy_nonoverlapping(result.as_ptr(), ptr, len);

    *out_ptr = ptr;
    *out_len = len;

    ERR_SUCCESS
}

/// Free memory allocated by `rrpc_call`
///
/// # Safety
/// Must only be called once per buffer returned by `rrpc_call`.
/// Pointer must not be used after calling this function.
#[no_mangle]
pub unsafe extern "C" fn rrpc_free(ptr: *mut u8, _len: usize) {
    if !ptr.is_null() {
        libc::free(ptr as *mut libc::c_void);
    }
}

/// Get the global registry (for testing/advanced usage)
pub fn get_registry() -> Option<&'static Mutex<Registry>> {
    GLOBAL_REGISTRY.get()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        unsafe {
            let result = rrpc_init();
            assert_eq!(result, ERR_SUCCESS);
        }
    }

    #[test]
    fn test_registry() {
        unsafe { rrpc_init() };
        
        let registry = get_registry().unwrap();
        let mut reg = registry.lock();
        
        reg.register("test", |input| Ok(input.to_vec()));
        
        let result = reg.call("test", b"hello").unwrap();
        assert_eq!(result, b"hello");
    }
}
