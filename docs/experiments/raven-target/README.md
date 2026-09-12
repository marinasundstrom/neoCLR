# Raven target emission probe

This is slice 2 of the [Raven target experiment](../../raven-target-experiment.md).
It tests the existing compiler API and inventories emitted PE metadata. The current
follow-up imports a bounded static subset and executes it against neoCLR's real System
library through the separate runtime verification command below. Earlier slice notes
retain their historical limits. This is not general Raven or direct PE execution.

## Reproduce

Current probe validated on 2026-09-12 with Raven revision
`1d7341fa64a66b514e5e68031b6d072d8140ea3a` on `codex/neoclr-target-resolution`,
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
