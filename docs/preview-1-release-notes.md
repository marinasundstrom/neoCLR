# neoCLR Preview 1 — draft release notes

Status: draft for review. No release candidate, final version, tag or publication date
has been selected. Cargo's development version is currently 0.1.0. See the
[release checklist](preview-1.md) and [validation evidence](preview-1-validation.md).

neoCLR is an experimental, standalone .NET-inspired virtual machine. This first
source preview demonstrates a platform where values are the default and allocation,
reference access and memory-management policies are explicit choices. Familiar CLI
concepts provide a starting point while leaving room for deliberate changes.

## Included in the source preview

- A neoIL assembler, metadata loader, optional typed control-flow verifier and
  cross-platform interpreter, built together as one executable.
- Primitive arithmetic, control flow, free functions, overloads, generic types,
  ordinary records, constructors, accessibility and properties/indexers.
- A small platform-written System library, split into namespace folders, with
  console byte input and line output, strings, numeric operations and bounded file input.
- Ordinary Option and Result carriers with nested cases and overloaded
  TryGet(out Case& value) extraction; recoverable failures use typed Error values.
- Terminal Faults with logical stack frames available to the CLI and embedding host.
- Explicit native allocation/free and pointers, stack/heap array views and a small
  native-layout ArrayList<T> with explicit release.
- Call-scoped managed references, out and conditional-output contracts, explicit
  reference receivers and interface views. List<T> and Equatable<T> demonstrate
  collection access and typed equality without implicit boxing or ownership.
- Minimal read-only System.Type inspection, experimental Rust embedding and a
  scalar/pointer native-call subset.

[The walkthrough](preview-1-walkthrough.md) includes HelloWorld, console calculation,
array loops, file summaries, error handling, union extraction, references and equality.
[Raven-like pseudocode](preview-1-programs.md) explains how a future language could
lower these programs. It is explanatory notation, not an implemented compiler.

## Build and try it

Rust 1.85.0 is the minimum. Install the native build prerequisites documented in the
[README](../README.md#build-and-run-a-sample), then run from the source root:

```sh
cargo run --locked -- run examples/hello.neoil
cargo run --locked -- run examples/equatable.neoil
cargo run --locked -- run examples/console_input.neoil
```

Cargo downloads locked dependencies on the first build. No .NET installation is
needed. The System library is bundled automatically. The README also covers separate
assembly, verification, external System artifacts, embedding and native sample builds.

## Deliberate differences and current limits

Types do not have a class/struct bit choosing reference versus value semantics. An
ordinary T is copied by value; T& explicitly accesses a caller's slot, and T* exposes
a native pointer contract. Managed references currently stay within active guest
calls. Writable aliases are permitted; a Rust-style borrow checker is not required.
Value semantics do not prove physical native-stack allocation in this interpreter.

Void is a real value and type argument. A Void-returning call leaves one Void value
on the evaluation stack. Expected failures use Result rather than guest exceptions;
Faults terminate execution and do not provide catch/unwind or automatic cleanup.

System.Value remains temporary interpreter storage for library payloads and some
host boundaries. Packing allocates an owned host value tree and copying recursively
copies owned payloads. This is not a settled native representation. Its
[retirement](value-storage.md#retirement-decision) needs a coordinated payload,
copy and lifetime migration. The tagged Void* example covers borrowed native-layout
storage, not arbitrary String/error/nested-carrier ownership.

The current artifact is prototype JSON format 4, not a CLI binary. It removes old
union-specific encodings; reassemble older artifacts, including System. Standard
CLI instruction names are retained where appropriate, and extensions/temporary
operations are listed in the [opcode reference](neoil.md#opcode-compatibility-policy).

Only interpretation is implemented. JIT and native AOT inform the architecture but
are not execution modes in this preview. There is no completed reference-counted
Ref<T>, GC, automatic destruction, concurrency model, high-level compiler or .NET
assembly importer. Reflection, collections and interop intentionally cover small
subsets. The verifier is optional and conservative; successful verification does
not guarantee memory safety for arbitrary native pointer operations. No production
sandbox, stable ABI, binary compatibility or performance parity is claimed.

## Validation and remaining release gates

Local macOS ARM64 evidence includes a clean Rust 1.85.0 build, an uninterrupted
521-test run, and source/artifact, external-library, embedding and native workflows.
Stable-toolchain regression coverage and strict Clippy are also recorded. CI is
configured for stable and minimum Rust on Linux, macOS and Windows; results for an
exact release candidate have not yet been recorded. These draft notes do not announce
validated support for every OS/CPU combination.

Before publication, choose the candidate/version, collect its platform results,
complete first-install and final source-package checks, and finalize these notes.
The project is MIT-licensed. [Third-party notices](../THIRD_PARTY_NOTICES.md) inventory
locked dependencies, their license texts and the separate native libffi/build-tool
notices. Dependencies are downloaded by Cargo and are not vendored in the source archive.
