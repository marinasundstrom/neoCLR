# neoCLR Preview 3 — runtime-library foundations

Version: **0.1.0-preview.3** · Tag: **v0.1.0-preview.3** · Date: **2026-09-08**.
Source-only GitHub prerelease; no prebuilt binaries or crates.io publication.

Preview 3 makes the platform more useful through a small, familiar runtime library
and the object/callable features needed to build it. Neo remains a companion concept
language for exercising and explaining the runtime, rather than a full compiler.
There is no .NET installation requirement or .NET assembly compatibility claim.

Values remain the default. `T&` explicitly selects managed reference semantics for
frame-owned or managed-heap storage, and Neo handles target access automatically.
Reference views preserve the concrete owner and its lifetime; base-class and
interface projections do not implicitly box or slice values. The .NET comparisons
and deliberate tradeoffs are recorded in the linked API design documents.

## Useful library building blocks

- [Comparable, Iterable and Iterator](common-interfaces.md), with readonly observation
  and explicit disposal. ArrayList iterators retain their initial backing buffer and
  extent; they do not use .NET List's mutation-version invalidation policy.
  [Predicate searches](predicate-search.md) return Option<T> for absence and use
  shared Func delegate contracts.
- [Ordinal String operations](ordinal-text.md), [Char classification](character-classification.md)
  backed by pinned Unicode 16 data, and [fundamental Math helpers](math.md).
  Neo adds character and finite Double literals. Culture-sensitive text operations
  and implicit numeric widening are outside this subset.
- Separate [Date and Time](date-time.md) values with checked construction, useful
  components and comparison. [Clock.GetLocalNow](local-clock.md) captures the actual
  local date, time and UTC offset in one reading. Clock precision and timezone
  fallback are documented; formatting, parsing and globalization are deferred.
- [Environment](environment.md) exposes guest arguments, current directory and
  process-variable lookup. Missing values use Option and fallible reads use Result.
  [Path](path.md) provides lexical combination and filename extraction.
- [Bounded UTF-8 file output](file-output.md) complements ReadAllText. WriteAllText
  returns Result<Void,FileWriteError>, checks the byte limit before opening, and
  creates/replaces a regular file without adding a BOM. It is not an atomic-save API:
  a later failure can leave partial output.

## Runtime and Neo foundations

[Readonly references](readonly-storage.md) now work across parameters, receivers,
fields, locals, arrays and return signatures. Permissions survive derived addresses
and views; widening to writable access faults. Readonly is shallow: another writable
alias may change the underlying data. Immutable bindings remain language policy.

The [object model](class-dispatch.md) adds inherited layout, managed base views,
virtual/override dispatch and abstract classes. [Constructor chaining](constructor-chaining.md)
keeps storage unpublished until initialization and base completion. Neo adds ordinary
classes, field initializers, bounded default-constructor synthesis and default(T).
Defaults do not call constructors or manufacture invalid managed references.
Base-first field-initializer ordering deliberately differs from C#.

[Interface inheritance](interface-inheritance.md), [explicit implementations](explicit-interfaces.md)
and [default bodies](default-interface-implementations.md) use managed receivers and
preserve lifetime, readonly and output rules. Most-specific selection and ambiguous
diamonds are checked. Reflection's MethodInfo, FieldInfo and PropertyInfo now share
an abstract [MemberInfo base](reflection-hierarchy.md), using chained construction
and readonly readers. Introspection remains metadata-only and declared-only.

[Generic functions and static methods](function-generics.md) support explicit type
arguments and Neo argument-based inference. [Delegates](delegates.md) provide the
shared callable abstraction: contextual method groups, normal invocation syntax,
and [lambdas/closures](delegates.md) lower onto managed delegates and capture storage.
Func supports Void results, so no separate Action family is needed. Captured mutable
bindings share storage, including returned closures; unsafe captures are rejected.
These closures allocate managed cells/environments and do not promise allocation-free
execution. Multicast delegates, variance and generic instance methods remain later work.

## Build and try

Install Rust 1.85.0 or stable and the native C build prerequisites in the
[README](../README.md#build-and-run-a-sample). From the extracted source root:

```sh
cargo run --locked -- run examples/source/common-interfaces.neo
cargo run --locked -- run examples/source/predicate-search.neo
cargo run --locked -- run examples/source/constructor-chaining.neo
cargo run --locked -- run examples/source/default-interfaces.neo
cargo run --locked -- run examples/source/closures.neo
cargo run --locked -- run examples/source/date-time.neo
cargo run --locked -- run examples/source/local-clock.neo
cargo run --locked -- run examples/source/environment.neo -- "guest argument"
mkdir preview-output
cargo run --locked -- run examples/source/file-report.neo -- README.md preview-output
```

The report example reads at most 1 MiB and writes at most 4096 bytes to
`preview-output/summary.txt`, replacing it if it exists. It demonstrates arguments,
paths, text, bounded I/O and recoverable errors together. The CLI currently prints
the guest return value; a nonzero Main result is not a process exit-code contract.
Use `debug` instead of `run` to inspect a program, including guest arguments after `--`.
See the [Neo guide](neo.md), [grammar](neo-grammar.md) and [debugger guide](debugger.md).

## Migration and limits

Recompile applications and external System artifacts together with this runtime.
The provisional JSON format and Rust embedding API are not stable. New readonly,
inheritance, constructor, mapping and delegate metadata/instructions require the
matching runtime; Rust metadata literals and exhaustive enum matches may need updates.

Equatable, scalar Equals, Comparable and reflection descriptor readers now require
readonly managed receivers. Neo borrows automatically; direct IL and host wrappers
must supply the matching receivers. List implementers must supply inherited
GetIterator and readonly getter contracts. Readonly references cannot be stored in
unqualified writable destinations. Derived constructors use managed receivers, and
managed stfld returns Void while value-form stfld still returns an updated value.
Whole-value operations through base views fault rather than slice the complete owner.

Neo reserves class/default/readonly and generated neoCLR.Compiler names. Rust
ExecutionOptions gains arguments; exhaustive literals must initialize that field or
use defaults. Use array indexers in Neo; optional empty heap-array braces are unnecessary.

Nullability metadata, enums/flags, generic constraints, async, LINQ, dynamic hooks,
general string indexing and broader framework coverage remain future work. There is
no JIT, native AOT, .NET assembly importer, stable ABI or production-sandbox promise.
GC does not call Dispose/Close; guest destructors, automatic cleanup, pinning and
persistent host roots remain separate work. Raw pointers remain explicit native
interop. The debugger launches guest programs rather than attaching to arbitrary OS
processes. See the [roadmap](roadmap.md) and [changelog](../CHANGELOG.md).

## Validation and distribution

Publication requires a green six-job CI run for the exact versioned commit: Linux,
macOS and Windows on Rust 1.85.0 and stable. Each job checks a fresh source archive,
locked notices, all test targets, 26 Neo source/artifact scenarios and native interop.
Stable jobs also run formatting and strict Clippy. Runner results do not establish
support for every OS/CPU configuration.

The GitHub release records the validated commit/run and supplies the source archive,
SHA-256 checksums and per-job reports. Earlier development runs do not certify this
versioned candidate. See [validation](next-preview-validation.md) and
[source packaging](source-release.md). The project is MIT-licensed; the
[third-party notices](../THIRD_PARTY_NOTICES.md) cover 47 locked registry packages
and retained native build-tool notices. Dependency sources are downloaded by Cargo,
not vendored into this source preview.
