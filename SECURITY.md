# Security Policy

## Security Model

**rRPC is designed for trusted, performance-critical code only.**

rRPC provides a high-performance FFI (Foreign Function Interface) bridge between .NET and Rust with sub-microsecond latency. This performance comes at the cost of process isolation and sandboxing capabilities.

### Threat Model

**rRPC is suitable for:**
- Desktop applications where you control all code
- Performance-critical components in trusted environments
- Internal tool development
- Applications where process crash is an acceptable failure mode

**rRPC is NOT suitable for:**
- Executing untrusted user code
- Plugin systems with third-party code
- Multi-tenant environments
- Security boundaries between components
- Any scenario requiring code sandboxing

## Fundamental Security Limitations

These are **architectural limitations** that cannot be fixed without fundamentally changing rRPC's design:

### 1. No Memory Isolation

rRPC runs F#/.NET and Rust code in the **same process with shared memory space**.

**Implications:**
- A bug in Rust can corrupt .NET memory
- A bug in .NET marshaling can corrupt Rust memory
- Buffer overflows can affect the entire process
- No protection against use-after-free across the FFI boundary

**Mitigation:** Thoroughly test all FFI boundaries and validate all inputs.

### 2. No Privilege Separation

All rRPC handlers execute with **full process privileges**.

**Implications:**
- Rust handlers can access any file the process can access
- Handlers can make network connections
- Handlers can spawn processes
- No capability-based security enforcement (yet)

**Mitigation:** Treat Rust handlers as first-class application code, not plugins.

### 3. Panic = Process Crash

Rust panics **will crash the entire application**, including the .NET host.

**Implications:**
- Index out of bounds in Rust → entire app dies
- Assertion failures → entire app dies
- No graceful recovery from Rust panics

**Mitigation:** 
```rust
// Use Result instead of panic
fn handler(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let value = input.get(index)
        .ok_or(RpcError::DecodeFailed("Index out of bounds".into()))?;
    // ...
}
```

### 4. WASM: Shared Memory Space

When compiled to WebAssembly, rRPC's linear memory is **fully accessible to JavaScript**.

**Implications:**
- JavaScript can read all WASM memory
- Secret keys in Rust memory can be scanned
- Side-channel attacks (timing, Spectre-like) are possible
- No memory protection between WASM and JavaScript

**Mitigation:** Never store secrets in WASM memory. Use Web Crypto API for sensitive operations.

### 5. Supply Chain Attacks

rRPC and its dependencies execute at **build time and runtime** with full privileges.

**Implications:**
- Malicious dependencies can inject code
- Compromised crates can steal data
- Build scripts (`build.rs`) run with full system access
- Transitive dependencies create large attack surface

**Mitigation:** 
- Audit dependencies regularly with `cargo audit`
- Pin dependency versions
- Review `build.rs` files
- Use `cargo-vet` for supply chain verification

### 6. Type Safety Holes

FFI passes **raw byte arrays** with no compile-time type checking.

**Implications:**
- Client can send malformed data
- Deserialization vulnerabilities possible
- Type confusion attacks
- Schema mismatches cause undefined behavior

**Mitigation:** Use schema-driven code generation (v0.2+) and validate all inputs.

### 7. No DoS Protection

Rate limiting is **advisory only** and can be bypassed.

**Implications:**
- Malicious client can exhaust process resources
- Thread pool starvation
- Memory exhaustion
- CPU saturation

**Mitigation:** Run in resource-limited containers/VMs. Implement application-level rate limiting.

## Fixable Security Issues

These issues can be mitigated through proper implementation:

### Input Validation

**Problem:** Unbounded input sizes can cause memory exhaustion.

**Solution:**
```rust
const MAX_INPUT_LEN: usize = 10 * 1024 * 1024; // 10 MB

pub extern "C" fn rrpc_call(/* ... */, in_len: usize, /* ... */) -> i32 {
    if in_len > MAX_INPUT_LEN {
        return ERR_INPUT_TOO_LARGE;
    }
    // ...
}
```

### Null Pointer Validation

**Problem:** Unchecked null pointers cause crashes.

**Solution:**
```rust
pub extern "C" fn rrpc_call(
    method_ptr: *const c_char,
    /* ... */
) -> i32 {
    if method_ptr.is_null() {
        return ERR_NULL_POINTER;
    }
    // ...
}
```

### Error Information Leakage

**Problem:** Detailed error messages expose internal implementation.

**Solution:**
```rust
impl RpcError {
    pub fn to_client_message(&self) -> String {
        match self {
            RpcError::UnknownMethod(_) => "Method not found".into(),
            RpcError::DecodeFailed(_) => "Invalid input".into(),
            // Never expose internal details
            _ => "Internal error".into(),
        }
    }
}
```

### Audit Logging

**Problem:** No visibility into who called what.

**Solution:**
```rust
fn log_call(method: &str, input_len: usize, result: &Result<Vec<u8>, RpcError>) {
    info!(
        method = method,
        input_len = input_len,
        success = result.is_ok(),
        timestamp = Utc::now().to_rfc3339(),
    );
}
```

## Best Practices

### 1. Validate All Inputs

```rust
fn create_user(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    // Validate size
    if input.len() > 1024 {
        return Err(RpcError::DecodeFailed("Input too large".into()));
    }
    
    // Validate format
    let req = CreateUserRequest::decode(input)
        .map_err(|e| RpcError::DecodeFailed(e.to_string()))?;
    
    // Validate business rules
    if req.name.is_empty() || req.name.len() > 100 {
        return Err(RpcError::ExecutionFailed("Invalid name".into()));
    }
    
    // Safe to proceed
    create_user_impl(req)
}
```

### 2. Use Result Instead of Panic

```rust
// ❌ Bad - will crash process
fn divide(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let a = decode_i32(&input[0..4]);
    let b = decode_i32(&input[4..8]);
    let result = a / b;  // Panics if b == 0
    Ok(encode_i32(result))
}

// ✅ Good - returns error
fn divide(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let a = decode_i32(&input[0..4]);
    let b = decode_i32(&input[4..8]);
    if b == 0 {
        return Err(RpcError::ExecutionFailed("Division by zero".into()));
    }
    Ok(encode_i32(a / b))
}
```

### 3. Sanitize Error Messages

```rust
// ❌ Bad - leaks internal paths
return Err(RpcError::ExecutionFailed(
    format!("Failed to open /internal/secret/path/config.toml: {}", e)
));

// ✅ Good - generic message
return Err(RpcError::ExecutionFailed("Configuration error".into()));
```

### 4. Use Least Privilege

```rust
// ❌ Bad - handler can do anything
fn read_file(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let path = String::from_utf8(input.to_vec())?;
    std::fs::read(path)  // Can read ANY file
        .map_err(|e| RpcError::ExecutionFailed(e.to_string()))
}

// ✅ Good - restricted to safe directory
fn read_file(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let filename = String::from_utf8(input.to_vec())?;
    
    // Only allow files in data directory
    let safe_path = Path::new("/app/data").join(&filename);
    if !safe_path.starts_with("/app/data") {
        return Err(RpcError::ExecutionFailed("Access denied".into()));
    }
    
    std::fs::read(safe_path)
        .map_err(|_| RpcError::ExecutionFailed("File not found".into()))
}
```

### 5. Audit Critical Operations

```rust
fn delete_user(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let req = DeleteUserRequest::decode(input)?;
    
    // Log before destructive operation
    warn!(
        action = "delete_user",
        user_id = %req.id,
        timestamp = %Utc::now().to_rfc3339(),
    );
    
    delete_user_impl(req)?;
    
    // Log success
    info!(action = "delete_user", user_id = %req.id, status = "success");
    
    Ok(vec![])
}
```

### 6. Set Resource Limits

```rust
use std::time::{Duration, Instant};

fn expensive_operation(input: &[u8]) -> Result<Vec<u8>, RpcError> {
    let start = Instant::now();
    let max_duration = Duration::from_secs(5);
    
    for item in items {
        if start.elapsed() > max_duration {
            return Err(RpcError::ExecutionFailed("Operation timeout".into()));
        }
        process(item);
    }
    
    Ok(result)
}
```

## Deployment Recommendations

### For Production Use

1. **Run in isolated containers**
   ```yaml
   # Docker: Limit resources
   services:
     app:
       image: myapp
       deploy:
         resources:
           limits:
             cpus: '2'
             memory: 1G
   ```

2. **Use read-only file systems where possible**
   ```bash
   docker run --read-only --tmpfs /tmp myapp
   ```

3. **Drop unnecessary capabilities**
   ```bash
   docker run --cap-drop=ALL --cap-add=NET_BIND_SERVICE myapp
   ```

4. **Enable seccomp filtering (Linux)**
   ```json
   {
     "defaultAction": "SCMP_ACT_ERRNO",
     "syscalls": [
       { "name": "read", "action": "SCMP_ACT_ALLOW" },
       { "name": "write", "action": "SCMP_ACT_ALLOW" }
     ]
   }
   ```

5. **Monitor and alert on anomalies**
   - Unexpected crashes
   - High error rates
   - Unusual resource consumption
   - Calls to unexpected methods

## Vulnerability Reporting

If you discover a security vulnerability in rRPC, please report it responsibly.

**DO NOT open a public GitHub issue.**

### Reporting Process

1. Email security details to: [your-email@example.com]
2. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)
3. Allow 90 days for fix before public disclosure

### Response Timeline

- **24 hours:** Acknowledgment of report
- **7 days:** Initial assessment and severity classification
- **30 days:** Fix developed and tested (for critical issues)
- **90 days:** Public disclosure (if applicable)

## Security Advisories

Security advisories will be published at:
- GitHub Security Advisories: https://github.com/Imnsol/rRPC/security/advisories
- Project releases page

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Security Features Roadmap

### v0.2 (Planned)
- [ ] Input size validation
- [ ] Null pointer checks
- [ ] Basic rate limiting
- [ ] Audit logging framework
- [ ] Schema validation (MSL)

### v0.3 (Planned)
- [ ] Capability-based security (compile-time)
- [ ] Handler timeout enforcement
- [ ] Memory usage limits per call
- [ ] Sanitized error messages

### v0.4 (Planned)
- [ ] Runtime capability enforcement
- [ ] Encrypted RPC payloads (optional)
- [ ] Handler signing/verification
- [ ] Advanced audit logging

### Future
- [ ] Process isolation mode (sacrifices performance)
- [ ] WASM sandboxing improvements
- [ ] Formal security audit

## Additional Resources

- [OWASP FFI Security](https://owasp.org/www-community/vulnerabilities/Unsafe_use_of_FFI)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [Memory Safety in Rust](https://doc.rust-lang.org/nomicon/)
- [.NET P/Invoke Best Practices](https://docs.microsoft.com/en-us/dotnet/standard/native-interop/best-practices)

## Acknowledgments

We appreciate responsible disclosure and will acknowledge security researchers who help improve rRPC's security.

---

**Remember:** rRPC prioritizes performance over isolation. If you need strong security boundaries, consider using gRPC or similar network-based RPC instead.
