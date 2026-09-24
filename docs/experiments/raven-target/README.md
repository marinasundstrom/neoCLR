# Raven targeting neoCLR

## Development compiler policy (2026-09-24)

The shared neoCLR build props now require Raven's AllowNullableValueTypes option
(general compiler commit `48466d258`, integrated as `04b19639f`). They select
RavenAllowNullableValueTypes=false: RAV0407 rejects nullable value declarations with
"Value types can't be declared as nullable". Nullable references remain valid.
The `--nullable-value-types` CLI switch can override a project policy, but enabling it
does not add neoCLR runtime support for Nullable<T>. Unconstrained generics and
inferred/imported values are outside this declaration-only restriction.

The preceding general fix `37b3ad81f` (integration `1b2c6afee`) also preserves inherited
nullable metadata on generated record Object.Equals. The target-only follow-up `fd2fd7881` narrows a guarded Object? argument before
record-struct unboxing. No record Runtime Contract configuration or runtime
instruction changes are required. The checked
[record sample](../records/README.md) covers class/struct nullable Object comparisons
and the new rejection diagnostic. Use a matching development compiler; previously
packaged SDKs are not updated by these source changes.


Post-Preview-4 source work: [application classes/interfaces](../../raven-application-types.md)
and the [order-workflow demo](../../raven-order-workflow.md) now exercise user-defined
objects against the runtime library. These require the updated experimental Raven
branch; published Preview 4 binaries remain unchanged.

Start with the [runtime and Raven release walkthrough](../../runtime-raven-preview.md)
and [current VS Code setup](VSCODE.md). The existing runtime API pass is complete for
the bounded POC, with coverage and limits [recorded here](../../raven-runtime-api-coverage.md).
The source-probe sections below retain the history of earlier integration slices;
their old tool versions and admission limits do not describe the latest bundle.

Normal `neoclr run` now prints only guest output. Older recorded transcripts below
include a historical `=> Void` runner suffix; it is no longer emitted by default.
Use `--show-result` to inspect return values on stderr. Current verification scripts
expect clean guest output.

This is slice 2 of the [Raven target experiment](../../raven-target-experiment.md).
It tests the existing compiler API and inventories emitted PE metadata. The current
follow-up imports a bounded static subset and executes it against neoCLR's real System
library through the separate runtime verification command below. Earlier slice notes
retain their historical limits. This is not general Raven or direct PE execution.

## Typed record equality integration (2026-09-24)

General Raven commit `df9b7211b` (integrated as `832549785`) annotates synthesized
record-class Equals with the nullable record parameter and keeps typed record-struct
parameters non-nullable. It also aligns top-level nullable reference annotations
when emitting interface dispatch. Target-only follow-up `5d7f3d5c8` selects nested
record-class component Equals by the underlying parameter type, preserving null guards.
RuntimeRecordContract configuration and the public Equatable<T> interface are unchanged.

The [record sample](../records/README.md) exercises nullable Key locals and literal
null, alongside Object/interface calls and the existing value-copy cases. The .NET
comparison has 32 assertions on pinned SDK 10.0.100/runtime 10.0.0. Generic record-class
checks in Raven's .NET tests do not expand neoCLR's supported component profile.

## Current Result propagation follow-up

The current sources require Raven `22cea6fa1` or later on
`codex/neoclr-target-resolution`. Earlier revisions recorded below describe historical
slices. Local `0.1.12-neoclr.4` tools include propagation; see [VS Code instructions](VSCODE.md). To reproduce from source, build the compiler and
language server from that experiment checkout:

```sh
dotnet build src/Raven.Compiler/Raven.Compiler.csproj -f net11.0 -p:WarningLevel=0
dotnet build src/Raven.LanguageServer/Raven.LanguageServer.csproj -f net11.0 -p:WarningLevel=0
```

From the neoCLR repository, with a built runtime, use fresh output paths:

```sh
dotnet run --project docs/experiments/raven-target/Probe.csproj -p:RavenRoot=/absolute/path/to/Raven -p:BuildProjectReferences=false -p:WarningLevel=0 -- --interfaces /tmp/neoclr-propagation
python3 docs/experiments/raven-target/prepare_editor.py /tmp/neoclr-propagation /absolute/path/to/Raven --collections --runtime /absolute/path/to/neoclr
python3 docs/experiments/raven-target/verify_project.py /tmp/neoclr-propagation/editor/Demo.rvnproj --collections --raven /absolute/path/to/Raven --runtime /absolute/path/to/neoclr
```

This verifies the existing fundamentals plus `library-propagation.rvn`, whose output
is `Continued`, `42`, then `Overflow propagated` on separate lines. To run it manually,
copy that sample to the generated editor project's `Main.rvn` and run `run_project.py`
with the same project, `--raven` and `--runtime` arguments. The second call returns early
without printing `Continued`. See the [contract and limits](../../propagation-contract.md):
this slice admits Result<Int32,OverflowError>, Option<Int32> and
Result<Void,OverflowError>. The generated editor opens the combined propagation
workflow, with ArrayList constructors and iteration. `verify_project.py` exercises
all three propagation forms.

## Match syntax validation

The [Raven match matrix](../../raven-match-matrix.md) records verified forms, diagnostics,
and outstanding arm-return discrepancies. Run the `--matches` probe and
`verify_matches.py` before changing the published support claims.

## Reproduce

Current probe validated on 2026-09-12 with Raven revision
`995a4c982fbf5df97b82e417c9319c2e87164461` on `codex/neoclr-target-resolution`,
neoCLR starting revision `26068dc`,
.NET SDK `11.0.100-rc.1.26425.128` and Mono.Cecil `0.11.6`.
Use a separate Raven checkout at that revision. The local `global.json` pins the SDK;
restore needs Raven's package feeds/dependencies and the .NET 11 reference pack.

First build the Raven compiler from its checkout (a warm checkout with unchanged
generator inputs; follow Raven's own AGENTS.md for a fresh checkout):

```sh
dotnet build src/Raven.Compiler/Raven.Compiler.csproj -f net11.0 -p:UseRavenCoreReference=false -p:WarningLevel=0
```

From this directory, pass the absolute Raven checkout path and a new output directory:

```sh
dotnet run --project Probe.csproj -p:RavenRoot=/absolute/path/to/Raven -p:WarningLevel=0 -- /tmp/raven-neoclr-probe
```

The output directory must not exist. Read `report.json`; the DLLs are inspection
artifacts only. The fixture's `WriteLine` is an empty placeholder, not a working Console.
Do not ship it or use it as a runtime implementation. Generated output is not committed.
The checked-in [results.json](results.json) records the observed metadata and diagnostics.
Instruction text is exploratory evidence, not a stable compiler-output assertion.

## Checks and observations

The input imports `System.Console.*` and declares `Main` calling
`WriteLine("Hello from Raven on neoCLR")`.

- The .NET control emits a call scoped to `System.Console`.
- Removing the reference-pack Console assembly and adding the fixture binds that call
  to `NeoCLR.Probe.System`. Existing `EmitOptions` core retargeting removes output
  references to System.Console, System.Runtime and System.Private.CoreLib.
- A nonexistent `MissingWriteLine` produces a binding error.
- **Missing-library isolation fails:** omitting both Console references still binds.
  The probe records this as `MissingLibraryIsolationPassed: false` and inventories the
  resulting image. A passing emission probe is not a passing target-isolation check.
- The incomplete declaration fixture references `mscorlib`. It introduces that
  core identity; output retargeting does not close all dependencies. The fixture lacks
  Object, ValueType and attribute definitions referenced by the target. The report
  also shows a `System.String` reference scoped to the target module without a local
  definition. The whole signature graph needs validation, beyond AssemblyRef names.
- Even this small source emits Unit, nullable-attribute and entry-point helpers.
  Inspect the complete report, not just the source Main body.

The targeted compiler build passed with zero warnings/errors. Control and fixture
emission and the missing-member check passed; the missing-library negative exposed the
isolation gap. No Raven source changes, Raven xUnit suite, or neoCLR execution are claimed.

The next contract work is documented in [the minimal target map](../../raven-minimal-target.md).

## Dependency-closure audit (slice 3)

The same command now runs `ClosureAudit` against the emitted application and fixture.
It rejects the incomplete target and host-fallback artifact without searching installed
frameworks. The report's `Closure` section records these errors separately from compiler
binding diagnostics. This does not change Raven's resolver or make the target executable.

Positive fixtures cover a self-contained metadata assembly and a consumer with an
explicit external dependency. Negative fixtures cover an omitted dependency, wrong
assembly version, missing type, missing method, changed parameter signature and duplicate
identity. These use ordinary `Probe.Root` types to isolate metadata lookup from core
primitive-type recognition; they do not establish a CLI-conforming core library.
The negative checks require the relevant diagnostic, not just any failure.
Cecil rewriting can also introduce mscorlib references into mutated fixtures; these are
reported rather than hidden. The positive fixtures require zero resolution errors.

The audit covers AssemblyRef, TypeRef and MemberRef lookup. It is not a signature/IL
verifier or a hardened parser for hostile inputs. All fixtures remain metadata-only.
See [the binary-profile decision](../../raven-binary-profile.md) for its limits and the
planned CLI container/translation boundary. The slice 3 probe starts from neoCLR commit
`7dfaca6`, with the same earlier Raven revision and SDK as slice 2.

## Explicit-only compiler imports (slice 4)

The current probe requires the Raven revision above, which adds `MetadataImportOptions`.
Earlier slices 2–3 used `d92b02812740ae052f277c23151e9cc208f7672d`; that revision
cannot compile the updated probe. Slice 2 began at neoCLR `0a60f9b`.

The new option names `System.Runtime` as the metadata core and excludes implicit host
assembly seeding, including the empty-reference-list host-core shortcut. With the explicit .NET reference pack plus Console fixture, target
binding and emission succeed. Omitting Console now produces binding diagnostics and
`GetTypeByMetadataName("System.Console")` returns null. The report retains the legacy
mode's failing isolation result for comparison and records `ExplicitOnlyIsolationPassed`
separately. Compiler execution still uses its normal host environment.

This fixes the tested metadata-import fallback, not the entire target library. The
probe still supplies .NET reference-pack declarations, and the retargeted artifact
still has incomplete core definitions. The dependency audit remains necessary.

Validation for this slice: 7 framework-targeting baseline tests passed; the final focused
set of 15 framework/import tests passed, including missing core, missing Console after
host cache warm-up, incremental policy changes, option copying and explicit-mode emission.
The new constructor argument requires compiler API consumers to rebuild; it remains
optional for existing source callers. No guest neoCLR execution is claimed.

Raven's `scripts/test-target-framework-matrix.sh` also passed: Raven.Core and Raven.Macros
built for .NET 10/11, and the representative .NET 10 macro and .NET 11 runtime-async
programs executed successfully. No changes were needed to those samples.

## Core-only reference artifact (slice 5)

The same command now creates `NeoCLR.CoreProbe.dll` as a metadata-only reference assembly
using Roslyn 4.12.0 and no input references. Raven binds/emits the `CoreOnly`, `CoreEmpty`,
`CoreNested` and `CoreInt32` programs with only this artifact supplied. The probe checks
zero core AssemblyRefs using the raw metadata reader, the reference-assembly marker,
application/core dependency resolution in both input orders, and missing Console / wrong
parameter-type diagnostics. The existing framework-based controls remain separate.

See [the core declaration contract](../../raven-core-declarations.md) for the distinction
between compiler declarations and runtime implementations. The placeholder definitions
are not neoCLR's System implementation. This slice makes no Raven repository changes;
it uses the same Raven commit as slice 4, from neoCLR starting revision `33a216b`.

The report/audit now snapshot AssemblyRefs before Cecil type resolution can synthesize
an in-memory mscorlib reference. Earlier application inventories included such a synthetic
reference. The old Console fixture still has a real mscorlib dependency and still fails.

The complete updated probe passed, including the four core-only emissions and expected
negative diagnostics. Raven remains unchanged on `codex/neoclr-target-resolution`.

## First runtime-library execution milestone

Validated 2026-09-12 with the same Raven commit and SDK above, starting from neoCLR
`f49a8c9`. No Raven changes were needed. `StaticImport` now reads the four core-only
application DLLs using Cecil and writes corresponding `.neoil` files. The compiler binds
against the supplied declaration assembly; the imported Console call runs against the
actual neoCLR System.Console implementation. The core fixture's empty method never runs.

After building Raven as described above, run the probe from this directory with a new
output directory. A warm build can use `-p:BuildProjectReferences=false` to reuse the
already-built compiler:

```sh
dotnet run --project Probe.csproj -p:RavenRoot=/absolute/path/to/Raven -p:BuildProjectReferences=false -p:WarningLevel=0 -- /tmp/raven-neoclr-execution
```

Then, from the neoCLR repository root:

```sh
cargo build --locked
python3 docs/experiments/raven-target/verify_runtime.py target/debug/neoclr /tmp/raven-neoclr-execution
```

The script requires the completed probe report, verifies every imported module, runs each
on neoCLR and asserts the output. `CoreOnly` prints `Hello from Raven on neoCLR`; empty,
nested-call and Int32/local programs also succeed. The CLI additionally prints `=> Void`
as its host result envelope. [runtime-results.json](runtime-results.json) records the run.
The [imported fixtures](imported/CoreOnly.neoil) are regression evidence, not a substitute
for rerunning Raven emission. Their maps record original method tokens, byte offsets,
output lines and SHA-256 hashes of application/core inputs.

### Import contract and remaining gaps

The bridge belongs to neoCLR experiment tooling. It is not a Raven backend rewrite and
is not the Rust runtime's PE reader. It reuses the standard metadata/CIL contract described
in the [binary profile](../../raven-binary-profile.md) and the existing explicit closure
audit. Only reachable ordinary static methods with Void/Int32/String signatures are
admitted, using straight-line instructions: constants, arguments, Int32 locals, dup/pop,
static calls and returns. Declared maxstack, argument/result types, initialization and
return stacks are checked before output. Local default initialization is preserved.
Branches, instance methods, generic signatures, initializers, exception regions, native
methods and unknown external bindings are rejected in this profile. Inputs are bounded
to 16 MiB, reachable methods to 128, bodies to 64 KiB and local counts to 256.

All supplied metadata dependencies are audited before selecting reachable bodies.
Unreachable Raven Unit/attribute/constructor helpers are retained in the source image
and closure audit but are not imported for execution. This is an explicit reachable-code
policy, not full validation of helper IL or hostile PE metadata. Cecil remains a trusted
compiler-artifact tool here, not a hardened production admission boundary.

Console binding is deliberately narrow: the supplied core's exact resolved
`System.Console.WriteLine(String) -> void` maps to the actual runtime function. A neoCLR
wrapper discards that existing System method's inhabited Void result, while imported
ordinary void calls preserve their empty-stack semantics. Migrating System's signatures
can remove the wrapper later. No core declaration body is used as a runtime implementation.

The alternative was waiting for the full native metadata reader or adding a Raven writer.
This bridge gives immediate library-integration evidence with no Raven changes, at the
cost of a temporary .NET/Cecil tool and duplicated import logic. It is not a performance
claim or the final deployment pipeline. Direct binary loading, broader library reference
coverage and class-program execution remain subsequent stages.

The probe also patches the original PE instruction bytes (without rewriting metadata)
to reject underflow, surplus return values and unsupported `ldnull`. Their diagnostics
are recorded in `report.json`. Rewriting these negatives with Cecil initially introduced
a synthetic mscorlib reference; byte-only mutation avoids changing the test's metadata
question. No dependency was removed to force acceptance.

## Shared library surface and completion probe (2026-09-12)

The [target surface](TargetSurface.cs) now drives both compiler declarations and exact
runtime bindings. It exposes five signatures: Console.WriteLine(String),
Console.WriteLine(Int32), Math.Min(Int32,Int32), Math.Max(Int32,Int32) and Math.Sign(Int32).
Math calls go directly to the actual System implementations. Console wrappers only adapt
the existing inhabited Void return to the declared no-result convention.

The editable [Raven sample](samples/library-basics.rvn) is compiled by the same probe and
runs as `CoreLibrary`. Using the reproduction commands above now validates five programs.
It prints `42`, `1`, `0`, and `Library calls from Raven`. These results come from the real
runtime library, not the declaration stubs. The sample is copied to the probe output by
MSBuild so the probe does not depend on its invocation working directory.

The [API policy](../../api-policy.md) already distinguishes neoCLR's Result-returning
Math.Abs(Int32) and Clamp from .NET's throwing APIs. Those signatures are intentionally
not projected as plain Int32 methods. The [runtime library](../../runtime-library.md)
provides the contract baseline; union support will expose the real signatures next.
This is staged API coverage, not a change to those runtime implementations.

The probe also invokes Raven's existing `Compilation.GetCompletions` with the **same
explicit-only core reference**. Math completion returns Max, Min and Sign; Console
returns WriteLine. Abs, Clamp, ReadLine and host System.Storage.File are absent. The report
records these results under `TargetCompletions`. This demonstrates target-aware completion
in the compiler API, not VS Code integration.

Read-only inspection of Raven revision `1d7341fa64a66b514e5e68031b6d072d8140ea3a` shows
`WorkspaceManager.EnsureRavenCoreReference` can add a framework Raven.Core reference when
one is missing. Project loading and that policy must be checked against the explicit target
configuration before claiming the editor MVP. No Raven edits were made in this slice.

Validation: the complete emission/import probe passed, including its existing negative
checks and the new completion checks. All five imports verify and run through
`verify_runtime.py`; `cargo test --test raven_import` passes against the refreshed fixtures.
The next MVP program must consume a real Result/Option API and handle both outcomes.

## Running the Result demo (2026-09-12)

The same reproduction command checks `samples/library-result.rvn`, using a separate
`union-probe/NeoCLR.CoreProbe.dll`. `report.json` includes `UnionProbe`: the valid
Result patterns bind and emit, while passing a string to Math.Abs fails with RAV1503.
The generated `CoreUnion.dll` passes dependency-closure checks. Both generic out-case
extractors must be referenced; ordinary object type-test lowering is rejected. The
compact checked-in outcome is `union-results.json`.

`UnionImport` now produces `CoreUnion.neoil` and its source/provenance map.
The runtime verification command above verifies and runs all eight programs, including
this Result sample. To run only the checked-in demo from the neoCLR repository root:

```bash
cargo run -- verify docs/experiments/raven-target/imported/CoreUnion.neoil
cargo run -- run docs/experiments/raven-target/imported/CoreUnion.neoil
```

Expected output:

```text
42
Overflow
=> Void
```

These are results from the actual System.Math.Abs and System.Result implementations.
The final Void line is the CLI's no-result display; it does not demonstrate generic Void.
The emitter declarations never execute. `union-results.json` records emission and
import rejection checks; `runtime-results.json` records execution.

The Result profile admits only Int32/OverflowError and their closed cases, static app
methods, local addresses, and the verified branch subset needed here. It rejects
observable default Result carriers, wrong out-case types, uninitialized receivers,
and incompatible stack merges. Its adapters translate TryGetValue to TryGet, copy the
value receiver through its managed address, initialize the out case on both outcomes,
and map Boolean results to CLI Int32 stack values. Console adapters consume legacy
inhabited Void returns. Direct calls to the recognition-only `Value` property are
not supported. See the [Result import contract](../../raven-target-experiment.md#result-execution-profile-2026-09-12)
for the limits and remaining POC work.

## Running the Option demo (2026-09-12)

`samples/library-option.rvn` defines a small application lookup, `FindPrice`, returning
`System.Option<int>`. It constructs Some(42) for product 7 and None for a missing
product. Case/carrier constructors and extraction bind to the actual neoCLR Option
library; FindPrice itself is application code, not a new runtime API.

The reproduction command emits and imports `CoreOption.dll`. Run the checked-in
artifact from the neoCLR repository root:

```bash
cargo run -- verify docs/experiments/raven-target/imported/CoreOption.neoil
cargo run -- run docs/experiments/raven-target/imported/CoreOption.neoil
```

Expected output:

```text
42
Product not found
=> Void
```

`verify_runtime.py` now verifies seven programs. The Option probe rejects unwritten
carrier reads, wrong out cases, and a wrong constructor case. `option-results.json`
records emission and these checks; runtime results remain in `runtime-results.json`.

The importer was renamed from ResultImport to UnionImport, with the same control-flow
and lifetime checks. Its newobj bindings admit only the selected real Option constructors;
case local defaults remain supported, while default carrier reads are rejected. It does not add
general object construction, application classes, or a new runtime instruction. Generic
Void and VS Code project completion remain the next POC steps.

## Running the generic Void demo (2026-09-12)

`samples/library-void.rvn` returns `Option<System.Void>`: Some(()) represents completion
without a payload; None represents no completion. Reproduce with the same probe and
runtime verification commands above; the script now checks eight programs. Run the
saved import from the neoCLR repository root:

```sh
cargo run -- verify docs/experiments/raven-target/imported/CoreVoid.neoil
cargo run -- run docs/experiments/raven-target/imported/CoreVoid.neoil
```

Expected output:

```text
Completed without a payload
Not completed
=> Void
```

The final CLI display is separate from the generic Void value in the carrier.
`VoidProjection` preserves Raven's emitted `CoreVoid.raw.dll` and produces
`CoreVoid.dll`, replacing generic VOID markers with named value-type references to the
supplied System.Void. Ordinary method return VOID signatures remain unchanged. This is
an experimental neoCLR target adapter, not native Raven backend support or a claim of
.NET execution compatibility. No Raven source change was needed in this slice.

The importer recognizes only the compiler-generated empty Unit.Value literal with its
validated static readonly field and no type initializer, mapping it to neoCLR's existing
`ldvoid`. Actual Option<Void> constructors and extractors execute in the runtime library.
The VM still represents Void with an inhabited stack marker; zero-stack/storage handling
is future work. Arbitrary static fields, general generic Void APIs and default carrier
reads are not admitted by this profile.

`void-results.json` records emission, dependency closure and three rejection probes:
raw generic VOID signatures, an integer substituted for the Void payload, and a field
other than the Unit literal. The probe also checks the named token in the binary method
signature and confirms host .NET rejects Void as a generic argument. See the
[design comparison](../../raven-target-experiment.md#generic-void-execution-2026-09-12).
VS Code project completion is the next POC step.

## VS Code project completion (2026-09-12)

The [VS Code walkthrough](VSCODE.md) covers the locally installed experimental extension,
project preparation, expected suggestions, rebuilding, logs, and the separate runtime
execution command. This editor slice requires Raven `37ae9730409d52f876b6b6e47abfa950d8300064`;
the older emission revision at the top remains the baseline for the earlier slices.
`prepare_editor.py` generates an isolated project using `RavenMetadataCoreAssemblyName`.
`verify_editor.py` checks actual LSP requests against the configured server. Math
completion was also verified in the installed VS Code extension. These checks do not
claim the normal Build/Run buttons target neoCLR yet.

## Saved project build/run milestone (2026-09-12)

Raven `5b773ae3536f52ef077c8897867950249d6dde90` fixes generated host-TFM attributes for
explicit metadata targets. `run_project.py` now compiles the editable project through
RavenWorkspace, retargets emission and imports/verifies/runs the resulting saved source.
`prepare_editor.py` creates dedicated VS Code tasks; `configure_tasks.py` updates an
existing folder. See [edit/build/run instructions](VSCODE.md#edit-build-and-run-the-saved-project).
`verify_project.py` checks Result, Option, generic Void, a changed output and failure
behavior. The normal Raven toolbar remains a separate pipeline. Propagation and
interfaces are not claimed by this milestone.

## Interface contract probe (2026-09-12)

The separate `--interfaces` mode now compiles and imports two Int32 collection programs.
Use the adapted runtime System profile to verify and execute them; see the
[collection execution instructions](../../raven-interface-contract.md#executable-raven-collection-import-2026-09-12).
The default saved-project profile remains unchanged.

## Executable Raven arrays (2026-09-12)

The `--arrays` probe compiles [library-arrays.rvn](samples/library-arrays.rvn) with
Raven at `5b773ae3536f52ef077c8897867950249d6dde90`, using only the supplied neoCLR
core declarations. It imports standard `Int32[]` signatures as runtime `arrayref<Int32>`
and emits `newarr`, element operations and `ldlen`. The sample returns an array, shares
it between bindings, mutates it through a parameter and prints its length. No Raven
compiler change is required. The core declares `System.Array.Length` so Raven can bind
and lower the property to `ldlen`; that declaration body never executes.

From the neoCLR repository root, use a new output directory:

```sh
cargo build --locked
dotnet run --project docs/experiments/raven-target/Probe.csproj \
  -p:RavenRoot=/absolute/path/to/Raven -p:BuildProjectReferences=false -p:WarningLevel=0 \
  -- --arrays /tmp/raven-neoclr-arrays
python3 docs/experiments/raven-target/verify_arrays.py /tmp/raven-neoclr-arrays \
  --runtime /absolute/path/to/neoclr/target/debug/neoclr
```

If building with `CARGO_TARGET_DIR`, point `--runtime` to that directory's `debug/neoclr`.
The verifier script checks the generated array sample and null-default fixture, runs
them and writes `array-execution-results.json`. Expected positive output:

```text
42
2
=> Void
```

The probe also mutates emitted IL and requires rejection of wrong allocation element
types, unsupported stores, wrong index stack types, multidimensional signatures and
uninitialized-local reads. Rejected inputs must not produce executable neoIL. The
null-default fixture instead passes verification and faults at runtime on array access.
Recorded evidence is in [array-results.json](array-results.json).

This bounded bridge now labels maps `result-option-void-arrays-v4`. It admits Int32
vectors in locals, static-function parameters and returns; InitLocals arrays get typed
null defaults. Array covariance, other element types, instance-field import, null literals,
array-reference comparisons and interface/class conversions remain unsupported. Runtime
capabilities are broader than this importer profile. The actual ArrayList/iterator
library adaptation and union propagation are still pending.

For the existing saved-project/VS Code workflow, use `samples/library-arrays.rvn` as the
project source and refresh its `NeoCLR.CoreProbe.dll` from this probe's output. That core
also includes the existing Result/Option declarations. Old local core files lack the
Length declaration. The bridge runner must come from this checkout; no new Raven SDK or
VS Code extension build is needed for this slice. This turn verified the compiler/import/
runtime path, not a new manual VS Code session.

## Runtime collection profile

The [adapted collection profile](../../raven-interface-contract.md#adapted-runtime-collection-profile-2026-09-12)
provides generation, verification and execution instructions for the actual collection
algorithms using nominal classes/interfaces and managed arrays. The `--interfaces`
probe now imports the matching Raven programs; use `verify_collections.py` with the
explicit generated System profile to verify their execution.
[Target-specific Raven contracts](../../raven-target-contracts.md) are proposed for
future implicit iteration without changing the default .NET target.

## Target-selected Raven for loops

The current probe sources require Raven commit
`c28657859dda6b473d182445d91e5d44469a6974` on `codex/neoclr-target-resolution`
(or a descendant). Rebuild Raven.CodeAnalysis before building the probe with
BuildProjectReferences=false. Earlier revision references above describe earlier slices.

`--interfaces` now also compiles [library-foreach.rvn](samples/library-foreach.rvn),
selecting Iterable/Iterator through RuntimeIterationContract. The existing
`verify_collections.py` command runs it with the explicit System collection profile;
its output is `41, 42, 41, 41`. See [contract support and limits](../../raven-target-contracts.md#first-implemented-compiler-slice-2026-09-12).

This is compiler-API integration, not a new VS Code/project switch. Automatic iterator
Dispose is a recorded Raven gap; cleanup/defer support remains future work.


The collection profile can also be selected from an editable Raven project, with the
same iteration settings used by the language server and saved-project runner. This
requires Raven `1e3f7ff07d8a9785ed54105b74d1fcda96c8795f` or a compatible descendant;
see [collection project setup and checks](VSCODE.md#collection-project-follow-up).


## Fundamental Raven demonstration

The collection declaration profile now also exposes Result, Option and generic Void.
`library-workflow.rvn` uses an ArrayList through List, iterates with the configured
Iterable/Iterator protocol, handles missing products with Option and arithmetic failure
with Result, and returns an Option<Void> completion marker. No exception handling or
union propagation is synthesized. The existing System implementations execute; metadata
stub bodies do not. No new Raven changes or runtime opcodes are needed for this slice.

Regenerate a fresh `--interfaces` probe and follow the [collection project instructions](VSCODE.md#collection-project-follow-up).
`verify_project.py --collections` now runs these checks against that same declaration
assembly and adapted runtime library:

| Demonstration | Evidence | Scope |
| --- | --- | --- |
| Class reference assignment | `library-collection-aliases.rvn`: mutating an alias changes the original list | Ordinary nominal ArrayList/List references, without explicit managed-reference syntax |
| Array reference assignment and parameters | `library-arrays.rvn`: a called function changes the original array, output 42/2 | Int32 vectors and standard CLI array operations |
| Value copying | `library-value-copy.rvn`: replacing an Option binding preserves its prior copy, output 42/7 | Known Option<Int32> value carrier; not arbitrary mutable struct import |
| Library naming and iteration | `library-foreach.rvn` and `library-workflow.rvn` | Configured Iterable/Iterator/GetIterator, with List/ArrayList and indexers |
| Result, Option and Void | Standalone samples plus combined workflow | Existing bounded union import and Void projection; no propagation |
| Editor experience | `verify_editor.py --collections --files` | Collection and union names, selected Math API and inferred int loop binding |

Expected workflow output is 42, Completed, Price overflow, Skipped, Product not found,
Skipped, then `=> Void`. The successful, overflow and missing-product paths all execute.
The six malformed-import checks and null receiver fault fixture remain in the probe.
Recorded [project results](collection-project-results.json) and
[language-server results](collection-editor-results.json) capture this verification.

This is a fundamental demonstration, not full integration. Raven remains on its separate
experimental branch; compiler changes and differences will be evaluated separately as
the experiment advances. The existing .NET baseline is preserved. Full application class
import, broader collection elements, automatic cleanup, propagation and a production
binary loader remain outside this demonstration. Earlier research on
[CLI class behavior](../../class-semantics.md) and
[collection contracts](../../common-interfaces.md) supplies the comparison: familiar
reference assignment and dispatch are retained, while library names and result-based
APIs are deliberate target differences. Temporary profile generation/import restrictions
are tooling costs, not new platform semantics.


### Executable integer Math surface

The saved-project importer now uses the same TargetSurface catalog as compiler
metadata for the existing static Int32 Math.Min, Math.Max and Math.Sign APIs. These
members were visible in completion but previously rejected by this importer. Calls
retain their standard CLI static-call shape and execute the existing System.Math
implementations. Reference and resolved-definition signatures must agree; this does
not admit arbitrary static methods or new overloads.

`library-math.rvn` checks Min/Max at both Int32 limits, equal operands, and Sign for
negative/zero/positive values. `verify_project.py` runs it for both the smaller union
profile and the combined collection profile. Expected output is -2147483648,
2147483647, 7, -7, -1, 0, 1 and `=> Void`.

This reuses the [existing Math contract and .NET comparison](../../math.md): no new
runtime behavior, opcode or Raven change is introduced. Result-based Math.Abs remains
a separately bound target-library difference. Other Math overloads remain outside the
bounded importer even though they exist in the neoCLR runtime library.

The [file API slice](../../raven-file-api.md) now covers bounded UTF-8 read/write,
Result propagation and conditional-output validation.
See the [existing API coverage plan](../../raven-runtime-api-coverage.md) for the
full POC scope and the [experimental release procedure](RELEASING.md) for separate
Raven SDK/VSIX packaging without a full Raven product release cycle.

[Shared signature projection](../../raven-signature-projection.md) documents the
file/collection signature checker and its `--signatures` regression probe.

[The String API slice](../../raven-string-api.md) adds eight existing helpers, Unicode
boundary examples and `verify_editor.py --strings` completion checks.

The `--slices` probe and `verify_slices.py` verify UTF-8 slicing, typed error
propagation and conditional-output safety; the [String API documentation](../../raven-string-api.md)
contains commands and the remaining error-carrier limits.

The [lexical Path projection](../../raven-path-api.md) adds `Combine` and
`GetFileName`, with a saved-project sample and completion checks under `--files`.

The [Int32 parsing projection](../../raven-parsing-api.md) includes `--parsing`
probe/verification, a saved-project example and optional editor completion checks.

The [integer division example](../../raven-division-api.md) covers recoverable zero
and overflow errors. Numeric completion checks under `--parsing` include Divide.

The [Int32 instance example](../../raven-integer-api.md) covers Equals, CompareTo
and ToString on local and parameter receivers, with numeric completion checks.

The [integer Clamp example](../../raven-clamp-api.md) demonstrates inclusive bounds
and Result propagation for invalid ranges, completing the current Int32 Math subset.

The [Double Math example](../../raven-floating-math-api.md) exercises all current
floating-point Math methods using concrete Double values and comparison calls.

The [primitive and character projection](../../raven-primitive-api.md) requires
Raven experiment `26907410f` or later for explicit integral casts. It includes all
current Char classifiers and bounded concrete primitive comparisons.

The [calendar and local-clock projection](../../raven-calendar-api.md) includes
fixed validation examples, `--calendar` completion checks and `verify_clock.py`
for validating a live snapshot against the host clock.

The [error-value API projection](../../raven-error-api.md) includes constructors,
checked extraction, descriptions, message errors and `--errors` completion checks.


## Prototype queries (source experiment)

The current collection profile adds `import System.Linq.*` with deferred `Filter` and
`Map` and eager `ToList`. See [the query contract](../../raven-query-api.md),
`samples/library-queries.rvn`, and `verify_queries.py`. Use fresh declaration metadata
and the matching generated library; archived .7 packages do not contain this API.
`verify_editor.py --queries` checks completion on lists and query results.

### TypeInfo member hierarchy (2026-09-19)

The reference profile and importer now expose TypeInfo as the fourth sealed
MemberInfo case. DeclaringType is Option<TypeInfo>; unwrap ordinary member owners,
and handle None for top-level types. Name, Module and MetadataToken are inherited.
Rebuild consumers and update exhaustive matches. Runtime Contract typeof settings
are unchanged. Raven's same-file sealed-family rule is satisfied by compiling all
member contracts and providers together in Descriptors.rvn (74 generated slices).

Nested source types retain a module-scoped declaring_type_token in .origin metadata,
including the owner's definition when only the nested type is used. Missing or
cyclic ownership is rejected. The nested-type sample exercises imported ownership
and four-way matching. This does not add nested type enumeration or dynamic loading.

When an instance method is lowered to a free function (notably a value-type
constructor), the source metadata's parameter-token list includes a zero entry
for its explicit receiver. The receiver has no CLI Param row. Ordinary declared
parameter tokens retain their positions after that synthetic slot. This prevents
valid source programs from failing metadata admission after receiver lowering.

### Minimal UTF-8 API (2026-09-19)

The selected System.Runtime adds Utf8.Encode(String) → Sequence<Byte> and
Utf8.Decode(Sequence<Byte>) → Result<String,InvalidUtf8Error>. The bridge checks
exact static signatures and supplies managed-array adapters for the two native
services. String.IsEmpty is now a property. Runtime Contract configuration remains
unit/() → System.Void; no Raven compiler changes are required. Rebuild reference
metadata and generated runtime together. See [contracts, comparison and limitations](../../raven-string-api.md#strict-utf-8-conversion-2026-09-19).

### Development Option and Result operators

After Preview 8, regenerate the collection-profile reference core and System library
together to use the [outcome operator port](../../../raven-outcome-operators.md).
`verify_outcome_operators.py PROJECT.rvnproj --bridge Probe.dll --system System.neoil
--runtime neoclr` validates the executable examples, branch selection, callback
faults and invalid calls. `verify_editor.py --unions` now also checks outcome
operator completion; include the other flags matching the reference profile.
The runtime sources use Raven extension declarations, validated against the
bootstrap extension metadata. Published Preview 8 bundles remain unchanged.


## Experimental worker notification (2026-09-23)

[Delayed Copy](../delayed-copy/README.md) builds a temporary worker-library adapter
that calls bootstrap-only `RuntimeServices.NotifyWorker(handle, callback)`. The
importer binds the exact Int32/Func<Void> signature to the new runtime service and
discards its Void value for the CIL no-result call. The bootstrap C# declaration
uses the existing PropagationUnit-to-Void metadata projection. Rebuild the bridge,
bootstrap core and runtime together; old bundles cannot run the adapter.

Normal application references still omit RuntimeServices. Normal worker-library
snapshots are unchanged: the experiment replaces their generated methods/helpers
only in a temporary System library. It supports the default TaskQueue and retains
callback graphs until VM dispatch; explicit queues and public worker migration
remain open. No Raven compiler semantic/emission changes or Runtime Contract flags
are added. Existing heap async state machines and cancellation propagation settings
remain in use. The sample verifier compiles both the library adapter and ordinary
application, checks exact output and requires actual guest collections. Runtime
worker/Task tests cover admission, roots, dispatch and teardown.


The notification experiment now polls after a callback returns to the System
library's default TaskQueue.Drain, then appends ready notification work using Post.
The hook validates the library module, owner, default receiver identity and
Func<Void> invocation boundary. Updating the queue implementation must preserve or
reassess that boundary. No Raven lowering, reference-core signature or Runtime
Contract setting changes in this follow-up. The shared Copy.rvn consumer is tested
with both allocation pressure and a self-reposting BackgroundWork callback; an
explicit-queue runtime test verifies default notifications do not run there.


### Development file streams

The reference/import profile admits System.Streams.FileInputStream and
FileOutputStream with exact signatures and private resource constructors. Their
Raven implementations call bootstrap-only RuntimeServices bindings for open,
create, read-into, write, flush and close; application code cannot call those raw
services. StreamError uses the existing typed error-carrier machinery. The compiler
and Runtime Contract configuration are unchanged: ordinary reference classes,
arrays and Result metadata use the existing CIL path. No Raven repository changes
or new suspension semantics are part of this slice.

Regenerate the library fragments and reference assembly together. The
[provider sample](../storage-provider/README.md) uses normal SDK/MSBuild compilation;
its verifier separately checks wrapper error paths, managed-array aliasing and
resource-slot reuse. Stream wrappers are synchronous and invocation-bound; see the
[stream guide](../../../api-docs/streams.md) for API details and the DocFX Flush
renderer exclusion. This is development work after Preview 9.


### Development Storage metadata adapter

The reference catalog includes System.Storage.Metadata.GetKind(string), EntryKind
and StorageLookupError. The implementation is Raven-authored; the bootstrap-only
RuntimeServices.StorageKind maps to the existing neoCLR.Runtime.StorageKind service.
The reference uses the existing value-carrier/Result metadata contract. There is no
Raven compiler, Runtime Contract configuration or emission-policy change. The normal
Storage provider SDK sample and LookupContracts exercise reference import and execution.
Metadata takes a native string; application-owned provider GetFile consumes its
validated logical Path. These are development additions after Preview 9.


### Integrated Path reference contract

System.Storage.Path is now a sealed reference type with private construction,
Parse returning Result<Path, InvalidPathError>, read-only properties and nonvirtual
Equals/ToString methods. Existing static Combine/GetFileName string signatures
remain. The catalog recognizes Path instance calls, including Raven-emitted
callvirt for nonvirtual reference members, and emits the existing runtime instance
call mechanism. The bootstrap owner preserves both static and instance methods.
RavenMetadataCoreAssemblyName/RavenTargetCoreAssemblyName remain NeoCLR.CoreProbe;
no Runtime Contract setting, compiler semantic rule or Raven emission change is
introduced. Ordinary overload resolution and metadata access checks now see the
platform type rather than the sample-local one. Use matching regenerated core and
runtime artifacts. The companion integration note is on Raven's isolated feature
branch in docs/compiler/runtime-contracts.md; no experiment is merged into Raven main.

### Directional stream interface integration (2026-09-23)

System.Streams.InputStream and OutputStream are ordinary CLI interfaces, authored
in Raven and emitted into the development runtime library. FileInputStream and
FileOutputStream implement them directly; custom application providers use the
same imported contracts. The importer validates the exact core interface members
and admits only the corresponding concrete-to-interface conversion. It does not
add a catch-all conversion or identify application types by their spelling.

The matching reference/runtime snapshots are required. Existing concrete methods
and native services keep their behavior. Runtime Contract options are unchanged;
new interface implementations use ordinary CLI method matching and interface
calls, without new VM instructions. The Storage sample exercises Read/Write/Flush/
Close through these interfaces for both disk and memory, including short transfers.
Input cannot call Write and output cannot call Read; source negatives cover this.

Integration exposed a general Raven array-symbol identity defect: imported byte[]
parameters carried a different construction container from source arrays and failed
implicit interface matching. The independent ordinary .NET regression and fix are on
Raven main (`f0c06f7f8`), integrated as `5fa6516fe`; the target-specific stream catalog remains here. Use a development
compiler containing that fix and the configured-unit identity fix when regenerating the snapshots or building custom
stream implementations. Published Preview 9 compilers do not contain these fixes.

Flush also exposed a configured-unit identity mismatch inside imported generic
returns. Raven main `44ae9f242` (integration `849347a97`) fixes equality/hash matching
against the explicitly selected value type. Its ordinary .NET regression uses
System.ValueTuple, with opt-in success and default rejection; 39 focused unit, symbol
and interface checks pass. No neoCLR-specific policies were merged into Raven main.


### Integrated Storage descriptors — 2026-09-23

The development reference now declares System.Storage.File as a sealed descriptor
class, preserving its static native text helpers. Directory resolves child addresses;
StorageLookup inherits StorageProvider and adds typed file lookup. The strict bridge
admits only these constructors/members and the exact inherited lookup-to-byte
conversion, including arrays of descriptors. Matching runtime and reference snapshots
are required; no compiler or Runtime Contract configuration change is needed.
See the [Storage evidence](../storage-provider/README.md#platform-filedirectory-integration--2026-09-23)
and [member reference](../../../api-docs/storage-items.md). Native static file/Path
artifact regressions remain the compatibility check.


### Storage interface hierarchy migration — 2026-09-23

StorageItem now closes the kind hierarchy to the File and Directory interfaces;
providers can implement either branch. `--storage-hierarchy <new-folder>` verifies
that ordinary C# CIL cannot add a third branch through the importer, and that missing
closure metadata is rejected. The SDK Storage contract also checks Raven rejection
and common Name/Path access across provider-owned File and Directory objects.
Raw neoIL does not enforce the closed marker. Provider resolution remains a follow-up.

The development native text calls are FileText.ReadAllText/WriteAllText; File is now
an interface with no constructors or static helpers. Legacy raw File helper names
remain supported. The source/reference/runtime snapshot must be updated together.


StorageLookup additionally admits GetDirectory(Path) -> Result<Directory,
StorageLookupError>. Reference declarations, strict signature validation and the
Raven bootstrap contract include it. Provider implementations return the core
interface; the importer requires no provider-specific concrete class. This is a
development contract change with no Raven compiler or Runtime Contract setting change.


Provider-resolution consolidation: System.Storage.StorageProvider now admits only
GetItem/GetFile/GetDirectory, returning interface-valued Results. StorageLookup and
its inherited byte-provider conversion are removed from declarations, strict binding
and runtime bootstrap selection. Byte routing stays in provider implementations.
This is a development API migration requiring matching generated artifacts; no
Raven compiler changes or Runtime Contract settings are introduced.


FileSystem is now a Raven-authored public host provider with internal LocalFile and
LocalDirectory dependencies. The reference/importer maps only the explicit public
constructor and lookup signatures. Directory traversal and bounded GetItems return
core interfaces; StorageList and StorageNames are bootstrap transport helpers. No
Raven compiler or Runtime Contract configuration changes are introduced. Generated
core/runtime artifacts must match this development contract.

## Record and HashCode integration — 2026-09-24

The experimental Raven compiler now has an opt-in RuntimeRecordContract. The SDK
sets RavenRecordAssemblyName=NeoCLR.CoreProbe, RavenRecordEquatableType=System.Equatable`1
and RavenRecordHashCodeType=System.HashCode. Use a matching compiler/reference/library
bundle; Preview 9 artifacts do not supply this contract. Default .NET record synthesis
is unchanged.

The current slice admits non-generic record classes with integer, non-null string
and same-compilation record-class components (including nullable references), preserving
reference assignment while generating value equality, matching hashes, display and
Deconstruct. The [record sample](../records/README.md) is the executable gate. RAVT004
rejects nullable string/value components, external record components, record structs and
generic/inherited shapes rather than suggesting full record parity. The target hash
contract requires Add(int), Add(string) and ToHashCode. Nested records use typed
Equals/GetHashCode; strings use their content equality operator. HashCode is a mutable
Raven value type with Add(int/string), ToHashCode and Combine(int,int), without generic
component or comparer support. See [the design comparison](../../hash-code-design.md).

The importer admits private CLI initonly fields with stores restricted to declaring
constructors or recognized init accessors. IsExternalInit is accepted only as an exact
core return modifier on a property setter. Integer, string and application-reference output parameters support generated
Deconstruct. Application calls preserve each declared output's assignment obligation;
reference indirect stores require an exact supported type and a declared output. A standalone metadata marker has no runtime instance. Initialization-only
assignment is enforced by Raven; the runtime does not yet preserve an initonly field
flag or expose application property metadata. This follows the compiler/runtime split
of [.NET init accessors](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/init),
without claiming general CLI modifier support. `--record-metadata-checks` verifies
ordinary, static, foreign-marker and unassociated-setter rejection.

Shared candidate deferred for independent main validation: a source-defined generic
runtime-interface fixture exposed a reentrant negative metadata-name lookup cache
issue. The compiler regression uses a separately emitted provider assembly, matching
the real target boundary. No shared source-cache fix or Raven main release is claimed.

Nullable record-reference follow-up: generated members resolve the underlying record
type while retaining nullable source properties. Null components compare/hash/display
without dereferencing; Deconstruct retains null. The importer materializes literal
nulls only for application-reference call/constructor arguments through typed adapters,
preserving mixed/multiple argument positions. Intrinsic null strings remain rejected.
The sample verifies both paths; no new RuntimeRecordContract settings or public
library members are added.

Record comparison-operator follow-up (2026-09-24): generated class `==` and `!=`
preserve nullable reference annotations; struct operands remain values. No new
RuntimeRecordContract setting is needed. General Raven equality null guards use
Object.ReferenceEquals to avoid recursive operator calls. The target branch also
uses identity guards for record-class components in equality, hashing and display;
intrinsic string guards keep their existing target-supported lowering. User-defined
operators cannot redefine those internal null tests. Nullable value support is unchanged.

Boxed Boolean Object follow-up (2026-09-24): the existing compiler/importer boxing
path now reaches bounded runtime Equals/GetHashCode dispatch for Boolean, alongside
Int32. No Runtime Contract setting, emission change or reference signature changes.
The Object sample checks copied flags, exact-type/null comparisons, distinct identity
and 1/0 hashes. This does not enable Boolean record components or typed Boolean APIs.

### Transitional ref async protocol (2026-09-24)

The application reference now exposes generic ref Start<TState> and
AwaitOnCompleted<TAwaiter,TState>. The importer validates these exact core contracts
and specializes them for nongeneric application state machines and target Task
awaiters. Startup invokes MoveNext on existing storage. Pending registration retains
one state owner in the shared class builder, boxing only on the first suspension of
a value state. Later suspensions use the same owner; completion/cancellation clears
the builder's retained reference. No frame-backed ref escapes.

The bootstrap reference keeps the by-value helper signatures implemented in Raven;
they are not executed as declaration stubs. Source-generated state MoveNext and
SetStateMachine preserve no-result signatures even with byref value receivers.
The importer now admits checked application value loads and writable field addresses.
Runtime class field addresses retain their GC owner and enforce null/access checks.
Raven’s existing generic/ref builder selection supplies the protocol. A target-branch
policy correction separates target Task recognition and awaitless builder lowering
from the heap/value choice; ordinary .NET policy remains unchanged. RavenHeapAsyncStateMachines=false selects a value
state when written after the neoCLR props import. The default remains true.

The compiler-facing builder APIs are transitional and may be removed for runtime
suspension. This does not add generic async methods, custom awaiters, CLR exception
capture or a new scheduler. See [the experiment](../value-async/README.md) and the
[on-site protocol reference](../../../api-docs/async-builders.md).

Raven target policy fix: `89a40051e` on `codex/async-preview-readiness`; six existing
state-machine tests pass. The strict target matrix runs twelve Release cases,
including awaitless completion. No target changes were integrated into Raven main.

### Library Object consumers — 2026-09-24

Path implements Equatable<Path> and consistent Object equality/hash/display. Typed
equality takes T, not T?; Object.Equals(Object?) is the explicit null-aware boundary.
The reference marks Equatable's T operand non-nullable, though Raven currently
accepts a null literal through a constructed generic interface: a diagnostic gap,
not a nullable contract. No compiler source change is made in this slice. The
importer preserves Object ancestry and base construction for library class overrides.
Path reference/interface conversions are admitted and tested through HashMap callbacks.
The archived Neo profile keeps generated Path.bootstrap fragments because it lacks
the platform Object/HashCode classes; the Raven profile uses the complete source.

RuntimeTypeInfo now overrides Object equality/hash/display without changing the
opaque-handle layout, TypeIdentity or Raven configuration. Equality uses TypeEquals;
the bounded library hash uses FullName and can collide across distinct definitions.
The introspection fixture exercises generic/array shapes and boxed GetType under GC
pressure. Typed TypeInfo/Equatable operands stay non-nullable. No compiler code changes
are needed, and other descriptor identity contracts remain open.

Assembly/module wrappers now use their scoped catalog keys. Field/method/property
wrappers use kind, closed declaring type and definition index, with Name display.
The shared member-base reference projection normalizes virtual flags only on getters,
so explicit Object overrides retain dispatch metadata. Its closed hierarchy and native
field layouts are unchanged. Runtime Contract configuration and Raven compiler behavior
are unchanged; the member fixture checks reference imports, inherited Object display,
generic owner distinctions and map retention. Parameter ownership and reflected-context
semantics remain open, and member queries currently enumerate declarations only.

Parameter snapshots now carry a closed declaring TypeInfo, owner-kind integer and
member-definition index for Object equality/hash together with position. Native factory
layout and bridge validation changed together; rebuild the development runtime, library
and SDK as a unit. The archived value profile is unchanged. Public parameter ownership
resolution is not added; metadata remains descriptive. Runtime Contract configuration,
compiler semantics and emission are unchanged. The parameter fixture covers owner-aware
map lookup under GC; raw metadata regressions cover zero tokens, generic owners and
property/accessor distinctions. No Raven compiler changes are required.

### Object in generic payload signatures — 2026-09-24

The generic API mapper now recognizes CLI ELEMENT_TYPE_OBJECT independently of Cecil's
host-core scope, using the same rule as ordinary import signatures. Target-core Object
references remain accepted. This closes the HashMap<Object, ...> admission gap without
admitting arbitrary unresolved named types. Runtime Contract configuration, public
reference metadata, compiler semantics, library algorithms and native layout are unchanged;
no Raven compiler patch is needed. The mixed Object map fixture checks key dispatch,
boxed values, Option<Object> payloads, collisions, replacement, rehashing and GC. A
freshly built bridge is required; existing runtime/library fragments remain valid.

## Attributed custom unions (2026-09-24)

`ErrorBindings` emits `System.Runtime.CompilerServices.UnionAttribute` for carriers
with cases; the corresponding hand-authored runtime structs carry the same marker.
Empty standalone error values remain structs. Raven's general CLI metadata reader
recognizes the explicit marker with nested case constructors and matching public
`IsCase: bool` / `GetCase() -> Case` accessors as `IUnionSymbol`, on .NET or neoCLR.
RavenDoc consequently shows union signatures, U icons and grouped cases, preserving
legacy case links. This requires no runtime Union interface or Runtime Contract
option and does not change the Stored Value representation.

Compared with the boxed `Value` C# union contract, this projection preserves the
existing typed accessor ABI without introducing boxing. The cost is a separate
recognized metadata shape. Recognition does not synthesize `TryGetValue` or add
pattern extraction lowering: typed accessor use remains the validated custom
carrier contract. The general Raven tests use independent .NET class and struct
fixtures, reject unmarked/malformed shapes and verify documentation grouping and
same-named unrelated type visibility.

Runtime implementations now construct cases directly instead of wrapping a case
that already projects to its carrier. A clean compiler rebuild also required
Raven main's existing configured-unit identity fix on the neoclr branch
(`e86e1c773`): with the existing System.Void RuntimeUnitContract, imported generic
Flush results must compare equal to source `Result<unit, StreamError>` signatures.
No target option was added.

The same clean validation brought Raven main's structural imported-array identity
fix to neoclr (`0fee69abd`) so byte-array interface implementations retain their
virtual/final metadata flags. Focused .NET unit/array interface tests passed.
All bootstrap slices regenerated successfully with unchanged emitted `.neoil`
fragments; source/compiler/reference fingerprints changed. The 14 carrier
admission checks and targeted error runtime tests passed.

### String sequence construction — 2026-09-24

The target reference exposes String(Sequence<Char>) and a read-only grapheme indexer.
String implements Sequence with a private explicit Collection.Count; public Length
remains unchanged. The importer validates the trusted core constructor signature and
lowers newobj to the managed StringFromSequence transport. That facade invokes an
internal Raven factory; native services only concatenate snapshot chars and index
UTF-8 graphemes. Explicit Count matching uses MethodImpl identity rather than the
different explicit-property names emitted by C# and Raven.

Rebuild the bridge, target reference and selected Raven runtime together. Runtime
Contract settings and Raven compiler emission are unchanged. The archived Neo
bootstrap excludes the Raven collection contracts and factory. The
[construction sample](../string-sequence/README.md) documents validation and the
existing direct-empty-literal compiler limitation. No Iterable overload is selected.

### Explicit String interning — 2026-09-24

The String reference catalog exposes static Intern(String) -> String. The exact
signature binds to the Raven-authored method, which calls the trusted StringIntern
runtime service. The VM supplies one bounded pool per execution; loaded metadata
remains immutable. Isolated workers and separate host invocations have independent
pool state and inherited quota values. No literal rewriting or compiler interning is
performed. Rebuild the bridge/reference and runtime together; Runtime Contract
configuration and Raven emission are unchanged. See the
[checked sample](../string-interning/README.md) for quotas, lifetime and differences
from the longer-lived CLR pool. Wrong parameter shapes and static callvirt are
rejected by the bridge probe.


String parameter names now agree between authored methods, reference metadata and
runtime introspection. Replace value0/value1 named arguments with the member's
meaningful names: characters, text, left/right, other, substring, prefix, suffix or
byteStart/byteLength. Indexers retain index. Positional signatures do not change.
The String sequence sample verifies named arguments, including reordered arguments.


### Development TCP client integration

SocketBindings admits only the selected System.Networking.Sockets.Socket factory,
Send, Receive and Close signatures from the matching core reference. SocketError has the
existing union-carrier shape. SocketConnectCompletion and SocketTransferCompletion,
Socket's handle constructor and error decoder remain internal, and native operations
are bootstrap-only RuntimeServices calls. Native IDs are erased private Int64 values;
they are never part of the public API. Rebuild the reference, bootstrap library and
importer together. No Raven compiler change, Runtime Contract option, emitted state
machine convention or nullable policy changes in this slice. Socket awaits use the
existing Task/Promise and generated-state-machine contracts. See the
[client sample](../socket-client/README.md) for validation and current restrictions.


The send slice reuses a private SocketTransferCompletion for receive/send outcomes.
Socket.Send has the same Task<Result<int, SocketError>> shape as Receive. Its exact
reference binding calls the Raven-authored method; no new compiler or Runtime Contract
setting is required. Bootstrap-only SocketTransferResult replaces SocketReceiveResult;
rebuild all matching artifacts, since private services are not compatibility contracts.


Dns now uses the networking binder for public GetHostAddresses and private
DnsCompletion. The native result is copied into a managed array, exposed as
Sequence<String> through Task<Result<...,DnsError>>. Rebuild the reference, importer,
bootstrap and runtime together. No Runtime Contract configuration or compiler
implementation changes are required. The socket client tests hostname resolution.
Array-in-Result metadata emission and nested callback capture failures remain open;
see the socket design for the observed errors and independent reproduction gates.
