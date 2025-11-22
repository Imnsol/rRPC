namespace RRpc

open System
open System.Runtime.InteropServices

module Native =
    [<DllImport("rrpc_core", CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)>]
    extern int rrpc_init()

    [<DllImport("rrpc_core", CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)>]
    extern int rrpc_call(IntPtr method_ptr, UIntPtr method_len, IntPtr in_ptr, UIntPtr in_len, out IntPtr out_ptr, out UIntPtr out_len)

    [<DllImport("rrpc_core", CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi)>]
    extern void rrpc_free(IntPtr ptr, UIntPtr len)

module RRpc =
    let init() = 
        let res = Native.rrpc_init()
        if res <> 0 then failwithf "rrpc_init failed: %d" res

    let call (methodName:string) (input:byte[]) : Result<byte[], string> =
        // Pin/allocate input
        let methodBytes = System.Text.Encoding.UTF8.GetBytes(methodName)
        let methodPtr = Marshal.AllocHGlobal(methodBytes.Length)
        Marshal.Copy(methodBytes, 0, methodPtr, methodBytes.Length)

        let inputLen = if isNull input then 0 else input.Length
        let inPtr = if inputLen = 0 then IntPtr.Zero else Marshal.AllocHGlobal(inputLen)
        if inputLen > 0 then Marshal.Copy(input, 0, inPtr, inputLen)

        let mutable outPtr = IntPtr.Zero
        let mutable outLen = UIntPtr.Zero

        try
            let rc = Native.rrpc_call(methodPtr, UIntPtr(uint32 methodBytes.Length), inPtr, UIntPtr(uint32 inputLen), &outPtr, &outLen)
            if rc <> 0 then
                Error (sprintf "rrpc_call returned %.d" rc)
            else
                let outLenInt = int outLen
                if outPtr = IntPtr.Zero || outLenInt = 0 then
                    Ok [||]
                else
                    let result = Array.zeroCreate<byte> outLenInt
                    Marshal.Copy(outPtr, result, 0, outLenInt)
                    // free the buffer returned by Rust
                    Native.rrpc_free(outPtr, UIntPtr(uint32 outLenInt))
                    Ok result
        finally
            if methodPtr <> IntPtr.Zero then Marshal.FreeHGlobal(methodPtr)
            if inPtr <> IntPtr.Zero then Marshal.FreeHGlobal(inPtr)
