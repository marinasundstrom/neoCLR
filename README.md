# neoCLR

[![CI](https://github.com/marinasundstrom/neoCLR/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/marinasundstrom/neoCLR/actions/workflows/ci.yml)
[![Preview 3](https://img.shields.io/badge/release-v0.1.0--preview.3-blue)](https://github.com/marinasundstrom/neoCLR/releases/tag/v0.1.0-preview.3)
[![MIT license](https://img.shields.io/badge/license-MIT-green)](LICENSE)
[![Rust 1.85+](https://img.shields.io/badge/Rust-1.85%2B-orange)](Cargo.toml)

**An experimental, .NET-inspired virtual machine with values by default and explicit control over memory and references.**

neoCLR is a managed, type-safe virtual machine with garbage-collected heap storage
and explicit value versus reference semantics. Its intentionally low-level instruction
set also provides explicit memory access, typed pointers, native calls,
and allocation controls when a program needs them. It is a runtime platform rather
than a high-level language: language authors may build safer or more ergonomic
abstractions above the same capabilities.

neoCLR is the codename for an experimental runtime for an unnamed, .NET-inspired
platform. This repository contains a small standalone Rust interpreter, a neoIL
assembler, a prototype metadata format, and executable samples. It does not need
an installed .NET runtime.

The default is .NET/CLR semantics wherever neoCLR does not intentionally diverge.
Prototype shortcuts are limitations to remove, not new platform semantics.
Consumer APIs should preserve familiar .NET namespaces, names, overloads, and
contracts wherever the deliberate differences permit; see [API policy](docs/api-policy.md).

Assembly source syntax is free to evolve. The intended emitted metadata and CIL
remain based on the CLI model so existing parsing/tooling can be reused where
possible; deviations should be driven by a semantic requirement. The current JSON
output is a temporary internal format, not that final binary representation. See
[format direction](docs/format-direction.md) and
[assembler expressiveness](docs/assembler-design.md).

The goal is familiar runtime and library structure with deliberately different
semantics: one model for data types, value semantics by default, explicit storage
and lifetime choices, a real `Void` value, functions outside types, `Option<T>` for absence,
`Result<T,E>` for recoverable errors, and terminal Faults instead of exceptions.
Interfaces use ordinary names without an `I` prefix.
[Borrowed interface references](docs/interfaces.md) make dispatch explicit without boxing or ownership.
[Managed references in the runtime and Neo](docs/managed-reference-semantics.md) explains
frame ownership, GC reachability and automatic source-level access.
[Typed equality](docs/equality.md) uses System.Equatable<T> and Equals(T).
[Explicit cloning](docs/cloning.md) uses System.Clonable<T> and Clone(), independently
of ordinary value copies. The [lifecycle design](docs/lifecycle.md) separates
value lifetimes, managed heap collection and resource cleanup.
[Explicit cleanup](docs/disposal.md) is available through System.Disposable and
System.Closable<E>; automatic destruction remains future work.

## Try Neo

[Neo](docs/neo.md) is a small Raven-inspired concept language for this runtime.
We keep this companion compiler updated alongside neoCLR to test and explain platform
features. It is not currently intended to be a complex, full-fledged compiler; see
the [upcoming slices](docs/neo-roadmap.md) for its bounded development plan.
The first example exercises value copies, explicit references and managed heap storage:

```sh
cargo run -- run examples/source/counter.neo
cargo run -- run examples/source/counter.neo --gc-stats --gc-events
```

It prints `1`, `2`, and `42`. See the [Neo guide](docs/neo.md) for source syntax,
artifact compilation and current limits. The [bounded calculator](docs/neo-calculator.md)
combines control flow, union results and console input:

```sh
cargo run -- run examples/source/calculator.neo
```

## Release history

This README describes the current development tree. The [changelog](CHANGELOG.md)
separates unreleased work from published capabilities. Every commit updates it using
[the changelog workflow](docs/changelog.md); published entries remain unchanged.
The [next-preview validation guide](docs/next-preview-validation.md) documents the
reproducible source-archive check and exact-commit release gates.

## Preview 3

**Preview 3 (v0.1.0-preview.3)** is the runtime-library foundation preview: readonly
references, inheritance and constructor chaining, interface implementations, delegates
and closures, plus comparison/iteration, text/Math helpers, Date/Time, the local clock
and bounded file I/O. See the [release notes](docs/preview-3-release-notes.md) for
examples, migration guidance and limitations. Publication is gated on exact-commit
CI; the GitHub prerelease records validation and source checksums.

Managed references can address frame-owned values or managed heap objects. Neo reads
and writes their targets automatically; `ReferenceEquals` explicitly compares their
locations. Returning a reference into the current frame faults. GC reclaims unreachable
managed heap objects; native allocation/free remains explicit and separate.
[ArrayList<T>](docs/array-list.md) now uses a managed backing array and requires no Free.
Automatic destruction and resource cleanup remain future work.

[Preview 1's frozen release notes](docs/preview-1-release-notes.md) describe the earlier
release. Its [walkthrough](docs/preview-1-walkthrough.md) remains historical evidence;
Preview 2 also retains its [frozen release notes](docs/preview-2-release-notes.md).
Use the current [Neo guide](docs/neo.md) and examples for Preview 3.

## Build and run a sample

Run these commands from the repository root. Rust 1.85.0 is the minimum supported
toolchain (edition 2024); current stable Rust is also tested. Install Rust and a native
C toolchain for vendored libffi: compiler/make on Unix,
or MSVC tools on Windows. No .NET installation is required.

Cargo records this minimum as `rust-version = "1.85"`. CI is configured to test
1.85.0 and stable on Linux, macOS and Windows; recorded local validation is macOS
ARM64. See the [validation guide](docs/next-preview-validation.md) and the GitHub release
for exact-commit platform evidence. Dependencies must be downloaded on the first
build; the commands use Cargo.lock through `--locked`.

The assembler and interpreter are built together as the `neoclr` executable.
The bundled `runtime/System.neoil` library is assembled automatically; it does not
need a separate build for normal sample execution.

### Quick start

```sh
cargo run --locked -- run examples/hello.neoil
```

Cargo builds the executable if needed, then neoCLR assembles and runs the source.
Expected output:

```text
Hello, world!
=> Void
```

### Inspect the IL generated from Neo

```sh
cargo run --locked -- emit-il examples/source/enums.neo
cargo run --locked -- emit-il examples/source/generic-receivers.neo /tmp/receivers.neoil
```

The command validates the source and emits readable, reassemblable IL without running
it. File output refuses overwrites. See [IL inspection](docs/il-inspection.md) for
round trips and source-mapped debugging; JSON artifact disassembly remains future work.

### Build, assemble, verify, and run separately

```sh
cargo build --locked
./target/debug/neoclr assemble examples/hello.neoil hello.neo.json
./target/debug/neoclr verify hello.neo.json
./target/debug/neoclr run hello.neo.json
```

On Windows the executable is `target\debug\neoclr.exe`. `hello.neo.json` contains
prototype metadata and IL, not native machine code or a .NET executable. The assembler
refuses to overwrite an existing output: use a new filename or remove your previous
sample artifact before assembling again.

`verify` checks evaluation-stack types, definite local initialization, and return
contracts without executing the program. `check` performs structural metadata and
operand validation only. `run` accepts either `.neoil` source or a JSON artifact;
it does not automatically run the opt-in typed verifier.

### Build the runtime library explicitly (optional)

The runtime sources are organized by namespace under `runtime/System/` and
`runtime/neoCLR/Runtime/`. `runtime/System.neoil` is the ordered source manifest;
its included files assemble into one System module. Cargo bundles those same files.

```sh
cargo run --locked -- assemble runtime/System.neoil System.neo.json
cargo run --locked -- run hello.neo.json --system System.neo.json
```

The second command uses the HelloWorld artifact assembled above. These commands
select your compiled System library instead of the bundled source. The same
output-file overwrite rule applies to `System.neo.json`.

### Test representative programs

```sh
cargo run --locked -- run examples/strings.neoil
cargo run --locked -- run examples/errors.neoil
cargo run --locked -- run examples/file_input.neoil
cargo run --locked -- run examples/console_input.neoil
printf 'A' | cargo run --locked -- run examples/console_typed.neoil
cargo run --locked -- run examples/properties.neoil
cargo run --locked -- run examples/constructors.neoil
cargo run --locked -- run examples/ordinary_carrier.neoil
cargo run --locked -- run examples/ordinary_unions.neoil
cargo run --locked -- run examples/accessibility.neoil
cargo run --locked -- run examples/field_access.neoil
cargo run --locked -- run examples/type_visibility.neoil
cargo run --locked -- run examples/type_arities.neoil
cargo run --locked -- run examples/arrays.neoil
cargo run --locked -- run examples/array_bounds.neoil
```

The [accessibility sample](docs/accessibility.md) demonstrates public methods and restricted helpers.
The [class sample](docs/classes-and-defaults.md) demonstrates body fields, constructor
synthesis, typed defaults and frame/heap base views.
The [generic function sample](docs/function-generics.md#neo-projection) demonstrates explicit
type arguments on free functions and static methods.
The [property sample](docs/properties.md) demonstrates generic properties backed by ordinary methods.
The [constructor sample](docs/constructors.md) initializes generic records through public constructors with private fields.
The [ordinary carrier sample](docs/value-storage.md) uses explicit typed value storage and ordinary methods without union-specific instructions.
The [ordinary System union sample](docs/union-convention.md) uses ordinary System.Option/Result library types, including the migrated I/O boundaries.
The [ordinary carrier hosting example](docs/erased-inputs.md) passes a guest-created Result through the Rust host and back into IL.
The [parsing boundary](docs/int32-parse.md) now returns ordinary System.Result; the Error sample uses it throughout.
The [type arity sample](docs/type-arities.md) lets a non-generic companion coexist with generic types of the same name.
The [console sample](docs/console-io.md) prompts for a number, reads input, and doubles it.
The string sample demonstrates Unicode text and recoverable slice Errors. The
[file-input sample](docs/file-input.md) reads text, computes a result, and handles invalid input. The
[error sample](docs/errors.md) constructs and reports Error messages, then continues. The array
sample prints `10`, `42`, `10` and frees its buffer. The bounds sample deliberately
terminates with a nonzero exit code and a Fault stack trace; that failure is expected.

Run the repository test suite with:

```sh
cargo test --locked
```

### Multiple modules

```sh
cargo run --locked -- run examples/modules/app.neoil \
  --module examples/modules/operations.neoil --module examples/modules/models.neoil
```

`assemble`, `check`, and `verify` accept the same dependency flags. Use `--system`
to select a runtime library; see the [CLI module workflow](docs/cli-module-sets.md).

### More samples

The feature tour checks record copying, explicit heap identity, free functions,
loops, arithmetic, Option<Void>, and Result<Void,Error>. The additional samples
cover the implemented instruction set:

```sh
cargo run -- run examples/features.neoil
cargo run -- run examples/overloads.neoil
cargo run -- run examples/types.neoil
cargo run -- run examples/names.neoil
cargo run -- run examples/pointers.neoil
cargo run -- run examples/memory.neoil
cargo run -- run examples/stack.neoil
cargo run -- run examples/control-flow.neoil
cargo run -- run examples/arguments.neoil
cargo run -- run examples/comparison-branches.neoil
cargo run -- run examples/compact.neoil
cargo run -- run examples/layout.neoil
cargo run -- run examples/generic-metadata.neoil
cargo run -- run examples/generic-values.neoil
cargo run -- run examples/generic-methods.neoil
cargo run -- run examples/generic-memory.neoil
cargo run -- run examples/attributes.neoil
cargo run -- run examples/member-identities.neoil
cargo run -- verify examples/generic-methods.neoil
cargo run -- run examples/native-integers.neoil
cargo run -- run examples/integers.neoil
cargo run -- run examples/bits.neoil
cargo run -- run examples/floating.neoil
cargo run -- run examples/checked-conversions.neoil
cargo run -- run examples/fault.neoil
```

`examples/fault.neoil` deliberately produces a terminal Fault and nonzero exit code.

Declarations and calls carry parameter signatures, for example
`.function Describe(int32) -> string` and
`call System.Console.WriteLine(string)` or `call Initialize()`. Functions overload
by name and ordered parameter types; return types alone do not distinguish them.
The overload sample demonstrates both user-defined and bootstrap library overloads.
See [the assembly reference](docs/neoil.md) for primitive aliases and syntax.

## Native interop sample

```sh
cargo run --locked --example build_native
cargo run --locked -- run examples/pinvoke.neoil
```

This builds a native C-ABI library, then demonstrates a native function modifying
explicitly allocated guest storage. The CLI executes native imports as trusted code;
metadata checking does not load libraries. See [P/Invoke](docs/native-interop.md) for
supported signatures, embedding APIs, and the current memory boundaries.

## Runtime library

The runtime library is written for neoCLR in neoIL, then assembled into our metadata
and instruction representation. Public library functions run through ordinary guest
calls. Division error checks and absolute-value logic execute in IL; host primitives
remain for I/O and the temporary string parsing/conversion surface.

By default, the executable embeds the library source and assembles/caches it once.
It can also use an explicitly compiled System artifact:

```sh
cargo run -- assemble runtime/System.neoil System.neo.json
cargo run -- run examples/hello.neoil System.neo.json
```

Explicit module sets can link additional libraries, with optional direct dependency
lists and exact revision pins. Lookup still requires unique type names across the
load set; automatic dependency discovery and side-by-side versions remain deferred.
See [module sets](docs/module-sets.md) and [runtime library design](docs/runtime-library.md), and
[API contracts](docs/api-design.md).

## What is implemented

- Module, type/field, and free-function metadata, with typed parameters and locals.
- Signature-based overload resolution and structured call references.
- Optional parameter/local names (`string value`, `.local Point point`), preserved
  in metadata; named operands assemble to indices.
- Canonical System primitive definitions, type-owned static methods, copied value
  receivers, and explicit managed reference receivers; see [type system](docs/type-system.md)
  and [reference contracts](docs/reference-slots.md).
- Signed/unsigned 8-, 16-, 32-, and 64-bit integer storage, UTF-16 Char,
  integer conversions, bitwise/shift/remainder operations, and indirect loads/stores; see [integer types](docs/integer-types.md).
- Checked `conv.ovf.*` numeric conversions with explicit source signedness and overflow Faults.
- `Single`/`Double` storage, floating-point arithmetic, conversions, comparisons,
  and finite checks; see [floating point](docs/floating-point.md).
- `System.IntPtr`/`System.UIntPtr` (`nint`/`nuint`), native-width arithmetic,
  conversions, and native storage; see [native integers](docs/native-integers.md).
- Native `Ptr<T>`/`T*` values, explicit heap allocation/free, casts, byte offsets,
  field addresses, typed loads/stores, and native-sized pointer fields.
- `Void`, `Int32`, `Boolean`, `String`, `Error`, records, and constructed
  `Option<T>` and `Result<T,E>` types, including nested uses of `Void`, plus managed `T&` references.
- An iterative interpreter with explicit call frames and an IL-style evaluation stack.
- Frame-owned aggregate copying and explicit heap allocation/sharing.
- A line-oriented assembler with quoted strings, labels, source-line diagnostics,
  and forward function/type references.
- A platform-written [System library](runtime/System.neoil): overloaded
  `System.Console.WriteLine`, `System.Int32.Parse`, `System.Int32.Divide`, and
  `System.Math.Abs`, plus instance `Int32.ToString()`, assembled into the same metadata/IL representation as apps.
- Explicit `.pinvoke` imports with dynamic library loading and scalar/pointer C-ABI calls.
- Explicit bootstrap host bindings for console I/O, text, Error values, file input, and numeric conversion,
  explicitly declared with CLR-style `MethodImpl`/`InternalCall` metadata.
- Configurable limits on instruction count, call depth, stack slots per frame,
  explicit heap object count, native payload bytes, and allocation identities.

This is an original experiment informed by the
[ECMA-335 CLI specification](https://ecma-international.org/publications-and-standards/standards/ecma-335/),
particularly its metadata and CIL partitions. It is not a fork of CoreCLR and does
not yet import CLI metadata, execute .NET assemblies, or provide binary compatibility.
The prototype's JSON format is not a proposed replacement binary encoding.
Current format version 5 makes heap.new produce T& directly and removes the Ref
signature and heap.load/store instructions. Reassemble source and System artifacts
from earlier formats before loading them.

## Boundaries

“Stack by default” is a **guest semantic model** here: interpreter frames own
values, and copying an object copies its fields. Rust currently uses `Vec`,
`String`, and `Box` internally, so this does not demonstrate physical native-stack
allocation or its performance. The managed heap uses a nonmoving tracing collector;
T& is the unified reference feature for frame and heap storage. Native `Ptr<T>` allocations separately support individual free.
GC does not provide deterministic resource cleanup or guest finalizers. See
[memory layers](docs/memory-model.md) and [GC](docs/garbage-collection.md).

Option/Result still use temporary [System.Value storage](docs/value-storage.md),
which explicitly packs payloads into owned host value trees and recursively copies
owned values. This has allocation costs and is not a settled native representation.
Its planned removal requires a complete payload-storage and lifetime migration;
managed references and the Void* sample alone do not complete that migration.

The library is a bootstrap surface, not a complete BCL. General user-defined
generic instance methods, static lifetime verification, full native marshalling,
threading, runtime async, JIT compilation, and full verification are unimplemented.
Resource limits are guardrails, not a memory quota or a hostile-code sandbox.
The CLI writes console output immediately and flushes each line, so prompts are visible
before input. Default Rust embedding still captures output until successful completion;
an explicitly supplied host console enables live I/O. See [console I/O](docs/console-io.md).

See [semantics](docs/semantics.md), [assembler and opcodes](docs/neoil.md),
[arrays and pointers proposal](docs/arrays-and-pointers.md), and
[migration and roadmap](docs/roadmap.md). Candidate high-level frontends are a modified
C# dialect or a Raven subset; existing .NET code migration follows the necessary runtime
and OOP foundations. Work and validation are recorded in
[the work log](docs/work-log.md).

## Development

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

The CI matrix runs these checks on Linux, macOS, and Windows. The
[validation record](docs/preview-1-validation.md) includes a passing six-job
cross-platform baseline and identifies the tested snapshots. New changes still
require their own validation; earlier CI evidence does not certify the working tree.

Typed memory initialization/copying (`initobj`, `cpobj`) and byte-range operations
(`initblk`, `cpblk`) are implemented with explicit pointer checks. See
[heap and pointer semantics](docs/heap-and-pointers.md#copying-and-initializing-memory).

Parameter, local, and qualified field names are assembly conveniences mapped to
canonical indices. See [identifier mappings](docs/neoil.md#identifier-mappings-and-field-aliases).

Library-defined Option and Result are implemented through ordinary generic carrier
types, nested cases, constructors, properties and conditional output methods.
See the [union convention](docs/union-convention.md). Their remaining storage milestone
is retiring temporary System.Value after payload layout, copying and lifetime
contracts support String, errors and nested carriers; see [value storage](docs/value-storage.md).

Indexed generic parameters, constructed references, field substitution, and closed
generic record values, methods, and native layouts are implemented; see [generic types](docs/generic-metadata.md).
Next are further type metadata and storage fundamentals needed for ordinary library
types. Dedicated union opcodes are not planned.

[Marker custom attributes](docs/custom-attributes.md) are supported on types and
methods/functions. The System library supplies UnionAttribute as an ordinary marker;
case behavior lives in ordinary library methods. [Reflection introspection](docs/reflection.md)
now enumerates fields, methods, properties and parameter signatures. Run
`cargo run -- run examples/source/reflection.neo` for the Neo example. Reflective
invocation and guest attribute discovery remain deferred.

Following the strategy review, the [control-flow verifier foundation](docs/verification.md)
is implemented as an explicit `verify` command. It checks stack types, call/field
operands, local initialization, returns and managed reference use. Runtime guards
enforce liveness and output assignment; broader lifetime analysis remains pending. The
[construction, mutation, and initialization proposal](docs/construction-and-initialization.md)
connects the next type-system decisions with verification and a future high-level
language for the runtime library. Whole-value constructors and reference receivers
are implemented; partial initialization, readonly access and placement construction
remain proposals.

[Execution architecture](docs/execution-architecture.md) treats interpretation, JIT,
and native AOT as platform-wide targets, with shared semantics and explicit capability
boundaries. Only interpretation exists today. Embedding and a future high-level
language are separate architectural requirements; hosting does not define the modes.

[Function identities](docs/member-identities.md) now preserve module-local call targets
through linking and generic specialization. Explicit `@ Module:index` references can
distinguish overloads with identical substituted signatures.

[Type identities](docs/type-identities.md) preserve module-local type definition rows
and expose resolved closed signature keys, including generic arguments and pointers.
Exact revision identities and scoped origin checks are implemented. Internal scoped
type keys, colliding type names and content provenance remain future work.

[LoadedProgram](docs/loaded-program.md) prepares an immutable metadata snapshot shared
by execution, verification, and type queries. Run `cargo run --example loaded_program`
to prepare HelloWorld once and execute it twice with fresh guest state.

[Resolved function invocation](docs/invocation.md) supports repeated calls to static and instance
IL functions with typed primitive and [owned record arguments](docs/record-inputs.md),
including libraries without an entry point. [Bootstrap Option/Result inputs](docs/union-inputs.md)
are validated by type, case, and payload.
Run `cargo run --example invoke` for the Rust embedding sample.
`cargo run --example record_inputs` demonstrates copied record inputs and result reuse.
`cargo run --example instance_invocation` demonstrates explicit copied generic receivers.
`cargo run --example union_inputs` demonstrates Option/Result arguments and result reuse.
[Cooperative cancellation](docs/cancellation.md) is available through ExecutionOptions.
`cargo run --example cancellation` stops an invocation from another host thread.
[Interrupt and native boundaries](docs/interrupts-and-native.md) define the VM-level
contracts available to managed and native-oriented language profiles.
[Native lowering](docs/native-lowering.md) keeps the typed IL directly translatable
to machine instructions, with runtime services remaining explicit boundaries.
[Opcode policy](docs/neoil.md#opcode-compatibility-policy) records CLI-aligned
instructions, neoCLR additions, deviations, and temporary bootstrap operations.
[Dynamic dispatch](docs/dynamic-dispatch.md) is an opt-in VM service for dynamic
languages, with language-provided handlers and explicit safety/resource contracts.
[Compatibility strategy](docs/compatibility-strategy.md) describes how familiar CLR
concepts and deliberate neoCLR improvements coexist during migration.
[Closed call-graph analysis](docs/reachability.md) follows explicit roots for backend planning.
`cargo run --example reachability` reports HelloWorld's IL calls, runtime import, and
[runtime-service requirements](docs/runtime-services.md).
[Fault stack snapshots](docs/stack-traces.md) preserve logical frames and method identities.
`cargo run --example stack_trace` formats a snapshot after the loaded program is dropped.
Debug-source resolution and guest StackTrace/StackFrame types remain pending.
[Explicit target layouts](docs/target-layout.md) support storage planning independently of the host.
`cargo run --example target_layout` compares two pointer-width/alignment choices.
[Initial String methods](docs/text-model.md) provide concatenation, equality, and checked UTF-8 slicing.
`cargo run -- run examples/strings.neoil` demonstrates Unicode text and recoverable slice errors.
[System.Array<T> buffer descriptors](docs/arrays-and-pointers.md) provide explicit allocation/free
and checked access for native-layout elements. `cargo run -- run examples/arrays.neoil`
demonstrates descriptor aliasing and mutation; `examples/array_bounds.neoil` produces a bounds Fault.
`cargo run -- run examples/arrays_stack.neoil` demonstrates a frame-local array backed by `localloc`.

[Explicit module sets](docs/module-sets.md) support additional libraries, cross-module
generic calls, and field-name aliases. Run `cargo run --example modules` for a three-module
sample. Optional [`.references` lists](docs/module-references.md) enforce direct module
dependencies. [Scoped type operands](docs/scoped-types.md), such as `[Models]Box<Int32>`,
check the definition's module. Optional [artifact revision labels](docs/module-revisions.md)
support exact dependency pins and definition identities. Duplicate type names and
side-by-side module versions remain pending.

[Ordinary nested types](docs/nested-types.md) support generic cases under non-generic
companions. Run `cargo run -- run examples/nested_types.neoil` for a minimal example.
Non-generic unions can nest cases directly; `cargo run -- run examples/non_generic_union.neoil`
demonstrates that form.
`cargo run -- run examples/system_companions.neoil` constructs a nested System result case.
`cargo run -- run examples/union_extract.neoil` demonstrates discriminator-then-accessor extraction.

neoCLR is licensed under the [MIT License](LICENSE). [Third-party notices](THIRD_PARTY_NOTICES.md)
record the locked dependency licenses, including native libffi and its build-tool notices.

[Preview 1 release notes](docs/preview-1-release-notes.md) describe this experimental
source release. [Source packaging](docs/source-release.md) records the audit and
archive validation process.

[Managed references in Raven-like pseudocode](docs/references-in-pseudocode.md)
explains &value, T&, output parameters, reference receivers, interface views and
union TryGet methods alongside their executable neoIL equivalents.

[Checked reference returns](examples/reference_returns.neoil) demonstrates returning
a reference to a caller-owned field and updating that field through the result.

[Managed value initialization](docs/managed-initialization.md) defines initobj for
existing typed slots, outputs and constructor receivers.

[Managed arrays](docs/managed-arrays.md) now support frame-owned values and GC heap
allocations with common checked element operations and managed element references.

[Neo interfaces](docs/neo-interfaces.md) demonstrate declared contracts, record
implementations and managed reference projections without boxing. Run
`cargo run --locked -- run examples/source/interfaces.neo --gc-stats`.

## Debug a running program

Launch the [interactive terminal debugger](docs/debugger.md):

```sh
cargo run --locked -- debug examples/source/debugger.neo
```

Use `source`, `bt`, `stack`, `heap`, `step`, `next` and `continue`. `watch` enables
live inspection; `pause` freezes execution at an instruction boundary. Neo source
locations and local labels survive compilation into JSON artifacts.


### Managed callbacks

[Delegates](docs/delegates.md) provide checked static/heap-bound callbacks with ordinary
Invoke calls. Neo automatically wraps method groups when a delegate type is expected
and uses function-style invocation. The Func family includes Void results, so
Array.ForEach takes Func<T,Void> instead of a separate Action<T> delegate.
Contextual lambdas share captured bindings in managed storage, including returned
and nested closures; captured references must satisfy heap-lifetime checks.

```sh
cargo run --locked -- run examples/source/func-callbacks.neo
cargo run --locked -- run examples/source/closures.neo
```

### Common comparison and iteration

[Comparable, Iterable and Iterator](docs/common-interfaces.md) provide scalar ordering
and managed collection traversal. ArrayList supports independent iterators through
readonly Iterable views; Iterator exposes MoveNext, Current and Dispose.

```sh
cargo run --locked -- run examples/source/common-interfaces.neo
```

[Eager predicate searches](docs/predicate-search.md) add ArrayList.Find, FindIndex
and Exists using Func<T,Boolean>. Find returns Option<T>; custom equality uses
readonly Equatable implementations. LINQ remains future work.

```sh
cargo run --locked -- run examples/source/predicate-search.neo
```

[Ordinal text operations](docs/ordinal-text.md) provide explicit comparison and
prefix/suffix/containment checks. The file-name search example combines text,
ArrayList predicates and Option matching:

```sh
cargo run --locked -- run examples/source/ordinal-text.neo
```

[Character classification](docs/character-classification.md) adds familiar System.Char predicates and
Neo character literals, including Unicode IsDigit and explicit IsAsciiDigit.

[Fundamental Math operations](docs/math.md) add integer helpers, typed Clamp and core
Double functions. Neo supports Double literals and same-type arithmetic/comparison;
`examples/source/math.neo` demonstrates the APIs.

[Date and Time core values](docs/date-time.md) provide validated factories, readonly
components and value comparison. Run `examples/source/date-time.neo` for leap-date,
time-of-day and typed-error handling; parsing and formatting remain planned.

The [local system clock](docs/local-clock.md) provides Date, Time and UTC offset in one reading.
Run its Neo sample with `cargo run --locked -- run examples/source/local-clock.neo`.

The [Environment API](docs/environment.md) exposes guest arguments, current directory and
optional process variables; see `examples/source/environment.neo`.

The [file report example](docs/file-output.md) combines guest arguments, paths and bounded
UTF-8 input/output with typed Results.
