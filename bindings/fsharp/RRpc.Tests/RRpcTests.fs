module RRpc.Tests

open Expecto
open RRpc.RRpcClient

let test_init _ =
    match init() with
    | Ok rc ->
        test <@ rc = 0 @> |> ignore
    | Error msg ->
        // Native lib missing is acceptable for local dev — treat as skipped
        Test.skiptestf "native not available: %s" msg

let test_call_unknown_method _ =
    match init() with
    | Ok _ ->
        match call("no_such_method", [||]) with
        | Ok _ -> failwith "Expected rrpc_call to fail for unknown method"
        | Error _ -> () // expected
    | Error msg ->
        Test.skiptestf "native not available: %s" msg

let test_callJson_error _ =
    match init() with
    | Ok _ ->
        match callJson<obj>("no_such_method", null) with
        | Ok _ -> failwith "Expected callJson to return error for unknown method"
        | Error _ -> ()
    | Error msg -> Test.skiptestf "native not available: %s" msg

let tests =
    testList "RRpc bindings" [
        testCase "rrpc_init" (fun _ -> test_init())
        testCase "rrpc_call unknown method" (fun _ -> test_call_unknown_method())
        testCase "callJson error handling" (fun _ -> test_callJson_error())
    ]

[<EntryPoint>]
let main args =
    runTestsWithArgs defaultConfig args tests
