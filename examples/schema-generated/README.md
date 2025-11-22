# Schema-generated code examples

This folder demonstrates a minimal schema (MSL) and generated typed models + codecs for multiple languages.

Schema: `examples/schema/workspace.msl`

Generated outputs and tests:

- fsharp — `Generated.fs` + `Generated.Tests.fs` (Expecto)
- rust   — `src/lib.rs` with serde_json roundtrip tests
- go     — `node.go` + `node_test.go` using encoding/json
- ts     — `node.ts` (TypeScript types) and `test_roundtrip.js` (NodeJS JSON roundtrip)

What these show:
- One canonical schema can produce typed models for multiple languages
- JSON codecs (serde/json/System.Text.Json) allow roundtrip validation
- This is the initial prototype of an MSL → codegen workflow

Next steps:
- Build a small `msl-compiler` that emits these files automatically from MSL
- Add binary codecs (bincode/protobuf/msgpack) for performance
- Wire generated Rust types to the rRPC registry and F# bindings to call them end-to-end
