# Preview 1: public source release

Target: a runnable developer source preview containing the neoIL assembler, interpreter,
small platform-written runtime library, and demonstrative programs. This publication
shape was selected on 2026-09-07. Prebuilt binaries and a high-level compiler are not
required. This document defines the proposed feature and release gates for that target;
it does not declare the current checkout ready for Preview 1 or authorize publication.

The milestone's name is **Preview 1**. Its behavioral scope is defined by the six
[Raven-like program contracts and IL mappings](preview-1-programs.md). This checklist
tracks the supporting capabilities and publication work. The pseudocode is explanatory;
hand-authored neoIL equivalents supply runnable acceptance evidence until a frontend exists.

## What a reader should be able to do

From a fresh clone, a developer can build without .NET installed, assemble and verify
a small program, run it, and understand the deliberate semantic differences. They can
read console input, compute a result, handle an expected Error, and see an actionable
Fault trace when an execution contract is violated. They can inspect the library's own
IL and see ordinary types implementing Option and Result rather than hidden union opcodes.

Position the release as an original, experimental .NET-inspired VM with familiar CLI
concepts. Do not claim .NET binary compatibility, a production runtime, a security
sandbox, stable metadata/hosting ABI, or replacement of the .NET ecosystem. The platform
may remain unnamed; neoCLR is sufficient as the runtime codename for the preview.

## Required capability and current gap

| Area | Required for Preview 1 | Current state / remaining work |
| --- | --- | --- |
| Build and tooling | Locked source build; assemble, check, verify, and run workflows; clear malformed-input diagnostics | Implemented; fresh-clone and declared toolchain/platform validation remain release gates |
| Execution fundamentals | Primitive values, free functions, calls/overloads, indexed parameters/locals, control flow, real Void, value copying | Implemented; preserve semantics through remaining changes |
| Metadata and modules | Explicit types/members/signatures, generic type definitions and closed use, properties/accessors, module references and member identities | Implemented; current JSON is a documented prototype representation |
| Ordinary type contracts | Public/internal type visibility, public/internal/private members, usable constructors with defined receiver and initialization behavior | Visibility, properties and whole-value constructor initialization implemented; carrier requirements still need proving |
| Ordinary unions | Carrier/variant convention sufficient for Option<T> and Result<T,TError>, including Void, nested carriers, and Result<T,T> | Selected convention, System carriers and bounded erased host input implemented; existing API/native/bootstrap host-path migration remains |
| No union-specific IL | Library, samples, native result construction and host inputs use ordinary types; remove special union operations and encodings | Not done; current some/none/ok/err/is.case/ldcase and special Option/Result handling are bootstrap debt |
| Minimal library and console | Useful Int32/String/Error APIs; console input/output; EOF distinct from input failure; an interactive program | Implemented using raw byte input and immediate line output; `ReadByte` uses the ordinary nested Result/Option boundary. General ReadLine is not required |
| Minimal reflection | Obtain a System.Type-style descriptor from a type token or value, compare identity, read its name, inspect closed generic arguments | Existing host type identities are groundwork; guest read-only type inspection is not implemented |
| Memory and arrays | Explicit pointers/allocation/free, record storage, small usable array/buffer example, documented lifetime and copy rules | Implemented; native-layout Array<T> descriptors are sufficient. No promise of general owned arrays or automatic cleanup |
| Errors and diagnostics | Recoverable failures through Result; terminal Faults with owned logical stack frames shown by CLI/host | Implemented with bootstrap Results; migrate them. Source-line maps and guest StackTrace classes are not required for Preview 1 |
| Embedding and native boundary | One runnable embedding example and one supported scalar/pointer native interop example | Implemented experimental Rust hosting and P/Invoke subset; validate the published examples and platform requirements |
| Publication readiness | Accurate README/design limits, explicit license, release notes, tested supported platforms and source release instructions | Work remains; no LICENSE file was found during this planning review |

"Implemented" does not mean release-validated on every platform. Local validation so far
is on macOS ARM64; CI is configured for Linux, macOS, and Windows, but this plan does not
claim that a particular release commit has passed those remote jobs.

Preview 1 can retain narrow, explicitly labeled bootstrap host helpers for console, text,
numeric conversion and I/O where platform code still lacks the necessary primitives.
It cannot retain union-specific execution as the final Option/Result implementation.
A JSON artifact format is acceptable for the source preview; a CLI binary reader/writer
and compatibility with existing .NET tooling are later work, not achieved by using JSON.

## Remaining implementation order

1. **Use the executable construction/storage foundation.** [Constructors](constructors.md)
   initialize whole values; [explicit typed storage](value-storage.md) lets a carrier
   retain one complete wrapper without inactive defaults. The prototype needs no
   addressed receivers, native overlapping layout or implicit ownership model.
2. **Use the selected member convention.** The [constructor/query contract](union-convention.md)
   defines permitted wrappers and match lowering through ordinary calls/branches.
   A future compiler/tooling recognizer is separate; the VM attaches no special semantics.
   Add [non-generic companions with ordinary nested generic cases](nested-types.md):
   name/arity distinction, nested ownership and read-only type descriptors are implemented; library case migration
   is next.
3. **Preserve the tested ordinary library carriers through migration.** System.Option and
   System.Result cover None versus Some<Void>, Ok<Void>, Result<T,T>, nested carriers,
   failed queries and independent value copies. Fully qualified names select them while
   short Option/Result spellings still denote bootstrap categories. Migrate existing
   APIs and host/native adapters before removing that temporary distinction.
   Int32.Parse, Math.Abs, Console.ReadByte and File.ReadAllText use canonical ordinary
   nested-case returns, including the console/file native boundaries. There are no
   parallel Typed APIs. Divide and slicing still use bootstrap carriers.
4. **Replace and remove the bootstrap union system.** Migrate library methods, native/host
   adapters, samples, and tests to ordinary types and calls. Delete the six special
   instructions and special Option/Result type/value dispatch. Make the metadata break
   explicit: version the new representation and clearly reject unsupported old artifacts;
   an automatic migration tool is not required for this preview.
5. **Add minimal read-only type inspection.** Expose type identity, names and closed
   generic arguments through a small System.Type-style API. Specify descriptor lifetime
   and identity scope. Inspection grants no invocation, construction or mutation rights.
   Use existing metadata; do not introduce a second type model or reflection-dependent
   union execution. Member enumeration and dynamic invocation remain outside Preview 1.
6. **Freeze the demonstration set and complete release validation.** Correct documentation
   against the final implementation, run the release checks below, and resolve licensing.
   Fix failures and contradictions before adding more features.

The interpreter's carrier-storage decision and limits are recorded in
[value storage](value-storage.md). API/host/native migration, bootstrap removal and
minimal guest inspection remain substantial work. The
prototype is not evidence of a settled native ABI or a publication date.

## Required demonstrations

Each demonstration must be checked in, work offline after dependencies are installed,
have documented commands and expected output/exit status, and run in automated validation.
The program contracts P1–P6 in the [separate mapping document](preview-1-programs.md)
define the central behavioral set; embedding/native checks below cover its platform boundaries.

- **HelloWorld:** build, assemble to an artifact, verify, run, and print a line.
- **Interactive computation:** prompt before input, read a number, compute a result,
  distinguish EOF/empty input, report an ordinary input Error, and finish normally.
- **Ordinary library types and unions:** construct a type with non-public representation,
  read it through an accessor, copy/update explicitly, and handle Option/Result cases
  through the final ordinary member contract. No bootstrap union opcodes may remain.
- **Read-only type inspection:** obtain and compare descriptors for primitives and a
  closed generic record, display their names, and inspect the generic arguments. Verify
  distinct type/module identities and document descriptor lifetime.
- **Explicit memory and arrays:** allocate, initialize, read/update, and free a small
  buffer. A separate invalid-access example must terminate with a useful Fault trace.
- **Embedding and native calls:** invoke a public function from a host with owned inputs
  and host-supplied console I/O; build a tiny local native library and call the supported
  ABI subset. Neither example requires networking or external accounts.

The existing file-input example can ship as an additional bounded integration. It does
not justify expanding Preview 1 into streams, filesystem abstractions or networking.

## Release acceptance checklist

- [ ] The ordinary constructor/carrier milestones above are implemented and tested.
- [ ] No special union instructions or Option/Result runtime categories remain; final
      samples and library code execute through ordinary metadata and IL operations.
- [ ] Minimal guest type inspection is implemented without dynamic invocation, reflective
      construction, field mutation, or an accessibility bypass.
- [ ] All required demonstrations run from source and, where applicable, assembled artifacts.
- [ ] `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, and
      `cargo test --locked` pass for the exact proposed release commit.
- [ ] Linux, macOS and Windows source builds/tests and representative console/native samples
      pass in CI for that commit. Record actual architectures; do not imply every OS/CPU
      combination is supported. Narrow any unsupported claim explicitly before release.
- [ ] Verify the minimum Rust toolchain claimed in the README, record it in package metadata,
      and test it as well as the current stable toolchain. CI currently uses stable only.
- [ ] A fresh clone follows the README successfully, including native build prerequisites,
      external sample data paths, expected failure exits and output-file overwrite rules.
- [ ] README and API/metadata documents distinguish implemented behavior, intentional
      divergences, temporary helpers, unsafe/trusted boundaries, and deferred features.
- [ ] Select and add a repository license approved by the project owner; verify source
      provenance and required dependency notices. Do not infer a license from public hosting.
- [ ] Choose the preview version/tag and add release notes with capabilities, known limits,
      breaking metadata changes, build instructions, and tested platform evidence.
- [ ] Confirm the source release contains the lockfile, runtime IL, fixtures, samples,
      docs and tests, with no generated local artifacts or credentials.

A crates.io release is not part of this target; Cargo.toml currently has publish=false.
Publishing a work-in-progress repository earlier is a separate choice from declaring a
Preview 1 release. This plan performs neither a remote push nor a publication.

## Explicitly outside Preview 1

- A C# or Raven frontend, compiler self-hosting, or rewriting the library in that language.
- JIT or native AOT execution. Preserve their architectural requirements without making
  backend implementation a source-preview gate.
- GC, production reference counting, automatic destruction, or a universal ownership model.
  Existing bootstrap Ref behavior must not be advertised as a completed counted Ref<T>.
- Inheritance, interface/virtual dispatch, closed hierarchies, member enumeration, dynamic
  invocation, reflective construction/mutation, broad reflection or redesigned
  access modifiers. Keep the familiar implemented access levels for now.
- A Stream hierarchy, sockets, HttpClient, runtime async or general concurrency libraries.
- Full Unicode indexing/decoding APIs, general collections, multidimensional/covariant arrays,
  and comprehensive framework parity.
- Guest StackTrace/StackFrame APIs, source-level debugger integration and source-line mapping;
  the first release still requires terminal Fault traces through the existing host/CLI path.
- .NET assembly import/migration, CLI binary compatibility, production hardening, sandboxing,
  performance parity, stable ABI guarantees and prebuilt release packages.

These remain future directions. Add an item to Preview 1 only when it is needed to satisfy
one of the required demonstrations or a concrete correctness/release requirement.

## Minimal reflection boundary

Include read-only type inspection because it makes the platform's type model observable
and demonstrates that primitives and records share that model. The required behavior is
obtaining a descriptor for a statically named type and for a value, comparing canonical
identity, reading a display/name representation, and inspecting closed generic arguments.
Exact method names and handle encoding are implementation choices to settle in that slice.

Reflection should describe neoCLR's own type and execution model. Familiar System.Type
concepts are a starting point, not a requirement to reproduce the .NET reflection object
hierarchy, historical distinctions, or behavioral conventions. Its public structure and
behavior may intentionally evolve with the runtime. Document those choices and any
migration implications; the preview makes no reflection compatibility or stability promise.
This freedom does not expand Preview 1 beyond the small read-only subset defined here.

A descriptor is metadata, not an allocation policy, universal object reference, or native
address. Its lifetime must remain valid for every supported use, including values returned
to an embedding host. Names alone must not define identity. Inspecting an internal type
must not create permission to call its members or construct it through a type token.

Keep this independent of member enumeration, runtime code generation and JIT availability.
Document the metadata needed by the descriptors so a future AOT backend can retain it
explicitly. Read-only method/field/property enumeration can follow after Preview 1 if it
proves useful; it is not needed to execute the ordinary union convention.
