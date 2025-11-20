# Error Handling Best Practices

Comprehensive guide to handling errors in rRPC applications.

## Error Types

### Core RPC Errors

```rust
#[derive(Debug)]
pub enum RpcError {
    UnknownMethod(String),
    DecodeFailed(String),
    EncodeFailed(String),
    ExecutionFailed(String),
}
```

### When Each Error Occurs

**UnknownMethod**
- Method name not registered in registry
- Typo in method name
- Client/server version mismatch

**DecodeFailed**
- Invalid input format
- Deserialization error
- Schema mismatch

**EncodeFailed**
- Output serialization failed
- Type doesn't implement encoder
- Memory allocation failure

**ExecutionFailed**
- Handler threw an exception
- Business logic error
- Resource unavailable (DB, file, etc.)

## Error Handling Patterns

### 1. Early Validation (Rust)

```rust
fn create_user(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    // Validate input size
    if input.len() < 16 {
        return Err(RpcError::DecodeFailed(
            format!("Expected at least 16 bytes, got {}", input.len())
        ));
    }
    
    // Decode
    let request = CreateUserRequest::decode(input)
        .map_err(|e| RpcError::DecodeFailed(e.to_string()))?;
    
    // Validate business rules
    if request.name.is_empty() {
        return Err(RpcError::ExecutionFailed("Name cannot be empty".into()));
    }
    
    if !request.email.contains('@') {
        return Err(RpcError::ExecutionFailed("Invalid email format".into()));
    }
    
    // Execute
    let user = create_user_impl(request)?;
    
    // Encode
    user.encode()
        .map_err(|e| RpcError::EncodeFailed(e.to_string()))
}
```

### 2. Result Propagation

```rust
fn get_user_with_posts(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let req = GetUserRequest::decode(input)
        .map_err(|e| RpcError::DecodeFailed(e.to_string()))?;
    
    // ? operator propagates errors automatically
    let user = fetch_user(req.id)?;
    let posts = fetch_user_posts(req.id)?;
    
    let response = UserWithPosts { user, posts };
    response.encode()
        .map_err(|e| RpcError::EncodeFailed(e.to_string()))
}
```

### 3. Error Context

```rust
use anyhow::Context;

fn get_user_detailed(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let req = GetUserRequest::decode(input)
        .context("Failed to decode GetUserRequest")
        .map_err(|e| RpcError::DecodeFailed(e.to_string()))?;
    
    let user = fetch_user(req.id)
        .context(format!("User not found: {}", req.id))
        .map_err(|e| RpcError::ExecutionFailed(e.to_string()))?;
    
    user.encode()
        .context("Failed to encode user")
        .map_err(|e| RpcError::EncodeFailed(e.to_string()))
}
```

### 4. Custom Domain Errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("User not found: {0}")]
    NotFound(Uuid),
    
    #[error("Email already exists: {0}")]
    DuplicateEmail(String),
    
    #[error("Invalid role: {0}")]
    InvalidRole(String),
    
    #[error("Permission denied")]
    PermissionDenied,
}

impl From<UserError> for RpcError {
    fn from(err: UserError) -> Self {
        RpcError::ExecutionFailed(err.to_string())
    }
}

fn create_user_impl(req: CreateUserRequest) -> Result<User, UserError> {
    if email_exists(&req.email) {
        return Err(UserError::DuplicateEmail(req.email));
    }
    // ...
}

// Automatically converts UserError -> RpcError
fn create_user(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let req = CreateUserRequest::decode(input)?;
    let user = create_user_impl(req)?;  // UserError auto-converted
    Ok(user.encode()?)
}
```

## Client-Side Error Handling

### F# Result-Based API

```fsharp
type RpcResult<'T> =
    | Ok of 'T
    | Error of string

module RRpcSafe =
    let tryCall (method: string) (input: byte[]) : RpcResult<byte[]> =
        try
            let output = RRpc.call method input
            Ok output
        with
        | :? DllNotFoundException as ex ->
            Error $"rRPC library not found: {ex.Message}"
        | :? AccessViolationException as ex ->
            Error $"Memory access error (possible invalid pointer): {ex.Message}"
        | ex ->
            Error $"RPC call failed: {ex.Message}"

// Usage
match RRpcSafe.tryCall "get_user" input with
| Ok output ->
    let user = UserCodec.decode output
    printfn "User: %s" user.Name
| Error msg ->
    eprintfn "Error: %s" msg
```

### F# Railway-Oriented Programming

```fsharp
module Result =
    let bind f result =
        match result with
        | Ok x -> f x
        | Error e -> Error e
    
    let map f result =
        match result with
        | Ok x -> Ok (f x)
        | Error e -> Error e

let getUserWorkflow userId =
    RRpcSafe.tryCall "get_user" (encodeUuid userId)
    |> Result.bind (fun bytes ->
        try
            Ok (UserCodec.decode bytes)
        with ex ->
            Error $"Decode failed: {ex.Message}"
    )
    |> Result.bind (fun user ->
        if user.IsActive then
            Ok user
        else
            Error "User is not active"
    )
```

### TypeScript Error Classes

```typescript
export class RpcError extends Error {
    constructor(
        message: string,
        public readonly method: string,
        public readonly code?: number
    ) {
        super(message);
        this.name = 'RpcError';
    }
}

export class RpcClient {
    call(method: string, input: Uint8Array): Uint8Array {
        const result = rrpc_call(method, input);
        
        if (result.length === 0) {
            throw new RpcError(
                `Call to ${method} returned empty result`,
                method
            );
        }
        
        return result;
    }
    
    async callSafe(method: string, input: Uint8Array): Promise<Uint8Array> {
        try {
            return this.call(method, input);
        } catch (err) {
            if (err instanceof RpcError) {
                console.error(`RPC error calling ${err.method}:`, err.message);
            }
            throw err;
        }
    }
}

// Usage with error boundaries
try {
    const user = await client.callJson<User>('get_user', { id: userId });
    console.log(user.name);
} catch (err) {
    if (err instanceof RpcError) {
        showErrorToast(err.message);
    } else {
        throw err;  // Unexpected error
    }
}
```

## Structured Error Responses

### Error Code System

**Rust:**
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: i32,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

pub const ERR_UNKNOWN_METHOD: i32 = 1001;
pub const ERR_DECODE_FAILED: i32 = 1002;
pub const ERR_ENCODE_FAILED: i32 = 1003;
pub const ERR_NOT_FOUND: i32 = 2001;
pub const ERR_DUPLICATE: i32 = 2002;
pub const ERR_PERMISSION_DENIED: i32 = 2003;

fn encode_error(code: i32, message: String, details: Option<serde_json::Value>) -> Vec<u8> {
    let error = ErrorResponse { code, message, details };
    serde_json::to_vec(&error).unwrap_or_else(|_| {
        // Fallback if encoding error fails
        b"{\"code\":9999,\"message\":\"Failed to encode error\"}".to_vec()
    })
}

fn create_user(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let req = match CreateUserRequest::decode(input) {
        Ok(r) => r,
        Err(e) => return Ok(encode_error(
            ERR_DECODE_FAILED,
            e.to_string(),
            Some(json!({ "input_length": input.len() }))
        )),
    };
    
    match create_user_impl(req) {
        Ok(user) => user.encode().map_err(Into::into),
        Err(UserError::DuplicateEmail(email)) => Ok(encode_error(
            ERR_DUPLICATE,
            "Email already exists".into(),
            Some(json!({ "email": email }))
        )),
        Err(e) => Ok(encode_error(
            ERR_UNKNOWN,
            e.to_string(),
            None
        )),
    }
}
```

**F# Client:**
```fsharp
type ErrorResponse = {
    Code: int
    Message: string
    Details: Map<string, obj> option
}

type RpcResponse<'T> =
    | Success of 'T
    | ErrorResponse of ErrorResponse

let call<'T> (method: string) (input: byte[]) : RpcResponse<'T> =
    let output = RRpc.call method input
    
    // Try to decode as error first
    try
        let error = JsonSerializer.Deserialize<ErrorResponse>(output)
        if error.Code > 0 then
            ErrorResponse error
        else
            raise (InvalidOperationException())
    with _ ->
        // Not an error, decode as success
        let result = JsonSerializer.Deserialize<'T>(output)
        Success result

// Usage
match call<User> "get_user" input with
| Success user ->
    printfn "User: %s" user.Name
| ErrorResponse err ->
    match err.Code with
    | 2001 -> printfn "User not found"
    | 2003 -> printfn "Permission denied"
    | _ -> printfn "Error %d: %s" err.Code err.Message
```

## Logging and Monitoring

### Rust Logging

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(input))]
fn get_user(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    info!("get_user called with {} bytes", input.len());
    
    let req = match GetUserRequest::decode(input) {
        Ok(r) => {
            info!("Decoded request for user {}", r.id);
            r
        }
        Err(e) => {
            warn!("Failed to decode request: {}", e);
            return Err(RpcError::DecodeFailed(e.to_string()));
        }
    };
    
    match fetch_user(req.id) {
        Ok(user) => {
            info!("Successfully fetched user {}", user.id);
            user.encode().map_err(Into::into)
        }
        Err(e) => {
            error!("Failed to fetch user {}: {}", req.id, e);
            Err(RpcError::ExecutionFailed(e.to_string()))
        }
    }
}
```

### Centralized Error Logging

```rust
use std::sync::Mutex;
use chrono::Utc;

static ERROR_LOG: Mutex<Vec<ErrorLogEntry>> = Mutex::new(Vec::new());

struct ErrorLogEntry {
    timestamp: DateTime<Utc>,
    method: String,
    error: String,
    context: HashMap<String, String>,
}

fn log_error(method: &str, error: &RpcError, context: HashMap<String, String>) {
    let entry = ErrorLogEntry {
        timestamp: Utc::now(),
        method: method.to_string(),
        error: format!("{:?}", error),
        context,
    };
    
    ERROR_LOG.lock().unwrap().push(entry);
}

// In rrpc_call wrapper
pub extern "C" fn rrpc_call(/* ... */) -> i32 {
    let result = registry.call(method, input);
    
    match result {
        Ok(_) => { /* success */ }
        Err(e) => {
            log_error(method, &e, HashMap::from([
                ("input_len".into(), input.len().to_string()),
            ]));
        }
    }
}
```

## Recovery Strategies

### 1. Retry with Backoff

```fsharp
let rec retryCall maxAttempts delay method input =
    async {
        try
            return RRpc.call method input |> Ok
        with ex when maxAttempts > 1 ->
            do! Async.Sleep delay
            return! retryCall (maxAttempts - 1) (delay * 2) method input
    }

// Usage
let! result = retryCall 3 100 "flaky_operation" input
```

### 2. Circuit Breaker

```fsharp
type CircuitState =
    | Closed
    | Open of DateTime
    | HalfOpen

type CircuitBreaker(threshold: int, timeout: TimeSpan) =
    let mutable state = Closed
    let mutable failures = 0
    
    member this.Call(method: string, input: byte[]) =
        match state with
        | Open openTime when DateTime.UtcNow - openTime < timeout ->
            Error "Circuit breaker is open"
        | Open _ ->
            state <- HalfOpen
            this.TryCall(method, input)
        | _ ->
            this.TryCall(method, input)
    
    member private this.TryCall(method, input) =
        try
            let result = RRpc.call method input
            failures <- 0
            state <- Closed
            Ok result
        with ex ->
            failures <- failures + 1
            if failures >= threshold then
                state <- Open DateTime.UtcNow
            Error ex.Message
```

### 3. Fallback Values

```fsharp
let getUserOrDefault userId =
    try
        let input = encodeUuid userId
        let output = RRpc.call "get_user" input
        UserCodec.decode output
    with _ ->
        // Return default/cached user
        { Id = userId; Name = "Unknown"; Email = "" }
```

## Testing Error Scenarios

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_decode_error() {
        let invalid_input = vec![0xFF; 10];  // Invalid data
        let result = get_user(&invalid_input);
        
        assert!(matches!(result, Err(RpcError::DecodeFailed(_))));
    }
    
    #[test]
    fn test_not_found_error() {
        let req = GetUserRequest {
            id: Uuid::nil(),  // Non-existent user
        };
        let input = req.encode().unwrap();
        let result = get_user(&input);
        
        assert!(matches!(result, Err(RpcError::ExecutionFailed(_))));
    }
    
    #[test]
    fn test_error_message_format() {
        let error = RpcError::UnknownMethod("missing_fn".into());
        let message = format!("{:?}", error);
        assert!(message.contains("missing_fn"));
    }
}
```

### Integration Tests

```fsharp
[<Test>]
let ``calling unknown method returns error`` () =
    RRpc.init()
    
    let result = RRpcSafe.tryCall "non_existent_method" [||]
    
    match result with
    | Error msg ->
        Assert.IsTrue(msg.Contains("Unknown method"))
    | Ok _ ->
        Assert.Fail("Expected error")
```

## Best Practices

1. **Always validate input early**: Fail fast with clear messages
2. **Use typed errors**: Domain-specific error types are clearer than strings
3. **Log errors with context**: Include method name, input size, timestamp
4. **Return structured errors**: Use error codes and details for programmatic handling
5. **Document error conditions**: Specify which errors each function can return
6. **Test error paths**: Unit test failure scenarios
7. **Graceful degradation**: Provide fallbacks where possible
8. **Monitor error rates**: Alert on unusual error patterns

## See Also

- [API Reference](api-reference.md) - Error types
- [Getting Started](getting-started.md) - Common issues
- [Schema Guide](schema-guide.md) - Schema validation errors
