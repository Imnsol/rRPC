# API Reference

Complete API documentation for rRPC.

## Core FFI Functions

### `rrpc_init`

Initialize the rRPC runtime and function registry.

```rust
#[no_mangle]
pub extern "C" fn rrpc_init() -> i32;
```

**Returns:**
- `0` on success
- `-1` on failure (e.g., already initialized)

**Usage:**
```rust
let result = unsafe { rrpc_init() };
assert_eq!(result, 0);
```

**Notes:**
- Must be called exactly once before any `rrpc_call()` invocations
- Thread-safe: Uses `OnceLock` internally
- Idempotent: Subsequent calls return error but don't crash

---

### `rrpc_call`

Call a registered RPC function by name.

```rust
#[no_mangle]
pub extern "C" fn rrpc_call(
    method_ptr: *const c_char,
    method_len: usize,
    in_ptr: *const u8,
    in_len: usize,
    out_ptr: *mut *mut u8,
    out_len: *mut usize,
) -> i32;
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `method_ptr` | `*const c_char` | Pointer to method name UTF-8 bytes |
| `method_len` | `usize` | Length of method name in bytes |
| `in_ptr` | `*const u8` | Pointer to input buffer |
| `in_len` | `usize` | Length of input buffer |
| `out_ptr` | `*mut *mut u8` | Output: pointer to result buffer |
| `out_len` | `*mut usize` | Output: length of result buffer |

**Returns:**
- `0` on success (check `out_ptr` and `out_len` for result)
- `-1` on error (unknown method, execution failed, etc.)

**Memory Contract:**
- **Input**: Caller allocates and owns; Rust borrows during call
- **Output**: Rust allocates; caller must free via `rrpc_free()`
- **Lifetime**: Output buffer valid until `rrpc_free()` called

**Example (Rust):**
```rust
let method = "echo";
let input = b"Hello, World!";
let mut out_ptr: *mut u8 = std::ptr::null_mut();
let mut out_len: usize = 0;

let result = unsafe {
    rrpc_call(
        method.as_ptr() as *const i8,
        method.len(),
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
    println!("Result: {:?}", output);
    
    // IMPORTANT: Free the buffer
    unsafe { rrpc_free(out_ptr, out_len); }
}
```

**Example (F# P/Invoke):**
```fsharp
[<DllImport("myrrpc.dll", CallingConvention = CallingConvention.Cdecl)>]
extern int rrpc_call(
    IntPtr method_ptr,
    UIntPtr method_len,
    IntPtr in_ptr,
    UIntPtr in_len,
    IntPtr& out_ptr,
    UIntPtr& out_len
)

let call (method: string) (input: byte[]) : byte[] =
    let methodBytes = System.Text.Encoding.UTF8.GetBytes(method)
    use methodHandle = fixed methodBytes
    use inputHandle = fixed input
    
    let mutable outPtr = IntPtr.Zero
    let mutable outLen = UIntPtr.Zero
    
    let result = rrpc_call(
        NativePtr.toNativeInt methodHandle,
        UIntPtr(uint methodBytes.Length),
        NativePtr.toNativeInt inputHandle,
        UIntPtr(uint input.Length),
        &outPtr,
        &outLen
    )
    
    if result = 0 then
        let output = Array.zeroCreate (int outLen)
        Marshal.Copy(outPtr, output, 0, int outLen)
        rrpc_free(outPtr, outLen)
        output
    else
        failwith "RPC call failed"
```

---

### `rrpc_free`

Free a buffer allocated by `rrpc_call()`.

```rust
#[no_mangle]
pub extern "C" fn rrpc_free(ptr: *mut u8, len: usize);
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `ptr` | `*mut u8` | Pointer to buffer (from `rrpc_call`) |
| `len` | `usize` | Length of buffer (from `rrpc_call`) |

**Usage:**
```rust
// After rrpc_call()
unsafe { rrpc_free(out_ptr, out_len); }
```

**Safety:**
- ⚠️ Must be called exactly once per `rrpc_call()` output
- ⚠️ Pointer must not be used after `rrpc_free()`
- ⚠️ Double-free causes undefined behavior
- ✅ Safe to call with null pointer (no-op)

---

## Registry API

### `Registry::new`

Create a new function registry.

```rust
pub fn new() -> Self
```

**Returns:** Empty registry

**Example:**
```rust
let mut registry = Registry::new();
```

---

### `Registry::register`

Register a function handler.

```rust
pub fn register<F>(&mut self, name: &str, handler: F)
where
    F: Fn(&[u8]) -> Result<Vec<u8>, RpcError> + 'static
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `name` | `&str` | Method name (must be unique) |
| `handler` | `F` | Function taking bytes, returning bytes |

**Handler Signature:**
```rust
fn handler(input: &[u8]) -> Result<Vec<u8>, RpcError>
```

**Example:**
```rust
fn echo(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    Ok(input.to_vec())
}

fn add(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    if input.len() != 8 {
        return Err(RpcError::DecodeFailed("Expected 8 bytes".into()));
    }
    let a = u32::from_le_bytes([input[0], input[1], input[2], input[3]]);
    let b = u32::from_le_bytes([input[4], input[5], input[6], input[7]]);
    let sum = a + b;
    Ok(sum.to_le_bytes().to_vec())
}

let mut registry = Registry::new();
registry.register("echo", echo);
registry.register("add", add);
```

---

### `Registry::call`

Invoke a registered function.

```rust
pub fn call(&self, method: &str, input: &[u8]) -> Result<Vec<u8>, RpcError>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `method` | `&str` | Method name |
| `input` | `&[u8]` | Input bytes |

**Returns:**
- `Ok(Vec<u8>)` on success
- `Err(RpcError)` on failure

**Errors:**
- `RpcError::UnknownMethod` if method not registered
- `RpcError::ExecutionFailed` if handler returns error

**Example:**
```rust
let result = registry.call("echo", b"test")?;
assert_eq!(result, b"test");
```

---

## Error Types

### `RpcError`

Represents errors during RPC execution.

```rust
#[derive(Debug)]
pub enum RpcError {
    UnknownMethod(String),
    DecodeFailed(String),
    EncodeFailed(String),
    ExecutionFailed(String),
}
```

**Variants:**

#### `UnknownMethod(String)`
Method name not found in registry.

```rust
Err(RpcError::UnknownMethod("missing_function".into()))
```

#### `DecodeFailed(String)`
Input deserialization failed.

```rust
Err(RpcError::DecodeFailed("Invalid UTF-8".into()))
```

#### `EncodeFailed(String)`
Output serialization failed.

```rust
Err(RpcError::EncodeFailed("JSON encode error".into()))
```

#### `ExecutionFailed(String)`
Handler execution error.

```rust
Err(RpcError::ExecutionFailed("Database connection lost".into()))
```

**Display:**
```rust
match error {
    RpcError::UnknownMethod(name) => 
        eprintln!("Unknown method: {}", name),
    RpcError::DecodeFailed(msg) => 
        eprintln!("Decode failed: {}", msg),
    // ...
}
```

---

## Constants

### Error Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `-1` | General error |
| `-2` | Not initialized |
| `-3` | Unknown method |
| `-4` | Invalid input |

---

## Type Aliases

### Handler Function

```rust
type HandlerFn = dyn Fn(&[u8]) -> Result<Vec<u8>, RpcError>;
```

The signature for all RPC handler functions.

---

## Best Practices

### 1. Always Free Output

```rust
// ✅ Good
let result = unsafe { rrpc_call(/* ... */) };
// ... use output ...
unsafe { rrpc_free(out_ptr, out_len); }

// ❌ Bad - memory leak
let result = unsafe { rrpc_call(/* ... */) };
// ... use output ...
// (forgot to free)
```

### 2. Validate Input Early

```rust
fn get_user(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    if input.len() != 16 {
        return Err(RpcError::DecodeFailed(
            format!("Expected 16 bytes (UUID), got {}", input.len())
        ));
    }
    // ... process valid input ...
}
```

### 3. Use Type-Safe Wrappers

```rust
// Instead of raw bytes
fn get_user_raw(input: &[u8]) -> Result<Vec<u8>, RpcError> { /* ... */ }

// Use strongly-typed codecs
fn get_user(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let request = GetUserRequest::decode(input)
        .map_err(|e| RpcError::DecodeFailed(e.to_string()))?;
    
    let user = fetch_user(request.id)?;
    
    user.encode()
        .map_err(|e| RpcError::EncodeFailed(e.to_string()))
}
```

### 4. Handle Errors Gracefully

```rust
fn divide(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let a = i32::from_le_bytes([input[0], input[1], input[2], input[3]]);
    let b = i32::from_le_bytes([input[4], input[5], input[6], input[7]]);
    
    if b == 0 {
        return Err(RpcError::ExecutionFailed("Division by zero".into()));
    }
    
    let result = a / b;
    Ok(result.to_le_bytes().to_vec())
}
```

---

## Platform-Specific Notes

### Windows (DLL)

```fsharp
[<DllImport("myrrpc.dll")>]
extern int rrpc_init()
```

### Linux (SO)

```python
# Python example
lib = ctypes.CDLL("./libmyrrpc.so")
lib.rrpc_init.restype = ctypes.c_int
result = lib.rrpc_init()
```

### macOS (dylib)

```swift
// Swift example
let lib = dlopen("libmyrrpc.dylib", RTLD_NOW)
let rrpc_init = dlsym(lib, "rrpc_init")
```

---

## Version Compatibility

| rRPC Version | Minimum Rust | C ABI Stable |
|--------------|--------------|--------------|
| 0.1.x | 1.70 | ✅ Yes |
| 0.2.x | 1.75 | ✅ Yes |

C ABI is stable across versions - binaries compiled with different rRPC versions are compatible.

---

## See Also

- [Getting Started Guide](getting-started.md)
- [Architecture Overview](ARCHITECTURE.md)
- [Examples](../examples/)
