# Introspection Object contracts

Development, 2026-09-24. Build a matching development bundle and run:

```sh
cargo build --example measure_async
python3 docs/experiments/introspection-object/verify.py \
  --toolchain-root /path/to/bundle --runner target/debug/examples/measure_async
```

The fixture checks repeated TypeInfo queries with separate allocation identities,
typed/EquatableTo/Object equality, MemberInfo views, null/wrong-type rejection at the
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

The fixture also checks assembly and module wrappers through Object equality/hash/display,
rejects null and other descriptor kinds, and retains descriptor-keyed HashMap entries
through collection. Assembly identity is its full loaded catalog identity; module
identity adds the module name. A Rust catalog regression deliberately reuses short
assembly names, module names and tokens across distinct scopes. Member and parameter
snapshots still require ownership investigation; this fixture makes no claim of value
equality for them. TypeInfo, MemberInfo, AssemblyInfo and ModuleInfo have generated
API reference; remaining gaps are tracked in api-docs/README.md.

The original TypeInfo-only run allocated/reclaimed 422 managed objects with peak 64,
ten collections and zero retained objects (heap limit 96). The expanded fixture
continues to require multiple collections and full reclamation. These totals include
fixture churn, maps and hashing temporaries; they are correctness evidence, not an
allocation or timing comparison with .NET.

Expanded run: 476 allocated/reclaimed objects, peak 96, ten collections and zero
retained objects at completion under the same 96-object limit. The scoped catalog
regression also passes with matching short names and module names.
