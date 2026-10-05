# Source-built JSON object mapping — 2026-10-05

The production JSON serializer/object mapper and introspection sources now compile to
an independent native assembly. Two consumers compile against emitted artifacts only
and execute on NeoCLR. No production mapper, serializer, descriptor or existing sample
source was rewritten to make this gate pass.

## Executable gate

`bootstrap/verify_json_mapping.py` builds 17 production files plus four internal native
adapters into JsonIntrospection.dll, then builds unchanged ResultOperators.rvn as a
separate library. It compiles, verifies and runs:

- `bootstrap/json-object-consumer.rvn`: nested objects; Unicode string/int/Boolean
  properties; explicit auto-accessors and a counted custom setter; integer and jagged
  arrays; mutation through a shared child reference; exact serialized output; missing
  fields and wrong element types. Invalid nested input produces no constructor/setter
  output between the validation markers. Successful exit: 42.
- The unchanged `../json-object-mapping/Mapping.rvn` and `Main.rvn` pair: reflection
  property mapping, construction, Result error conversions and stream round trips.
  Successful output: `JSON object mapping checks passed`, exit 0.

The first consumer's exact stdout is:

```text
Model constructed
Name assigned
Model constructed
Invalid input begins
Invalid input ends
Native JSON object mapping passed
```

The two construction lines include the source initializer on Batch.Child. The runtime
executes actual constructors and setters; it does not substitute DTO storage writes.

## Architecture and .NET comparison

The metadata library retains ordinary CLI Sealed flags for final reference classes in
builders, manual definitions, readers and introspection. This fixes the mapper rejecting
ordinary Raven model classes as open. Closed families remain distinct. Boolean is now
an explicit canonical scalar provider alongside the numeric implementations; it retains
the existing Boolean element code and native scalar storage. No format version changes.

Raven's portable adapter emits explicit auto-accessors from existing backing-field
symbols, conversion declarations and already-bound exact user conversion calls. The
ordinary .NET Reflection/Emit backend is unchanged. Importer facts stay in symbols;
emission does not reopen imported metadata objects.

Runtime queries produce internal snapshot recipes. Materialization uses the selected
service return signature, resolving descriptor providers in that contract's exact
assembly/module and verifying interface assignability and field signatures. Logical
service names never become aliases in the loaded type catalog. RuntimeTypeInfo and
its nominal/function providers retain actual handles. Arrays become owned managed
arrays; ParameterSnapshot is an internal immutable wrapper over a ParameterInfo array.
Source Option values use the source union's checked case layout. This is a bounded
runtime/library ABI, not a general reflection-object factory or reflection dependency
in the host metadata facade. Provider field/case layout changes require coordinated
runtime changes. Allocation and recursive materialization enforce existing budgets.

Source service declarations retain MethodImpl(InternalCall), as on the CLI, but only
known NeoCLR signatures bind. Property-set execution admits no-result calls and runs
the real setter; other services retain their previous no-result restrictions. This
corresponds to .NET property invocation semantics without introducing binder coercion.
Boolean interface dispatch uses the same canonical member resolution as other source
primitives. Existing UTF-8 text and exact boxed scalar contracts remain unchanged.

## Reproduce and ownership

Build Raven with the native adapter explicitly enabled:

```sh
dotnet build /path/to/Raven/src/Raven.Compiler/Raven.Compiler.csproj -f net10.0 \
  -p:NeoClrMetadataProject=/path/to/neoclr/tools/metadata/NeoCLR.Metadata.Experimental/NeoCLR.Metadata.Experimental.csproj
cargo build
python3 docs/experiments/extended-cli-metadata/bootstrap/verify_json_mapping.py \
  --compiler /path/to/rvnc.dll --runtime target/debug/neoclr \
  --core /path/to/IntrospectionCore.dll \
  --seed-source /path/to/text-gate/seed.neoil \
  --ownership /path/to/streams-gate/ownership.json \
  --base-library /path/to/text-gate/Numbers.dll \
  --encoding-library /path/to/encoding-gate/Encoding.dll \
  --streams-library /path/to/streams-gate/TextStreams.dll \
  --output /tmp/native-json-mapping-fresh
```

Use a fresh output directory. Build the explicit core with the existing Probe
`--reference-comparer-storage-core` mode from the matching CoreDeclarations (including
public ParamArrayAttribute and MethodImpl markers). Its CLI assembly supplies primitive
signature/marker bootstrap only. Obtain the numeric/text library and retained seed from
[the text gate](source-text-2026-10-04.md), then
[Encoding](source-encoding-2026-10-04.md) and
[TextStreams](source-text-streams-2026-10-04.md), whose output ownership.json is the input
above. Do not use a compiler build that omits NeoClrMetadataProject.

The script extends only the retained service seed with checked-in handle/construction
wrappers, Object reference equality and the ParamArray marker constructor. Source
JsonIntrospection owns Boolean and typeof/descriptor definitions; Numbers owns the
other source primitives and iteration/collection contracts. This incremental Boolean
placement is explicit and can move into a cumulative source-built System later.
Application and rebuilt-library imports remain native. Generated ownership.json and
System.neoil record the exact selections; dependency artifact hashes are recorded.
No public application declaration is supplied by a seed stub.

Execution explicitly permits 100,000,000 instructions for the combined parser/mapping
workload. This gate establishes correctness under that budget, not default-budget
coverage or a performance improvement. Hashes and commands identify the tested
artifacts, even where revision fields name the working-tree base before commit.

## Validation and limits

[Executable commands, source/dependency hashes and stdout](source-json-mapping-2026-10-05.json).
Compiler commit: `8f015386e`; metadata prerequisite: `df6d8e51`. Runtime changes are
in the commit containing this record. Tests: 153/153 C# metadata contract groups;
eight focused runtime reflection/scoped-provider checks; 24 native runtime checks
(including the new no-result signature rejection test); eight focused .NET property/conversion/Result tests.
Earlier nine .NET pattern checks and the mapping-guards consumer remain relevant.

The API snapshot check still reports the previously recorded stale full guest snapshot.
New host metadata APIs are documented in the manual reference; internal adapters add
no public guest API. No partial library was substituted for the full reference, and no
website build or publication was performed. The previously documented integration-only
static-interface default-value binding regression is not claimed fixed here.

This closes the selected native JSON mapping gate, not all reflection APIs or the entire
System library. The HTTP-dependent Public.rvn sample, .NET source-library parity,
Enum/Uuid/Option mapping expansion and socket/HTTP integration remain separate work.
Next reassess the cumulative source-library inventory, then use the existing HTTP
sample to expose network-service, resource-lifetime and async gaps. Preserve this
artifact-only mapping gate while expanding coverage.
