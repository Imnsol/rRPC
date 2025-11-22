# rRPC — Type-safe schema-driven FFI for .NET ↔ Rust

rRPC is a lightweight runtime and toolchain for safe, schema-driven cross-language function calls between .NET (F#/C#) and Rust. Instead of adding a network layer, rRPC uses a small C ABI (cdylib) shim and generated bindings so calls are fast, type-safe, and easy to generate.

## Overview

Why rRPC?

- Use-case: when you need safe, low-latency interop between F#/.NET and Rust (desktop apps, deterministic tooling, real-time UDG/graph ops)
- Approach: schema-first bindings + a tiny native runtime (cdylib + P/Invoke) — no HTTP transport by default
- Outcome: compile-time guarantees across languages, minimal marshaling overhead, and consistent codegen for Rust/F#/TS/WASM.

## Key features

- Sub-millisecond local call latency (native FFI)
- Schema-driven code generation for strong typing across languages
- Multi-platform paths: native cdylib (desktop), WASM (browser)
- Low dependency surface: generated bindings + a lightweight runtime
- Licensed under Apache-2.0 (patent protection)

## Try it — quick example

Minimal, end-to-end example showing how a Rust `registry` exposes functions and an F# client calls them.

**Rust (example handler):**
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

## Performance (summary)

Local FFI calls avoid HTTP framing and provide a much faster path for in-process cross-language interactions. Benchmarks are available in `docs/benchmarks.md`.

## Getting started

From the project root you can build and run the demo example (Rust example registry + internal demo runner):

```powershell
# Build all Rust targets
cargo build --release

# Run the small demo (runs example registry handlers)
cargo run --example demo
```

F# and other language bindings will be added to `bindings/` as codegen matures; see `docs/getting-started.md` for up-to-date instructions.

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

## Project status

rRPC is an early-stage research project. The repo owner is currently developing rRPC solo and is not accepting outside contributions. The project is Apache-2.0 licensed — you're welcome to use and fork it.

If you'd like to follow progress, see the short public roadmap in `docs/ROADMAP.md` and the longer living plan `NEXT_STEPS.md` (internal, detailed).

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

## Roadmap (short public view)

See `docs/ROADMAP.md` for a short timeline of our priority milestones. For a detailed, iterative plan, the repository also contains `NEXT_STEPS.md` (developer-focused, living roadmap).

## License

Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)

**Note**: This is early-stage software. The API will evolve before 1.0.
