namespace RRpc

open System
open System.Runtime.InteropServices

module Native =
    // Library name - the cdylib compiled by the rrpc-core crate
    // On Windows this will produce `rrpc_core.dll`. On Unix `librrpc_core.so`.
    // When using the library in dev, ensure the native DLL is on PATH or next to the executable.
#if WINDOWS
    [<Literal>]
    let LibName = "rrpc_core"
#else
    [<Literal>]
    let LibName = "rrpc_core"
#endif

    [<DllImport(LibName, CallingConvention = CallingConvention.Cdecl, EntryPoint = "rrpc_init")>]
    extern int rrpc_init()

    [<DllImport(LibName, CallingConvention = CallingConvention.Cdecl, EntryPoint = "rrpc_call")>]
    extern int rrpc_call(IntPtr method_ptr, IntPtr in_ptr, UIntPtr in_len, out IntPtr out_ptr, out UIntPtr out_len)

    [<DllImport(LibName, CallingConvention = CallingConvention.Cdecl, EntryPoint = "rrpc_free")>]
    extern void rrpc_free(IntPtr ptr, UIntPtr len)
