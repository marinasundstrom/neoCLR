# String through Object

The development sample converts text to Object, compares exact contents, checks
matching hashes and unchanged display, and uses text keys with explicit Object
callbacks in HashMap under GC pressure. Strings are reference types, not boxed values;
the current runtime uses temporary wrappers when adapting intrinsic text to Object.

```sh
cargo build --example measure_async
python3 docs/experiments/string-object/verify.py \
  --toolchain-root /path/to/matching-development-bundle \
  --runner target/debug/examples/measure_async
dotnet run --project docs/experiments/string-object/dotnet/Baseline.csproj
```

Validation on 2026-09-24 reclaimed all 421 allocations across nine collections
(peak 64, zero live). The verifier requires multiple collections and zero live objects. The .NET 10
baseline checks content equality, matching hashes and display, not identical hash
numbers. The expanded development sample now checks String aliases, separate equal
constructions, Object/Sequence conversions, ToString, array storage, null/mixed types
and identity after GC. String reference comparison and explicit Object base hashes
follow the shared text owner; virtual hashes remain content-based.
Cast/type-test wrapper allocations now participate in pre-allocation GC; retained
wrappers survive small-heap pressure. No interning, Unicode normalization, culture-sensitive comparison or nullable string
storage is introduced. See the [Object review](../../object-model-review.md).


Expanded identity validation on 2026-09-24 reclaimed all 446 allocations across ten
collections (peak 64, zero live). Explicit Object base hashing is exercised by native
IL tests because no new source-level identity-hash helper is introduced. Those tests
also recreate wrappers after collection and check the retained String hash. Distinct
objects are allowed to have colliding hashes; no test assumes otherwise.
