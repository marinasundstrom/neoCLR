# Declared member Object contracts

Development, 2026-09-24. Build a matching development bundle, then run:

```sh
cargo build --example measure_async
python3 docs/experiments/introspection-members/verify.py \
  --toolchain-root /path/to/bundle --runner target/debug/examples/measure_async
cargo test --test raven_reflection member_object_identity_includes_kind_closed_owner_and_definition
```

The Raven sample queries Date field, method and property declarations, compares
fresh snapshots through Object and MemberInfo views, and distinguishes allocation
identity from represented-member equality. It rejects null, unrelated objects and
other member kinds, checks hash/display consistency and compares property accessors
with direct method queries. A member-keyed HashMap retains all three kinds through
collection. The runner verifies the imported program, requires multiple collections
under a 256-object heap limit and requires full reclamation after completion.

The raw catalog test checks two different declarations for each kind, repeated queries,
different closed generic owners and separate definitions with identical member names.
The key is (kind, closed declaring type, definition index), never name or token alone.
The fixture does not claim CLR reflected-context equality or inherited enumeration:
current queries list declarations, and no ReflectedType is exposed. ToString returns
Name rather than a full signature. Hashes may collide and are not stable identifiers.
Parameter ownership/equality remains separate work.

Recorded run: 785 allocated/reclaimed objects, peak 252, ten collections and zero
retained objects under the 256-object limit. These are correctness measurements,
not an allocation/performance comparison with .NET.

After parameter owner keys were added, the same fixture passes at the unchanged
256-object limit: 815 allocated/reclaimed, peak 256, ten collections, zero retained.
