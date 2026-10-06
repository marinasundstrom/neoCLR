# Callable nullable metadata (development, 2026-10-06)

The native GC bootstrap consumer exposes a shared metadata gap: a source
`KeepAlive(object?)` becomes `KeepAlive(object)` after separate compilation and
native import. The user asked whether nullability could be added another way or
should be deferred. The chosen first experiment preserves annotations separately
from physical signatures; new runtime nullability semantics remain deferred.

## Comparison and decision

.NET nullable-reference metadata uses `NullableAttribute` byte/byte[] payloads and
optional context compression. These are compiler conventions encoded in standard
CLI custom attributes, not different runtime reference layouts. Source:
[Roslyn nullable metadata specification](https://github.com/dotnet/roslyn/blob/main/docs/features/nullable-metadata.md),
retrieved 2026-10-06 (living specification, not a pinned runtime implementation).
Explicit annotations alone can express the needed facts without context compression.

Preserving that representation benefits artifact-only import and .NET interoperability.
It costs metadata bytes and requires recursive interpretation of reference positions by
the compiler. A new nullable signature category would require new format and runtime
contracts without solving an additional demonstrated need here. Treating every imported
reference as nullable, or special-casing GC, would lose declaration intent. Deferring all
preservation would leave a valid separately compiled API unusable. No performance
improvement is claimed.

## Implemented bounded slice

The host metadata model now authors and reads explicit parameter/return attributes in
CLI PE files. Builders delegate to definitions; introspection exposes the raw immutable
annotation. Uniform scalar and positional vector forms remain distinct. Validation is
bounded to 4096 flags and rejects malformed explicit attributes. The caller owns matching
the transform to a signature. This fail-closed reader is stricter than Roslyn's handling
of unrecognized payloads. Native encoding rejects authored annotations rather than
silently losing them. Public contracts and exceptions are documented in the
[host API reference](../../api-docs/experimental-metadata.md#explicit-callable-nullable-annotations-development-2026-10-06).

Focused C# tests compare emitted attributes with the host .NET `NullabilityInfoContext`
and execute the emitted identity methods. Array roots and elements, scalar/vector forms,
manual definitions, builder convenience, immutable payloads, clearing, facade access,
invalid authoring and malformed blobs are covered. All 160 C# metadata contract groups
pass, and `scripts/build-api-docs.py --check` passes. This slice changes no Raven compiler,
Runtime Contract configuration, native runtime behavior, bridge mapping or guest API.

## Next integration boundary

Native writer/reader materialization must preserve the same annotation facts and admit
them through the runtime metadata schema without changing storage. Then Raven emission
must use semantic nullable information before physical type erasure; native import must
reconstruct nullable symbols from introspection independently of the emitter. Test nested
generic positions and constructed scopes there. Context, field and property support stay
explicitly pending rather than defaulting missing data to non-null.

Completion remains the existing artifact-only GC consumer compiling and executing its
`KeepAlive(null)` call. The current CLI contract tests do not satisfy that gate and the
primitive bootstrap/retained seed stay unchanged.
