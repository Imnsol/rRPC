# Getting Started with rRPC

This guide will help you set up and use rRPC in your project.

## What is rRPC?

rRPC (Rusty Remote Procedure Call) is a type-safe, schema-driven FFI (Foreign Function Interface) library that enables seamless communication between .NET (F#/C#) and Rust with native performance and compile-time safety.

Unlike traditional RPC solutions:
- **No HTTP overhead**: Direct C ABI calls via cdylib
- **Type-safe**: Schema-driven code generation
- **Sub-microsecond latency**: 10-100x faster than gRPC for local calls

## Prerequisites

### Rust Development
- Rust 1.70+ ([install from rustup.rs](https://rustup.rs))
- Cargo (included with Rust)

### .NET Development (Optional)
- .NET 6.0+ SDK ([download](https://dotnet.microsoft.com/download))
- F# or C# support

## Installation

### Rust Library

Add rRPC to your `Cargo.toml`:

```toml
[dependencies]
rrpc-core = "0.1"

[lib]
crate-type = ["cdylib"]  # Required for FFI
```

### Build the Library

```powershell
cargo build --release
```

This produces a shared library:
- **Windows**: `target/release/your_lib.dll`
- **Linux**: `target/release/libyour_lib.so`
- **macOS**: `target/release/libyour_lib.dylib`

## Quick Start: Echo Server

### Step 1: Create a Rust Library

**Cargo.toml:**
```toml
[package]
name = "my-rrpc-server"
version = "0.1.0"
edition = "2021"

[dependencies]
rrpc-core = "0.1"

[lib]
crate-type = ["cdylib"]
```

**src/lib.rs:**
```rust
use rrpc_core::{Registry, RpcError};
use std::sync::OnceLock;

static REGISTRY: OnceLock<Registry> = OnceLock::new();

// Your RPC function
fn echo(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    Ok(input.to_vec())
}

fn reverse(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let mut output = input.to_vec();
    output.reverse();
    Ok(output)
}

#[no_mangle]
pub extern "C" fn rrpc_init() -> i32 {
    let mut registry = Registry::new();
    registry.register("echo", echo);
    registry.register("reverse", reverse);
    
    REGISTRY.set(registry).unwrap();
    0  // Success
}

#[no_mangle]
pub extern "C" fn rrpc_call(
    method_ptr: *const std::os::raw::c_char,
    method_len: usize,
    in_ptr: *const u8,
    in_len: usize,
    out_ptr: *mut *mut u8,
    out_len: *mut usize,
) -> i32 {
    // Safety: Assumes valid pointers from caller
    let method = unsafe {
        let slice = std::slice::from_raw_parts(method_ptr as *const u8, method_len);
        std::str::from_utf8(slice).unwrap()
    };
    
    let input = unsafe { std::slice::from_raw_parts(in_ptr, in_len) };
    
    let registry = REGISTRY.get().unwrap();
    match registry.call(method, input) {
        Ok(result) => {
            // Allocate output buffer
            let boxed = result.into_boxed_slice();
            unsafe {
                *out_len = boxed.len();
                *out_ptr = Box::into_raw(boxed) as *mut u8;
            }
            0  // Success
        }
        Err(e) => {
            eprintln!("RPC error: {:?}", e);
            -1  // Error
        }
    }
}

#[no_mangle]
pub extern "C" fn rrpc_free(ptr: *mut u8, len: usize) {
    if !ptr.is_null() {
        unsafe {
            let _ = Box::from_raw(std::slice::from_raw_parts_mut(ptr, len));
        }
    }
}
```

### Step 2: Build the Library

```powershell
cargo build --release
```

### Step 3: Test in Rust

Create `examples/test.rs`:

```rust
use rrpc_core::Registry;

fn main() {
    // Initialize
    unsafe { rrpc_init(); }
    
    // Call echo function
    let input = b"Hello, rRPC!";
    let mut out_ptr: *mut u8 = std::ptr::null_mut();
    let mut out_len: usize = 0;
    
    let result = unsafe {
        rrpc_call(
            "echo".as_ptr() as *const i8,
            4,
            input.as_ptr(),
            input.len(),
            &mut out_ptr,
            &mut out_len,
        )
    };
    
    if result == 0 {
        let output = unsafe {
            std::slice::from_raw_parts(out_ptr, out_len)
        };
        println!("Echo result: {}", String::from_utf8_lossy(output));
        
        // Free memory
        unsafe { rrpc_free(out_ptr, out_len); }
    }
}
```

Run it:
```powershell
cargo run --example test
# Output: Echo result: Hello, rRPC!
```

## Next Steps

### For F# Integration
See [F# Bindings Guide](fsharp-bindings.md) for P/Invoke integration.

### For TypeScript/WASM
See [WASM Guide](wasm-guide.md) for browser integration.

### Advanced Features
- [Schema-Driven Development](schema-guide.md) - Use MSL for type-safe contracts
- [Error Handling](error-handling.md) - Best practices for RPC errors
- [Performance Tuning](performance.md) - Zero-copy and optimization tips

## Common Issues

### `cdylib` Not Found
Ensure `[lib] crate-type = ["cdylib"]` is in your `Cargo.toml`.

### Crashes on Function Call
- Verify all pointers are valid
- Check that buffers are properly allocated
- Ensure `rrpc_init()` is called before `rrpc_call()`

### Memory Leaks
Always call `rrpc_free()` for every buffer returned by `rrpc_call()`.

## Example Projects

See the [examples/](../examples/) directory for complete working projects:
- `echo/` - Simple echo server
- `calculator/` - Math operations with typed schemas
- `wasm-demo/` - Browser-based demo

## Community & Support

- **Issues**: [GitHub Issues](https://github.com/Imnsol/rRPC/issues)
- **Discussions**: This is a solo research project; contributions not accepted at this time
- **License**: Apache-2.0

## Further Reading

- [Architecture Overview](ARCHITECTURE.md)
- [API Reference](api-reference.md)
- [Performance Benchmarks](benchmarks.md)
