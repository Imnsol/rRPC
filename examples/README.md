# rRPC Examples

This directory contains complete working examples demonstrating rRPC usage.

## Available Examples

### 1. Echo Server (`demo.rs`)

**What it does:** Simple echo and reverse functions demonstrating basic rRPC patterns.

**Run it:**
```powershell
cargo run --example demo
```

**Output:**
```
Initializing rRPC...
Registering functions...

Testing echo function:
Input: "Hello, rRPC!"
Output: "Hello, rRPC!"

Testing reverse function:
Input: "Hello, rRPC!"
Output: "!CPRr ,olleH"

Testing error handling:
Calling unknown method 'missing'...
Error: Unknown method: missing
```

**Key concepts:**
- Function registration
- Basic error handling
- Memory management (allocate/free)

**Code highlights:**
```rust
// Register handlers
registry.register("echo", |input| Ok(input.to_vec()));
registry.register("reverse", |input| {
    let mut output = input.to_vec();
    output.reverse();
    Ok(output)
});

// Call via FFI
let result = rrpc_call(/* ... */);
```

---

### 2. Calculator (Coming in v0.2)

**What it does:** Type-safe math operations using MSL schemas.

**Features:**
- Add, subtract, multiply, divide
- Strong typing with schema validation
- Error handling for division by zero

**Schema (calculator.msl):**
```yaml
types:
  MathRequest:
    operation: enum[Add, Sub, Mul, Div]
    a: i32
    b: i32
    
  MathResponse:
    result: i32
```

**Usage:**
```fsharp
// F# client
let req = { Operation = Add; A = 10; B = 32 }
let resp = RRpc.call<MathResponse> "calculate" req
printfn "Result: %d" resp.Result  // 42
```

---

### 3. User Management (Coming in v0.2)

**What it does:** CRUD operations for user data with typed schemas.

**Features:**
- Create, read, update, delete users
- UUID-based identifiers
- JSON serialization
- In-memory storage

**Schema (user.msl):**
```yaml
types:
  User:
    id: uuid
    name: string
    email: string
    created: timestamp
    
  CreateUserRequest:
    name: string
    email: string
    
  GetUserRequest:
    id: uuid
```

**API:**
```rust
// Rust handlers
fn create_user(input: &[u8]) -> Result<Vec<u8>, RpcError>
fn get_user(input: &[u8]) -> Result<Vec<u8>, RpcError>
fn list_users(input: &[u8]) -> Result<Vec<u8>, RpcError>
fn delete_user(input: &[u8]) -> Result<Vec<u8>, RpcError>
```

---

### 4. WASM Web Demo (Coming in v0.2)

**What it does:** Browser-based rRPC running in WebAssembly.

**Features:**
- Same Rust code as desktop
- TypeScript/JavaScript client
- Real-time browser console

**Build:**
```powershell
cd examples/wasm-demo
wasm-pack build --target web
python -m http.server 8080
# Open http://localhost:8080
```

**Usage:**
```typescript
// JavaScript in browser
import init, { rrpc_init, rrpc_call } from './pkg/rrpc_wasm.js';

await init();
rrpc_init();

const input = new TextEncoder().encode("Hello from browser!");
const result = rrpc_call("echo", input);
console.log(new TextDecoder().decode(result));
```

---

### 5. F# Integration (Coming in v0.2)

**What it does:** Complete F# client library with P/Invoke wrappers.

**Features:**
- Automatic marshaling
- F#-friendly API
- Async/Task support
- Type-safe schemas

**Project structure:**
```
fsharp-client/
├── RRpc.Core.fsproj       # P/Invoke bindings
├── RRpc.fs                # High-level API
└── Example.fsx            # Demo script
```

**Usage:**
```fsharp
#r "nuget: RRpc.FSharp"
open RRpc

// Initialize
RRpc.init()

// Type-safe call
let user = RRpc.call<User> "get_user" { Id = userId }
printfn "User: %s (%s)" user.Name user.Email

// Async variant
let! users = RRpc.callAsync<User list> "list_users" ()
users |> List.iter (printfn "%A")
```

---

### 6. Performance Benchmark (Coming in v0.3)

**What it does:** Compares rRPC vs gRPC vs tRPC latency and throughput.

**Metrics:**
- Latency (p50, p95, p99)
- Throughput (calls/sec)
- Memory usage
- CPU utilization

**Run:**
```powershell
cargo run --example bench --release
```

**Sample output:**
```
Benchmarking rRPC vs gRPC (localhost)
═══════════════════════════════════════════════
rRPC:
  Latency (p50):  0.8μs
  Latency (p99):  2.1μs
  Throughput:     1,200,000 calls/sec
  
gRPC:
  Latency (p50):  45μs
  Latency (p99):  120μs
  Throughput:     9,500 calls/sec

rRPC is 56x faster (latency) and 126x higher throughput
```

---

## Running All Examples

```powershell
# Echo demo
cargo run --example demo

# (Future examples)
cargo run --example calculator
cargo run --example users
cargo run --example bench --release
```

---

## Creating Your Own Example

### Step 1: Create `examples/my_example.rs`

```rust
use rrpc_core::{Registry, RpcError};

fn my_function(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    // Your logic here
    Ok(input.to_vec())
}

fn main() {
    let mut registry = Registry::new();
    registry.register("my_function", my_function);
    
    // Test it
    let result = registry.call("my_function", b"test").unwrap();
    println!("Result: {:?}", result);
}
```

### Step 2: Run it

```powershell
cargo run --example my_example
```

---

## Example Project Template

Use this as a starting point for your own rRPC server:

```
my-rrpc-project/
├── Cargo.toml
├── src/
│   └── lib.rs              # FFI exports + registry
├── examples/
│   └── test_client.rs      # Rust test client
└── bindings/
    ├── fsharp/             # F# P/Invoke
    └── typescript/         # TS/WASM
```

**Cargo.toml:**
```toml
[package]
name = "my-rrpc-project"
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

// Your functions here
fn hello(_input: &[u8]) -> Result<Vec<u8>, RpcError> {
    Ok(b"Hello from rRPC!".to_vec())
}

#[no_mangle]
pub extern "C" fn rrpc_init() -> i32 {
    let mut registry = Registry::new();
    registry.register("hello", hello);
    // Register more functions...
    
    REGISTRY.set(registry).unwrap();
    0
}

// Standard rrpc_call and rrpc_free implementations
// (See core/src/lib.rs for reference)
```

---

## Tips for Writing Examples

1. **Keep it simple**: Focus on one concept per example
2. **Add comments**: Explain non-obvious code
3. **Include expected output**: Show what success looks like
4. **Handle errors**: Demonstrate error handling patterns
5. **Test edge cases**: Division by zero, empty input, etc.

---

## Contributing Examples

This is a solo research project. See [README.md](../README.md) for contribution policy.

---

## See Also

- [Getting Started Guide](../docs/getting-started.md)
- [API Reference](../docs/api-reference.md)
- [Architecture](../docs/ARCHITECTURE.md)
