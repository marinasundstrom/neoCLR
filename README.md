# neoCLR

neoCLR is the codename for an experimental runtime for an unnamed, .NET-inspired
platform. This repository contains a small standalone Rust interpreter, a neoIL
assembler, a prototype metadata format, and executable samples. It does not need
an installed .NET runtime.

Assembly source syntax is free to evolve. The intended emitted metadata and CIL
remain based on the CLI model so existing parsing/tooling can be reused where
possible; deviations should be driven by a semantic requirement. The current JSON
output is a temporary internal format, not that final binary representation. See
[format direction](docs/format-direction.md).

The goal is familiar runtime and library structure with deliberately different
semantics: one model for data types, frame-owned values by default, explicit shared
allocation, a real `Void` value, functions outside types, `Option<T>` for absence,
`Result<T,E>` for recoverable errors, and terminal Faults instead of exceptions.
Interfaces will use ordinary names without an `I` prefix.

## Run

Install stable Rust with support for edition 2024 (Rust 1.85 or newer), then:

```sh
cargo run -- run examples/hello.neoil
cargo run -- run examples/features.neoil
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

The assembler refuses to overwrite an existing output file. `check` validates
metadata and instruction operands; it is **not** a complete static verifier.
Evaluation-stack types, initialization, and return contracts are checked as code
executes. `run` accepts `.neoil` source directly or serialized JSON modules.

## What is implemented

- Module, type/field, and free-function metadata, with typed parameters and locals.
- `Void`, `Int32`, `Boolean`, `String`, `Error`, records, and constructed
  `Option<T>`, `Result<T,E>`, and `Ref<T>` types, including nested uses of `Void`.
- An iterative interpreter with explicit call frames and an IL-style evaluation stack.
- Frame-owned aggregate copying and explicit heap allocation/sharing.
- A line-oriented assembler with quoted strings, labels, source-line diagnostics,
  and forward function/type references.
- Three bootstrap library intrinsics: `System.Console.WriteLine`,
  `System.Int32.Parse`, and `System.Int32.Divide`.
- Configurable limits on instruction count, call depth, stack slots per frame,
  and explicit heap object count.

This is an original experiment informed by the
[ECMA-335 CLI specification](https://ecma-international.org/publications-and-standards/standards/ecma-335/),
particularly its metadata and CIL partitions. It is not a fork of CoreCLR and does
not yet import CLI metadata, execute .NET assemblies, or provide binary compatibility.
The prototype's JSON format is not a proposed replacement binary encoding.

## Boundaries

“Stack by default” is a **guest semantic model** here: interpreter frames own
values, and copying an object copies its fields. Rust currently uses `Vec`,
`String`, and `Box` internally, so this does not demonstrate physical native-stack
allocation or its performance. Explicit guest references address a heap arena
retained until the execution result is dropped. There is no GC or individual free.

The library is a bootstrap surface, not a complete BCL. General user-defined
generics/unions, interfaces/dispatch, arrays, borrows, raw pointers, native interop,
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
