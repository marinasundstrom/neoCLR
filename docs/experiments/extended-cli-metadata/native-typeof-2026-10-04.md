# Native typeof boundary — 2026-10-04

Raven `1c3e14bc7` on codex/metadata-consumer emits configured typeof through semantic
operands and the independent metadata IILGenerator. The metadata token foundation is
neoCLR `ad375d0b`; this follow-up recognizes a RuntimeTypeHandle TypeDef inside the
exact selected core as well as its AssemblyRef encoding. No runtime opcode change.

The explicit primitive bootstrap now declares the opaque handle and the real TypeName
service. This uses the existing runtime implementation, not an executable CLI body.
The separate test provider stores the handle in a descriptor and asks that service
for its name. A consumer compiled with provider sources absent checks typeof(T) after
Int32 substitution and typeof on the imported Descriptor. Runtime verification passes;
execution returns 42 with empty stdout. Names are System.Int32 and
HandleContract.Descriptor. The fixture is not System.Introspection and does not count
as completing production JSON reflection support.

Reproduce after building the native compiler and bridge tool:

```sh
dotnet docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --reference-comparer-storage-core /tmp/HandleCore.dll
python3 docs/experiments/extended-cli-metadata/bootstrap/verify_type_handles.py \
  --compiler /absolute/path/to/Raven/src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  --runtime target/debug/neoclr --core /tmp/HandleCore.dll \
  --seed-source /absolute/path/to/source-text/seed.neoil \
  --base-library /absolute/path/to/source-text/Numbers.dll \
  --ownership /absolute/path/to/source-text/ownership.json \
  --output /tmp/typeof-fresh
```

The source-text inputs are the explicit [text bootstrap](source-text-2026-10-04.md).
The driver extends its retained seed with only TypeName and records the resulting
source, ownership, artifact hashes, revisions, commands and execution. The manifest
selects HandleProvider's Info/Context as RuntimeTypeOfContract for the consumer; it
leaves the original source-library ownership intact. No application reference falls
back to a CLI projection. See [execution evidence](native-typeof-2026-10-04.json).

Validation: 147 C# metadata groups, generated type-token PE verify/run42, catalog
selection checks and 17 Raven typeof tests pass. The latter retain the existing .NET
configured/default execution controls. The existing native JSON DOM/internal-codec
gate [passes with the updated core/seed](json-after-typeof-2026-10-04.json);
it remains separate from object mapping.
Website production capability claims and the guest API snapshot remain unchanged;
the host API is documented in the experimental metadata reference.

Next is the production introspection closure: descriptor factories and inheritance,
Object.GetType, ParameterSnapshot, and real member enumeration/access/creation bindings.
The C# metadata-only facade remains separate from these execution-bound descriptors.
Open generic definitions and virtual typeof context resolvers reject in this native
profile. .NET behavior and its Reflection/Emit backend remain unchanged.
