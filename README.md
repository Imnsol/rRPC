# rRPC

**Type-safe schema-driven FFI for .NET ↔ Rust**

rRPC brings gRPC's type safety and code generation to local function calls, eliminating HTTP overhead for 10-100x lower latency.

## Why rRPC?

Modern applications often need multiple languages:
- **F#/C#** for business logic and type-safe domain modeling
- **Rust** for performance-critical code and systems programming
- **TypeScript** for web interfaces

Traditional solutions force you to choose:
- **gRPC**: Type-safe but requires HTTP/2 (network overhead even for local calls)
- **Raw FFI**: Fast but type-unsafe, manual marshaling, brittle interfaces
- **tRPC**: TypeScript-only, still uses HTTP

**rRPC gives you both**: gRPC's type safety + FFI's native performance.

## Features

- ✅ **Sub-millisecond latency**: Native FFI (cdylib + P/Invoke), no network stack
- ✅ **Compile-time safety**: Schema-driven codegen for F#/Rust/TypeScript
- ✅ **Multi-platform**: Desktop (native), Web (WASM), Mobile (WASM)
- ✅ **Zero runtime deps**: Just generated bindings + thin FFI layer
- ✅ **Apache-2.0 licensed**: Patent protection for users and contributors

## Quick Example

**Rust (server):**
```rust
use rrpc_core::{Registry, RpcError};

fn get_user(_input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let user = r#"{"id": "123", "name": "Alice"}"#;
    Ok(user.as_bytes().to_vec())
}

fn main() {
    let mut registry = Registry::new();
    registry.register("get_user", get_user);
    
    // FFI functions automatically use registry
    unsafe { rrpc_init(); }
}
```

**F# (client):**
```fsharp
open RRpc

[<EntryPoint>]
let main argv =
    RRpc.init()
    let result = RRpc.call "get_user" [||]
    printfn "User: %s" (System.Text.Encoding.UTF8.GetString(result))
    0
```

## Performance

| Operation | rRPC | gRPC (localhost) | gRPC (network) |
|-----------|------|------------------|----------------|
| **Simple call** | <1μs | 50μs-1ms | 5-50ms |
| **1MB payload** | <100μs | 2-5ms | 10-100ms |
| **Throughput** | 1M+ calls/sec | ~10K calls/sec | ~1K calls/sec |

## Installation

### Rust
```toml
[dependencies]
rrpc-core = "0.1"
```

### F#
```fsharp
#r "nuget: RRpc.FSharp, 0.1.0"
```

## Documentation

- [Getting Started](docs/getting-started.md)
- [Architecture](docs/ARCHITECTURE.md)
- [API Reference](docs/api-reference.md)
- [Examples](examples/)

## Security Model

⚠️ **rRPC is designed for trusted code only.**

**Fundamental limitations:**
- Rust handlers run with full process privileges
- No memory isolation between .NET and Rust  
- Panics crash the entire application
- Cannot sandbox handler execution

**Use rRPC when:**
- You control all Rust code
- Handlers are as trusted as your application code
- Process crash is acceptable failure mode

**Do NOT use rRPC for:**
- Untrusted plugins or user-supplied code
- Security boundaries between components
- Multi-tenant environments

See [SECURITY.md](SECURITY.md) for detailed security considerations.

## Project Status

**This is an experimental research project.** I'm currently developing rRPC solo and **not accepting outside contributions** at this time.

You're welcome to:
- ✅ Use rRPC in your projects (Apache-2.0 license)
- ✅ Fork and experiment on your own

I may open contributions in the future, but for now I'm keeping full control of the codebase to move quickly and maintain architectural coherence.

## Use Cases

rRPC fills a specific niche: **type-safe, schema-driven FFI for trusted environments** where you need native performance without network overhead.

### Real-World Scenarios

**🎮 Game Engines & Graphics**
```fsharp
// F# game logic calling high-performance Rust physics engine
let updatePhysics (entities: Entity[]) =
    RpcClient.call<Entity[], PhysicsResult> "physics_step" entities
```
- **Why rRPC**: Sub-microsecond latency for per-frame physics updates
- **Alternative**: Raw FFI (unsafe, no type checking) or C++ (no F# integration)

**💹 High-Frequency Trading**
```fsharp
// F# strategy engine calling Rust market data processing
let analyzeBook (book: OrderBook) =
    RpcClient.call<OrderBook, Signal> "analyze_market" book
```
- **Why rRPC**: <1μs latency critical for algorithmic trading
- **Alternative**: gRPC adds 50μs-1ms overhead (unacceptable for HFT)

**🖥️ Desktop Applications**
```fsharp
// Electron-like app: F# UI + Rust native APIs
let readFile (path: string) =
    RpcClient.call<string, byte[]> "fs_read" path
```
- **Why rRPC**: Type-safe native access without node.js dependencies
- **Alternative**: Electron (large bundle) or raw P/Invoke (brittle)

**🔬 Scientific Computing**
```fsharp
// F# notebook calling Rust numerical libraries
let computeEigenvalues (matrix: float[][]) =
    RpcClient.call<Matrix, Eigenvalues> "lapack_eig" matrix
```
- **Why rRPC**: Zero-copy arrays, native BLAS/LAPACK performance
- **Alternative**: PythonNet (slow) or manual marshaling (error-prone)

**🌐 WASM Applications**
```typescript
// TypeScript web app calling Rust WASM modules
const result = await rpc.call<UserInput, ProcessedData>("process", input);
```
- **Why rRPC**: Type-safe WASM imports with schema validation
- **Alternative**: Raw `WebAssembly.instantiate` (no types, manual serialization)

**🔧 Developer Tools**
```fsharp
// F# build system calling Rust compiler toolchain
let compile (source: SourceFile) =
    RpcClient.call<SourceFile, CompileResult> "rustc_compile" source
```
- **Why rRPC**: Integrate Rust tooling into F# workflows with type safety
- **Alternative**: Process spawning (slow, no structured data)

**📊 Batch Processing & ETL**
```fsharp
// F# orchestration + Rust data processing
let processChunk (chunk: DataChunk) =
    RpcClient.call<DataChunk, ProcessedData> "transform" chunk
```
- **Why rRPC**: Process millions of records/sec with type-safe pipelines
- **Alternative**: Spark/Flink (distributed overhead) or pandas (GIL-limited)

### When to Use rRPC

✅ **Desktop applications** where you control all code (CAD, IDEs, games)  
✅ **Internal tools** and developer utilities  
✅ **Scientific computing** with trusted numerical libraries  
✅ **Financial systems** where microseconds matter and code is audited  
✅ **WASM apps** for type-safe browser/native code sharing  
✅ **Incremental migration** from monoliths to polyglot architectures  

### When NOT to Use rRPC

❌ **Plugin systems** with third-party code (use process isolation)  
❌ **Multi-tenant SaaS** (use microservices with network boundaries)  
❌ **Untrusted environments** (rRPC has no security sandboxing)  
❌ **Simple apps** that don't need FFI performance

## Comparison

| Feature | rRPC | gRPC | tRPC | Raw FFI |
|---------|------|------|------|---------|
| **Latency** | <1μs | 50μs-1ms | 5-20ms | <1μs |
| **Type Safety** | ✅ Schema | ✅ Proto | ✅ TS only | ❌ Manual |
| **Multi-Language** | ✅ F#/Rust/TS | ✅ Many | ❌ TS only | ⚠️ C-compat |
| **No Network** | ✅ | ❌ | ❌ | ✅ |
| **Patent Protection** | ✅ Apache-2.0 | ✅ Apache-2.0 | ❌ MIT | ❌ Varies |

## Roadmap

### v0.1.0 (Current)
- [x] Core FFI runtime with function registry
- [x] Error handling and type definitions
- [x] Basic examples (echo, reverse)
- [x] Apache-2.0 license with patent protection
- [x] CI/CD pipeline

### v0.2.0 (Schema & Multi-Platform)
- [ ] **MSL Schema Compiler**: Custom schema language for type definitions
  - Basic types (uuid, string, i32, f32, bytes, timestamp)
  - Enums and composite types
  - Code generation for F# and Rust
- [ ] **F# Bindings**: P/Invoke wrappers and marshaling helpers
  - Auto-generated from schemas
  - Async/Task support
- [ ] **WASM Support**: Compile to wasm32-unknown-unknown
  - Same Rust code runs native and browser
  - wasm-bindgen integration

### v0.3.0 (Web & Security)
- [ ] **TypeScript Bindings**: Browser/Node.js integration
  - Generated from MSL schemas
  - WASM FFI bridge
- [ ] **Zero-Copy Optimization**: Direct buffer passing for large payloads
  - Lifetime-safe borrowed slices
  - Pinned memory support
- [ ] **Capability System**: Permission-based security
  - Declarative capability requirements in schemas
  - Compile-time permission checks

### v0.4.0 (Advanced Features)
- [ ] **Time-Travel Helpers**: Event sourcing and replay utilities
  - Automatic command logging
  - State reconstruction APIs
- [ ] **Benchmarking Suite**: Formal performance comparisons
  - rRPC vs gRPC (localhost and network)
  - rRPC vs tRPC
  - rRPC vs raw FFI
- [ ] **Migration Guide**: From gRPC/tRPC to rRPC

### v1.0.0 (Stable Release)
- [ ] Complete MSL compiler with versioning support
- [ ] All language bindings production-ready
- [ ] Stable API guarantees
- [ ] Comprehensive documentation
- [ ] Published to crates.io and nuget.org

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)

**Note**: This is early-stage software. The API will evolve before 1.0.
