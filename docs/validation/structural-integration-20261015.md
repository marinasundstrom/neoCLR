# Structural type integration — 2026-10-15

Author direction: integrate the experimental structural type work, remove feature
flags and retire nominal ad-hoc Union/Tuple carriers. This integrates the existing
validated native implementation; it does not promise every structural combination.

## Integration

neoCLR `main` incorporates `codex/structural-union-intersection` through
`5e10ec336`. Main's three TypeInfo relationship commits were already carried on
the feature line. After resolving the merge, the runtime/compiler/library tree was
byte-for-byte the tested feature tree before integration-record edits. TypeInfo
relationships and JSON/reflection consumers are retained.

Raven's neoCLR line `codex/source-object-metadata-resolution` advances to
`codex/neoclr-union-intersection` through `f0ff3b5e8`. Raven main remains separate;
general candidates and main-based fixes keep their recorded provenance. Structural
binding/emission is selected by the neoCLR target contract; normal .NET behavior
and declared nominal Raven unions remain supported.

## Implemented migration

- Runtime structural Cargo features, compiler semantic/emitter switches, project
  switches, driver tuple switch and native reader opt-ins are removed.
- Native readers admit supported nominal and structural signatures together,
  with explicit dependency checks and no structural CLI projection.
- Runtime `System.Union` and `System.Tuple` carriers and the generated nominal
  Tuple slice are removed. Range returns named structural tuples directly and in
  Result. Rebuild consumers: these native signatures are not binary-compatible
  with the old nominal carriers.
- Tuple names are native signature data; compatibility remains positional.
  UnionTypeInfo, IntersectionTypeInfo and TupleTypeInfo remain introspectable.
- The five-library native API snapshot is rebuilt. Historical CLI documentation
  scaffolds are explicitly excluded from native API presentation.

## Validation

- 398 runtime unit/structural/constraint/identity checks pass; three existing tests
  remain ignored. Structural checks also run without default Cargo features.
- 175/175 host metadata contract tests pass, including attribute-row boundary
  coverage, structural reads/imports and explicit dependency failures.
- 50 focused Raven symbol, constraint and portable-pattern tests pass; native
  source tuple/mutation/import, composition/wrapper-call and external constraint
  probes pass with default readers.
- The fresh carrier-free Runtime/Data/Networking/Web/PostgreSQL bundle builds.
  [Range/Result consumer](../experiments/native-structural-types/carrier-free-results.json)
  passes interpreter, macOS ARM64 AOT and managed JIT, with zero arena allocation
  and AOT undefined/bounds sanitizers. A second fresh run with unified readers
  also passes (`target/readers-carrier-consumer/results.json`).
- Fresh [direct Task union](../experiments/native-structural-types/integrated-task-direct-results.json)
  and [closed-generic Task union](../experiments/native-structural-types/integrated-task-generic-results.json)
  consumers pass interpreter/AOT/JIT with eager/paced collection and cleanup against
  the carrier-free bundle. Exact binaries and source fingerprints are retained.
- The five TypeInfo relationship tests pass again on the merged tree.
- Native API rendering and reference snapshot checks pass. Website publication is
  separate; no release or deployment is implied.

## Explicit follow-up gaps

Bare generic-parameter alternatives (`T | Error`), tuple alternatives inside unions,
and arbitrary combinations of structural signatures remain unsupported in parts of
the authoring/compiler/backend pipeline. They must reject rather than silently use
nominal carriers. Whole-union boxing, generalized reflection construction and broader
platform/backend combinations retain their documented limits. These are gaps, not
permanent structural type-system rules. The supported scoped nominal generic forms,
including `Response<T>` constituents, are distinct from bare `T` alternatives.

Native execution evidence is macOS ARM64. JIT hosts are not sanitizer-instrumented.
Existing scalar/tuple benchmark evidence supports specific allocation/copy claims,
not a general performance guarantee for all mixed or asynchronous workloads.
