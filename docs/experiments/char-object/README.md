# Char Object contracts

The development sample uses a joined emoji as a Char and an Object-keyed map key.
It checks copied text, typed/Object equality, separate box identity, full display,
no implicit normalization of composed/decomposed text, and retained keys through GC.

```sh
cargo build --example measure_async
python3 docs/experiments/char-object/verify.py \
  --toolchain-root /path/to/matching-development-bundle \
  --runner target/debug/examples/measure_async
cargo test --test object_equality boxed_char_
dotnet run --project docs/experiments/char-object/dotnet/Baseline.csproj
```

Validation on 2026-09-24 reclaimed all 422 allocations across nine collections
(peak 64, zero live objects). The verifier checks exact output, multiple collections and zero live objects.
The .NET 10 comparison demonstrates Char Object behavior for a code unit and uses
StringInfo to contrast that with grapheme text. neoCLR keeps its existing grapheme
model: equal text must have equal hashes, but hashes are not compatible with .NET
Char and are not persistent identifiers. No normalization or collation is added.

See the [Object review](../../object-model-review.md) for the implementation tradeoffs.
