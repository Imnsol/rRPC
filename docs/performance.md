# Performance Tuning Guide

Optimize rRPC for maximum performance and minimal latency.

## Performance Overview

rRPC is designed for **sub-microsecond latency** on local calls. This guide helps you achieve optimal performance.

**Typical Performance:**
- Native FFI: <1μs per call
- WASM: 1-5μs per call
- Network RPC (comparison): 50μs-1ms

## Optimization Strategies

### 1. Zero-Copy Input

**❌ Bad - Unnecessary copy:**
```rust
fn process(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let data = input.to_vec();  // Unnecessary allocation
    let s = String::from_utf8(data).unwrap();
    // ...
}
```

**✅ Good - Direct slice access:**
```rust
fn process(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    // Work directly with the slice
    let value = u32::from_le_bytes([input[0], input[1], input[2], input[3]]);
    // ...
}
```

**✅ Better - Zero-copy parsing:**
```rust
fn process(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    // Use zerocopy crate for safe zero-copy parsing
    let value = zerocopy::LayoutVerified::<_, u32>::new(input)
        .ok_or(RpcError::DecodeFailed("Invalid size".into()))?
        .into_ref();
    // ...
}
```

### 2. Small Result Optimization

**For results ≤ 64 bytes, avoid heap allocation:**

```rust
// ❌ Bad - Always allocates
fn get_count(_input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let count: u64 = 42;
    Ok(count.to_le_bytes().to_vec())  // Heap allocation
}

// ✅ Good - Stack allocation
fn get_count(_input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let count: u64 = 42;
    Ok(count.to_le_bytes().into())  // From [u8; 8] to Vec
}

// ✅ Better - Inline result
fn get_count(_input: &[u8]) -> Result<SmallVec<[u8; 64]>, RpcError> {
    let count: u64 = 42;
    Ok(SmallVec::from_slice(&count.to_le_bytes()))
}
```

### 3. Pre-Allocated Buffers

**Reuse buffers across calls:**

```rust
use std::cell::RefCell;

thread_local! {
    static BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(4096));
}

fn process_many(inputs: &[&[u8]]) -> Vec<Vec<u8>> {
    BUFFER.with(|buf| {
        let mut buffer = buf.borrow_mut();
        inputs.iter().map(|input| {
            buffer.clear();
            // Reuse buffer for each operation
            buffer.extend_from_slice(input);
            // Process...
            buffer.clone()  // Only allocate for result
        }).collect()
    })
}
```

### 4. Batch Operations

**Combine multiple calls:**

```rust
// ❌ Bad - Many small calls
for id in user_ids {
    let result = rrpc_call("get_user", &id.to_bytes());
}

// ✅ Good - Single batched call
let ids_batch = user_ids.iter()
    .flat_map(|id| id.as_bytes())
    .copied()
    .collect::<Vec<_>>();
let results = rrpc_call("get_users_batch", &ids_batch);
```

**Rust handler:**
```rust
fn get_users_batch(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let count = input.len() / 16;  // 16 bytes per UUID
    let mut results = Vec::with_capacity(count);
    
    for chunk in input.chunks_exact(16) {
        let id = Uuid::from_slice(chunk)?;
        results.push(fetch_user(id)?);
    }
    
    // Encode all at once
    serde_json::to_vec(&results).map_err(Into::into)
}
```

### 5. Lazy Serialization

**Don't serialize if not needed:**

```rust
// ❌ Bad - Always serializes
fn maybe_get_user(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let req = GetUserRequest::decode(input)?;
    
    if !is_cached(req.id) {
        return Ok(vec![]);  // Still had to decode request
    }
    
    let user = fetch_user(req.id)?;
    user.encode().map_err(Into::into)
}

// ✅ Good - Early exit without deserialization
fn maybe_get_user(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    // Read UUID directly (first 16 bytes)
    if input.len() < 16 {
        return Err(RpcError::DecodeFailed("Input too small".into()));
    }
    
    let id = Uuid::from_slice(&input[0..16])?;
    
    if !is_cached(id) {
        return Ok(vec![]);  // Fast path
    }
    
    // Only decode full request if needed
    let req = GetUserRequest::decode(input)?;
    let user = fetch_user(req.id)?;
    user.encode().map_err(Into::into)
}
```

### 6. Inline Small Functions

```rust
// Compiler will inline these automatically
#[inline]
fn encode_u32(value: u32) -> [u8; 4] {
    value.to_le_bytes()
}

#[inline]
fn decode_u32(bytes: &[u8]) -> u32 {
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}
```

### 7. Use `&str` Instead of `String` Where Possible

```rust
// ❌ Bad - Allocates
fn process_name(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let name = String::from_utf8(input.to_vec())?;
    // ...
}

// ✅ Good - Borrows
fn process_name(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let name = std::str::from_utf8(input)?;
    // ...
}
```

## Benchmarking

### Using Criterion

**benches/latency.rs:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rrpc_core::Registry;

fn bench_echo(c: &mut Criterion) {
    let mut registry = Registry::new();
    registry.register("echo", |input| Ok(input.to_vec()));
    
    let input = b"Hello, benchmark!";
    
    c.bench_function("echo", |b| {
        b.iter(|| {
            registry.call("echo", black_box(input)).unwrap()
        })
    });
}

fn bench_decode_encode(c: &mut Criterion) {
    #[derive(serde::Serialize, serde::Deserialize)]
    struct User {
        id: uuid::Uuid,
        name: String,
        email: String,
    }
    
    let user = User {
        id: uuid::Uuid::new_v4(),
        name: "Alice".into(),
        email: "alice@example.com".into(),
    };
    
    c.bench_function("json_roundtrip", |b| {
        b.iter(|| {
            let bytes = serde_json::to_vec(&user).unwrap();
            let _decoded: User = serde_json::from_slice(&bytes).unwrap();
        })
    });
}

criterion_group!(benches, bench_echo, bench_decode_encode);
criterion_main!(benches);
```

**Run:**
```powershell
cargo bench
```

### Profiling with flamegraph

```powershell
cargo install flamegraph

# Profile release build
cargo flamegraph --bench latency

# Open flamegraph.svg in browser
```

### Memory Profiling

```powershell
# Install valgrind (Linux)
sudo apt install valgrind

# Run with memory profiling
cargo build --release
valgrind --tool=massif ./target/release/myrrpc

# Analyze
ms_print massif.out.*
```

## Comparison: Serialization Formats

| Format | Encode (ns) | Decode (ns) | Size (bytes) | Notes |
|--------|-------------|-------------|--------------|-------|
| **Raw bytes** | 0 | 0 | Minimal | Manual, type-unsafe |
| **bincode** | 50-100 | 50-100 | Compact | Fast, Rust-specific |
| **MessagePack** | 100-200 | 100-200 | Compact | Cross-language |
| **JSON** | 500-1000 | 800-1500 | Verbose | Human-readable |
| **Protobuf** | 200-400 | 300-500 | Compact | Schema-driven |

**Recommendation:**
- **Ultra-low latency**: Raw bytes (manual encoding)
- **Rust-only**: bincode
- **Cross-language**: MessagePack or Protobuf
- **Debugging**: JSON (switch to binary for production)

### Example: bincode

```toml
[dependencies]
bincode = "1.3"
```

```rust
use bincode::{serialize, deserialize};

fn create_user(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let req: CreateUserRequest = deserialize(input)
        .map_err(|e| RpcError::DecodeFailed(e.to_string()))?;
    
    let user = create_user_impl(req)?;
    
    serialize(&user)
        .map_err(|e| RpcError::EncodeFailed(e.to_string()))
}
```

**Benchmark:**
```
JSON:     1200ns encode, 1800ns decode
bincode:   120ns encode,  150ns decode
10x faster!
```

## WASM-Specific Optimizations

### 1. Reduce WASM Binary Size

**Cargo.toml:**
```toml
[profile.release]
opt-level = "z"     # Optimize for size
lto = true          # Link-time optimization
codegen-units = 1   # Better optimization, slower compile
panic = "abort"     # Remove unwinding code
strip = true        # Remove debug symbols
```

**Use wasm-opt:**
```powershell
wasm-opt -Oz -o output.wasm input.wasm
```

### 2. Avoid String Allocations

```rust
// ❌ Bad - String allocation in WASM
#[wasm_bindgen]
pub fn process(input: String) -> String {
    input.to_uppercase()
}

// ✅ Good - Direct memory access
#[wasm_bindgen]
pub fn process(input: &[u8]) -> Vec<u8> {
    input.iter().map(|b| b.to_ascii_uppercase()).collect()
}
```

### 3. Use WebAssembly SIMD (Experimental)

```rust
#[cfg(target_arch = "wasm32")]
use std::arch::wasm32::*;

#[cfg(target_arch = "wasm32")]
fn sum_simd(data: &[f32]) -> f32 {
    // Use SIMD for parallelism
    unsafe {
        let mut sum = f32x4_splat(0.0);
        for chunk in data.chunks_exact(4) {
            let v = v128_load(chunk.as_ptr() as *const v128);
            sum = f32x4_add(sum, v);
        }
        f32x4_extract_lane::<0>(sum)
            + f32x4_extract_lane::<1>(sum)
            + f32x4_extract_lane::<2>(sum)
            + f32x4_extract_lane::<3>(sum)
    }
}
```

## F# Client Optimizations

### 1. Reuse Encoders/Decoders

```fsharp
// ❌ Bad - Recreates encoder on every call
let call method (data: User) =
    let bytes = JsonSerializer.SerializeToUtf8Bytes(data)
    RRpc.call method bytes

// ✅ Good - Reuse serializer options
let serializerOptions = JsonSerializerOptions()
serializerOptions.PropertyNamingPolicy <- JsonNamingPolicy.CamelCase

let call method (data: User) =
    let bytes = JsonSerializer.SerializeToUtf8Bytes(data, serializerOptions)
    RRpc.call method bytes
```

### 2. Use Span<byte> for Zero-Copy

```fsharp
open System
open System.Buffers

let callWithSpan (method: string) (input: ReadOnlySpan<byte>) : byte[] =
    // Pin span and pass pointer
    use handle = MemoryMarshal.AsMemory(input).Pin()
    let ptr = NativePtr.toNativeInt (NativePtr.ofNativeInt (handle.Pointer))
    
    let mutable outPtr = IntPtr.Zero
    let mutable outLen = UIntPtr.Zero
    
    let result = rrpc_call(
        method,
        uint input.Length,
        ptr,
        uint input.Length,
        &outPtr,
        &outLen
    )
    
    // ... rest of implementation
```

### 3. Pool Buffers

```fsharp
let bufferPool = ArrayPool<byte>.Shared

let callPooled method (data: User) =
    let buffer = bufferPool.Rent(4096)
    try
        use ms = new MemoryStream(buffer)
        JsonSerializer.Serialize(ms, data)
        let input = buffer.[0 .. int ms.Position - 1]
        RRpc.call method input
    finally
        bufferPool.Return(buffer)
```

## Monitoring Performance

### Add Instrumentation

```rust
use std::time::Instant;

pub extern "C" fn rrpc_call(/* ... */) -> i32 {
    let start = Instant::now();
    
    let result = registry.call(method, input);
    
    let elapsed = start.elapsed();
    if elapsed.as_micros() > 100 {
        eprintln!("Slow call: {} took {:?}", method, elapsed);
    }
    
    // ... rest of implementation
}
```

### Track Metrics

```rust
use std::sync::atomic::{AtomicU64, Ordering};

static CALL_COUNT: AtomicU64 = AtomicU64::new(0);
static TOTAL_LATENCY_US: AtomicU64 = AtomicU64::new(0);

pub fn record_call(method: &str, latency_us: u64) {
    CALL_COUNT.fetch_add(1, Ordering::Relaxed);
    TOTAL_LATENCY_US.fetch_add(latency_us, Ordering::Relaxed);
}

pub fn get_avg_latency() -> f64 {
    let count = CALL_COUNT.load(Ordering::Relaxed);
    let total = TOTAL_LATENCY_US.load(Ordering::Relaxed);
    if count == 0 {
        0.0
    } else {
        total as f64 / count as f64
    }
}
```

## Best Practices Summary

1. **Avoid allocations in hot paths**: Use `&[u8]` directly
2. **Batch operations**: Combine multiple calls
3. **Choose efficient serialization**: bincode > MessagePack > JSON
4. **Pre-allocate buffers**: Reuse `Vec<u8>` across calls
5. **Inline small functions**: Use `#[inline]`
6. **Profile before optimizing**: Measure, don't guess
7. **Benchmark regularly**: Catch regressions in CI
8. **Monitor in production**: Track latency percentiles

## Real-World Example

**Before (slow):**
```rust
fn get_user(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let json_str = String::from_utf8(input.to_vec())?;
    let req: GetUserRequest = serde_json::from_str(&json_str)?;
    let user = fetch_user(req.id)?;
    let response_json = serde_json::to_string(&user)?;
    Ok(response_json.into_bytes())
}
// Latency: ~2μs
```

**After (fast):**
```rust
fn get_user(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let req: GetUserRequest = bincode::deserialize(input)?;
    let user = fetch_user(req.id)?;
    bincode::serialize(&user).map_err(Into::into)
}
// Latency: ~0.3μs (6.6x faster)
```

## See Also

- [API Reference](api-reference.md)
- [WASM Guide](wasm-guide.md)
- [Benchmarks](benchmarks.md)
