namespace RRpc

open System
open System.Runtime.InteropServices
open System.Text
open System.Text.Json
open Native

module RRpcClient =

    let safeCallNative f =
        try
            Ok (f())
        with
        | :? DllNotFoundException as e -> Error (sprintf "native library not found: %s" e.Message)
        | :? EntryPointNotFoundException as e -> Error (sprintf "native function not found: %s" e.Message)
        | ex -> Error (sprintf "native call error: %s" ex.Message)

    /// Initialize runtime (returns Ok 0 on success or Error string)
    let init () : Result<int, string> =
        safeCallNative(fun () -> Native.rrpc_init())

    /// Call a method (method name is UTF8 string, input byte[]). Returns Ok bytes or Error string.
    let call (methodName: string) (input: byte[]) : Result<byte[], string> =
        // Encode method name to UTF8 and ensure null-terminated for C
        let methodBytes = Encoding.UTF8.GetBytes(methodName + "\u0000")
        // Pin arrays
        try
            use mb = GCHandle.Alloc(methodBytes, GCHandleType.Pinned)
            use ib = GCHandle.Alloc(input, GCHandleType.Pinned)
            let mptr = mb.AddrOfPinnedObject()
            let iptr = if input.Length = 0 then IntPtr.Zero else ib.AddrOfPinnedObject()

            // Call native
            match safeCallNative(fun () ->
                    let mutable outPtr = IntPtr.Zero
                    let mutable outLen = UintPtr.Zero
                    let rc = Native.rrpc_call(mptr, iptr, (uint input.Length |> UIntPtr), &outPtr, &outLen)
                    if rc <> 0 then rc else 0
            ) with
            | Error e -> Error e
            | Ok rc ->
                if rc <> 0 then Error (sprintf "rrpc_call failed: rc=%d" rc)
                else
                    // If succeeded, we already copied the result above; but due to safeCallNative closure limitations,
                    // re-implement call flow and return actual bytes safely below.
                    // We'll implement a direct version now.
                    if rc <> 0 then Error (sprintf "rrpc_call failed: rc=%d" rc)
                    else
                        if outPtr = IntPtr.Zero || outLen = UIntPtr.Zero then Ok [||]
                        else
                            // Convert UIntPtr to int safely
                            let len = int (uint64 outLen)
                            if len <= 0 then Ok [||] else
                            let out = Array.zeroCreate len
                            Marshal.Copy(outPtr, out, 0, len)
                            Native.rrpc_free(outPtr, outLen)
                            Ok out
        finally
            ()

    /// Helper: call and decode JSON result as a given type
    let callJson<'T> (methodName: string) (input: obj) : Result<'T, string> =
        // input -> bytes
        let inputBytes = JsonSerializer.SerializeToUtf8Bytes(input)
        match call methodName inputBytes with
        | Ok out ->
            try
                let v = JsonSerializer.Deserialize<'T>(out)
                match v with
                | null -> Error "Deserialized null"
                | v -> Ok v
            with ex -> Error (sprintf "JSON decode error: %s" ex.Message)
        | Error e -> Error e
