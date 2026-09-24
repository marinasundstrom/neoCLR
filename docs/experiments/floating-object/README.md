# Floating Object keys

Development sample: boxed Single and Double use exact-type value equality. NaNs of
the same type compare equal; signed zeros compare equal and have matching hashes.
Floating `==` still follows IEEE comparisons. The sample uses explicit Object
callbacks in HashMap to find NaN keys and reject duplicate zero keys, while preserving
Single/Double distinctions, copied values and separate box identity through GC.

```sh
cargo build --example measure_async
python3 docs/experiments/floating-object/verify.py \
  --toolchain-root /path/to/matching-development-bundle \
  --runner target/debug/examples/measure_async
cargo test --lib floating_object_nan_payloads
cargo test --test object_equality floating_object
dotnet run --project docs/experiments/floating-object/dotnet/Baseline.csproj
```

Validation on 2026-09-24 reclaimed all 426 allocations across ten collections,
with peak 64 and zero live objects. The runner requires multiple collections and zero live objects after completion.
Native tests cover varied NaN bit patterns, subnormal hashes, infinities and exact
Object dispatch. The .NET 10 baseline checks the comparison rules independently.

Floating boxed formatting, typed Equals/GetHashCode members and generic math are
not added. This is a runtime Object-slot intrinsic, with no compiler or managed
layout change. Hash codes are not persistent identifiers. See the
[Object review](../../object-model-review.md) for comparisons and tradeoffs.
