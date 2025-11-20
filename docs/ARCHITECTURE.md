# rRPC Architecture

This document explains the technical design and architecture of rRPC.

## Overview

rRPC is a **type-safe, schema-driven FFI runtime** that bridges .NET and Rust without network overhead.

```
┌─────────────────────────────────────────────────────────┐
│                     Client Layer                        │
│  ┌──────────┐    ┌──────────┐    ┌──────────────┐     │
│  │  F# App  │    │  C# App  │    │  TypeScript  │     │
│  └────┬─────┘    └────┬─────┘    └──────┬───────┘     │
│       │ P/Invoke      │ P/Invoke         │ JS FFI      │
└───────┼───────────────┼──────────────────┼─────────────┘
        │               │                  │
        └───────────────┴──────────────────┘
                        │
        ┌───────────────▼────────────────────────┐
        │         C ABI Interface                │
        │  rrpc_init(), rrpc_call(), rrpc_free() │
        └───────────────┬────────────────────────┘
                        │
        ┌───────────────▼────────────────────────┐
        │        Function Registry               │
        │  HashMap<String, Handler>              │
        └───────────────┬────────────────────────┘
                        │
        ┌───────────────▼────────────────────────┐
        │         User Functions                 │
        │  fn echo(input: &[u8]) -> Result<..>   │
        │  fn get_user(input: &[u8]) -> Result   │
        └────────────────────────────────────────┘
```

## Core Components

### 1. FFI Layer (`core/src/lib.rs`)

The FFI layer exposes three C-compatible functions:

```rust
#[no_mangle]
pub extern "C" fn rrpc_init() -> i32;

#[no_mangle]
pub extern "C" fn rrpc_call(
    method_ptr: *const c_char,
    method_len: usize,
    in_ptr: *const u8,
    in_len: usize,
    out_ptr: *mut *mut u8,
    out_len: *mut usize,
) -> i32;

#[no_mangle]
pub extern "C" fn rrpc_free(ptr: *mut u8, len: usize);
```

**Design Decisions:**

- **`#[no_mangle]`**: Prevents Rust from mangling function names, making them callable from C/FFI
- **`extern "C"`**: Uses C calling convention for ABI compatibility
- **Raw pointers**: Direct memory access for zero-copy performance
- **Return codes**: `i32` return values (0 = success, negative = error)

### 2. Function Registry (`core/src/registry.rs`)

The registry maps method names to handler functions:

```rust
pub struct Registry {
    handlers: HashMap<String, Box<dyn Fn(&[u8]) -> Result<Vec<u8>, RpcError>>>,
}

impl Registry {
    pub fn register<F>(&mut self, name: &str, handler: F)
    where
        F: Fn(&[u8]) -> Result<Vec<u8>, RpcError> + 'static,
    {
        self.handlers.insert(name.to_string(), Box::new(handler));
    }
    
    pub fn call(&self, method: &str, input: &[u8]) -> Result<Vec<u8>, RpcError> {
        match self.handlers.get(method) {
            Some(handler) => handler(input),
            None => Err(RpcError::UnknownMethod(method.to_string())),
        }
    }
}
```

**Key Features:**

- **Type erasure**: `Box<dyn Fn>` allows heterogeneous function types
- **Lazy evaluation**: Functions registered once, called many times
- **Thread-safe**: Wrapped in `OnceLock` for concurrent access

### 3. Error Handling (`core/src/error.rs`)

```rust
#[derive(Debug)]
pub enum RpcError {
    UnknownMethod(String),
    DecodeFailed(String),
    EncodeFailed(String),
    ExecutionFailed(String),
}
```

**Error Propagation:**

1. Rust function returns `Result<Vec<u8>, RpcError>`
2. Registry converts to integer error codes
3. FFI layer returns code to caller
4. Client layer interprets code and throws appropriate exception

### 4. Memory Management

**Ownership Model:**

```
┌─────────────────────────────────────────────────┐
│  Client (F#/C#)                                 │
│  1. Allocates input buffer                      │
│  2. Calls rrpc_call()                           │
│  3. Receives output pointer                     │
│  4. Reads output                                │
│  5. Calls rrpc_free() to release                │
└────────┬────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────┐
│  Rust (cdylib)                                  │
│  1. Borrows input (no copy)                     │
│  2. Processes data                              │
│  3. Allocates output via Box::into_raw()        │
│  4. Returns raw pointer                         │
│  5. Waits for rrpc_free() to reclaim            │
└─────────────────────────────────────────────────┘
```

**Critical Rules:**

- Input: Caller owns, Rust borrows
- Output: Rust allocates, caller must free via `rrpc_free()`
- Lifetime: Output valid until `rrpc_free()` called

## Data Flow

### Synchronous Call Sequence

```
F# Client                   FFI Layer               Registry              Handler
    │                          │                       │                     │
    ├──rrpc_call()────────────>│                       │                     │
    │  ("get_user", bytes)     │                       │                     │
    │                          ├──lookup("get_user")──>│                     │
    │                          │                       ├──call(bytes)───────>│
    │                          │                       │                     │
    │                          │                       │<──Result<Vec<u8>>───┤
    │                          │<──Result<Vec<u8>>─────┤                     │
    │                          │                       │                     │
    │<─(ptr, len)──────────────┤                       │                     │
    │                          │                       │                     │
    ├──read output─────────────┤                       │                     │
    │                          │                       │                     │
    ├──rrpc_free(ptr, len)────>│                       │                     │
    │                          │                       │                     │
```

**Latency Breakdown:**

| Phase | Time | Notes |
|-------|------|-------|
| Method lookup | ~10ns | HashMap O(1) average |
| Handler call | Variable | User function time |
| Memory allocation | ~100ns | Box allocation |
| Total overhead | **<200ns** | Excluding handler execution |

Compare to gRPC localhost: ~50μs (250x slower)

## Schema System (Planned v0.2)

### MSL (Mycelium Schema Language)

```yaml
# user.msl
schema: mycelium/v1

types:
  User:
    id: uuid
    name: string
    email: string
    created: timestamp
    
  GetUserRequest:
    id: uuid
    
  GetUserResponse:
    user: User
    
functions:
  get_user:
    input: GetUserRequest
    output: GetUserResponse
```

### Code Generation

```
user.msl
   │
   ├──msl-compiler──>  F# Types + Codecs
   │                   (User.fs)
   │
   └──msl-compiler──>  Rust Types + Codecs
                       (user.rs)
```

**Generated Rust:**
```rust
// Auto-generated from user.msl
pub struct User {
    pub id: uuid::Uuid,
    pub name: String,
    pub email: String,
    pub created: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub fn encode(&self) -> Vec<u8> { /* ... */ }
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> { /* ... */ }
}
```

**Generated F#:**
```fsharp
// Auto-generated from user.msl
type User = {
    Id: Guid
    Name: string
    Email: string
    Created: DateTimeOffset
}

module UserCodec =
    let encode (user: User) : byte[] = (* ... *)
    let decode (bytes: byte[]) : User = (* ... *)
```

## Performance Optimizations

### 1. Zero-Copy Input

```rust
fn process(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    // Direct slice access - no allocation
    let value = u32::from_le_bytes([input[0], input[1], input[2], input[3]]);
    // ...
}
```

### 2. Pre-Allocated Buffers

```rust
// Reuse buffer for multiple calls
fn process_batch(inputs: &[&[u8]]) -> Vec<Vec<u8>> {
    let mut results = Vec::with_capacity(inputs.len());
    for input in inputs {
        results.push(process_one(input).unwrap());
    }
    results
}
```

### 3. Inline Small Results

```rust
// For small responses, use stack allocation
fn get_count(_input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let count: u32 = 42;
    Ok(count.to_le_bytes().to_vec())  // Only 4 bytes
}
```

## Security Model (Planned v0.3)

### Capability-Based Permissions

```yaml
# schema with capabilities
types:
  TerminateProcessCommand:
    requires: capability[process.terminate]
    pid: u32
```

**Enforcement:**

1. Schema declares required capability
2. Code generator creates phantom type
3. Caller must provide capability token at compile time

```fsharp
// F# client
let capability = Capability.ProcessTerminate  // Granted by user
let cmd = { Pid = 1234u }
RRpc.call<TerminateProcessCommand> "terminate" cmd capability
//                                                  ^^^^^^^^^^
//                                          Compile-time proof
```

## WASM Integration (Planned v0.2)

### Browser Architecture

```
┌─────────────────────────────────────┐
│  Browser                            │
│  ┌───────────────────────────────┐ │
│  │  TypeScript/Fable             │ │
│  │  ┌─────────────────────────┐  │ │
│  │  │  wasm-bindgen JS glue   │  │ │
│  │  └───────────┬─────────────┘  │ │
│  │              │                 │ │
│  │              ▼                 │ │
│  │  ┌─────────────────────────┐  │ │
│  │  │  rRPC.wasm (Rust)       │  │ │
│  │  │  Same lib.rs code!      │  │ │
│  │  └─────────────────────────┘  │ │
│  └───────────────────────────────┘ │
└─────────────────────────────────────┘
```

**Build Command:**
```bash
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/rrpc_core.wasm \
  --out-dir web/pkg --target web
```

## Comparison: rRPC vs Alternatives

### vs gRPC

| Aspect | rRPC | gRPC |
|--------|------|------|
| **Transport** | Direct FFI | HTTP/2 |
| **Latency** | <1μs | 50μs-1ms |
| **Schema** | MSL (custom) | Protobuf |
| **Use Case** | Local IPC | Distributed services |
| **Dependencies** | None | gRPC runtime |

### vs Raw FFI

| Aspect | rRPC | Raw FFI |
|--------|------|---------|
| **Type Safety** | Schema-driven | Manual |
| **Error Handling** | Structured | Error codes |
| **Memory Safety** | Managed | Manual |
| **Versioning** | Schema-based | None |

### vs tRPC

| Aspect | rRPC | tRPC |
|--------|------|------|
| **Languages** | F#/Rust/TS | TypeScript only |
| **Transport** | FFI/WASM | HTTP |
| **Performance** | Native | Network-bound |
| **Type Safety** | Compile-time | Inferred (TS) |

## Future Directions

### Time-Travel Debugging (v0.4)

```rust
// Auto-log every RPC call
struct EventLog {
    calls: Vec<(String, Vec<u8>, Vec<u8>)>,  // (method, input, output)
}

fn replay_from(log: &EventLog, index: usize) -> AppState {
    // Reconstruct state by replaying calls 0..index
}
```

### Streaming API (v0.5)

```rust
fn stream_events(
    input: &[u8],
    callback: extern "C" fn(*const u8, usize),
) -> i32 {
    // Invoke callback for each event
}
```

## Contributing

This is a solo research project. See [README.md](../README.md) for contribution policy.

## References

- [C ABI Specification](https://en.wikipedia.org/wiki/Application_binary_interface)
- [Rust FFI Guide](https://doc.rust-lang.org/nomicon/ffi.html)
- [Protocol Buffers](https://protobuf.dev/)
- [WebAssembly Interface Types](https://github.com/WebAssembly/component-model)
