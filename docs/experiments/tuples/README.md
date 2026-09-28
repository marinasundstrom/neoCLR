# Tuple support (development, 2026-09-28)

The [contract](../../tuples.md) records `System.Tuple` as neoCLR's value tuple name.
The Raven source is `runtime/raven/src/System/Tuple.rvn`; its imported generic
constructors live in the generated Tuple library slice. No Rust runtime change is
needed: field layout, copying, boxing, GC and arrays use existing value machinery.

## Reproduce

Build Raven in the isolated `codex/neoclr-tuples` worktree, then build the target
bridge with that RavenRoot. Regenerate the Tuple slice:

```sh
dotnet build docs/experiments/raven-target/Probe.csproj \
  -p:RavenRoot=/path/to/Raven-neoclr-tuples -p:BuildProjectReferences=false -p:WarningLevel=0
python3 docs/experiments/raven-target/build_runtime_library.py \
  --compiler /path/to/Raven-neoclr-tuples/src/Raven.Compiler/bin/Debug/net11.0/rvnc.dll \
  --bridge docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll --slice Tuple
dotnet docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --tuple-checks /path/to/NeoCLR.CoreProbe.dll
python3 docs/experiments/raven-target/collection_library.py /path/to/System.neoil
python3 docs/experiments/tuples/verify.py \
  --runtime target/release/neoclr \
  --bridge docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --reference /path/to/NeoCLR.CoreProbe.dll --system /path/to/System.neoil \
  --evidence docs/experiments/tuples/validation.json
```

Use the collection-library builder to produce System.neoil from runtime/System.neoil;
it includes the generated Tuple slice. `--tuple-checks` produces an ordinary consumer
reference and checks valid and mutated tuple layouts and constructor/field signatures.
`verify.py` independently compiles every saved source, checks emitted value-type/core
identity (including nested arguments), verifies typed stacks/control flow and executes
on neoCLR. Each attempt has a fresh output directory.

## Evidence and compiler separation

The [saved consumer evidence](validation.json) covers nine passing cases on macOS
arm64: syntax/labels/deconstruction/copying/nesting, all seven explicit arities and
zero defaults, and boxing/unboxing after GC plus tuple array elements. The main
consumer also passed using the debug native runtime. The compiler-reference fixture
checks 70 valid/malformed layout and member cases, including fields, explicit
layouts, packing and constructed member signatures. The target branch passes 41
focused compiler tests after integration.

Raven main `90b996b1b` fixes a general target metadata issue, independently reproduced
with standard .NET ValueTuple: tuple projections lost nested generic identities,
and applying MakeGenericMethod to a metadata factory proxy failed. The main-based
fixture checks metadata and executes to 42. All 31 focused tests passed before
integration. The fix is cherry-picked as `ee3a23d15` in the target worktree, retaining
that branch's nominal Void and async metadata handling.

Target commit `79a5d7ffc` selects System.Tuple for NeoCLR.CoreProbe. It has no main
counterpart. The target's existing void-pointer projection omission was also found
by the broader focused filter; the correction already exists on main in adaaa3db2
and is restored here without importing that mixed commit's unrelated changes.

An eight-element expression probe is rejected with `Emission failed while processing
method`; this is a known diagnostic limitation, not supported wider tuple syntax.
A dedicated unsupported-arity diagnostic remains open alongside wider tuple support.

This is source/development validation, not packaged SDK qualification, publication,
or evidence of execution on Windows, Linux, .NET Framework or NanoFramework.
Existing non-tuple library fragments are reused; their shared input fingerprints are
refreshed for the additive source/project registration without claiming regeneration.


## Pre-existing audit finding

The broader `verify_source_ownership.py` audit reports the same two unowned bodies
with both the original HEAD collection builder and this change's builder:
`neoCLR.Runtime.StringFromSequence(System.Collections.Sequence<Char>)` and an
instance `get_Value() -> System.Object`. Neither body is introduced or edited here.
This audit is not reported as passing. Tuple library snapshot hashes, the focused
API render (7 types, 7 constructors, 28 fields) and API source fingerprints pass.
The full website build was not run, following the focused-validation policy.
