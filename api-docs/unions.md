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
shared reference region uses actual nominal conformance for tests and casts. Enums,
Object/scalar-interface views requiring boxing and intrinsic String alternatives
remain unsupported by this native lowering. Whole-union boxing is also rejected pending structural boxed identity;
it must not silently become a cast of the payload to Object. Native union descriptor
reflection, array elements, dedicated JIT/collector stress and full Task application
qualification remain open. These are implementation gaps, not restrictions on the
union type's intended value/reference alternatives.
