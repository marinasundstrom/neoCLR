# Raven target and binary artifact experiment

Planned 2026-09-12 on `codex/raven-neoclr-target`, starting from `64024d6`.
This document scopes an experiment. No Raven backend, binary format or loader support
is implemented by the planning slice. The subsequent
[slice 1 integration map](raven-backend-integration-map.md) now pins and inspects Raven's
backend: it identifies existing core-library retargeting support, reflection/PE coupling,
framework discovery and the Void/Unit and exception-projection gaps. Binary format and
runtime-target contracts remain undecided.

## Objective and compatibility boundary

Use the existing Raven compiler, which currently targets .NET according to the author,
to compile a small program for neoCLR through a new target. The program must resolve
neoCLR's own runtime-library declarations, emit an artifact, and execute against that
library on neoCLR. Neo remains a demonstration and regression frontend; the runtime is
the main focus. This experiment tests interoperability with an independently developed
compiler rather than only improving Neo's source projection.

Seek the smallest useful compatibility surface across metadata, IL behavior, calls,
type/member resolution and library APIs. “ABI” includes only part of this boundary;
matching binary layouts or opcode names alone does not establish compatible execution.
C# targeting remains a longer-term goal, not a second compiler implementation in this
experiment. Running existing .NET binaries unchanged is not the acceptance criterion.

The author explicitly accepts source/API adaptation for deliberate differences:
expected failures should use Result, absence should generally use Option, and terminal
Faults should not recreate .NET's ordinary catchable exception flow. Do not silently
translate arbitrary try/catch into Result or claim unchanged source compatibility.
Fault containment and cleanup rules still need precise contracts. Nullability remains
a capability the platform should support, even where library APIs prefer Option;
this experiment does not settle its representation.

The [reference-defaults evaluation](reference-defaults-evaluation.md) remains an
investigation. This branch does not select a new class/value model, require a ValueType
hierarchy, or authorize changing all current storage contracts as a prerequisite.
Record actual frontend obstacles before deciding which runtime differences to change.

## First end-to-end milestone

A Raven source program calls neoCLR's System.Console.WriteLine with a string and
returns normally. It compiles using the new target, produces a documented binary
artifact, loads through neoCLR, and prints the expected text using neoCLR's System
implementation. A compiler-readable library declaration artifact and executable
library may be distinct if the backend requires it; their identities and signatures
must agree. The runtime library may remain authored in neoIL.

The milestone must demonstrate:

- An explicit Raven target selection and reproducible build/run commands.
- Compilation against neoCLR's library surface, without silently resolving the call
  to the installed .NET framework instead.
- Binary metadata and method-body emission/loading sufficient for the program and
  its actual library dependencies, with clear rejection of unsupported inputs.
- Correct binding to the loaded neoCLR library, observable output and normal completion.
- A regression path for both a successful call and a missing/mismatched member reference.

Do not assume HelloWorld implies a tiny loader change: inspect the dependency closure
of the System library. Choose whether to support that closure or produce a documented
minimal subset from the same library. A temporary JSON transport can diagnose compiler
integration, but does not complete the binary-artifact milestone.

## Investigation and slice order

1. **Inspect Raven's current backend and framework discovery.** Locate its repository
   and instructions, identify metadata/assembly emission dependencies and their pinned
   versions, and trace a simple program through symbol resolution, lowering and output.
   Determine where it assumes CLI core types, value/reference categories, Void, object
   construction, assembly identities, generics and exception handling. Do not assume
   Roslyn, Reflection.Emit or a particular metadata writer is used before inspection.
2. **Map the minimal target contract.** Compare the required emitted operations and
   library signatures to neoCLR. Classify each difference as reusable behavior, compiler
   adaptation, artifact encoding, library projection or missing runtime contract.
   Preserve useful CLR semantics and justify each deviation with a concrete need.
3. **Choose a binary strategy from evidence.** Evaluate reuse of a CLI container and
   compatible metadata/IL subset versus a versioned neoCLR container reusing compatible
   tables/signatures/operands. Include the cost to Raven's actual writer and neoCLR's
   loader. Define library-reference artifacts if needed. Do not select a format merely
   because it serializes current Rust structures conveniently.
4. **Implement and test the bounded artifact path.** Encode identities, references,
   signatures and method bodies needed by the milestone, including its library closure.
   Define feature/version rejection, malformed-input limits and diagnostics. Keep the
   existing source/JSON path useful as a control while bringing up the binary path.
5. **Add the Raven target and execute HelloWorld.** Compile against neoCLR declarations,
   load the resulting artifacts and bind to neoCLR's System. Document exact toolchains,
   source, commands, expected output and remaining unsupported language constructs.
6. **Expand only after the first path works.** Add a user-defined type and call, then
   a generic Result/Option API with success/error/absence cases. Use these to expose
   semantic gaps instead of promising all Raven features at once.

Keep implementation slices separate and update the changelog for each. The present
request creates the branch and documents the experiment; the listed implementations
are planned work, not reported completion.

## Binary format decisions that need explicit answers

Reuse the existing [format direction](format-direction.md) and
[design research process](design-research.md), with ECMA-335 Partitions II and III as
the metadata/IL baseline. Deepen the primary-source comparison against the actual
Raven emitter before implementing a contract; no new binary conformance claim is made here.

| Question | Why it matters |
| --- | --- |
| Standard CLI container, adapted container or custom header? | Determines writer/tool reuse and whether unsupported features are distinguishable before execution. |
| What identifies neoCLR core types and System? | Prevents accidental .NET core-library binding; must include module/assembly identity policy. |
| How are free functions, inhabited Void and extended managed references encoded? | Current platform semantics cannot be assumed to fit every standard CLI consumer. |
| Which signatures and opcodes retain standard meaning? | Compatible spelling/bytes must not hide different construction, storage or call behavior. |
| How are library declarations exposed to Raven? | The compiler needs usable symbols, not merely a runtime-readable file. |
| What happens to unknown features, invalid tokens and unsupported exception regions? | Reject clearly rather than interpreting them with different semantics. |
| How do versioning, linking and diagnostics survive serialization? | Source and binary loading must select the same definitions and report useful failures. |

No native machine-code ABI, JIT, AOT, full PE ecosystem interoperability or .NET binary
execution is promised by this first artifact experiment.

## Evidence and decisions to retain

Record Raven revision/toolchain, neoCLR revision, imported System identity, emitted
references, binary version/features, exact commands and validation results. Compare
source/JSON and binary execution where possible. Include unresolved compatibility gaps
and explicitly distinguish expected Result errors from terminal Faults.

Success means an existing compiler demonstrably targets our runtime and library.
It does not mean all its language features work or that migration requires no changes.

## Slice 2 evidence (2026-09-12)

The [minimal target contract](raven-minimal-target.md) and
[emission probe](experiments/raven-target/README.md) now record an executable compiler
baseline, target Console binding and the actual PE inventory. Missing-library isolation
fails and core declarations remain incomplete. Slice 3 must resolve those constraints
alongside its binary-format and no-result-call decision; no target execution is claimed.

## Slice 3 decision (2026-09-12)

The [binary profile](raven-binary-profile.md) selects CLI PE reuse with explicit target
admission and signature-driven translation. The emission probe now includes a resolver
that audits only supplied metadata dependencies, with positive and negative fixtures.
This detects the target's known dependency gaps; it does not yet fix Raven's resolver or
provide its complete core declarations. Those prerequisites, helper handling and input
stack verification must be completed before the planned reader can execute HelloWorld.

## Compiler isolation slice (2026-09-12)

The updated [probe](experiments/raven-target/README.md) uses Raven's opt-in
`MetadataImportOptions`. Explicit-only mode now excludes host resolver seeding and
the empty-reference-list host-core shortcut, and prevents cross-policy incremental
reuse. Console binds when supplied and is unavailable when omitted. This is tested
with an explicit .NET reference pack; providing an independent neoCLR core library
remains the next prerequisite, alongside generated-helper handling.

## Core declaration slice (2026-09-12)

The [minimal core reference artifact](raven-core-declarations.md) now supports binding
and emission of the static-call corpus without .NET framework references. Its metadata
and the emitted application resolve as a closed dependency set. This completes the
bounded declaration prerequisite, not the executable runtime library. Binary reading,
helper policy and validated call/return translation are next.

## Runtime-first compatibility direction

Most adaptation belongs in neoCLR. Changes to Raven should remain minimal and generally
useful for metadata/target handling. The [no-result return convention](no-result-methods.md)
implements the next static-call compatibility requirement directly in neoCLR, superseding
blanket call/return rewriting. It does not yet load Raven binaries.

The intended later type-model direction is alignment with .NET value-type/reference-type
semantics. Do not accumulate adaptations merely to preserve the original experiment's
value-default model. Deviations should be justified by the direction being demonstrated
(for example generic Void), with costs made explicit. This direction is recorded before
implementation; the existing allocation/type model has not been migrated by this slice.

## Next milestone: useful Raven subset

Author clarification, recorded 2026-09-12: the objective is enabling a minimal, useful
subset of Raven to target neoCLR. Type semantics should align with .NET's value/reference
classification rather than routing imported classes through the original explicit-reference
model. Standard metadata and instructions are the first choice; an internal semantic
change does not itself justify changing the external contract. Keep deliberate Void and
runtime-library improvements, with adaptation straightforward for existing compilers.

The PE container/body slice is complete as bounded inspection, not executable loading.
The next implementation order is:

1. Define and implement the minimal runtime type-classification contract. Class-typed
   locals, arguments and fields carry object references; value-typed storage copies values.
   Keep managed byrefs distinct from ordinary object references. Audit construction,
   assignment, calls, fields and GC together rather than introducing importer-specific
   implicit-reference rewrites. Use ordinary CLI class/value signature distinctions at
   the external boundary. Decide the exact internal representation during this slice.
2. Use one small Raven acceptance program to prove two aliases observe a mutation on a
   class instance, while copying a small value preserves independence. Include construction,
   a field, a static function call and Console output. Confirm the pinned compiler's
   supported source syntax before fixing the fixture. Compare behavior on .NET and neoCLR.
3. Complete only the metadata resolution, instruction coverage and real System binding
   needed to execute that program. Require explicit dependencies and clear unsupported
   diagnostics. Extend the corpus with a Void-returning function; test the intentional
   generic Void extension separately from ordinary no-result calls.

Each stage must report what actually runs. The static emission fixtures and test-bound
instruction execution are groundwork, not completion of this milestone. No new Raven
changes are assumed; any necessary changes remain isolated on its feature branch.

## Nominal reference storage foundation (2026-09-12)

The [first class-semantics slice](class-semantics.md) now implements heap-only ordinary
object handles under nominal signatures, alongside managed byrefs to slots. Runtime tests
prove class aliasing, value copying and the distinction between parameter rebinding and
byref replacement. This partially completes the first step above. Instance constructors,
null/default class references, field-store CIL semantics and the Raven acceptance program
remain next; the existing value-model library and Neo projection have not been migrated.

## Class construction follow-up (2026-09-12)

Class constructor-token operations now allocate/default supported fields, invoke a no-result
body and yield the object after successful return. Direct instance calls and class stores
have matching stack behavior. Standard constructor/field operand decoding is available.
The remaining immediate gaps are Object/base-constructor handling and reference defaults,
then metadata/core binding for the Raven program. Legacy value/byref field stores and
System no-result APIs still need alignment; the compiler must not compensate silently.

## First executable library integration (2026-09-12)

Raven's four static core-only programs now compile, pass a bounded neoCLR-owned import
bridge and execute on neoCLR. The HelloWorld program calls the real System.Console;
empty/nested calls and Int32 returns/locals also verify and run. See the
[reproduction workflow](experiments/raven-target/README.md#first-runtime-library-execution-milestone).
This completes the first narrow library-consumption milestone, not the class program or
native PE loading. No Raven source changes were needed.

Next library stages should expand the real reference/implementation surface around a small
Raven program, using type-semantics work where needed. Avoid treating parsing or synthetic
fixtures as the end goal. Retain explicit dependencies, familiar metadata/instructions and
clear boundaries for intentional Void and Result-based library differences.

## VS Code development experience

Author directive and clarification, recorded 2026-09-12: the MVP is **code completion in
Raven files/projects targeting neoCLR**, using the actual neoCLR reference/API surface.
A full debugger experience is not required. Runtime-library and compiler-target support
comes first so editor completion describes code that can really compile and run.

The immediate editor acceptance test is opening a Raven project configured for neoCLR
and receiving completion for the supported System APIs without host .NET API leakage.
Determine how Raven's existing language server/project loading consumes the explicit core
and reference settings; reuse that support wherever possible. Project configuration,
reference discovery and completion must select the same target as command-line compilation.

Build/run tasks and diagnostics can support this workflow. Source-mapped debugging is
optional later work, not an MVP dependency. These editor capabilities are planned, not
implemented by the import-bridge slice. The author corrected the assistant's earlier
build/run-first staging to make completion the minimum editor outcome.
