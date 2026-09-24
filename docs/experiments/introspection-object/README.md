# Introspection Object contracts

Development, 2026-09-24. Build a matching development bundle and run:

```sh
cargo build --example measure_async
python3 docs/experiments/introspection-object/verify.py \
  --toolchain-root /path/to/bundle --runner target/debug/examples/measure_async
```

The fixture checks repeated TypeInfo queries with separate allocation identities,
typed/Equatable/Object equality, MemberInfo views, null/wrong-type rejection at the
Object boundary, hash/display consistency, typeof versus boxed GetType, closed
generic arguments, array element shapes and HashMap callbacks. It retains descriptors
and map entries across repeated collection under a 96-object heap limit. The runner
verifies the imported program, checks multiple collections and requires no managed
objects to remain after the scalar entry returns.

RuntimeTypeInfo equality uses the existing loaded type-handle identity, not wrapper
identity or a display string. The current hash uses FullName via System.HashCode;
distinct definitions with the same name may collide. This is a bounded consistency
implementation, not a unique type ID, cross-build key or hash performance claim.
Typed Equals(TypeInfo) remains non-nullable. Object.Equals(Object?) explicitly handles
null; no nullable value type or automatic Option conversion is introduced.

Other introspection wrappers are investigated in the Object model review. This fixture
does not promise value equality for AssemblyInfo, ModuleInfo or member/parameter
snapshots. Their defining scope and owner must be represented before changing their
Object contracts. TypeInfo/MemberInfo API documentation is generated; remaining
introspection reference gaps are tracked in api-docs/README.md.

Recorded development run: 422 managed objects allocated/reclaimed, peak 64, ten
collections, zero retained objects at completion (heap limit 96). These totals include
fixture churn, maps and hashing temporaries; they are correctness evidence, not an
allocation or timing comparison with .NET.
