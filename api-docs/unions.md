# Union

Raven supports named nominal unions and ad-hoc unions written as type expressions.
The native feature-branch profile represents ad-hoc unions directly in metadata;
legacy CLI profiles retain nominal carriers.

## Nominal unions

Named unions give a domain its own identity and cases. They remain preferable for
many public APIs, especially when an error contract should be independent of the
operations used internally.

- [Option&lt;T&gt;](xref:System.Option`1) documents Some and None.
- [Result&lt;T, E&gt;](xref:System.Result`2) documents Ok and Error.

RavenDoc renders each declaration's cases, properties, methods and applicable
extensions. There is no universal case list shared by every declared union.

## Ad-hoc error unions — development

A return type such as `Result<Config, FileReadError | NumberParseError>` exposes
both possible error types. Each underlying error can propagate with `?` without
inheritance, a shared interface, a wrapper error or implicit converters written
solely for aggregation. Extract `Error(let error)` and use typed patterns to
inspect the listed alternatives. Unrelated errors are rejected.

Use a nominal union or dedicated type when the API needs domain-specific cases
and a stable abstraction. Use implicit converters for intentional domain mapping.
Review existing aggregation-only wrappers too, weighing clarity against public
signature compatibility and the meaning of named cases.
See the [configuration example](../features/outcomes/#adhoc-errors).

## Temporary carrier reference

The matching development library provides these ordinary nominal backing types
in System.Runtime; prefer Raven's `A | B` spelling in source:

| Source alternatives | Carrier |
| --- | --- |
| `A | B` | [Union&lt;T1, T2&gt;](xref:System.Union`2) |
| `A | B | C` | [Union&lt;T1, T2, T3&gt;](xref:System.Union`3) |
| `A | B | C | D` | [Union&lt;T1, T2, T3, T4&gt;](xref:System.Union`4) |
| `A | B | C | D | E` | [Union&lt;T1, T2, T3, T4, T5&gt;](xref:System.Union`5) |

Each carrier has one constructor and `TryGetValue(out value)` overload per
alternative. A successful extraction returns true and assigns the value; an
unsuccessful extraction returns false and clears the output to its default.
`HasValue` reports an active alternative. `Value` returns its boxed value, or null
for an inactive default carrier. Prefer typed patterns over boxed inspection.
`ToString()` is diagnostic text, not an alternative discriminant.

This is a temporary bridge, not native structural identity, normalization,
order-independent equality or union introspection construction. The two-to-five
arity bound belongs to the bridge, not the intended platform semantics. The native feature-branch profile replaces this carrier encoding with structural
signatures;
consumers must use matching compiler/library artifacts and rebuild when that
representation changes. No structural union type is being advertised as shipped.

[All structural families](structural-types.md)


<a id="branch-only-reference-experiment"></a>

## Native structural unions

A structural union is a value type whose signature lists its alternatives. The
alternatives may themselves be values or references. Copying a reference alternative
preserves the referenced object's identity. The union has no nominal declaration
or base type, and does not acquire members through nominal inheritance.

For example, the tested native task consumers use:

```raven
func Echo<T>(value: Response<T> | HttpError | JsonError) -> Response<T> | HttpError | JsonError {
    return value
}
```

The generic method executes with `T = int`. Separate consumers deliver all three
alternatives through `Task<Response | HttpError | JsonError>` and
`Task<Response<int> | HttpError | JsonError>`, including completed, queued and
async-forwarded paths. These are useful data-composition types; their use is not
restricted to API results.

Identity ignores alternative order and duplicates. Exact value alternatives use
checked boxing/extraction in the current interpreter; exact reference alternatives
use checked casts. Conversion to one alternative is explicit unless every possible
alternative already satisfies the target. This is membership in existing types,
not implicit implementation of an interface or synthesis of a nominal union.

On the development runtime branch, interpreter unions also accept function
alternatives such as `union<fn<Int32,Int32>,Int32>`. Narrow to the exact function
signature before invocation. Copies retain the target and bound receiver; wrong
signature casts fail. Bounded macOS ARM64 AOT storage now uses an inline tag and
separate callable slots, with receiver retention checked under eager/paced native
collection. Injection adds no union carrier allocation; binding retains its ordinary
callable-descriptor allocation. A separately compiled Raven library/consumer now
qualifies typed callable/scalar patterns and invocation under AOT collection.
JIT lifetime and native Object widening remain open. Function intersection bounds remain unsupported.

`UnionTypeInfo.Types: Sequence<TypeInfo>` exposes a constituent snapshot.
`TypeInfo.IsUnion` identifies structural unions; use
`NominalTypeInfo.IsUnionDeclaration` for named Raven union declarations.
`IsValueType` is true even when every alternative is a reference.

Native projects select structural capabilities through their Runtime Contract.
Supported paths include scoped nominal generic constituents, value wrappers,
locals, parameters/results, direct ref/out slots and bounded callbacks. Bare `T`
alternatives, directly captured union locals, whole-union boxing identity and
general AOT/JIT layout remain incomplete. The interpreter's boxed value-payload storage is
provisional and is not an efficiency claim or the final ABI.

The runtime branch is `codex/structural-union-intersection`, paired with Raven's
`codex/neoclr-union-intersection`. See the
[tested consumers and evidence](https://github.com/marinasundstrom/neoCLR/tree/codex/structural-union-intersection/docs/experiments/value-unions).

## Native reference-alternative storage

The structural branch now admits native unions whose alternatives are nominal
reference classes or interfaces. macOS ARM64 AOT checks cover membership in either
alternative, explicit extraction, copies, calls/results, null payloads and managed
output slots, with matching interpreter diagnostics for rejected alternatives.

The union remains a structural **value type**. Its current reference-only native
representation is one payload reference slot; the concrete object's existing tag
supports membership checks. Copying the union copies that slot and preserves the
payload object's identity, without allocating a union carrier. This storage choice
does not give the union nominal inheritance or make it a reference type.

Native unions of Int32, Int64 and Boolean have bounded tagged inline storage:
an adjacent scalar box/union-cast pair avoids allocating the box, and exact
extraction checks the discriminator. Immediate scalar type-test/null-test pairs
also compare the tag without boxing. A Raven-produced guarded match consumer
and a separate artifact-only library consumer with union returns/parameters both
return 42 in interpreter and macOS ARM64 AOT with zero arena use; general escaping
test results remain gated. Mixed Int32/Int64/Boolean and nominal reference alternatives now use three private
slots (tag, numeric bits, reference), tracing only the reference slot. AOT checks
cover copies, calls, replacement, null and failed casts with only explicit object
allocations. A fresh Raven `int | long | bool | Response` match consumer returns
42 in interpreter and AOT, using 16 arena bytes only for its explicit Response.
A separate artifact-only library consumer also passes mixed union returns and
parameters with reordered alternatives and the same explicit-object allocation.
An imported Response<int> alternative also preserves its generic field value
through the union return, parameter and typed pattern.
Closed nominal value records, optionally combined with Int32/Int64/Boolean, now
use a tag and disjoint inline payload regions, with inactive regions zeroed.
Tests cover exact nominal identity, field snapshots and embedded reference
identity with no payload-box allocation. A fresh artifact-only Raven struct
consumer also preserves Int32/Int64 fields through seven union-call/pattern checks,
returning 42 in interpreter/AOT with no native imports or arena allocation. The total layout is limited to 256 lanes;
its size is the sum of alternatives, so overlapping-storage optimization remains
future work. Direct nominal reference alternatives can accompany records; their
shared reference region uses actual nominal conformance for tests and casts. A
[fresh Raven consumer](../docs/experiments/native-structural-types/record-reference-import-project/README.md)
passes eight imported scalar/record/reference inputs with only its explicit
HttpError allocation (16 bytes). The
[closed-generic variant](../docs/experiments/native-structural-types/generic-record-reference-import-project/README.md)
also distinguishes Response<int> and Response<long> through nine imported inputs
and preserves substituted fields with the same allocation bound. A
[reference-valued substitution consumer](../docs/experiments/native-structural-types/reference-generic-record-import-project/README.md)
also preserves shared mutation through copied Response<Cell> records; ten inputs
allocate only the two explicit objects (32 bytes). This is not collector stress. Separate
[native collector tests](../docs/experiments/native-structural-types/README.md#native-collector-stress-for-union-roots-2026-10-15)
now cover record/reference/scalar payloads across calls and fault unwind with
eager/paced nonmoving AOT collection and complete post-entry reclamation. The
[fresh Raven consumer](../docs/experiments/native-structural-types/reference-generic-record-import-project/README.md#native-collection-run)
also passes 32 invocations per collector mode, reclaiming all explicit objects. Enums,
Object/scalar-interface views requiring boxing remain unsupported by this native lowering.
Intrinsic String alternatives now use a separate traced text slot in native union storage. Whole-union boxing is also rejected pending structural boxed identity;
it must not silently become a cast of the payload to Object. Native union descriptor
reflection, array elements, broader JIT/collector qualification and full Task application
qualification remain open. These are implementation gaps, not restrictions on the
union type's intended value/reference alternatives.

The fresh v4 Runtime/Data/Networking/Web bundle also passes this consumer in AOT
and managed JIT (32 invocations per eager/paced collector policy in each backend).
Each policy reclaims all 64 explicit allocations and leaves no roots or heap objects;
the synchronous JIT host completes cleanup before generated code is released. AOT
hosts run with undefined/bounds sanitizers; the JIT hosts are not sanitizer-instrumented.
This qualifies bounded synchronous callable-union ownership, not escaped callbacks
or async retention. [Reproduction and evidence](../docs/experiments/native-structural-types/callable-import-project/README.md).

Development native checked reservation now supports admitted structural union array
elements as inline snapshots, with indexed copy/replacement and initialization checks.
Default array allocation, borrowed element access and nominal array views remain
unqualified. Collection currently uses the existing conservative record-array scan;
see [storage details](../docs/experiments/native-structural-types/README.md).

### Fresh Task union native qualification (2026-10-15)

Both `Task<Response | HttpError | JsonError>` and
`Task<Response<int> | HttpError | JsonError>` now pass fresh Raven compilation,
interpreter execution, macOS ARM64 AOT and managed JIT against the rebuilt v4
class libraries. Each entry checks ten completed, queued and async-forwarded
alternative/identity paths and returns 42. For each consumer, eager and paced
collector policies each pass eight AOT entries with undefined/bounds sanitizers
and one managed JIT entry. The 64 KiB host checks task/socket scopes, roots and
heap cleanup before generated code release.

This closes the bounded Task delivery gate after structural generic arguments,
reserved arrays, record fields and managed record methods were admitted. It does
not qualify escaped callbacks, unrestricted open generics, other architectures,
exact allocation counts or union throughput. The JIT host is not sanitizer-built.
Runtime Contract configuration and native signature encoding are unchanged; no
CLI carrier replacement is introduced. Runtime/storage owns the newly admitted
operations; Raven continues emitting native structural signatures.

Evidence and reproducer: [native structural experiments](../docs/experiments/native-structural-types/README.md).

### Exact native union widening (2026-10-15)

Native unions now widen when every source alternative occurs exactly in the target.
Raven emits an explicit structural conversion instead of treating union widening as
an implicit no-op. AOT/JIT preserve compatible layouts directly and otherwise
repack the tag and active payload into the destination's private layout. Numeric
bits remain separate from traced reference slots, inactive banks stay zero, and
inactive/default values remain inactive. No union carrier or box is allocated.
This does not add payload-transforming conversions or general checked narrowing;
unsupported alternative sets remain rejected before object emission.

Validation covers scalar-to-scalar/mixed/record-union layouts, reordered record
payloads and inactive values. Callable, mixed-reference and reference-only sources
also widen across allocating calls under eager/paced collection. The collector
fixtures reclaim exactly 656 allocations for callable cases and 648 for object
cases per policy (eight entries), entirely from the existing explicit objects and
pressure allocations. They add no allocation for widening.

A fresh Raven consumer checks exact-over-wider overload selection, scalar-to-record
union widening and record union widening/reordering. Interpreter, macOS ARM64 AOT
and managed JIT each return 42 with zero arena use. AOT has no native imports and
passes undefined/bounds sanitizers. JIT uses the required GC/stack host and checks
roots/heap cleanup before returning; its host is not sanitizer-built. No Runtime
Contract or metadata encoding change is introduced.

Native Single/Double union alternatives now use distinct inline numeric regions,
with exact tests/extraction and allocation-free widening. Interpreter/AOT
qualification covers calls, fractional values and wrong extraction; other
primitive/enum alternatives and broader floating workloads remain open.

A fresh Raven float/double/int pattern consumer now also passes interpreter,
macOS ARM64 AOT and managed JIT with zero arena use. The bounded fixture checks
reordered/widened calls and exact alternative identity, not every floating edge
case or platform. See the [floating qualification](../docs/experiments/native-structural-types/floating-v4-results.json).
