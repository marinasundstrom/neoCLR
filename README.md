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

## Run

Install stable Rust with support for edition 2024 (Rust 1.85 or newer) and a native
C toolchain for vendored libffi (compiler/make on Unix, MSVC tools on Windows), then:

```sh
cargo run -- run examples/hello.neoil
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

HelloWorld prints:

```text
Hello, world!
=> Void
```

The feature tour checks record copying, explicit heap identity, free functions,
loops, arithmetic, `Option<Void>`, and `Result<Void,Error>`. Together the samples
contain every implemented opcode. The fault sample intentionally terminates with
a nonzero exit code and an instruction location.

Assemble to a standalone prototype module, then execute that module:

```sh
cargo run -- assemble examples/hello.neoil hello.neo.json
cargo run -- check hello.neo.json
cargo run -- run hello.neo.json
```

Declarations and calls carry parameter signatures, for example
`.function Describe(int32) -> string` and
`call System.Console.WriteLine(string)` or `call Initialize()`. Functions overload
by name and ordered parameter types; return types alone do not distinguish them.
The overload sample demonstrates both user-defined and bootstrap library overloads.
See [the assembly reference](docs/neoil.md) for primitive aliases and syntax.

The assembler refuses to overwrite an existing output file. `check` validates
metadata and instruction operands; it is **not** a complete static verifier.
Evaluation-stack types, initialization, and return contracts are checked as code
executes. `run` accepts `.neoil` source directly or serialized JSON modules.

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
[migration and roadmap](docs/roadmap.md). Work and validation are recorded in
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
