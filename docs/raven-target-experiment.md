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

## MVP acceptance criteria and union directive (2026-09-12)

The author added union usage to the MVP. The acceptance outcomes are now:

- Raven compiles a small program against the supplied neoCLR runtime-library declarations
  and it executes on neoCLR using the actual implementations. The five static programs
  now establish a narrow first stage, including Math and Console calls.
- A Raven program consumes a real Result or Option API and pattern-matches both outcomes.
  Math.Abs(Int32) is the proposed first case: ordinary success and Int32.MinValue overflow.
  Preserve neoCLR's Result contract; do not substitute .NET's throwing signature or a fake
  application-only result. This is not implemented yet.
- VS Code completion offers the supported target APIs in Raven files/projects without
  host-framework leakage. The existing compiler completion API now passes the explicit
  target probe; language-server/project integration remains outstanding.

Next support work should admit the generic union signatures/case representation and the
control flow needed for that Result sample, binding them to actual System metadata and
methods. Verify both outcomes and reject mismatched carrier/case types. Raven's existing
union lowering and the runtime library's layout/API must be compared before choosing the
import contract; this is not authorization to replace one silently with the other.

The reference declarations and importer share `TargetSurface` for the current five
signatures. This prevents editor/compiler API claims from drifting from the available
runtime bindings while the MVP grows. It is still a bounded experimental projection,
not the complete runtime class library or a native metadata loader.

## Union metadata probe and emission blocker (2026-09-12)

The checked-in `samples/library-result.rvn` probe now binds `Math.Abs(Int32)` as
`Result<Int32, OverflowError>` and type-matches `Result.Ok<Int32>` and
`Result.Error<OverflowError>`. It includes ordinary input and Int32.MinValue. This
is **binding evidence only**, not an executed Result demo. An incorrect string
argument is rejected. The separate reference assembly has no external dependencies;
its placeholder bodies never execute. It is deliberately excluded from the admitted
`TargetSurface` catalog and the five runtime samples.

Raven revision `1d7341fa64a66b514e5e68031b6d072d8140ea3a` recognizes the
`System.Runtime.CompilerServices.UnionAttribute` and discovers member types from
single-argument constructors (`Symbols/PE/PEUnionSymbols.cs`). Its type-pattern
emitter searches for `TryGetValue(out Case)` (`CodeGen/Generators/ExpressionGenerator.Patterns.cs`).
The probe supplies that protocol over the existing Result cases. neoCLR currently
names the equivalent operation `TryGet`; no alias or importer binding is implemented
by this probe. Choosing an alias versus a binding adapter remains open.

Emission fails before we can inventory the sample's IL: `Compilation.ResolveRuntimeType`
and `TypeSymbolExtensionsForCodeGen.GetClrTypeInternal` try to resolve the metadata
carrier as a host runtime type and report
``Unable to resolve runtime type for metadata symbol: System.Result`2``.
The probe recognizes exactly this failure and fails if the outcome changes, so later
work must replace the blocker assertion with positive emission checks. See the
[recorded probe outcome](experiments/raven-target/union-results.json).

This is an emitter limitation, not evidence that CLI generics need a different
encoding. The .NET baseline already separates reflection over metadata from runtime
loading: [MetadataLoadContext](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.metadataloadcontext?view=net-11.0-pp)
permits a designated core assembly and inspection across platforms (consulted
2026-09-12; the linked .NET 11 documentation is prerelease). Our provisional next
step is to preserve target metadata identity through Raven emission, on its existing
feature branch. Investigate that shared compiler capability before adding a special
neoCLR union lowering path. Loading fake executable host implementations would hide
the dependency and would not validate the actual runtime library.

After emission works, neoCLR still needs admission of closed generic library types,
value receiver/out-parameter calls, and verified branch/local state merges. Then bind
both cases to the real Result API and execute both outcomes. No union opcode or new
binary metadata format has been justified. The probe does not establish that a
metadata-only emitter change is small, nor that these two generic carrier layouts
are binary-compatible.

## Target metadata emission follow-up (2026-09-12)

The Result sample now emits at the Raven revision pinned in the probe README.
The compiler change stays on `codex/neoclr-target-resolution`. With explicit core
retargeting enabled, named metadata types and closed metadata constructions can pass
through persisted emission without becoming host runtime types. The existing method
proxy path now handles members on constructed metadata owners, preserves byref
parameters, and writes definition signatures (`!0`/`!1`) under those owners. Nested
case references retain their value-type marker. No neoCLR-specific syntax was added
to Raven; default emission is unchanged.

**Correction to the previous probe:** UnionAttribute and constructors were not enough
for Raven's union recognition. The preliminary declaration was missing its required
public `object Value` property. Consequently, successful binding alone did not prove
union pattern lowering. The declaration probe now includes that property, and checks
require both `TryGetValue(out Case)` references and reject ordinary box/isinst/unbox
pattern lowering. The new output passes the explicit dependency-closure audit and
references only NeoCLR.CoreProbe. The declaration's `Value` property is not implemented
or admitted as a neoCLR runtime API. Its projection, along with the TryGetValue/TryGet
name difference, must be resolved before exposing this as a supported target contract.

The sample has still **not executed on neoCLR**. `StaticImport` remains limited to its
existing primitive static subset. The next slice is the runtime/library binding and
verified generic/local/branch admission described above. Passing the metadata audit
is not proof of IL validity or runtime behavior. In particular, .NET InitLocals and
value-receiver/out-case storage must be preserved rather than synthesized away.

This follows the prior .NET comparison: keep standard generic signatures, managed
byrefs, and branches; fix the target-emission boundary instead of introducing union
instructions. Raven tests cover metadata-only generic signatures, nested case getters,
and out-case extraction. The five existing runtime programs remain the executable
baseline while the union importer is developed.

## Result execution profile (2026-09-12)

The first Result program now executes against the actual System.Math.Abs and
System.Result methods on neoCLR. The checked-in Raven source calls Abs(-42) and
Abs(Int32.MinValue), matches the corresponding case, and prints `42` and `Overflow`.
`ResultImport` is a separate bounded Cecil-to-neoIL profile. It admits the emitted
Int32/OverflowError carrier and cases, static application functions, local addresses,
and basic conditional/unconditional control flow. It does not load PE files directly
in the runtime or execute the declaration assembly. Neither new opcodes nor a new
binary metadata format were needed for this sample.

The CLI comparison remains the earlier metadata/byref/branch baseline, with explicit
library adapters where current neoCLR contracts differ:

- CLI value-type instance receivers are addresses. Adapters load the existing value
  through that address and call neoCLR's current value-receiver Result methods.
- Raven's `TryGetValue(out Case)` maps to existing `TryGet(out(true) Case&)`. Each
  adapter first stores the case's default value, so the output is assigned on both
  outcomes; this is the declared target contract, not an assumption that every CLI
  byref call writes its output. The Boolean result is converted to a CLI Int32 stack
  value. Only the exact two extraction signatures are admitted.
- InitLocals defaults are preserved for Int32 and the two case values. The carrier's
  default cannot be represented by the current runtime Result layout. Rather than
  choose a case implicitly, the importer rejects reads or receiver calls without a
  definite preceding assignment. This narrows accepted CLI programs; it is not a
  change to CLI default semantics or proof of general carrier layout compatibility.
- Exact stack types (including address provenance) must agree at branch joins;
  definite-assignment facts are intersected and rechecked to a fixed point. Branch
  targets, maxstack, method/body/local limits and call signatures are checked. Analysis
  has a work limit. Unreachable instructions are omitted from the executable output.
- Console's no-result projection still consumes the legacy inhabited Void return.
  No public throwing API or nullable result was substituted for Result.

The benefit is a runnable library demo without a Raven-specific runtime instruction;
the cost is an explicitly narrow import profile and temporary adapters. A general
metadata loader, default-carrier policy, broader generics and remaining class semantics
are still separate work. The union-recognition `Value` property stays declaration-only
and cannot be called through this profile. Preserve that restriction until its real
library contract is designed.

Validation includes six verified/executed programs, a Rust regression over the imported
Result artifact, and PE mutation probes rejecting default-carrier reads, uninitialized
receiver addresses, incompatible branch stacks and wrong out cases. Rejected inputs
must produce no executable artifact. Maps retain input hashes, method tokens and IL
byte offsets. See [run instructions](experiments/raven-target/README.md#running-the-result-demo-2026-09-12).

## POC demonstration priorities (2026-09-12)

The author clarified the intended demonstration: a .NET-like runtime with its own
runtime class library, Result and Option instead of exception/null-centered APIs,
Void accepted as a generic type argument, and Raven targeting with basic VS Code
completion. The POC should make that combination concrete before broadening the
platform surface.

| Demonstration | Current evidence | Remaining work |
| --- | --- | --- |
| Familiar runtime and own library | Raven Math/Console and Result program execute on neoCLR | Package a small cohesive sample/project |
| Result-based errors | Abs success and overflow execute | Broaden only as the sample needs |
| Option-based absence | Raven price lookup constructs/extracts real library Some/None values and executes | Incorporate the lookup into the cohesive demo |
| Generic Void | Raven Option<System.Void> constructs and matches both cases through a target adapter | Broaden target emission; zero-stack representation remains separate |
| VS Code completion | Installed local extension displays target Math completions; LSP checks cover Console and union names | Connect the editable project to the bounded runtime build/run pipeline |

The next priority is the Option demo, followed by generic Void and the project/editor
integration. Address runtime gaps when these scenarios expose them. The POC does not
require a full debugger or broad .NET application compatibility. This ordering is the
assistant's implementation plan in response to the author's acceptance criteria.

## Option lookup execution (2026-09-12)

The application price lookup in `samples/library-option.rvn` returns the actual
`System.Option<Int32>` carrier. Product 7 returns Some(42); product 99 returns None.
Both are matched and printed after Raven emission and neoCLR execution. This adds
application-to-library construction and a returned carrier to the earlier Result
scenario. It does not claim a new lookup API exists in the runtime library.

The `.NET/CLI` baseline here is value-type construction through newobj, storage into
locals, and managed-address receiver calls, using the same instruction family as
the prior Result profile. No runtime opcode or metadata extension was added. The
neoCLR-owned importer is now named `UnionImport`; selected constructor references map
to wrappers invoking the real Option.Some<Int32>, Option.None and Option<Int32>
constructors. Integer ceq results are projected to CLI Int32 stack values. Admitted
case locals retain their defaults; default carrier observation remains rejected
rather than being silently interpreted as None. This restriction is a preview profile
limit, not a claim that CLI default values behave differently.

Raven required a small feature-branch extension to its target metadata emission:
constructor tokens now follow the existing method-proxy approach. The final artifact
retains target constructor signatures and contains no proxy types. This avoids mixed
host/metadata generic arguments and Reflection.Emit's failure to encode the encountered
modified generic constructor parameter. Reference-only generic constructor regression
tests cover both the payload and carrier shape; the probe's dependency closure and
actual neoCLR execution provide the end-to-end check.

The payoff is a self-contained absence-handling demo without null, exceptions, or a
bespoke API added solely for the demo. Costs remain the bounded reference declarations
and temporary binding adapters; arbitrary constructors and generic parameters are not
newly supported. Validation now includes seven runtime programs, the Rust artifact
regression, and three new mutation checks for unwritten carriers, mismatched out cases,
and a mismatched constructor argument. Generic Void and actual VS Code completion
remain outstanding. See [Option run instructions](experiments/raven-target/README.md#running-the-option-demo-2026-09-12).

## Generic Void execution (2026-09-12)

Raven now demonstrates `Option<System.Void>` with Some(()) and None, using the actual
neoCLR Option library. This is a deliberately different contract from .NET:
[Type.MakeGenericType](https://learn.microsoft.com/en-us/dotnet/api/system.type.makegenerictype)
documents Void as an invalid generic argument (consulted 2026-09-12). The probe also
confirms that rejection on the pinned .NET 11 SDK. Ordinary method return VOID remains
an empty-stack result convention, independent of this generic type participation.

Raven's current output uses a primitive VOID signature marker inside generic arguments.
The provisional neoCLR-owned `VoidProjection` pass preserves that raw artifact and
replaces generic occurrences with named VALUETYPE references to the supplied System.Void.
It reuses existing metadata encodings, but changes their accepted semantics; this is
not a claim that the artifact satisfies ordinary CLI generic rules. Binary signature
checks establish the distinction, since Cecil presents both forms as MetadataType.Void.
The declaration and application dependencies still undergo the explicit closure audit.

Alternatives were to retain .NET's rejection, introduce a separate public Unit type,
or extend Raven's target emitter immediately. Rejection would miss the author's generic
Void goal; exposing Unit would change the intended API. The bounded post-emission adapter
lets us test that goal without another compiler change, at the cost of a temporary
projection stage. General metadata signatures and production compiler integration need
further work before adopting this as a universal encoding contract.

Raven's compiler-generated `()` literal loads its own empty Unit.Value field. Only that
validated field shape is mapped to `ldvoid`; arbitrary static fields and initializers
remain unsupported. The existing VM uses an inhabited Void marker on its evaluation
stack, including constructor arguments. This slice therefore establishes observable
generic behavior, not zero-stack or zero-storage optimization, ABI parity, or performance
improvement. Debug/provenance maps identify the projected input; the raw DLL is retained
for inspecting the compiler output before adaptation.

Validation: eight programs verify and execute; three new rejection probes cover raw
VOID generic signatures, a wrong payload, and a non-Unit field. These supplement the
four Result and three Option rejection probes and the earlier static profile checks.
The checked-in imports are covered by the Rust runtime regression. Actual VS Code
project completion remains the next POC priority.

## Project-backed editor execution (2026-09-12)

The explicit metadata-import policy now flows from Raven project evaluation into the
language server. A named `RavenMetadataCoreAssemblyName` disables automatic host framework
references; the language server also refrains from adding host Raven.Core/Macros support
references. This reuses the existing [core declaration contract](raven-core-declarations.md),
which uses .NET MetadataLoadContext's explicit resolver mechanism. No guest metadata,
IL, runtime opcode, or language syntax changes are introduced here.

A generic project property was chosen over a neoCLR-specific language-server branch.
The benefit is consistent references across compiler and editor without host APIs
leaking into the demo. The cost is that target authors must supply complete, correct
reference metadata; project configuration alone neither retargets emission nor validates
a runtime artifact. Existing .NET projects retain their default behavior. Explicit
package/reference items remain the caller's responsibility.

Raven changes remain isolated on `codex/neoclr-target-resolution` at `37ae973040`.
Validation covers 37 project-loading tests and three focused language-server tests.
The installed local VSIX displays Abs/Max/Min/Sign in VS Code, and the stdio protocol
probe checks Console and System (including Option/Result/Void, without System.IO).
The eight runtime programs still execute with the new compiler build. Full Build/Run
integration is separate; [instructions and limitations](experiments/raven-target/VSCODE.md)
make that boundary explicit.
