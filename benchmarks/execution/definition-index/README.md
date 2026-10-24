# Scoped definition identity lookup — 2026-10-24

The author's partitioned-ID suggestion and metadata-token question led to an exact
identity index for existing metadata definition lookup. It changes neither token
encoding nor the managed reflection API.

## Tokens and runtime identities

.NET tokens encode a table tag in the high byte and a row in the low 24 bits;
`0x06000004` identifies MethodDef row four. Tokens identify an element together with
its module; generic TypeSpec/MethodSpec resolution may additionally require generic
context. Sources: [token format](https://learn.microsoft.com/en-us/dotnet/standard/metadata-and-self-describing-components),
[module-scoped identity](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.memberinfo.metadatatoken),
[generic resolution context](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.module.resolvetype).

neoCLR's `src/metadata_tokens.rs` retains imported origin tokens and synthesizes
TypeDef/Field/MethodDef/Param/Property tokens with the corresponding CLI table tags
for native definitions without origins. This is support for the exposed definition
tokens, not all CLI token forms or a complete `Module.Resolve*` API. Structural type
forms currently return zero rather than inventing a source TypeDef row.

Three identities must remain distinct:

- The source module and original metadata token describe the source declaration.
- `TypeDefId` contains module, optional revision and a zero-based native definition
  index. Imported origin tokens need not equal this index plus one.
- The linked/native executable's type ordinal is local to that specialized image.

A source token can key lookup with its source module and suitable generic context.
It cannot be used blindly as an index into a linked type vector. The implemented
index accelerates **existing TypeDefId lookup**, not raw-token resolution.

## Implementation and limits

Within an existing immutable `runtime_lookup::with_module` scope, a lazy hash map
maps complete TypeDefId values to rows in the current module. Repeated definition
lookups no longer scan every type. Equality includes module and revision, and the
first row wins for duplicate identities, matching the old linear search. Missing
identities remain absent. Existing structural/array/reference handling stays in
`metadata_tokens::definition`; the unscoped fallback remains linear.

The index contains already-loaded definition identities and row numbers. It loads
no assemblies, reflection payloads, signatures or new type descriptions. Its owned
keys clone module/revision strings once per indexed definition: that is a real
memory/preparation cost. The map is built only when definition lookup uses the
scope, and is discarded with it. Nested scopes, unwinding, independent threads and
mutation between scopes retain the established lifetime rules. There is no global
cache, arbitrary string interning or cached generic-conformance result.

The tradeoff favors repeated queries. A single query can cost more than a linear
walk because it prepares the index; token prefix counting for synthetic fields and
parameters also remains a separate linear path. No whole-program or native HTTP
speedup is claimed for this slice.

## Validation and cost experiment

Focused tests cover module/revision separation, duplicate-first behavior, missing
identities, array/structural handling, nested unwind restoration, thread isolation,
rebuilding after mutation, lazy creation, independent owner-index state, source
origin preservation, token tag/row encoding and overflow rejection. Reflection
consumer results are retained in `focused.json`. In total, 32 focused functional
tests pass, plus the separately executed ignored cost experiment. The first consumer
link attempt used an incompatible default SDK; rerunning with the explicit known-good
SDK succeeds. The failed setup is retained in `default-sdk-failure.json.gz`.

The ignored release-mode `metadata_tokens::tests::definition_lookup_cost` test
queries the last of 10,000 definitions 20,000 times in four alternating-order trials.
Linear lookup takes 244–258 ms; warm indexed lookup takes 0.66–0.78 ms. Preparation,
including the existing name indexes and the new identity index, takes 2.5–2.9 ms.
This deliberately stresses repeated lookup and does not predict typical workload
speedup. The raw test log and `cost.json` retain every observation.

```sh
cargo test --lib metadata_tokens
cargo test --release --lib runtime_lookup
cargo test --release --test reflection --test reflection_hierarchy --test reflection_construction
cargo test --release --lib metadata_tokens::tests::definition_lookup_cost -- --ignored --nocapture
```

A first debug test launch waited in macOS `_dyld_start` before entering the harness;
its retained sample is a loader diagnostic, not an application deadlock or benchmark.

## Tracing recipe inventory

The ongoing GC investigation also inspected the current plaintext/JSON application
with the existing compiler `--inspect` path. Both pass admission. Compressed files
retain only object recipes from the full reports; commands and full-report hashes
are retained separately.

| Selected ordinary-object recipes | Plain | JSON |
| --- | ---: | ---: |
| Object types | 99 | 152 |
| Payload lanes | 1,059 | 1,263 |
| Reference trace slots | 324 | 389 |

These are static, unweighted counts, not allocation frequencies or expected GC
savings. They exclude some special/boxed/static/array allocations. The existing
recipes provide a starting point for small reference maps without loading reflection
names during collection, but they are not currently collector descriptors.

`text-arena.c::allocate_object` serves ordinary allocation and static storage;
static tables also use OBJECT_STORAGE with a count in their first word. Callable
carriers use UINT32_MAX and boxes have specialized payload rules. Therefore the
existing allocation kind and leading word cannot unambiguously select an ordinary
instance map. Integration needs explicit producer classification and immutable
image/arena lifetime ownership; unsupported shapes must keep conservative tracing.
No collector map or new allocation-kind encoding is adopted by this inventory.
