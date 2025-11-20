# Schema-Driven Development Guide

Learn to use schemas for type-safe rRPC development.

## Overview

Schema-driven development means defining your data types and RPC contracts in a **schema file** first, then generating code for both Rust and client languages.

**Benefits:**
- ✅ Compile-time type safety across languages
- ✅ Automatic serialization/deserialization
- ✅ Versioning and backward compatibility
- ✅ Documentation as code
- ✅ Eliminates manual marshaling

## MSL (Mycelium Schema Language)

**Coming in v0.2**, rRPC will include a custom schema language optimized for FFI and local RPC.

### Why Not Protobuf?

While Protobuf is excellent, MSL offers:
- **Simpler syntax** (YAML-like, easier to read)
- **FFI-first design** (optimized for local calls, not network)
- **UDG integration** (built-in graph types, spatial data)
- **No external dependencies** (single Rust compiler)
- **Fine-grained control** (zero-copy hints, capability annotations)

### Basic Schema Example

**user.msl:**
```yaml
schema: mycelium/v1

# Type definitions
types:
  User:
    id: uuid
    name: string
    email: string
    created: timestamp
    roles: [string]
    metadata: map<string, any>
  
  CreateUserRequest:
    name: string
    email: string
  
  GetUserRequest:
    id: uuid
  
  UpdateUserRequest:
    id: uuid
    name: string?      # Optional field
    email: string?

# Function contracts
functions:
  create_user:
    input: CreateUserRequest
    output: User
    
  get_user:
    input: GetUserRequest
    output: User
    
  list_users:
    input: {}          # Empty input
    output: [User]     # Array of users
    
  update_user:
    input: UpdateUserRequest
    output: User
```

### Primitive Types

| MSL Type | Rust | F# | TypeScript |
|----------|------|----|-----------:|
| `bool` | `bool` | `bool` | `boolean` |
| `i32` | `i32` | `int32` | `number` |
| `i64` | `i64` | `int64` | `bigint` |
| `u32` | `u32` | `uint32` | `number` |
| `u64` | `u64` | `uint64` | `bigint` |
| `f32` | `f32` | `float32` | `number` |
| `f64` | `f64` | `float` | `number` |
| `string` | `String` | `string` | `string` |
| `bytes` | `Vec<u8>` | `byte[]` | `Uint8Array` |
| `uuid` | `Uuid` | `Guid` | `string` |
| `timestamp` | `DateTime<Utc>` | `DateTimeOffset` | `Date` |

### Complex Types

**Arrays:**
```yaml
tags: [string]           # Vec<String>, string list, string[]
numbers: [i32]
users: [User]
```

**Maps:**
```yaml
settings: map<string, string>
counters: map<string, i32>
metadata: map<string, any>
```

**Optionals:**
```yaml
middle_name: string?     # Option<String>, string option, string | undefined
age: i32?
```

**Enums:**
```yaml
types:
  Status:
    enum: [Pending, Active, Suspended, Deleted]
  
  Role:
    enum: [Admin, User, Guest]

  # Usage
  User:
    status: Status
    role: Role
```

### Generated Code

#### Rust

**user.rs (generated):**
```rust
// Auto-generated from user.msl

use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created: DateTime<Utc>,
    pub roles: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserRequest {
    pub id: Uuid,
}

// Codecs
impl User {
    pub fn encode(&self) -> Result<Vec<u8>, EncodeError> {
        serde_json::to_vec(self).map_err(EncodeError::from)
    }
    
    pub fn decode(bytes: &[u8]) -> Result<Self, DecodeError> {
        serde_json::from_slice(bytes).map_err(DecodeError::from)
    }
}

// Handler registration helpers
pub fn register_user_handlers(registry: &mut Registry) {
    registry.register("create_user", create_user_handler);
    registry.register("get_user", get_user_handler);
    registry.register("list_users", list_users_handler);
}

fn create_user_handler(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let request = CreateUserRequest::decode(input)?;
    let user = create_user_impl(request)?;
    user.encode().map_err(Into::into)
}

// You implement the actual logic:
fn create_user_impl(req: CreateUserRequest) -> Result<User, RpcError> {
    // Your business logic here
}
```

#### F#

**User.fs (generated):**
```fsharp
// Auto-generated from user.msl

namespace Generated

open System
open System.Collections.Generic

type User = {
    Id: Guid
    Name: string
    Email: string
    Created: DateTimeOffset
    Roles: string list
    Metadata: Map<string, obj>
}

type CreateUserRequest = {
    Name: string
    Email: string
}

type GetUserRequest = {
    Id: Guid
}

module UserCodec =
    open System.Text.Json
    
    let encodeUser (user: User) : byte[] =
        JsonSerializer.SerializeToUtf8Bytes(user)
    
    let decodeUser (bytes: byte[]) : User =
        JsonSerializer.Deserialize<User>(bytes)

module UserRpc =
    let createUser (req: CreateUserRequest) : User =
        let input = JsonSerializer.SerializeToUtf8Bytes(req)
        let output = RRpc.call "create_user" input
        JsonSerializer.Deserialize<User>(output)
    
    let getUser (req: GetUserRequest) : User =
        let input = JsonSerializer.SerializeToUtf8Bytes(req)
        let output = RRpc.call "get_user" input
        JsonSerializer.Deserialize<User>(output)
    
    let listUsers () : User list =
        let output = RRpc.call "list_users" [||]
        JsonSerializer.Deserialize<User list>(output)
```

#### TypeScript

**user.ts (generated):**
```typescript
// Auto-generated from user.msl

export interface User {
    id: string;        // UUID as string
    name: string;
    email: string;
    created: Date;
    roles: string[];
    metadata: Record<string, any>;
}

export interface CreateUserRequest {
    name: string;
    email: string;
}

export interface GetUserRequest {
    id: string;
}

// Codecs
export class UserCodec {
    static encodeUser(user: User): Uint8Array {
        const json = JSON.stringify(user);
        return new TextEncoder().encode(json);
    }
    
    static decodeUser(bytes: Uint8Array): User {
        const json = new TextDecoder().decode(bytes);
        return JSON.parse(json);
    }
}

// RPC client
export class UserRpc {
    constructor(private client: RRpcClient) {}
    
    async createUser(req: CreateUserRequest): Promise<User> {
        const input = UserCodec.encodeCreateUserRequest(req);
        const output = await this.client.call('create_user', input);
        return UserCodec.decodeUser(output);
    }
    
    async getUser(req: GetUserRequest): Promise<User> {
        const input = UserCodec.encodeGetUserRequest(req);
        const output = await this.client.call('get_user', input);
        return UserCodec.decodeUser(output);
    }
    
    async listUsers(): Promise<User[]> {
        const output = await this.client.call('list_users', new Uint8Array());
        return UserCodec.decodeUserList(output);
    }
}
```

## Advanced Features (v0.3+)

### UDG-Specific Types

```yaml
types:
  Node:
    id: uuid
    type: NodeType
    position: spatial3d
    relationships: [Relationship]
  
  Spatial3D:
    x: f32
    y: f32
    z: f32
  
  Relationship:
    from: uuid
    to: uuid
    type: enum[Parent, Child, Reference, Dependency]
    weight: f32
```

### Capability Annotations

```yaml
types:
  TerminateProcessCommand:
    requires: capability[process.terminate]
    pid: u32
  
  DeleteFileCommand:
    requires: capability[file.delete]
    path: string

functions:
  terminate_process:
    requires: capability[process.terminate]
    input: TerminateProcessCommand
    output: bool
```

**Generated F#:**
```fsharp
// Compile-time capability check
let terminateProcess (cmd: TerminateProcessCommand) (cap: Capability<ProcessTerminate>) =
    // Can only call if you have the capability token
    RRpc.call "terminate_process" cmd
```

### Versioning

```yaml
schema: mycelium/v2  # Version bumped

types:
  User:
    id: uuid
    name: string
    email: string
    avatar: string | since=v2      # New field in v2
    settings: UserSettings | since=v2
```

**Backward compatibility:**
```rust
// v1 client can still decode v2 responses
// (unknown fields ignored)

// v2 client can decode v1 responses
// (new fields set to None/default)
```

### Zero-Copy Hints

```yaml
types:
  LargeBuffer:
    data: bytes
    encoding: zero_copy  # Don't copy, borrow slice
```

**Generated Rust:**
```rust
pub struct LargeBuffer<'a> {
    pub data: &'a [u8],  // Borrowed, not owned
}
```

## Using the Schema Compiler (v0.2+)

### Installation

```powershell
cargo install msl-compiler
```

### Compile Schema

```powershell
# Generate all targets
msl-compiler compile schema/user.msl --out src/

# Output:
#   src/rust/user.rs
#   src/fsharp/User.fs
#   src/typescript/user.ts
```

### Watch Mode

```powershell
msl-compiler watch schema/ --out src/
# Recompiles on schema file changes
```

### CI Integration

**.github/workflows/schema.yml:**
```yaml
name: Schema Codegen

on:
  push:
    paths:
      - 'schema/**/*.msl'

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: cargo install msl-compiler
      - run: msl-compiler compile schema/ --out src/
      - run: git diff --exit-code  # Fail if generated code out of sync
```

## Best Practices

### 1. Keep Schemas Simple

```yaml
# ✅ Good - clear, focused types
types:
  User:
    id: uuid
    name: string
    email: string

# ❌ Bad - kitchen sink type
types:
  User:
    # 50+ fields...
```

### 2. Use Semantic Versioning

```yaml
schema: mycelium/v1.0.0

# Breaking changes → major version
schema: mycelium/v2.0.0

# New optional fields → minor version
schema: mycelium/v1.1.0
```

### 3. Document Types

```yaml
types:
  # Represents a user account in the system.
  # Users can have multiple roles and custom metadata.
  User:
    id: uuid           # Unique identifier
    name: string       # Display name
    email: string      # Primary email (must be unique)
    roles: [string]    # List of assigned role names
```

### 4. Validate at Schema Level

```yaml
types:
  Email:
    value: string
    constraints:
      pattern: "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
  
  Age:
    value: i32
    constraints:
      min: 0
      max: 150
```

## Migration from Manual Marshaling

### Before (Manual)

**Rust:**
```rust
fn get_user(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    // Manual parsing
    let id = Uuid::from_slice(&input[0..16])?;
    
    // Fetch user
    let user = fetch_user(id)?;
    
    // Manual encoding
    let mut output = Vec::new();
    output.extend_from_slice(user.id.as_bytes());
    output.extend_from_slice(user.name.as_bytes());
    // ... 20 more lines ...
    Ok(output)
}
```

### After (Schema-Driven)

**schema/user.msl:**
```yaml
types:
  GetUserRequest:
    id: uuid
  User:
    id: uuid
    name: string
    email: string
```

**Rust:**
```rust
fn get_user_impl(req: GetUserRequest) -> Result<User, RpcError> {
    fetch_user(req.id)  // That's it!
}
```

All marshaling is auto-generated!

## Temporary: JSON Until MSL Ready

For now (v0.1), use JSON as your schema:

**schemas/user.json:**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "User": {
      "type": "object",
      "required": ["id", "name", "email"],
      "properties": {
        "id": { "type": "string", "format": "uuid" },
        "name": { "type": "string" },
        "email": { "type": "string", "format": "email" }
      }
    }
  }
}
```

Use `serde_json` in Rust and `System.Text.Json` in F#.

## See Also

- [API Reference](api-reference.md) - Manual marshaling patterns
- [Error Handling](error-handling.md) - Schema error types
- [MSL Spec](https://github.com/Imnsol/msl-spec) - Full language specification (v0.2+)
