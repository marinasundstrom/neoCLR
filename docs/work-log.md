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
