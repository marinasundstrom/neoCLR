# Boxed Int64 Object contract

Development sample: copied Int64 values retain full-width equality through Object,
reject other concrete types and use a .NET-compatible XOR-of-halves hash. Separate
boxes keep distinct identities. Two unequal Int64 keys deliberately share a hash;
HashMap uses explicit Object equality/hash callbacks to distinguish them. Retained
keys are checked after allocation churn and garbage collection.

Run with a matching development bundle and rebuilt native runner:

```sh
cargo build --example measure_async
python3 docs/experiments/int64-object/verify.py \
  --toolchain-root /path/to/development-bundle \
  --runner target/debug/examples/measure_async
cargo test --test object_equality boxed_
dotnet run --project docs/experiments/int64-object/dotnet/Baseline.csproj
```

Validation on 2026-09-24: the Raven sample reclaimed all 830 allocations across
20 collections (peak 64), with zero live objects on exit. The verifier checks exact
output, multiple collections and full reclamation.
The .NET 10 baseline checks the same equality/hash rules at the numeric limits and
for deliberate collisions. Native tests additionally verify copied unboxing after GC.

This is an interpreter Object-slot intrinsic, not a new source-declared Int64 member.
Object ToString now also checks copied Int64 text after GC, decimal Int32 output
and Boolean True/False output. Integer formatting is culture-independent; no format
strings or providers are supported. Other primitive types and nullable value types
remain outside this slice.
Hashes must not be stored as persistent identifiers. See the
[Object model review](../../object-model-review.md) for the implementation comparison.
