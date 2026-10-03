# Dual-target end-to-end implementation

Author-approved plan, 2026-10-03. The first gate is a broad application plus a
source-built library subset, not the entire runtime library. The author accepted an
explicit CLI primitive bootstrap and documented runtime seed for this gate. Native
application/rebuilt-library references must use direct metadata import.

## Integration priority clarification (2026-10-03)

The author reaffirmed CLI metadata as the physical baseline, then clarified that the
immediate objective is Raven targeting neoCLR, without concentrating on new semantics.
Finish the selected end-to-end case first. Do not make a wholesale CLI-authoritative
container/loader migration a prerequisite for union support. Extend the existing metadata
infrastructure only where a reproduced compiler/importer/runtime failure requires it,
reusing ordinary CLI tables, signatures, attribute contracts and IL where expressible.

The existing PE/#Neo transport still executes its native payload and exposes conventional
CLI declarations as a reference projection. This is a documented interim representation,
not completion of the CLI-compatible encoding goal. Format reconciliation and experimental
semantics remain later work unless a concrete acceptance failure requires a bounded change.
Existing extensions remain supported within their stated limits; this does not expand the
scope of Self or structural Function work.

The next acceptance step remains union/case metadata preservation and imported symbol
reconstruction, followed by separately compiled union execution. Passing source-body
preflight alone is insufficient. No new runtime execution result is claimed by this
scope clarification.

## Ordered slices and acceptance

1. Paired driver-level Hello/helper and separate library/consumer execution baseline.
2. Route rvnc neoclr references through the direct native symbol importer, with exact
   dependency validation and failure before output publication.
3. Complete introspection constructor/classification/accessibility/parameter facts
   needed by these consumers; retain language rules in Raven.
4. External interface contracts across contracts, implementation and consumer assemblies;
   evolve metadata writer/reader/verifier/runtime together and preserve compatibility.
5. Establish one declared owner per bootstrap/library type, resolving source/seed iteration
   identity; build collection contracts, ArrayList, comparers/HashMap, Option/Result/query
   dependencies as independent libraries.
6. Run unchanged application-order-collections against those libraries on both targets;
   fix reduced regressions at their owning layer instead of rewriting consumers.
7. Focused regression stabilization and a refreshed full-library gap inventory.

Success requires ordinary compiler commands, consumer compilation without library source,
expected stdout/exit/identity/mutation behavior and explicit artifact/revision records.
Reference-only bridge emission does not count as runtime success.

## Boundaries and defaults

Keep the existing .NET Reflection/Emit backend; a Cecil migration is later work.
Metadata readers -> introspection -> Raven symbols; symbols/lowered program -> compiler
emission contracts -> target builders -> definitions -> metadata -> PE. Library ILGenerator
and compiler emission contracts are independent. Importer objects never become emitter
operands. Introspection owns metadata resolution/substitution; Raven owns binding,
inference, overload resolution, conversions and accessibility decisions.

Retain .NET-compatible metadata/instruction behavior where supported. Capabilities expose
extensions and unsupported categories explicitly. Keep structural Function on feature
branches. Isolate independently useful compiler fixes for later main-line validation.
Each verified slice gets changelog/documentation and a commit; no performance claim without
measurement. API changes get C# contract tests and host reference documentation. Broader
runtime-library completion remains subsequent work.

Initial paired-driver baseline: 3/4 cases pass; native library consumption fails with
NEOMETA001 undeclared instance field. Both .NET cases pass. See
[baseline evidence](../experiments/extended-cli-metadata/dual-driver-baseline-2026-10-03.json).


## Direct native compiler command (2026-10-03)

`rvnc neoclr --core-reference NeoCLR.CoreProbe.dll --reference Library.dll -o App.dll App.rvn`
now imports API-produced native assemblies using NeoClrMetadataReference and artifact-only
emission bindings. It selects CompilationOptions.NeoCLR and its explicit primitive/runtime
contracts. Native references never fall back to CLI projection. The core reference is
required when --reference is supplied; this is an experimental command compatibility change.
Without native references/core selection, the existing host primitive bootstrap remains
available. --system-symbols/--system-method is labelled legacy and remains an explicit
partial callable projection, not a native library import fallback.

Paired driver acceptance now uses `--dual-driver <rvnc.dll> <neoclr> <NeoCLR.CoreProbe.dll>
<fresh-output>`. Both .NET and NeoCLR Hello/helper and separate generic library consumers
execute (42); the consumer has no library source. Duplicate identities, missing transitive
dependencies, ordinary CLI references, malformed input and unsupported source reject without
publishing output. Host .NET runtime execution uses an explicitly hashed runtimeconfig.
The wider source-library/collections gate remains open; runtime encoding is unchanged.

Declaration-fact slice complete for the current by-value signature profile: metadata
accessibility/flags and constructors are facade-owned and consumed by Raven. Existing
accessor associations remain canonical. Byref/out still fail reader materialization.
Next: external interface declarations across separately compiled native assemblies.


External-interface gate baseline (2026-10-03): the paired harness now also supports
--dual-driver-external / --dual-driver-external-inventory with the same driver/runtime/core/
output arguments. Contracts, Box<T> implementation and consumer compile in separate
invocations; source is removed before downstream compilation. The .NET split executes 42.
Native Hello passes, but native implementation emission rejects in SourceTypePlan before
encoding because the interface identity is external. This is a failing future acceptance
case, not a claim that the external-interface slice is complete.

Required changes span SourceTypePlan/SourceInterfacePlan capability admission, symbol-only
external relationship authoring in the NeoCLR adapter, metadata relationship validation/
round-trip, and runtime dependency/dispatch verification. Current writer/reader relationship
attachment requires owned definitions. Do not simply remove admission checks or duplicate
contracts into the implementation assembly. No compiler/runtime behavior changes in this
baseline test slice.

## External interface gate completed (2026-10-03)

External relationships now use output-owned interface identities and complete semantic
contracts. Raven records every direct method (including accessors), inherited edge and
constructed argument before completing each contract. The metadata writer rejects
incomplete contracts and missing/mismatched public implementations. Only implementing
methods receive CLI virtual/final/newslot flags. Definition collections and builder
convenience methods share relationship validation.

The native reader preserves assembly-scoped interface relationships; Introspection resolves
them through the explicit catalog and substitutes inherited arguments. No runtime loading
occurs during symbol import or emission. Existing format-5 relationships and linked runtime
dispatch suffice: no schema/version or runtime implementation change was needed.

The PE writer accepts the authored graph directly, preserving its validated CLI projection.
Reconstructing that projection from native bytes alone cannot recover external method
contracts and rejects explicitly; compiler emission uses the authored-graph overload.
This is a transport/reference projection limitation, not a native semantic restriction.

The three-assembly driver test now includes a local generic interface inheriting two
external paths to one base, a generic implementation, property dispatch and alias mutation.
Both targets execute 42 with library sources absent. C# tests additionally execute CLI
and native dispatch, test definition/builder authoring, incomplete/missing/mismatched
contracts, completion freezing, exact flags and missing metadata dependencies.
See [driver evidence](../experiments/extended-cli-metadata/dual-driver-external-2026-10-03.json).

Next: the explicit bootstrap/source ownership manifest and source-built iteration contracts.
The broad collections/source-library completion gate is still open.


## Source-owned contract bootstrap (2026-10-03)

The first ownership stage is implemented with
[`bootstrap/iteration-ownership.json`](../experiments/extended-cli-metadata/bootstrap/iteration-ownership.json).
It assigns Disposable, Iterator, Iterable, Collection, Sequence, MutableSequence and List
to `NeoCLR.Collections`. The acceptance runner reads its source catalog and copies the
unchanged runtime-library units into a temporary build directory, removing those copies
before compiling the consumer against the emitted artifact. Both ordinary drivers read
`--bootstrap-ownership` to select runtime contracts and reject missing, competing or
wrongly owned declarations. Exact artifact identity checks remain in the native importer.

The primitive-only CoreProbe mode retains compiler marker declarations and the existing
small Console/Math bootstrap surface; it does not contain collection or union declarations.
This first stage references no retained System seed. It does not establish ownership of
the entire runtime library. The manifest explicitly leaves TypeOf and Propagation
contracts null because the bootstrap does not supply their dependencies. Unsupported
emission remains subject to ordinary target capability checks; the expanded default
NeoCLR profile is unchanged when no manifest is selected.

Acceptance exercises source-defined Collection/Iterable/Iterator inheritance, an external
implementation, interface Count/Current/MoveNext/Dispose calls, `for` iteration, explicit
array iteration and alias identity. Both targets return 42. Wrong owners, duplicate owner
entries, unsupported manifest versions and the expanded conflicting CoreProbe reject
without output. [Evidence](../experiments/extended-cli-metadata/source-iteration-ownership-2026-10-03.json).

Reproduce after building rvnc with its native metadata project:

```sh
dotnet run --project docs/experiments/raven-target/Probe.csproj \
  -p:RavenRoot=/absolute/path/to/Raven -p:UseRavenCoreReference=false -p:WarningLevel=0 \
  -- --reference-primitive-core /tmp/NeoCLR.PrimitiveCore.dll
python3 scripts/check-source-iteration.py --raven-root /absolute/path/to/Raven \
  --core /tmp/NeoCLR.PrimitiveCore.dll --output /tmp/fresh-iteration-check
```

Next source-library dependencies include CheckedStorage, Fail, callback signatures and
Option. The source/seed partition for those declarations remains open. The author also
explicitly requires native Self signatures: the runtime already understands SelfType,
but the Cecil-like SignatureType/readers/introspection/compiler path must preserve that
semantic form rather than the temporary CLI marker. This is additional required metadata
work, not a change to the structural Function branch boundary.

Source-owned contract driver implementation: Raven `codex/metadata-consumer` commit `f9841b0f6`, with neoCLR metadata `8c08829e`. The native runtime executable is unchanged.


## Checked-storage driver step (2026-10-03)

`rvnc neoclr --bootstrap-intrinsics --core-reference <core>` now passes the exact
registered primitive bootstrap to the existing target intrinsic contract. The flag is
opt-in, requires an explicit core and may be specified only once. It does not authorize
unrelated native dependencies or add an importer/emitter backchannel. `CheckedStorage`
must have the already validated public static `Reserve<T>(int) -> T[]` signature; it
emits the existing native array.reserve operation. Ordinary .NET defaults are unchanged.
Unlike CLR newarr, native reservation tracks uninitialized elements and faults on reads
before writes; the reference-only bootstrap body is never executable code.

The new `--reference-storage-core` generator mode adds only this declaration to the
primitive core, without competing source collection/union declarations. It is still a
bootstrap reference, not a retained System implementation or an executable .NET adapter.
No native format/runtime changes are needed. The C# API's existing BootstrapReference
validation and checked-storage tests remain the owning contracts.

`scripts/check-source-storage.py` first runs the existing dual-target source-iteration
acceptance. It then compiles a native generic helper library through rvnc, deletes its
source, imports its artifact into separate consumers without intrinsic opt-in, and verifies
execution. Mutation through an array alias returns 42; an unread reserved slot faults.
Disabled intrinsics, missing explicit core and repeated flags diagnose without publishing.
[Executable evidence](../experiments/extended-cli-metadata/source-storage-2026-10-03.json).

Reproduce with the normal bridge build, selecting `--reference-storage-core /tmp/storage.dll`,
then `python3 scripts/check-source-storage.py --raven-root <Raven> --core /tmp/storage.dll
--output <fresh-directory>`.

The [unchanged-source inventory](../experiments/extended-cli-metadata/arraylist-inventory-2026-10-03.json)
compiles Option and Propagatable alongside the seven collection contracts on .NET. Native
emission rejects the propagation interface's out-parameter signatures. Including ArrayList
also exposes missing System.Fail binding and minimal-bootstrap callback accessibility on
native; .NET lacks the runtime storage/Fail service adapters. These are separate dependency
and target-contract gaps. There is no ArrayList consumer or broad application success yet.
Next admit and preserve supported ref/out interface parameter modes through the native
metadata facade and compiler, then rerun the unchanged sources before widening emission.


## Native ref/out interface step (2026-10-03)

The existing native `ByRef` signatures and `out_parameters` indices now survive declaration
materialization and metadata-only introspection. ParameterInfo exposes element type plus
an explicit Value/Ref/Out mode. Raven imports those facts into parameter symbols and authors
member references from them. The portable interface plan admits writable ref/out only when
the target's managed-reference capability allows it; readonly variants remain unsupported.
Complete external contracts retain output indices when substituting generic owner arguments.
No runtime/schema change, importer-object emission dependency or new Runtime Contract option
is introduced. Ordinary .NET Reflection/Emit remains in place. Native semantics match the
existing CLR ref/out calling contract: out must be assigned before normal return; ref input
must already be initialized. CLI In metadata is not silently treated as writable ref.

A C# driver fixture compiles contracts, implementation and consumer independently, removes
library sources, then executes generic inherited interface out assignment and ref mutation
on both targets (42). Incompatible ref/out implementations reject without output publication.
Reproduce with NeoClrMetadataProbe `--dual-driver-parameter-modes <rvnc.dll> <neoclr>
<CoreProbe.dll> <fresh-output>`. C# metadata tests cover both containers, definition/builder
parity, mode conflicts, readonly rejection and open/constructed signature substitution.
See [evidence](../experiments/extended-cli-metadata/parameter-modes-2026-10-03.json).

The unchanged source Propagatable interface now emits with the source-owned collection
contracts. Option no longer fails interface admission; it now reaches the native source
union/declaration emission rejection. This is the next source-library gate, alongside the
previously recorded Fail/callback bootstrap dependencies for ArrayList. No source stubs or
manual union carriers replace the runtime library, and broad application completion remains open.

## Union payload foundation (development, 2026-10-03)

The metadata producer now accepts direct nominal and constructed fields in owned
value types, including a `Payload<T>` embedded in a `Carrier<T>`. Builder calls and
manually attached field definitions share validation. Writing rejects recursive inline
storage and limits owned layout traversal to depth 64 and 4096 visited constructions;
references and vectors terminate inline traversal. Native input validates the same
owned layouts. External dependency layouts still require explicit dependency resolution
and runtime linking; this check does not load dependencies implicitly.

`AssemblyDefinition.ReadNativeAssembly` now materializes unconstrained top-level
value declarations, signatures, fields and supported constructors/methods. `IsValueType`
reflects the native category; sealed/sequential flags are preserved without inventing
a CLI `System.ValueType` dependency. ImportReference overloads retain the explicit
matching output core requirement. Nested declarations, constrained owners and generic
instance methods remain outside this native snapshot profile. Native snapshots remain
immutable and preserve their original bytes on Write.

This matches CLR inline value storage and copy semantics: an executable tag/payload
fixture returns 42 on both runtimes after mutating an independent copy. Direct native
imports of nongeneric/generic value constructors and methods also execute on both.
The fixture uses an Int32 tag and is a metadata contract test, not a replacement for
Raven union lowering or proof that source Option compiles.

Raven maps the introspection value category to Struct and its semantic ValueType base;
generic field substitution remains in introspection. It does not reopen imported
metadata in emission. Source value declarations, nested union cases, the Byte tag,
synthesized members and symbol-authored external value operands remain the next
compiler work. Runtime-library union sources are unchanged. No format version change,
CLI projection fallback, runtime implementation change or performance claim is needed.

## Source value declarations for union emission (2026-10-03)

The native adapter now opts into an explicit portable `ValueType` declaration category.
Top-level ordinary structs, including unconstrained generic owners, map to metadata
value definitions. Reference classes retain their existing path. Nested declarations,
value-interface implementations, ref structs and constrained value owners remain outside
this bounded source profile; no source unions are rewritten into classes or manual carriers.

Shared body planning preserves an addressed receiver for member access and takes a value
copy when `self` is used as an expression. Synthesized parameterless struct constructors
zero-initialize fields before declared initializers. The adapter uses the existing metadata
ILGenerator and value-type builder contracts. There is no metadata format/runtime change,
new bootstrap selection or importer-to-emitter dependency. Ordinary .NET emission retains
its existing Reflection/Emit path; its portable profile does not opt into this new category.

The C# `NeoClrMetadataProbe --source-value-driver <rvnc.dll> <neoclr> <core.dll> <fresh-dir>`
checks ordinary driver compilation and execution on both targets: generic inline payloads,
explicit/default constructors, accessors, self copies, local field mutation and independent
copies return 42 with empty stdout/stderr. Unsupported value-interface implementation
rejects with NEOMETA001 and no output. This is same-compilation source value coverage,
not separate native library consumption or source Option completion. Source and artifact
hashes plus command results are recorded by the harness.

Remaining union dependencies include nested case/companion declarations, the Byte tag,
synthesized union methods/relationships and symbol-authored external value references.
The source library and broad application gates remain open. No independent binder fix
was needed here; shared changes add an explicitly selected emission capability.

## Native nested case metadata (development, 2026-10-03)

`AssemblyDefinition.ReadNativeAssembly` now retains supported nested class/value
ownership beneath nongeneric declaring types, including generic nested value cases.
Local signature lookup keys include the declaring token; external TypeRef rows use
nested TypeRef scopes. Same-named cases beneath different companions remain distinct.
Nested public/internal accessibility, canonical declaring views and constructor/member
signatures survive direct native reading without a CLI projection. ImportReference
accepts these scoped native definitions with the existing explicit matching core policy.

Raven publishes nested symbols as members of their declaring type, preserving their
namespace and containing-type identities; namespace member and simple-name lookup do
not flatten them. Generic nested field substitution stays in introspection. Emission
continues to reject native nested operands outside its symbol-authored capability profile.

This follows CLI NestedClass/TypeRef identity semantics using existing native relationship
encoding; no schema change or implicit dependency loading is required. C# checks cover
same-named local/external payloads, multiple nesting levels, internal visibility, missing
dependencies and cyclic owners. Direct native nested generic/nongeneric constructor
imports execute on CLR and NeoCLR (42). Native semantic probes retain owner/field identity.

This is a reader/importer prerequisite for union cases, not source union completion.
Nested types that capture generic enclosing parameters remain unsupported. Source union
declaration collection, nested definition emission, Byte discriminators and complete
synthesized union contracts remain pending. Existing immutable snapshot behavior is unchanged.

## Nested source declaration emission (2026-10-03)

The NeoCLR adapter now opts into the compiler-owned `NestedType` declaration capability.
It collects nested class/struct declarations in owner-first order and calls the metadata
library's existing nested builders with empty child namespaces and lexical ownership.
Supported children are nongeneric root classes and unconstrained generic/nongeneric
values under nongeneric supported owners. Static children, generic enclosing-type
capture and generic nested reference classes remain explicit unsupported categories.
The ordinary .NET Reflection/Emit backend does not opt into the new portable category.

Nested lookup may provide a substituted accessor under a nongeneric owner. The native
callable resolver now reuses its original source declaration before considering external
references. This uses compiler symbols only, with no name-based member matching or
importer access. It is part of the new native emission capability; no independently
reproduced .NET regression or binder fix is claimed.

`NeoClrMetadataProbe --nested-value-driver <rvnc.dll> <neoclr> <core.dll> <fresh-dir>`
compiles equivalent sources through ordinary commands on both targets and executes 42
with empty output. It checks two same-named payloads with different owners, a generic
nested value, nested class construction, zero default storage and independent value
copies. It reads the emitted native snapshot to verify enclosing identities and rejects
generic enclosing owners and value-interface implementations without publishing output.
The explicit CoreProbe bootstrap is unchanged; no metadata schema or runtime change is
required. Existing library IILGenerator remains behind Raven's emitter boundary.

Generated union declaration collection, Byte discriminators, full synthesized union
contracts and symbol-authored external value/case operands remain pending. This test
contains ordinary nested declarations; it does not claim that unchanged Option or the
broad class-library consumer compiles yet.
