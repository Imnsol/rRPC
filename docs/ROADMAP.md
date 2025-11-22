# rRPC — Public Roadmap (short view)

This file is the short, public-facing roadmap for rRPC. It's intentionally concise — a developer-focused, detailed plan lives in `NEXT_STEPS.md`.

Last updated: 2025-11-21

## High-level goals

- Deliver a production-ready schema-first FFI runtime for .NET ↔ Rust (native and WASM)
- Provide simple, strongly-typed bindings and patterns so teams can safely call across languages without HTTP
- Offer first-class support for UDG and time-travel-friendly event logging (for deterministic reproducibility)

## Milestones

### v0.1 (Done)
- Core FFI runtime (registry + API): ✅
- Basic examples (echo, reverse): ✅
- Apache-2.0 license (patent protection): ✅
- CI / tests across platforms: ✅

### v0.2 (Short-term — next 0–6 weeks)
- Public deliverables:
  - MSL (Mycelium Schema Language) minimal prototype (types + encoder/decoder)
  - F# P/Invoke bindings + a tiny F# demo client
  - WASM build target and a small browser demo
  - Basic benchmarks and docs (docs/benchmarks.md)

### v0.3 (Medium-term — 2–3 months)
- Public deliverables:
  - Full MSL codegen for F#/Rust/TypeScript
  - Zero-copy / large-buffer optimizations for high-throughput domains
  - Capability-based permissions and audit logging
  - Integration examples: UDG functions, Elmish TUI → rRPC → Bevy 3D demo

### v1.0 (6+ months, stable)
- Stabilize API surface and publish packages:
  - crates.io: rrpc-core
  - nuget.org: RRpc.FSharp
  - npm: @rrpc/core (TypeScript/WASM bindings)
- Production-ready docs, benchmarks and migration guides

## Usage & contribution policy

This repo is currently maintained by the owner and is published under Apache-2.0. At present, the owner is not accepting external contributions. You are welcome to fork and use the code.

If you want to follow day-to-day progress, the detailed living roadmap is `NEXT_STEPS.md` in the repository root.

---

For more details on the design and developer-centric plan, see `NEXT_STEPS.md` and `docs/ARCHITECTURE.md`.
