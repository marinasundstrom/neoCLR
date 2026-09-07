# neoCLR

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
Interfaces will use ordinary names without an `I` prefix.

## Current focus

The first native heap/pointer slice is implemented: explicit allocation/free,
native addresses, pointer casts and byte offsets, indirect access, and sequential
record layout. `localloc` provides explicit frame-local byte storage, released on
return. Native integers and explicit pointer/address conversions are also
available. See [heap and pointers](docs/heap-and-pointers.md).
Reference counting, GC, and higher-level lifetime management remain deferred.
Types describe values and behavior; allocation and lifetime are separate choices,
with no class/struct bit deciding either. The proof of concept aims to make migration
familiar where possible, without committing every program to one memory model.

## Build and run a sample

Run these commands from the repository root. Install stable Rust 1.85 or newer
(edition 2024) and a native C toolchain for vendored libffi: compiler/make on Unix,
or MSVC tools on Windows. No .NET installation is required.

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
cargo run --locked -- run examples/arrays.neoil
cargo run --locked -- run examples/array_bounds.neoil
```

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

This is bootstrap linking against one System library, not yet general assembly
reference resolution. See [runtime library design](docs/runtime-library.md).

## What is implemented

- Module, type/field, and free-function metadata, with typed parameters and locals.
- Signature-based overload resolution and structured call references.
- Optional parameter/local names (`string value`, `.local Point point`), preserved
  in metadata; named operands assemble to indices.
- Canonical System primitive definitions, type-owned static methods, and read-only
  instance receiver snapshots; see [type system](docs/type-system.md).
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
  `Option<T>`, `Result<T,E>`, and `Ref<T>` types, including nested uses of `Void`.
- An iterative interpreter with explicit call frames and an IL-style evaluation stack.
- Frame-owned aggregate copying and explicit heap allocation/sharing.
- A line-oriented assembler with quoted strings, labels, source-line diagnostics,
  and forward function/type references.
- A platform-written [System library](runtime/System.neoil): overloaded
  `System.Console.WriteLine`, `System.Int32.Parse`, `System.Int32.Divide`, and
  `System.Math.Abs`, plus instance `Int32.ToString()`, assembled into the same metadata/IL representation as apps.
- Explicit `.pinvoke` imports with dynamic library loading and scalar/pointer C-ABI calls.
- Three host primitives for console output, Int32 string conversion, and parsing,
  explicitly declared with CLR-style `MethodImpl`/`InternalCall` metadata.
- Configurable limits on instruction count, call depth, stack slots per frame,
  explicit heap object count, native payload bytes, and allocation identities.

This is an original experiment informed by the
[ECMA-335 CLI specification](https://ecma-international.org/publications-and-standards/standards/ecma-335/),
particularly its metadata and CIL partitions. It is not a fork of CoreCLR and does
not yet import CLI metadata, execute .NET assemblies, or provide binary compatibility.
The prototype's JSON format is not a proposed replacement binary encoding.
Current format version 3 records type representations, method owners, and instance
call form. Reassemble earlier source and System artifacts before loading them.

## Boundaries

“Stack by default” is a **guest semantic model** here: interpreter frames own
values, and copying an object copies its fields. Rust currently uses `Vec`,
`String`, and `Box` internally, so this does not demonstrate physical native-stack
allocation or its performance. Legacy `Ref<T>` values address an arena retained until the execution result is
dropped. Native `Ptr<T>` allocations separately support individual free. There is
no reference counting or GC. Counted `Ref<T>` remains a deferred ownership abstraction;
Rust's memory model does not define guest behavior. See [memory layers](docs/memory-model.md).

The library is a bootstrap surface, not a complete BCL. General user-defined
generics/unions, interfaces/virtual dispatch, arrays, borrows, full native marshalling,
threading, runtime async, JIT compilation, and full verification are unimplemented.
Resource limits are guardrails, not a memory quota or a hostile-code sandbox.
Console output is collected and emitted by the CLI only on successful completion.

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

The CI matrix runs these checks on Linux, macOS, and Windows. Local validation
has been performed on macOS ARM64; the other platforms still require CI execution.

Typed memory initialization/copying (`initobj`, `cpobj`) and byte-range operations
(`initblk`, `cpblk`) are implemented with explicit pointer checks. See
[heap and pointer semantics](docs/heap-and-pointers.md#copying-and-initializing-memory).

Parameter, local, and qualified field names are assembly conveniences mapped to
canonical indices. See [identifier mappings](docs/neoil.md#identifier-mappings-and-field-aliases).

The next fundamental milestone is library-defined Option and Result through minimal
generic support and a convention for ordinary carrier types, inspired by .NET 11.
See the [union convention](docs/unions-and-enums.md);
reflection remains outside that milestone.

Indexed generic parameters, constructed references, field substitution, and closed
generic record values, methods, and native layouts are implemented; see [generic types](docs/generic-metadata.md).
Next are further type metadata and storage fundamentals needed for ordinary library
types. Dedicated union opcodes are not planned.

[Marker custom attributes](docs/custom-attributes.md) are supported on types and
methods/functions. The System library supplies UnionAttribute as an ordinary marker;
union behavior and guest reflection remain pending.

Following the strategy review, the [control-flow verifier foundation](docs/verification.md)
is implemented as an explicit `verify` command. It checks stack types, call/field
operands, local initialization, and returns; reference-lifetime analysis remains pending. The
[construction, mutation, and initialization proposal](docs/construction-and-initialization.md)
connects the next type-system decisions with verification and a future high-level
language for the runtime library; its contracts are proposals, not implemented features.

[Execution architecture](docs/execution-architecture.md) treats interpretation, JIT,
and native AOT as platform-wide targets, with shared semantics and explicit capability
boundaries. Only interpretation exists today. Embedding and a future high-level
language are separate architectural requirements; hosting does not define the modes.

[Function identities](docs/member-identities.md) now preserve module-local call targets
through linking and generic specialization. Explicit `@ Module:index` references can
distinguish overloads with identical substituted signatures.

[Type identities](docs/type-identities.md) preserve module-local type definition rows
and expose resolved closed signature keys, including generic arguments and pointers.
Module-scoped type lookup and revision identities remain future work.

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

[Explicit module sets](docs/module-sets.md) support additional libraries, cross-module
generic calls, and field-name aliases. Run `cargo run --example modules` for a three-module
sample. Optional [`.references` lists](docs/module-references.md) enforce direct module
dependencies. [Scoped type operands](docs/scoped-types.md), such as `[Models]Box<Int32>`,
check the definition's module. Optional [artifact revision labels](docs/module-revisions.md)
support exact dependency pins and definition identities. Duplicate type names and
side-by-side module versions remain pending.
