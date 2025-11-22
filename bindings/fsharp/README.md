# rRPC — F# binding demo

This folder contains a minimal F# demo client that calls the `rrpc_core` runtime using P/Invoke.

Prerequisites
- .NET SDK 8/9 installed (this repo was tested with .NET 9)
- Rust toolchain (to build the `rrpc_core` cdylib)

Quick steps (PowerShell)

```powershell
# 1. Build rrpc_core (repo root)
Set-Location '..\..\..'  # repo root
cargo build --release

# 2. Build the F# demo
Set-Location 'bindings\fsharp'
dotnet build

# 3. Copy the native DLL into the F# app folder
Copy-Item -Path 'target\release\rrpc_core.dll' -Destination 'bindings\fsharp\bin\Debug\net9.0\rrpc_core.dll' -Force

# 4. Run the demo
dotnet run
```

Notes
- The demo uses `DllImport("rrpc_core")` — when the DLL is next to the executable, P/Invoke will find it.
- If you prefer the `debug` runtime you can `cargo build` without `--release` and copy from `target\debug` instead.
