# Raven target emission probe

This is slice 2 of the [Raven target experiment](../../raven-target-experiment.md).
It tests the existing compiler API and inventories emitted PE metadata. Slice 3 adds
a dependency-closure audit that resolves only explicitly supplied assemblies. It does **not**
execute the output, implement a neoCLR target, or provide neoCLR's runtime library.

## Reproduce

Recorded on 2026-09-12 with Raven revision
`d92b02812740ae052f277c23151e9cc208f7672d`, neoCLR starting revision `0a60f9b`,
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
- The target still references `mscorlib`. Cecil's declaration fixture introduces that
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
`7dfaca6`, with the same pinned Raven revision and SDK as slice 2.
