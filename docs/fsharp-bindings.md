# F# Bindings Guide

Complete guide to using rRPC from F# via P/Invoke.

## Overview

F# can call Rust rRPC functions using P/Invoke (Platform Invoke), .NET's FFI mechanism for calling native libraries.

**Architecture:**
```
F# Application
    ↓ P/Invoke
Native Library (myrrpc.dll / .so / .dylib)
    ↓ C ABI
Rust rRPC Core
```

## Quick Start

### Step 1: Build the Rust Library

Ensure your Rust library is built as a `cdylib`:

**Cargo.toml:**
```toml
[lib]
crate-type = ["cdylib"]
```

Build it:
```powershell
cargo build --release
```

This produces:
- **Windows**: `target/release/myrrpc.dll`
- **Linux**: `target/release/libmyrrpc.so`
- **macOS**: `target/release/libmyrrpc.dylib`

### Step 2: Create F# Bindings

**RRpc.fs:**
```fsharp
module RRpc

open System
open System.Runtime.InteropServices
open System.Text

// P/Invoke declarations
[<DllImport("myrrpc", CallingConvention = CallingConvention.Cdecl)>]
extern int rrpc_init()

[<DllImport("myrrpc", CallingConvention = CallingConvention.Cdecl)>]
extern int rrpc_call(
    IntPtr method_ptr,
    UIntPtr method_len,
    IntPtr in_ptr,
    UIntPtr in_len,
    IntPtr& out_ptr,
    UIntPtr& out_len
)

[<DllImport("myrrpc", CallingConvention = CallingConvention.Cdecl)>]
extern void rrpc_free(IntPtr ptr, UIntPtr len)

// High-level F# API
let init () : unit =
    let result = rrpc_init()
    if result <> 0 then
        failwith "Failed to initialize rRPC"

let call (methodName: string) (input: byte[]) : byte[] =
    let methodBytes = Encoding.UTF8.GetBytes(methodName)
    
    let mutable outPtr = IntPtr.Zero
    let mutable outLen = UIntPtr.Zero
    
    // Pin arrays so GC doesn't move them
    use methodHandle = fixed methodBytes
    use inputHandle = fixed input
    
    let result = rrpc_call(
        NativePtr.toNativeInt methodHandle,
        UIntPtr(uint methodBytes.Length),
        NativePtr.toNativeInt inputHandle,
        UIntPtr(uint input.Length),
        &outPtr,
        &outLen
    )
    
    if result <> 0 then
        failwithf "RPC call failed: %s" methodName
    
    // Copy output before freeing
    let output = Array.zeroCreate (int outLen)
    Marshal.Copy(outPtr, output, 0, int outLen)
    
    // Free Rust-allocated memory
    rrpc_free(outPtr, outLen)
    
    output
```

### Step 3: Use in Your Application

**Program.fs:**
```fsharp
open System
open RRpc

[<EntryPoint>]
let main argv =
    // Initialize rRPC
    RRpc.init()
    
    // Call echo function
    let input = System.Text.Encoding.UTF8.GetBytes("Hello from F#!")
    let output = RRpc.call "echo" input
    let result = System.Text.Encoding.UTF8.GetString(output)
    
    printfn "Echo result: %s" result
    
    0
```

## Type-Safe Wrappers

### Basic Type Encoding

**Primitives:**
```fsharp
module Encoding =
    let encodeInt32 (value: int32) : byte[] =
        BitConverter.GetBytes(value)
    
    let decodeInt32 (bytes: byte[]) : int32 =
        BitConverter.ToInt32(bytes, 0)
    
    let encodeString (value: string) : byte[] =
        System.Text.Encoding.UTF8.GetBytes(value)
    
    let decodeString (bytes: byte[]) : string =
        System.Text.Encoding.UTF8.GetString(bytes)
```

**Usage:**
```fsharp
// Add two numbers
let a = 10
let b = 32
let input = 
    Array.concat [
        Encoding.encodeInt32 a
        Encoding.encodeInt32 b
    ]

let output = RRpc.call "add" input
let sum = Encoding.decodeInt32 output
printfn "%d + %d = %d" a b sum  // 10 + 32 = 42
```

### JSON Serialization

**Using System.Text.Json:**
```fsharp
open System.Text.Json

type User = {
    Id: Guid
    Name: string
    Email: string
}

module UserRpc =
    let getUser (userId: Guid) : User =
        let request = {| Id = userId |}
        let input = JsonSerializer.SerializeToUtf8Bytes(request)
        let output = RRpc.call "get_user" input
        JsonSerializer.Deserialize<User>(output)
    
    let createUser (name: string) (email: string) : User =
        let request = {| Name = name; Email = email |}
        let input = JsonSerializer.SerializeToUtf8Bytes(request)
        let output = RRpc.call "create_user" input
        JsonSerializer.Deserialize<User>(output)

// Usage
let user = UserRpc.createUser "Alice" "alice@example.com"
printfn "Created user: %s (%A)" user.Name user.Id
```

## Async Support

### Wrapping Calls in Async

```fsharp
module RRpcAsync =
    open System.Threading.Tasks
    
    let callAsync (methodName: string) (input: byte[]) : Async<byte[]> =
        async {
            // Run on thread pool to avoid blocking
            return! Task.Run(fun () -> RRpc.call methodName input)
                    |> Async.AwaitTask
        }
    
    let callAsyncTask (methodName: string) (input: byte[]) : Task<byte[]> =
        Task.Run(fun () -> RRpc.call methodName input)

// Usage
let getUserAsync (userId: Guid) : Async<User> =
    async {
        let request = {| Id = userId |}
        let input = JsonSerializer.SerializeToUtf8Bytes(request)
        let! output = RRpcAsync.callAsync "get_user" input
        return JsonSerializer.Deserialize<User>(output)
    }

// Call it
let user = getUserAsync (Guid.NewGuid()) |> Async.RunSynchronously
```

## Error Handling

### Basic Error Handling

```fsharp
type RpcResult<'T> =
    | Success of 'T
    | Error of string

module RRpcSafe =
    let tryCall (methodName: string) (input: byte[]) : RpcResult<byte[]> =
        try
            let output = RRpc.call methodName input
            Success output
        with
        | ex -> Error ex.Message

// Usage
match RRpcSafe.tryCall "get_user" input with
| Success output -> 
    let user = JsonSerializer.Deserialize<User>(output)
    printfn "User: %s" user.Name
| Error msg -> 
    printfn "Error: %s" msg
```

### Result-Based API

```fsharp
module RRpcResult =
    let call (methodName: string) (input: byte[]) : Result<byte[], string> =
        try
            RRpc.call methodName input |> Ok
        with
        | ex -> Error ex.Message
    
    let callAndDecode<'T> (methodName: string) (input: byte[]) : Result<'T, string> =
        result {
            let! output = call methodName input
            try
                return JsonSerializer.Deserialize<'T>(output)
            with
            | ex -> return! Error $"Deserialization failed: {ex.Message}"
        }

// Usage with computation expression
let getUser userId =
    result {
        let request = {| Id = userId |}
        let input = JsonSerializer.SerializeToUtf8Bytes(request)
        return! RRpcResult.callAndDecode<User> "get_user" input
    }
```

## Platform-Specific Considerations

### Windows (DLL)

**Finding the DLL:**
```fsharp
// Option 1: Place myrrpc.dll next to executable
// Option 2: Add to PATH
// Option 3: Specify full path in DllImport

[<DllImport(@"C:\path\to\myrrpc.dll", CallingConvention = CallingConvention.Cdecl)>]
extern int rrpc_init()
```

**Using SetDllDirectory:**
```fsharp
[<DllImport("kernel32.dll", CharSet = CharSet.Unicode, SetLastError = true)>]
extern bool SetDllDirectory(string lpPathName)

// At startup
SetDllDirectory(@"C:\path\to\native\libs") |> ignore
```

### Linux (SO)

```fsharp
[<DllImport("libmyrrpc.so", CallingConvention = CallingConvention.Cdecl)>]
extern int rrpc_init()
```

**LD_LIBRARY_PATH:**
```bash
export LD_LIBRARY_PATH=/path/to/libs:$LD_LIBRARY_PATH
dotnet run
```

### macOS (dylib)

```fsharp
[<DllImport("libmyrrpc.dylib", CallingConvention = CallingConvention.Cdecl)>]
extern int rrpc_init()
```

### Cross-Platform Solution

```fsharp
module Platform =
    let libraryName =
        if RuntimeInformation.IsOSPlatform(OSPlatform.Windows) then
            "myrrpc.dll"
        elif RuntimeInformation.IsOSPlatform(OSPlatform.Linux) then
            "libmyrrpc.so"
        elif RuntimeInformation.IsOSPlatform(OSPlatform.OSX) then
            "libmyrrpc.dylib"
        else
            failwith "Unsupported platform"

// Use NativeLibrary.Load for explicit loading
open System.Runtime.InteropServices

let loadLibrary () =
    let path = Path.Combine(AppContext.BaseDirectory, Platform.libraryName)
    NativeLibrary.Load(path)
```

## Memory Safety

### Common Pitfalls

**❌ Don't forget to free:**
```fsharp
// BAD - memory leak
let output = RRpc.call "echo" input
// ... use output ...
// (never called rrpc_free)
```

**✅ Always free:**
```fsharp
// GOOD
let call methodName input =
    let mutable outPtr = IntPtr.Zero
    let mutable outLen = UIntPtr.Zero
    
    try
        // ... call rrpc_call ...
        let output = Array.zeroCreate (int outLen)
        Marshal.Copy(outPtr, output, 0, int outLen)
        output
    finally
        if outPtr <> IntPtr.Zero then
            rrpc_free(outPtr, outLen)
```

### Using IDisposable

```fsharp
type RpcResponse(ptr: IntPtr, len: UIntPtr) =
    member _.Pointer = ptr
    member _.Length = int len
    member _.ToArray() =
        let arr = Array.zeroCreate (int len)
        Marshal.Copy(ptr, arr, 0, int len)
        arr
    
    interface IDisposable with
        member _.Dispose() =
            if ptr <> IntPtr.Zero then
                rrpc_free(ptr, len)

// Usage
use response = callRaw "echo" input
let output = response.ToArray()
// Automatically freed when scope exits
```

## Advanced Patterns

### Computation Expression for RPC

```fsharp
type RpcBuilder() =
    member _.Return(x) = Ok x
    member _.ReturnFrom(x) = x
    member _.Bind(result, f) =
        match result with
        | Ok x -> f x
        | Error e -> Error e
    member _.Zero() = Ok ()

let rpc = RpcBuilder()

let workflow userId =
    rpc {
        let! user = getUser userId
        let! posts = getUserPosts userId
        let! comments = getUserComments userId
        return {| User = user; Posts = posts; Comments = comments |}
    }
```

### Batching Calls

```fsharp
module RRpcBatch =
    let callMany (calls: (string * byte[]) list) : Result<byte[], string> list =
        calls
        |> List.map (fun (method, input) ->
            RRpcResult.call method input
        )
```

## Testing

### Unit Testing with NSubstitute

```fsharp
// Mock interface for testability
type IRRpcClient =
    abstract member Call: string -> byte[] -> byte[]

type RRpcClient() =
    interface IRRpcClient with
        member _.Call methodName input =
            RRpc.call methodName input

// In tests
[<Test>]
let ``getUserById returns user`` () =
    let mockClient = Substitute.For<IRRpcClient>()
    let expectedUser = {| Id = Guid.NewGuid(); Name = "Alice" |}
    let responseBytes = JsonSerializer.SerializeToUtf8Bytes(expectedUser)
    
    mockClient.Call("get_user", Arg.Any<byte[]>()).Returns(responseBytes)
    
    // Test your code that uses mockClient
```

## Performance Tips

1. **Reuse buffers**: Pool byte arrays to reduce allocations
2. **Avoid string encoding**: Work with byte[] directly when possible
3. **Batch calls**: Combine multiple requests if supported
4. **Use Span<byte>**: For .NET Core 3.0+ zero-copy scenarios

**Example with Span:**
```fsharp
let encodeInt32Span (value: int32) (buffer: Span<byte>) =
    BitConverter.TryWriteBytes(buffer, value)
```

## Troubleshooting

### DllNotFoundException

**Error:** `Unable to load DLL 'myrrpc': The specified module could not be found.`

**Solutions:**
1. Ensure DLL is in same directory as executable
2. Add DLL directory to PATH
3. Use full path in `DllImport`
4. Check DLL dependencies with `dumpbin /dependents` (Windows) or `ldd` (Linux)

### BadImageFormatException

**Error:** `An attempt was made to load a program with an incorrect format.`

**Cause:** Architecture mismatch (x64 vs x86)

**Solution:** Build Rust library for correct target:
```powershell
# For x64
cargo build --release --target x86_64-pc-windows-msvc

# For x86
cargo build --release --target i686-pc-windows-msvc
```

### AccessViolationException

**Cause:** Invalid pointer usage or memory corruption

**Fixes:**
- Verify you're calling `rrpc_init()` before `rrpc_call()`
- Ensure all pointers are valid
- Check that buffers aren't garbage-collected mid-call (use `fixed`)
- Always call `rrpc_free()` exactly once per output

## Complete Example Project

See [examples/fsharp-client/](../examples/fsharp-client/) for a full working F# client.

## See Also

- [Getting Started](getting-started.md)
- [API Reference](api-reference.md)
- [Error Handling](error-handling.md)
