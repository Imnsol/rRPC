# msl-compiler (prototype)

This is a small prototype MSL (minimal schema language) compiler used in the rRPC project. It reads a YAML MSL file and generates typed model code for multiple target languages (F#, Rust, Go, TypeScript) in examples/schema-generated.

## Quick usage

Build and run locally from the repository root:

```pwsh
cd tools/msl-compiler
cargo run -- ../examples/schema/workspace.msl -o ../examples/schema-generated
```

This writes generated files under `examples/schema-generated` in language subfolders.

## Tests

There are integration tests that run the compiler against `examples/schema/workspace.msl` and compare outputs to the checked-in `examples/schema-generated` samples:

```pwsh
cd tools/msl-compiler
cargo test --test compare_to_examples -- --nocapture
```

There are also unit/debug tests that help inspect the YAML parsing and generator behavior.

## Extending

- `src/lib.rs` contains a small IR extractor and generation helpers.
- Add new target templates in `src/` and add tests under `tests/` to assert generation parity.

This is an early prototype; the code is intentionally minimal and should be extended with a proper AST/IR, richer type system, and templating for production use.
