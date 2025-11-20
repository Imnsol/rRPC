# WebAssembly (WASM) Guide

Run rRPC in the browser using WebAssembly.

## Overview

The same Rust code that powers native rRPC can compile to WebAssembly and run in browsers with near-native performance.

**Architecture:**
```
Browser
  └─ JavaScript/TypeScript
       └─ wasm-bindgen glue
            └─ rRPC.wasm (Rust compiled to WASM)
```

**Benefits:**
- ✅ Same codebase (desktop + web)
- ✅ Type-safe across boundaries
- ✅ 10-50x faster than pure JavaScript
- ✅ Runs in sandbox (secure)

## Prerequisites

Install WASM toolchain:

```powershell
# Add wasm32 target
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen CLI
cargo install wasm-bindgen-cli

# Install wasm-pack (optional, but recommended)
cargo install wasm-pack
```

## Quick Start

### Step 1: Update Cargo.toml

Add wasm-bindgen dependency:

```toml
[package]
name = "myrrpc"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]  # Required for WASM

[dependencies]
rrpc-core = "0.1"
wasm-bindgen = "0.2"

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
```

### Step 2: Add WASM Exports

**src/lib.rs:**
```rust
use wasm_bindgen::prelude::*;
use rrpc_core::{Registry, RpcError};
use std::sync::OnceLock;

static REGISTRY: OnceLock<Registry> = OnceLock::new();

// Register your functions
fn echo(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    Ok(input.to_vec())
}

// WASM-specific initialization
#[wasm_bindgen]
pub fn rrpc_init() -> i32 {
    // Optional: set panic hook for better error messages
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
    
    let mut registry = Registry::new();
    registry.register("echo", echo);
    
    match REGISTRY.set(registry) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[wasm_bindgen]
pub fn rrpc_call(method: &str, input: &[u8]) -> Vec<u8> {
    let registry = REGISTRY.get().expect("Not initialized");
    
    match registry.call(method, input) {
        Ok(result) => result,
        Err(e) => {
            // Log error to console
            web_sys::console::error_1(&format!("RPC error: {:?}", e).into());
            vec![]
        }
    }
}
```

### Step 3: Build for WASM

**Using wasm-pack (recommended):**
```powershell
wasm-pack build --target web
```

**Using cargo + wasm-bindgen:**
```powershell
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/myrrpc.wasm \
  --out-dir pkg \
  --target web
```

This produces:
- `pkg/myrrpc_bg.wasm` - The WebAssembly binary
- `pkg/myrrpc.js` - JavaScript glue code
- `pkg/myrrpc.d.ts` - TypeScript definitions

### Step 4: Use in Browser

**index.html:**
```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>rRPC WASM Demo</title>
</head>
<body>
    <h1>rRPC WASM Demo</h1>
    <button id="testBtn">Test Echo</button>
    <pre id="output"></pre>
    
    <script type="module">
        import init, { rrpc_init, rrpc_call } from './pkg/myrrpc.js';
        
        async function run() {
            // Load WASM module
            await init();
            
            // Initialize rRPC
            const initResult = rrpc_init();
            console.log('Init result:', initResult);
            
            // Test echo function
            document.getElementById('testBtn').addEventListener('click', () => {
                const encoder = new TextEncoder();
                const decoder = new TextDecoder();
                
                const input = encoder.encode("Hello from browser!");
                const output = rrpc_call("echo", input);
                const result = decoder.decode(output);
                
                document.getElementById('output').textContent = result;
            });
        }
        
        run();
    </script>
</body>
</html>
```

**Serve locally:**
```powershell
# Python
python -m http.server 8080

# Node.js
npx http-server -p 8080

# Rust
cargo install miniserve
miniserve . -p 8080
```

Open: http://localhost:8080

## TypeScript Integration

### Type Definitions

**types.ts:**
```typescript
export interface User {
    id: string;
    name: string;
    email: string;
}

export interface RpcClient {
    init(): number;
    call(method: string, input: Uint8Array): Uint8Array;
}
```

### Wrapper Class

**rrpc-client.ts:**
```typescript
import init, { rrpc_init, rrpc_call } from './pkg/myrrpc.js';

export class RRpcClient {
    private initialized = false;
    
    async init(): Promise<void> {
        if (this.initialized) return;
        
        await init();
        const result = rrpc_init();
        if (result !== 0) {
            throw new Error('Failed to initialize rRPC');
        }
        this.initialized = true;
    }
    
    call(method: string, input: Uint8Array): Uint8Array {
        if (!this.initialized) {
            throw new Error('Client not initialized. Call init() first.');
        }
        return rrpc_call(method, input);
    }
    
    callJson<T>(method: string, input: any): T {
        const encoder = new TextEncoder();
        const decoder = new TextDecoder();
        
        const inputJson = JSON.stringify(input);
        const inputBytes = encoder.encode(inputJson);
        
        const outputBytes = this.call(method, inputBytes);
        const outputJson = decoder.decode(outputBytes);
        
        return JSON.parse(outputJson);
    }
}

// Usage
const client = new RRpcClient();
await client.init();

const user = client.callJson<User>('get_user', { id: '123' });
console.log(user.name);
```

## Fable Integration

Fable compiles F# to JavaScript and works seamlessly with WASM:

**App.fs:**
```fsharp
module App

open Fable.Core
open Fable.Core.JsInterop

// Import WASM module
[<Import("default", from="./pkg/myrrpc.js")>]
let init: unit -> JS.Promise<unit> = jsNative

[<Import("rrpc_init", from="./pkg/myrrpc.js")>]
let rrpcInit: unit -> int = jsNative

[<Import("rrpc_call", from="./pkg/myrrpc.js")>]
let rrpcCall: string -> byte[] -> byte[] = jsNative

// High-level API
let initRRpc () =
    promise {
        do! init()
        let result = rrpcInit()
        if result <> 0 then
            failwith "Failed to initialize rRPC"
    }

let call (method: string) (input: byte[]) : byte[] =
    rrpcCall method input

// Elmish Model-View-Update
type Model = {
    Users: User list
    Status: string
}

type Msg =
    | LoadUsers
    | UsersLoaded of User list
    | Error of string

let update msg model =
    match msg with
    | LoadUsers ->
        let cmd = Cmd.OfPromise.either
            (fun () -> 
                promise {
                    let input = System.Text.Encoding.UTF8.GetBytes("")
                    let output = call "list_users" input
                    let json = System.Text.Encoding.UTF8.GetString(output)
                    return Thoth.Json.Decode.Auto.fromString<User list>(json)
                }
            )
            ()
            (function Ok users -> UsersLoaded users | Error e -> Error e)
            (fun ex -> Error ex.Message)
        { model with Status = "Loading..." }, cmd
    
    | UsersLoaded users ->
        { model with Users = users; Status = "Loaded" }, Cmd.none
    
    | Error msg ->
        { model with Status = sprintf "Error: %s" msg }, Cmd.none
```

## Performance Optimization

### 1. Minimize Allocations

```rust
// ❌ Bad - allocates on every call
#[wasm_bindgen]
pub fn process(input: &[u8]) -> Vec<u8> {
    let s = String::from_utf8(input.to_vec()).unwrap();
    s.to_uppercase().into_bytes()
}

// ✅ Good - minimal allocations
#[wasm_bindgen]
pub fn process(input: &[u8]) -> Vec<u8> {
    input.iter()
        .map(|&b| b.to_ascii_uppercase())
        .collect()
}
```

### 2. Use Shared Memory

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct SharedBuffer {
    data: Vec<u8>,
}

#[wasm_bindgen]
impl SharedBuffer {
    #[wasm_bindgen(constructor)]
    pub fn new(size: usize) -> Self {
        Self { data: vec![0; size] }
    }
    
    pub fn as_ptr(&self) -> *const u8 {
        self.data.as_ptr()
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn write(&mut self, offset: usize, bytes: &[u8]) {
        self.data[offset..offset + bytes.len()].copy_from_slice(bytes);
    }
}

// JavaScript can read/write directly to WASM memory
```

### 3. Batch Operations

```typescript
// Instead of:
for (const item of items) {
    rrpc_call("process", item);
}

// Batch:
const batch = new Uint8Array(items.flat());
rrpc_call("process_batch", batch);
```

## Debugging

### Console Logging

```rust
use web_sys::console;

#[wasm_bindgen]
pub fn debug_info() {
    console::log_1(&"Debug message".into());
    console::warn_1(&"Warning message".into());
    console::error_1(&"Error message".into());
}
```

### Better Panic Messages

```toml
[dependencies]
console_error_panic_hook = "0.1"
```

```rust
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}
```

### Source Maps

Build with debug info:
```powershell
wasm-pack build --dev
```

### Browser DevTools

1. Open browser console (F12)
2. Check "Enable WASM debugging"
3. Set breakpoints in Rust code (Chrome/Edge support this)

## Size Optimization

### 1. Release Build with Optimizations

**Cargo.toml:**
```toml
[profile.release]
opt-level = "z"  # Optimize for size
lto = true       # Link-time optimization
codegen-units = 1
panic = "abort"
strip = true     # Remove debug symbols
```

### 2. wasm-opt

```powershell
# Install
npm install -g wasm-opt

# Optimize
wasm-opt -Oz -o output.wasm input.wasm
```

### 3. Feature Flags

Disable unused features:
```toml
[dependencies]
serde = { version = "1", default-features = false }
```

**Typical sizes:**
- Debug: 2-5 MB
- Release: 500 KB - 2 MB
- Optimized: 100-500 KB

## Worker Threads

Run rRPC in a Web Worker for non-blocking UI:

**worker.js:**
```javascript
import init, { rrpc_init, rrpc_call } from './pkg/myrrpc.js';

let initialized = false;

self.onmessage = async (e) => {
    if (!initialized) {
        await init();
        rrpc_init();
        initialized = true;
    }
    
    const { method, input } = e.data;
    const output = rrpc_call(method, input);
    self.postMessage(output);
};
```

**main.js:**
```javascript
const worker = new Worker('worker.js', { type: 'module' });

worker.postMessage({
    method: 'process_large_data',
    input: new Uint8Array([/* ... */])
});

worker.onmessage = (e) => {
    const result = e.data;
    console.log('Worker result:', result);
};
```

## Cross-Origin Considerations

### CORS Headers

Server must set:
```
Access-Control-Allow-Origin: *
Content-Type: application/wasm
```

### Service Worker Caching

```javascript
// sw.js
self.addEventListener('install', (event) => {
    event.waitUntil(
        caches.open('rrpc-v1').then((cache) => {
            return cache.addAll([
                '/pkg/myrrpc_bg.wasm',
                '/pkg/myrrpc.js',
            ]);
        })
    );
});
```

## Comparison: Native vs WASM

| Metric | Native (cdylib) | WASM (browser) |
|--------|-----------------|----------------|
| **Latency** | <1μs | 1-5μs |
| **Throughput** | 1M+ ops/sec | 100K-500K ops/sec |
| **Startup** | Instant | 10-50ms (load + init) |
| **Memory** | Direct | Sandbox isolated |
| **Debugging** | Full | Limited (improving) |

WASM is ~2-5x slower than native but **10-50x faster than JavaScript**.

## Examples

See complete examples:
- [examples/wasm-demo/](../examples/wasm-demo/) - Basic browser demo
- [examples/fable-wasm/](../examples/fable-wasm/) - F# Fable integration
- [examples/worker-pool/](../examples/worker-pool/) - Multi-threaded processing

## Troubleshooting

### Module not found

Ensure you're using a local server (WASM can't load from `file://` URLs).

### Import errors

Check that your import path matches the `wasm-pack` output:
```javascript
// Correct
import init from './pkg/myrrpc.js';

// Wrong (missing .js extension)
import init from './pkg/myrrpc';
```

### Memory grows unbounded

Always release resources:
```rust
#[wasm_bindgen]
pub struct MyResource {
    data: Vec<u8>,
}

#[wasm_bindgen]
impl MyResource {
    // JavaScript calls this when dropping the object
    pub fn free(self) {
        // Explicit cleanup if needed
    }
}
```

## See Also

- [wasm-bindgen Book](https://rustwasm.github.io/wasm-bindgen/)
- [Fable Documentation](https://fable.io)
- [Getting Started](getting-started.md)
- [Performance Guide](performance.md)
