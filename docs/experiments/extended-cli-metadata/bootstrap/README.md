# Source-built union bootstrap (development)

This bounded native gate compiles the unchanged iteration contracts, Propagatable,
Option and Result into NeoCLR.Collections.dll. A second ordinary compiler invocation
imports only that artifact, then neoCLR verifies and executes the consumer. This is
not the complete dual-target library gate or the collection application gate.

Owners:

- Primitive semantic declarations: the explicit `--reference-storage-core` image.
  Its CLI bodies are never executed; it contains no source-owned union/collection types.
- Retained executable services: `union-seed.neoil`, module System. Its inventory is
  Void, Char, RuntimeTypeHandle, Object, String and Console. Native intrinsics implement
  string concatenation, type-handle display and console output.
- Source library: every declaration in `union-ownership.json`, consumed by the compiler
  as its ownership/iteration configuration. No seed copies of these declarations exist.

Object.ToString is a temporary target adapter: it asks the existing native type-handle
query for the display name directly instead of allocating the guest introspection facade.
That removes the Option/Result bootstrap cycle without a placeholder body. The runtime's
existing intrinsic boxed-primitive formatting and ordinary value overrides still dispatch.
Unlike a full .NET core library, this seed deliberately exposes only the services required
by this gate; unsupported core member references reject. Broader source-library ownership
and executable .NET adapters remain subsequent work.

From the neoCLR worktree, with an already built Raven driver and neoCLR runtime:

```sh
dotnet run --project docs/experiments/raven-target/Probe.csproj \
  -p:RavenRoot=/absolute/path/to/Raven -p:UseRavenCoreReference=false \
  -p:WarningLevel=0 -- --reference-storage-core /tmp/UnionCore.dll
python3 docs/experiments/extended-cli-metadata/bootstrap/verify_source_unions.py \
  --compiler /absolute/path/to/Raven/src/Raven.Compiler/bin/Debug/net10.0/rvnc.dll \
  --runtime target/release/neoclr --core /tmp/UnionCore.dll \
  --output /tmp/source-unions-fresh
```

The output directory must be fresh. Evidence records every command, source/artifact hash,
stdout and exit status. Expected output is `Option.Some(40)` then `Result.Error(7)`,
with exit 42. The consumer checks copy independence, false-output initialization, unit
residuals, error residuals, generic pattern matching and boxed virtual display. Missing
library input and duplicate source ownership in a seed must reject before publication.

The seven older native consumers use their own full CoreProbe/seed baseline; the small
core here intentionally does not supply their configured typeof facade.

## ArrayList assessment

Generate the next primitive profile with `--reference-collection-storage-core` in
place of `--reference-storage-core`, then pass that image and `--collections` to the
same Python driver. This adds public CLI Func/Action declarations for Raven's existing
callback binding and the marked namespace declaration for System.Fail. These are explicit
compiler bootstrap symbols; no source collection or union declaration is copied into core.
The callback declarations remain the existing temporary CLI transport, not a new structural
Function semantic design or a claim that the separate Function experiments are integrated.

`arraylist-ownership.json` adds unchanged ArrayList and its internal iterator to the
source library. `collection-seed.neoil` includes the union seed and supplies System.Fail
through the existing terminal runtime intrinsic. The negative-capacity execution test
checks the real diagnostic and failure exit, so its reference-only core body is never
mistaken for executable behavior.

Current result: the library and a separate native-import consumer compile and execute
successfully. The consumer has no library source inputs. Alias mutation, independent copy,
iteration and Find callback checks return 42. The negative-capacity consumer also compiles
against the library alone and terminates through System.Fail. Missing-library and duplicate
seed ownership guards remain required. The previous blocked assessment is retained as
historical evidence; the current driver expects successful separate execution.

Native reader function signatures now project through metadata-only FunctionTypeInfo views,
with generic substitution and explicit dependency resolution. Raven maps the Boolean
predicate shape to its existing callable symbols, and emission authors the callback operand
from those symbols. Explicit no-result callbacks remain an unsupported Raven import category;
metadata views preserve their distinction from inhabited-unit callbacks. Wider function
semantics, .NET class-library adapters and the full collection application remain separate work.

### Separately compiled HashMap and comparers

Use the same collection primitive core and compiler/runtime paths as the ArrayList
workflow, replacing `--collections` with `--hashmap` and choosing a fresh output directory.
`hashmap-ownership.json` extends source ownership to Comparer, FunctionComparer,
EqualityComparer, FunctionEqualityComparer, Map, MutableMap and HashMap. Their sources
are unchanged. The consumer receives only the emitted native library reference.

The consumer forces every key into one bucket, grows beyond initial capacity, rejects
an existing key, replaces and inserts through Set, checks missing Option payloads,
checks Keys snapshot independence, and mutates a shared ArrayList through a retrieved
value. Both equality/hash callbacks and ordering callbacks execute through imported
contracts. Expected stdout is empty and exit status is 42. Existing capacity-failure,
missing-reference and duplicate-ownership guards also run. HashMap does not currently
expose removal. This is native execution evidence, not the full .NET library gate.

See [recorded commands, revisions and hashes](../hashmap-import-2026-10-03.json).
No metadata, compiler or runtime changes were necessary for this extension. The source
contract mirrors the supported CLR generic interfaces and callable operations; no new
NeoCLR semantics or format version is introduced. Query composition and the broad
application remain the next acceptance boundary.

### Native extension declaration/import regression

Use `--extensions` with the same paths and primitive core. This runs the cumulative
HashMap gate, then builds `extension-library.rvn` as a second native library and compiles
`extension-consumer.rvn` against both artifacts with library sources absent. Generic
receiver predicates and a method-generic selector execute (empty stdout, exit 42).
This is an isolated compiler regression, not a replacement for runtime query sources.

Raven uses its existing CLR lowering: extension receiver type parameters become method
parameters on a nongeneric static container. The container carries the standard
`System.Runtime.CompilerServices.ExtensionAttribute` marker through the same bounded
embedded-attribute profile as native unions. Native namespace discovery reads the marker
through introspection, then leaves receiver applicability and inference to Raven binding.
No format version, instruction or runtime dispatch change was needed. Constrained
extensions, static extension members and extension properties are outside this slice.

`query-ownership.json` records the cumulative *assessment* sources, including unchanged
SingleError and Operators. Compilation now stops at the object-to-generic conversion in
`OfType<U>`, publishing no library. Do not use that manifest as proof of executable query
support. See [extension evidence](../extension-import-2026-10-03.json) and
[full source assessment](../query-source-assessment-2026-10-03.json). Next add the supported
CLI-equivalent unboxing/generic-cast operation through shared emission, metadata and
runtime validation, then resume the unchanged broad application.

### Unchanged full query library executes

Use `--queries` with the same compiler/runtime/core paths and a fresh output directory.
This selects `query-ownership.json`, compiles the complete unchanged SingleError and
Operators sources cumulatively with unions and collections, and then compiles only
`query-consumer.rvn` against the emitted native artifact. OfType, Filter, Map, ToList and
Single execute with empty stdout and exit 42. Heterogeneous boxed integers/string values
check filtering and unboxing; a retrieved ArrayList checks retained object identity.
The capacity-failure, missing-library and duplicate-ownership guards also run.

The earlier OfType assessment is historical: UnboxAny authoring and shared lowering now
close it using the existing CLI/native instruction. The unchanged broad application
still rejects `Order[].Filter` during binding. The next task is canonical array iteration
contract participation in extension receiver inference/conversion, followed by native
emission and runtime verification. No sample rewrite is used to bypass that failure.

### Nominal Array<T> assessment (not an execution gate)

Author direction on 2026-10-03 keeps arrays backed by nominal Array<T>; structural array
identity changes are deferred to structural-types work. Vector storage remains distinct
from the nominal member/iteration contract, just as CLI vectors have special storage.

`array-ownership.json` extends the cumulative source set with unchanged System/Array.rvn,
selects `ArrayShapeTypeName = System.Array`1`, and explicitly selects source-owned
Propagatable for NeoCLR propagation. Its library compiles and imports. The independent
`array-consumer.rvn` compiles, but execution fails while converting the vector to the
source-owned Iterable contract: the runtime cannot select a matching implementation.
This is a linking/backing contract gap, not grounds for structuralizing arrays or
rewriting the application. The ordinary `--queries` success gate remains unchanged.

Reproduce by using the same ordinary driver arguments as the query evidence, substituting
`array-ownership.json`, compiling its source list to `NeoCLR.Collections.dll` in a fresh
directory, then compiling only `array-consumer.rvn` against that artifact and running it
with the same seed. Exact commands and artifact hashes are recorded in
[the nominal array assessment](../nominal-array-assessment-2026-10-03.json).
Next connect vector storage to the output-owned nominal Array<T> descriptor and its
source-authored iterator through explicit identity-aware compiler/runtime contracts.
Do not rely on the old translated System.ArrayEnumerable adapter's hard-coded names.
