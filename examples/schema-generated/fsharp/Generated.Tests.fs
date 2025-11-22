module Schema.Generated.Tests

open Expecto
open Schema.Generated
open System

let tests =
    testList "F# generated roundtrip" [
        testCase "node roundtrip" <| fun _ ->
            let n = { Id = Guid.NewGuid(); Title = "Alice"; Position = [|1.0;2.0;3.0;4.0|] }
            let b = Codec.serialize n
            let n2 = Codec.deserialize<Node> b
            Expect.equal n.Id n2.Id "ids match"
            Expect.equal n.Title n2.Title "titles match"
            Expect.equal n.Position n2.Position "positions match"

        testCase "hyperedge roundtrip" <| fun _ ->
            let h = { Id = "he-1"; Nodes = [| Guid.NewGuid(); Guid.NewGuid() |]; Label = Some "edge" }
            let b = Codec.serialize h
            let h2 = Codec.deserialize<HyperEdge> b
            Expect.equal h.Id h2.Id "ids"
            Expect.equal h.Label h2.Label "labels"
    ]

[<EntryPoint>]
let main args = runTestsWithArgs defaultConfig args tests
