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
| Ordinary type contracts | Public/internal type visibility, public/internal/private members, usable constructors with defined receiver and initialization behavior | Implemented and exercised by ordinary carrier construction, extraction and copy tests |
| Ordinary unions | Carrier/variant convention sufficient for Option<T> and Result<T,TError>, including Void, nested carriers, and Result<T,T> | Implemented: ordinary carriers and overloaded TryGet(out Case&), using temporary System.Value payload storage; see the storage boundary below |
| No union-specific IL | Library, samples, native result construction and host inputs use ordinary types; remove special union operations and encodings | Implemented in format 4; old artifacts require reassembly |
| Minimal library and console | Useful Int32/String/Error APIs; console input/output; EOF distinct from input failure; an interactive program | Implemented using raw byte input and immediate line output; `ReadByte` uses the ordinary nested Result/Option boundary. General ReadLine is not required |
| Minimal reflection | Obtain a System.Type-style descriptor from a type token or value, compare identity, read its name, inspect closed generic arguments | Implemented via type-only ldtoken, System.Type and declared-type System.TypeOf<T>.Of; no dynamic object dispatch |
| Memory and arrays | Explicit pointers/allocation/free, record storage, small usable array/buffer example, documented lifetime and copy rules | Implemented: native-layout Array<T> views and ArrayList<T> with explicit release. No automatic cleanup or arbitrary-payload collection promise |
| Managed references and interfaces | Call-scoped typed references, output parameters, explicit receiver views and a small useful interface set | Implemented: T&, out/out(true), reference receivers, List<T> and Equatable<T>; no escaping references or automatic ownership |
| Errors and diagnostics | Recoverable failures through Result; terminal Faults with owned logical stack frames shown by CLI/host | All six reviewed library Result APIs use ordinary typed errors. Source-line maps and guest StackTrace classes are not required for Preview 1 |
| Embedding and native boundary | One runnable embedding example and one supported scalar/pointer native interop example | Implemented experimental Rust hosting and P/Invoke subset; validate the published examples and platform requirements |
| Publication readiness | Accurate README/design limits, explicit license, release notes, tested supported platforms and source release instructions | MIT license and dependency notice inventory present; release notes drafted; final provenance, candidate/platform and archive checks remain |

"Implemented" does not mean release-validated on every platform. Local validation so far
is on macOS ARM64; CI is configured for Linux, macOS, and Windows, but this plan does not
claim that a particular release commit has passed those remote jobs.

Preview 1 can retain narrow, explicitly labeled bootstrap host helpers for console, text,
numeric conversion and I/O where platform code still lacks the necessary primitives.
It cannot retain union-specific execution as the final Option/Result implementation.
A JSON artifact format is acceptable for the source preview; a CLI binary reader/writer
and compatibility with existing .NET tooling are later work, not achieved by using JSON.

## Current implementation milestone

The required execution and library foundation is implemented. It includes ordinary
constructors/properties/accessibility, nested generic case types, removal of
union-specific IL, read-only type inspection, arrays and explicit native allocation.
More recent slices add managed output references, reference receivers, interface
views, case-based TryGet extraction and generic typed equality. The console and file
programs use those extraction contracts in ordinary IL.

The [runnable walkthrough](preview-1-walkthrough.md) exercises source and artifact
paths for the principal demonstrations, including references, union extraction and
Equatable<T>. These are local acceptance fixtures; their existence does not certify
a proposed release commit across platforms.

## Temporary storage boundary

The working Preview 1 scope retains the current System.Value implementation as an
explicitly documented interpreter shortcut. This follows the existing
[retirement decision](value-storage.md#retirement-decision): removal is intended,
but a complete object model or payload-storage migration is not silently added to
the first source preview. Retention does not make System.Value a permanent public
platform direction or settle the future System.Object representation.

System.Value packing allocates an owned host value tree; copying recursively copies
owned payloads. This is a real allocation cost. It is not an allocation-free union
representation or a native ABI. Option/Result and several host boundaries still rely
on it, even though their cases, methods and extraction are ordinary library types
and IL. Managed slot references cannot be stored in it.

The tagged Void* sample proves borrowed storage for native-layout payloads. It does
not replace arbitrary String, typed error or nested carrier payload storage. Replacing
System.Value requires explicit representations and allocation/copy/release contracts
for all those cases, followed by one coordinated library, host-boundary and artifact
migration. Neither T& nor Void* alone supplies that ownership contract.

Release documentation must disclose this limitation and the retirement direction.
If complete removal is selected as a Preview 1 requirement, revise the milestone
scope and acceptance tests before starting that migration.

## Remaining work, in order

1. **Finish the documentation audit.** Keep the README, opcode table, API contracts and
   language mappings consistent. Explain value copies, native pointers, managed views,
   interpreter allocation costs, verification limits and the temporary storage above.
2. **Validate the build contract on claimed platforms.** Rust 1.85.0 is recorded in
   Cargo metadata and tested locally from a clean source snapshot. Minimum/stable CI
   jobs are configured for three operating systems. Collect their results and verify
   first-install prerequisites and clean-clone instructions for the release candidate.
3. **Finalize the source package.** Review the [dependency notices](../THIRD_PARTY_NOTICES.md)
   and [source audit](source-release.md) against the candidate. Finish provenance review
   and the [draft release notes](preview-1-release-notes.md), then select the version/tag
   explicitly. Recheck archive membership after any further changes.
4. **Validate the candidate commit.** Run formatting, strict Clippy, the full suite,
   representative embedding/native examples and source/artifact demonstrations on the
   exact candidate. Record OS, architecture and toolchain for local and CI evidence.
5. **Review publication readiness.** Resolve failures or narrow unsupported claims,
   then complete the checklist below. Publication, tagging and pushing remain separate
   actions; completing implementation does not perform them.

See [validation evidence](preview-1-validation.md) for the current local baseline and
what still needs to be recorded. Prioritize failures and gaps in these gates over
expanding the runtime library.

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
- **References and typed equality:** pass caller output slots, extract case values with
  TryGet, dispatch a reference receiver and compare a user-defined value through
  Equatable<T>&. Demonstrate that a view adds no ownership.
- **Embedding and native calls:** invoke a public function from a host with owned inputs
  and host-supplied console I/O; build a tiny local native library and call the supported
  ABI subset. Neither example requires networking or external accounts.

The existing file-input example can ship as an additional bounded integration. It does
not justify expanding Preview 1 into streams, filesystem abstractions or networking.

## Release acceptance checklist

- [x] The ordinary constructor/carrier milestones above are implemented and tested.
- [x] No special union instructions or Option/Result runtime categories remain; final
      samples and library code execute through ordinary metadata and IL operations.
- [x] Minimal guest type inspection is implemented without dynamic invocation, reflective
      construction, field mutation, or an accessibility bypass.
- [x] Managed reference/output contracts and explicit interface dispatch are implemented
      and regression-tested; Equatable<T> demonstrates typed equality.
- [ ] All required demonstrations run from source and, where applicable, assembled artifacts.
- [ ] `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, and
      `cargo test --locked` pass for the exact proposed release commit.
- [ ] Linux, macOS and Windows source builds/tests and representative console/native samples
      pass in CI for that commit. Record actual architectures; do not imply every OS/CPU
      combination is supported. Narrow any unsupported claim explicitly before release.
- [x] Declare the minimum Rust toolchain in Cargo metadata and configure minimum/stable
      CI coverage. Rust 1.85.0 build/tests passed locally; see the validation record.
- [ ] Minimum and stable toolchain jobs pass on the claimed platforms for the exact
      proposed release commit.
- [ ] A fresh clone follows the README successfully, including native build prerequisites,
      external sample data paths, expected failure exits and output-file overwrite rules.
- [ ] README and API/metadata documents distinguish implemented behavior, intentional
      divergences, temporary helpers, unsafe/trusted boundaries, and deferred features.
- [x] Add the project-owner-selected MIT license and Cargo license metadata.
- [ ] Release notes and public documentation disclose System.Value allocation/copy costs,
      its retirement direction and the limits of the Void* storage experiment.
- [x] Inventory locked dependency licenses and preserve notice texts with provenance/hashes,
      including native libffi and separately licensed build/test tooling.
- [ ] Complete maintainer provenance review and refresh dependency/source notices for the
      exact candidate and chosen distribution shape.
- [x] Draft release notes covering capabilities, known limits, format changes and build instructions.
- [ ] Choose the preview version/tag and finalize release notes with exact-candidate platform evidence.
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
- Inheritance, class virtual dispatch, interface inheritance/defaults/variance, closed hierarchies, member enumeration, dynamic
  invocation, reflective construction/mutation, broad reflection or redesigned
  access modifiers. Keep the familiar implemented access levels for now.
- A Stream hierarchy, sockets, HttpClient, runtime async or general concurrency libraries.
- Full Unicode indexing/decoding APIs, broad collection APIs beyond the small native-layout ArrayList<T>, multidimensional/covariant arrays,
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
The initial API and opaque handle contract are documented in [type inspection](type-inspection.md).

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

The implemented [interface subset](interfaces.md) supports explicit borrowed dispatch,
generic contracts and List<T> access to ArrayList<T>, without boxing or ownership.
