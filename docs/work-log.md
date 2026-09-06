# Work log

## 2026-09-06 — Initial interpreter and assembler proof of concept

Started from an empty repository. Added a standalone Rust crate with a loader,
typed metadata/IR, line-oriented assembler, iterative interpreter, and command-line
`assemble`, `check`, and `run` operations. JSON serialization is a temporary means
of testing the interpreter and assembler independently; final CLI-based binary
encoding is a separate milestone.

Implemented owned records, explicit shared heap references, first-class Void,
constructed Option/Result types, free functions, locals, arithmetic, branches,
terminal Faults, and three bootstrap library functions. The interpreter checks
executed operations and bounds execution resources. No guest null or exception
instructions exist.

Added HelloWorld, a feature tour containing all 35 implemented instruction forms,
and an intentionally failing Fault sample. The feature tour checks copy isolation,
heap aliasing, unit-valued unions, recoverable numeric errors, and control flow.
CLI integration tests cover assembly to disk, metadata checking, execution, output,
refusal to overwrite existing output, and terminal Fault exit status.

Recorded the type/storage philosophy, prototype contracts, opcode reference,
array/pointer proposals, migration direction, and next milestones. The README
contains build/run instructions and explicit implementation boundaries. Clarified
that assembly source syntax can change independently while metadata and CIL should
remain CLI-based wherever the intended semantics permit. Ordinary arithmetic wraps;
checked arithmetic uses the familiar `.ovf` forms and terminal Faults.

Validation on macOS ARM64 with Rust 1.95.0:

- `cargo fmt --check`: passed.
- `cargo clippy --locked --all-targets -- -D warnings`: passed.
- `cargo test --locked`: 19 integration tests passed; several are table-driven.
- `cargo test --locked --release --test runtime`: all 12 runtime tests passed.
- HelloWorld prints `Hello, world!` and returns Void.
- Feature tour prints `All feature checks passed.` and returns `Ok(Void)`.
- JSON round-trip execution agrees with execution from assembled source.

Added Linux/macOS/Windows CI. Only local macOS execution has been observed so far;
the CI matrix has not been run in this session.

Open work includes a full verifier, actual CLI binary metadata/CIL emission,
owned arrays and checked borrows, general generics/unions, interfaces, library
modules, heap reclamation, .NET migration tooling, and future async/native interop.
Physical native-stack allocation has not been demonstrated by this interpreter.

## 2026-09-06 — Explicit call signatures and overloads

Calls now use `Name(Type, ...)`, including `Name()` for parameterless functions.
HelloWorld uses `call System.Console.WriteLine(string)`. Primitive aliases are
normalized by the assembler; structured metadata retains canonical types. Nested
constructed types and their internal commas are accepted in parameter lists.

Function identity is name plus ordered parameter types, enabling overloads by
arity/type/order. Return types alone cannot distinguish overloads. The loader
rejects duplicate signatures and unresolved call targets; execution checks actual
arguments against the explicitly selected target. `.entry Main` selects `Main()`
regardless of declaration order. Exact intrinsic signatures are reserved while
other overloads of their names are allowed.

Added an Int32 overload of Console.WriteLine and an overload demonstration program.
Updated existing samples/tests to specify call signatures. Prototype metadata
format is now version 2, with call operands carrying name and parameter type array;
format 1 modules must be reassembled. The output remains temporary JSON IR.

Validation: all 28 integration tests pass, including nine new overload tests for
parsing, dispatch, nested types, metadata round trips, duplicate/unknown targets,
entry selection, and intrinsic overloads. Formatting and Clippy checks pass.

## 2026-09-06 — Inline declarations and platform-written System library

Added `.function Describe(int32) -> string` declarations using the same parameter
parser as call operands. Migrated samples and runtime library source to inline
signatures. Legacy headers with `.param` remain accepted, but cannot be mixed with
an inline parameter list.

Moved public System APIs out of interpreter intrinsic dispatch into
`runtime/System.neoil`. The library contains five ordinary functions, including
Result-based Divide and Abs implementations. Added signed `div` with truncation
toward zero and terminal Faults for invalid arithmetic; all 36 instruction forms
are represented in samples. Three host primitives remain for I/O and temporary
Int32 parsing/formatting. These host boundaries are documented.

Added library modules without entry points, bootstrap linking, cached default
System compilation, and an explicit compiled-library CLI/embedding execution path.
Tests prove a supplied serialized library body executes and that library calls use
guest frames/budgets. General dependency resolution and final CLI binary emission
remain unimplemented.

Recorded .NET/CLR semantics as the default outside intentional deviations. Added an
assembler design document requiring eventual full platform expressiveness, with
.NET assembly language as the capability baseline. The current parser and metadata
subset, including parameter-only overload keys, are not a permanent ceiling.

Validation: `cargo test --locked` passes all 36 integration tests; `cargo fmt --check`,
`cargo clippy --locked --all-targets -- -D warnings`, and `git diff --check` pass.

## 2026-09-06 — Native implementation metadata and API familiarity

Added `.methodimpl InternalCall` on bodyless runtime function declarations. It
emits `impl_flags: 4096`, using the CLR InternalCall value. Ordinary functions use
zero. This implements the metadata target for a future familiar MethodImpl attribute
form; general custom-attribute syntax remains pending.

Moved host binding into an explicit typed registry. Calls resolve a declaration
first, then its implementation flag selects IL or native execution. The binder
checks name, ordered parameters, and return type. Unknown bindings/flags, native
bodies/locals, and missing declarations fail validation. Tests confirm a familiar
native helper name without the flag runs its IL body instead of triggering host
code. The System library now declares its three host dependencies explicitly.

Added an API policy: retain familiar .NET namespaces, type/member names, overloads,
and behavior except where intentional platform semantics require change. Marked
Int32.Divide as an experimental extension and runtime helper names as implementation
details. API familiarity does not require identical implementation, and prototype
limitations are not permanent API design decisions.

Validation: all 41 integration tests pass, including five new metadata/binding
checks. Clippy, formatting, and diff whitespace checks pass.
