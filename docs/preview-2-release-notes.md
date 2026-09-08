# neoCLR Preview 2 — release notes

Version: **0.1.0-preview.2** · Tag: **v0.1.0-preview.2** · Date: **2026-09-08**.
A source-only GitHub prerelease; no prebuilt binaries or crates.io publication.

Preview 2 develops the managed-memory model and adds Neo, a small Raven-inspired
companion language for demonstrating and testing the runtime. Types remain values
by default. Explicit managed references can address frame-owned values or managed
heap objects, with checked lifetimes and transparent target access in Neo.

## Highlights

- A tracing, nonmoving mark-and-sweep GC reclaims unreachable managed heap graphs,
  including cycles. Interior references and interface views preserve their owners.
  GC statistics and bounded event history expose collection activity.
- Managed arrays and a growable ArrayList<T> support values and managed-reference
  elements. ArrayList<Foo&> stores references in a managed backing array; it needs no
  Free. Interface views provide virtual dispatch without boxing.
- Neo supports records, interfaces, functions, control flow, union-aware match,
  typeof, arrays, reference-aware calls and explicit output parameters. It is an
  intentionally small concept compiler, maintained with runtime development.
- The interactive terminal debugger shows call stacks, frame slots, evaluation
  stacks, managed heap objects and native allocations. It provides source mappings,
  breakpoints, step-in, source step-over and step-out.
- Type introspection exposes MethodInfo, FieldInfo, PropertyInfo and ParameterInfo.
  GetType on a live managed reference discovers its concrete target through interface
  views. Preliminary ReferenceEquals compares managed locations independently of
  value equality; the corresponding IL instructions are ref.type and ref.eq.
- Repeated Neo declarations renew local storage, allowing varying array lengths on
  successive iterations. Resetting storage while aliases remain live faults.

See the [changelog](../CHANGELOG.md) for the complete dated history and
[library API design](api-design.md) for value, reference and output contracts.

## Build and try

Install Rust 1.85.0 or stable and the native C build prerequisites in the
[README](../README.md#build-and-run-a-sample). From the extracted source root:

```sh
cargo run --locked -- run examples/source/counter.neo --gc-stats --gc-events
cargo run --locked -- run examples/source/collections.neo
cargo run --locked -- run examples/source/outputs.neo
cargo run --locked -- run examples/source/reflection.neo
cargo run --locked -- run examples/source/reference-identity.neo
cargo run --locked -- debug examples/source/counter.neo
```

Dependencies are downloaded by Cargo; the System library is bundled automatically.
Read the [Neo guide](neo.md), [grammar](neo-grammar.md), [debugger guide](debugger.md),
[reflection guide](reflection.md) and [reference identity guide](reference-identity.md).

## Migration and limits

Reassemble older artifacts and external System libraries with this runtime. JSON
format 5 is provisional. Earlier development Ref<T>/heap.load/heap.store APIs have
been replaced by T&, ldobj and stobj. Remove ArrayList.Free calls and use managed
reference receivers. Copying a list descriptor copies Count but shares its backing
array until growth; use ArrayList<T>& to share the whole mutable descriptor.

Neo automatically accesses managed-reference targets; remove explicit managed
dereferences from earlier development samples. Returning a reference into the current
frame remains invalid: use explicit managed heap allocation for an independent
lifetime. Frame ownership describes runtime lifetimes, not a promise of physical
native-stack placement in this interpreter.

GC does not call Dispose or Close. Guest destructors, finalizers, automatic block
cleanup, inheritance, pinning, persistent host roots and concurrency remain future
work. Reflection is metadata-only; there is no reflective invocation or field mutation.
The debugger launches programs and does not attach to arbitrary OS processes.
Neo is not a full language implementation. Output-callee obligations are enforced
at runtime; the verifier checks caller initialization. IL/artifact verification is
optional, while runtime lifetime checks remain mandatory. Raw pointer operations
are low-level native interop and do not acquire managed lifetime guarantees.

There is no JIT, native AOT, .NET assembly importer, stable ABI or compatibility
promise. This is an experimental preview, not a production sandbox.

## Validation and distribution

Publication requires successful checks on the exact tagged commit in all six CI
jobs: Linux, macOS and Windows, each on Rust 1.85.0 and stable. Each job tests an
extracted source archive, checks locked dependency notices and compares nine Neo
programs as source and artifacts, then exercises native interop. Stable jobs also
run formatting and strict Clippy. These are tested runner configurations, not
support for every OS/CPU combination.

The GitHub release links the exact passing run and includes the source archive,
its SHA-256 and per-job validation reports. See the [validation procedure](next-preview-validation.md).
The project is MIT-licensed; [third-party notices](../THIRD_PARTY_NOTICES.md) cover
locked dependencies and native build tooling. Dependency sources are not vendored.
