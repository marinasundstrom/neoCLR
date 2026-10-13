# Integrated runtime and memory-management direction

Author direction, 2026-10-13. Interpreter, JIT, native AOT and managed memory
management are parts of one neoCLR product. Historical experiment directories and
`neoclr-aot-poc` naming do not define separate products. Qualification gaps remain
explicit; integrating an engine does not qualify every target or enable a prototype.

## Build foundation

The runtime, JIT/AOT backend, shared security provider and TLS adapter now share the
root Cargo workspace and lockfile. Build both executable tools with:

```sh
cargo build --locked --workspace
```

Outputs are `target/debug/neoclr` and `target/debug/neoclr-aot-poc` (with `.exe` on
Windows). `--release` selects `target/release`. Existing backend manifest commands
also use this workspace and output directory. Old files under
`tools/aot-poc/target` are stale build artifacts, not fallback executables.

Run the shared conformance entry point from an ordinary macOS shell or an MSVC
Windows developer shell with Python, Rust and OpenSSL available:

```sh
python3 scripts/validate-runtime-product.py --output target/runtime-product-check
```

It retains per-command logs and a machine-readable report, combining the workspace
build, CLI mode selection, scalar interpreter/JIT/native comparisons, native GC/root
contracts and TLS native-adapter checks. ARM64 macOS also runs compiled rooted UTF-8
slicing. Platform-specific test guards still apply; this is not full HTTP, Windows
ARM64 or whole-library qualification. CI uses this same entry point.

The backend package/binary retains its existing name for bundle compatibility in
this slice. `cargo build -p neoclr` and the default root build still omit compiler
packages from the selected build; interpreter-only consumers do not acquire a
Cranelift dependency. Compiler dependencies remain in the shared lockfile.
The runtime's existing resolved dependency versions are retained; common backend
dependencies align with that resolution instead of carrying a second lockfile.

Native TLS static-library builds use `--profile native-ffi` to retain abort-on-panic
behavior. Ordinary runtime builds keep their existing panic policy. Explicit native
archive target directories are retained where packaging requires them. The separate
text/crypto static-library workspaces remain migration work, including preserving
text LTO/size settings. This is not a completed source-layout or packaging migration. `src/jit_cli.rs`
still launches the backend process; workspace membership alone does not create an
in-process JIT service or a complete embedding API.

## Memory management is a runtime service

**Architectural direction, not a new public ABI.** GC remains the default memory
manager for managed programs. The author also wants future constrained-device
profiles without tracing GC and distributions with specialized collectors. Select
an implementation when building a runtime distribution; do not implement live GC
replacement or collector migration inside a running execution.

The runtime/memory-manager boundary must cover:

- Aligned, zero-initialized allocation, overflow/limits and terminal exhaustion;
  result publication and admitted allocation retry must preserve existing roots.
- Object/reference layout descriptions, roots/handles, interior references and
  suspended-frame ownership. A moving manager requires updateable references or
  explicitly supported pinning; today's conservative addresses are insufficient.
- Reference stores and bulk copies, with collector-specific barriers when needed.
  Build-selected implementations can inline fast paths; a per-allocation or
  per-reference-store virtual call is not required by this architecture.
- Safe suspension/quiescence, concurrent mutation rules, host attachment and resource
  teardown. Managed liveness and deterministic native-resource release are separate
  responsibilities; fault exits and abandoned operations must remain safe.
- Limits and truthful diagnostics. Collector-specific generations/tuning remain
  optional capabilities, not invented properties of every memory manager.

A bounded arena/region profile could be useful for a closed invocation with a proven
lifetime boundary. It cannot reset memory while objects, host handles or suspended
operations escape that boundary. Reject unsupported program capabilities or prove
those lifetimes; do not silently leak, reset live objects, or call disabling GC a
complete small-device implementation. Reference counting is another possible
implementation, with cycle and store-overhead costs, not a selected replacement.

Distribution compatibility must account for layout, root and barrier requirements.
AOT artifacts and native hosts must match the selected memory profile. JIT and
interpreter execution must use its capabilities consistently. Exact profile identity,
metadata and ABI encoding remain to be designed before alternate managers ship.

## Comparison and tradeoffs

Primary sources reviewed 2026-10-13:

- [.NET collector architecture](https://github.com/dotnet/runtime/blob/main/docs/design/coreclr/botr/garbage-collection.md)
  separates allocation and reclamation responsibilities and describes the execution
  engine supplying allocation context. We retain familiar managed lifetime semantics
  while leaving implementation choice behind a runtime-owned boundary.
- [.NET Native AOT build integration](https://github.com/dotnet/runtime/blob/main/docs/workflow/building/coreclr/nativeaot.md)
  links a runtime library with generated code. Native publishing is not equivalent
  to removing runtime services. Separate compilation tools can belong to one product.
- [Embedded Rust collections](https://doc.rust-lang.org/embedded-book/collections/index.html)
  distinguishes allocator-backed and fixed-capacity storage. Predictable capacity
  is useful for constrained devices, but Rust's ownership guarantees do not transfer
  automatically to arbitrary managed object graphs.

The proposed variation is distribution-level memory-manager choice, not a promise
that arbitrary managed programs run without GC. Benefits are footprint/workload
specialization and direct fast paths. Costs are capability validation, collector
conformance, matching native artifacts and a larger supported-profile matrix.
No alternative collector or no-GC device distribution is implemented by this slice.

## Next integration slices

1. Keep one build and focused conformance entry point for CLI/interpreter, JIT,
   compiled native consumers and GC/root contracts; retain platform-specific tests.
2. Move shared native runtime implementation out of documentation experiment paths,
   keeping fixtures and historical evidence separate. Update packaging/import recipes
   together; do not duplicate implementations or promote incomplete APIs by renaming.
3. Extract the backend library/hosting boundary from the provisional CLI launcher;
   a runtime-owned execution contract must account for session, fault and resource
   lifetime. Avoid a runtime/backend dependency cycle: common metadata and runtime
   services must be usable without importing the compiler. The exact crate split is
   still to be selected. AOT can remain a separate publishing tool in the same product.
4. Consolidate static-library builds, distribution manifests and backend naming.
5. Extract the memory-manager contract from current allocation/root/cleanup behavior,
   then qualify build-selected implementations. Finish current helper-root and idle
   cleanup work before changing pacing; measure improvements independently of this
   build integration. Generational GC remains a candidate, not a selected policy.

## Qualification — 2026-10-13

The local integrated gate and the hosted macOS job at `bb5a3d2b0` pass: workspace
build, CLI selection, 24 scalar execution/compiler tests, two compiled rooted-text
tests, 20 native GC/root executions, security/TLS adapter checks and packaging
policy tests. Reports are retained in [validation/runtime-product](validation/runtime-product/).
The local report predates the implementation commit; hosted evidence has the exact
revision and input hashes. The root-only dependency graph excludes Cranelift.

The first Windows run exposed line-ending conversion of the hashed Cranelift
license, fixed by preserving notice bytes in checkout. The next run passed notice
checks, the workspace build and CLI selection, but exposed a CRLF-sensitive
negative compiler fixture. That fixture now normalizes line endings and verifies
each invalid-input mutation is effective; the focused local rerun passes. The [Windows rerun](https://github.com/marinasundstrom/neoCLR/actions/runs/38431039111)
at `4d1efde54` passes the full integrated gate, including all 12 applicable scalar
compiler/JIT tests, 20 GC/root executions, TLS/security and packaging checks.
The retained Windows reports include exact inputs and commands. These are build/test
integration fixes, not a measured runtime throughput improvement.
