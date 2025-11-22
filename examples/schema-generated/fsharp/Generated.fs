namespace Schema

open System
open System.Text.Json
open System.Text.Json.Serialization

[<CLIMutable>]
type Node = {
    Id: Guid
    Title: string
    Position: float[]
}

[<CLIMutable>]
type HyperEdge = {
    Id: string
    Nodes: Guid[]
    Label: string option
}

module Codec =
    let serialize<'T> (x: 'T) = JsonSerializer.SerializeToUtf8Bytes(x)
    let deserialize<'T> (b: byte[]) : 'T = JsonSerializer.Deserialize<'T>(b)
