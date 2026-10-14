# Union

Raven supports named nominal unions and ad-hoc unions written as type expressions.
A native structural union representation remains a separate design direction.

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
arity bound belongs to the bridge, not the intended platform semantics. Native
structural union metadata and value operations will replace the carrier encoding;
consumers must use matching compiler/library artifacts and rebuild when that
representation changes. No structural union type is being advertised as shipped.

[All structural families](structural-types.md)


## Branch-only reference experiment

The unpublished `codex/structural-union-intersection` branch admits neoIL
`union<A,B>` and `intersection<A,B>` behind the `experimental-compositions` build
feature. It supports reference locals, free-function parameters/results, checked
casts and guaranteed interface dispatch in the interpreter. This is not Raven
source support or a replacement for published union carriers. There are no public
compound reflection constructors or members to enumerate yet.

Fields, generics, byrefs, arrays, boxed/intrinsic reference values and JIT/AOT remain
unsupported. Typed nulls follow ordinary reference behavior; non-null membership
preserves the same class allocation. These are experiment limits, not permanent
platform contracts.
