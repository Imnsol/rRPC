open System
open RRpc

// Top-level demo runner (easier to ensure runtime executes)
printfn "Starting F# rRPC demo client..."

try
    // Initialize rRPC runtime
    RRpc.init()

    let input = System.Text.Encoding.UTF8.GetBytes("Hello from F#")
    match RRpc.call "echo" input with
    | Ok bytes -> printfn "Echo returned: %s" (System.Text.Encoding.UTF8.GetString(bytes))
    | Error e -> printfn "Call failed: %s" e

    match RRpc.call "reverse" input with
    | Ok bytes -> printfn "Reverse returned: %s" (System.Text.Encoding.UTF8.GetString(bytes))
    | Error e -> printfn "Reverse failed: %s" e

    // Unknown method test
    match RRpc.call "missing" [||] with
    | Ok _ -> printfn "Unexpected success"
    | Error e -> printfn "Missing method error: %s" e
with ex ->
    printfn "Demo failed: %s" (ex.ToString())

// block until user hits Enter so test output remains visible
printfn "Demo complete — press Enter to exit"
System.Console.ReadLine() |> ignore
