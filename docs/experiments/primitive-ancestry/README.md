# Primitive and core-enum ancestry regression

## Primitive and core-enum foundations — 2026-10-18

The native source-core emitter now defines owned ValueType/Enum foundations before
intrinsic value declarations and core enums, then binds their exact local bases.
Intrinsic scalar storage is unchanged; nominal ancestry introduces no fields or
reference receiver. Runtime ancestry validation admits only the verified fieldless
ValueType/Object chain, with Enum as the additional enum link. Native primitive
boxes retain this ancestry while keeping their distinct payload tags.

This matches [.NET ValueType ancestry](https://learn.microsoft.com/en-us/dotnet/api/system.valuetype?view=net-10.0)
and the [enum inheritance and boxing contract](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/enums).
The temporary CLI projection retains its ordinary CLR bases; native metadata now
records the owned identities. No Runtime Contract configuration or ordinary .NET
compiler behavior changes. Primitive methods continue through existing runtime
dispatch, independently of ordinary class virtual-slot validation.

Freshly rebuilding all four core libraries exposed two earlier Object-base gaps:
closed reference roots can now bind the selected external Object, and imported
Object-slot references retain their virtual classification for override checking.
The native metadata layer owns both repairs; ordinary derived-base restrictions
remain enforced. Local ordinary value-record bases are separate from the new
intrinsic/core-enum authoring contracts.

Validation is retained in neoCLR's `docs/experiments/primitive-ancestry/`, including
the rebuilt-library hashes, matched execution modes and metadata ownership tests.

`Main.rvn` checks BaseType and IsSubclassOf for the twelve fixed/native-width
numeric primitives, Boolean, Char and RuntimeTypeHandle. It checks imported
StringComparison's Enum base and box identity, direct ValueType boxing, GetType,
exact unboxing, and rejection of String/Enum ancestry mismatches.
`baseline.json.gz` retains the original missing SByte base failure.
Run `verify.py --help` for the artifact/output inputs.

This ancestry slice retains existing native primitive boxing profiles; additional
primitive box shapes are validated in a subsequent slice. No installed bundle is
modified. Char continues to use neoCLR's documented grapheme representation.

The ancestry consumer passes interpreter/JIT/AOT with native GC. Metadata primitive,
local enum, closed Object-base override and existing Object-root checks pass; 20
runtime Object-root tests pass. `library-build.json.gz` records the source-built
Runtime/Data/Networking/Web cohort, and `results.json.gz` records consumer inputs.

The existing JSON record consumer also passes all three modes against this rebuilt
cohort; `json-results.json.gz` retains its commands, hashes and output.
