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
Rust runtime tests cover varied NaN bit patterns, subnormal hashes, infinities and exact
Object dispatch. The .NET 10 baseline checks the comparison rules independently.

Floating boxed formatting, typed Equals/GetHashCode members and generic math are
not added. This is a runtime Object-slot intrinsic, with no compiler or managed
layout change. Hash codes are not persistent identifiers. See the
[Object review](../../object-model-review.md) for comparisons and tradeoffs.


## Native backend qualification — 2026-10-20

`NativeFloatingObject.rvnproj` and `NativeMain.rvn` use the current source-owned
class library. The legacy project and measurement script above retain their original
bundle requirements. `check-native.py` runs the new consumer through interpreter,
macOS ARM64 JIT and AOT; all three pass in `native-validation-20261020.json`.
The compiler remains Raven `8da3a45f8` on `codex/neoclr` (local bundle label
`4c051e332-dirty-record-policy`); library signatures and runtime ABI are unchanged.

Native boxing already retained Single/Double storage and exact-type queries. This
change admits their Object Equals/GetHashCode slots and generates floating comparisons
in the existing dispatch body. It preserves the interpreter/.NET value contract:
NaNs of the same type compare equal, signed zero compares equal, Single never equals
Double, and null/other types compare false. Floating operators retain IEEE behavior.
Hashing normalizes NaNs and signed zeros, then folds Double's bits to Int32. Hashes
match the current interpreter; they are not persistent or unique identifiers.
See [.NET Double.Equals](https://learn.microsoft.com/dotnet/api/system.double.equals)
and the existing .NET baseline for the semantic comparison.

The executable backend test checks multiple positive/negative and signaling NaN bit
patterns, infinities, subnormal and finite hashes, nulls, another numeric type,
separate box identities and the unchanged NaN equality operator. The source consumer
also checks exact unboxing/type tests, default floating fields in ordinary and nested
generic structs, HashMap key identity and retained values after explicit GC.

The existing box allocation and GC layout are reused; this adds no new storage kind
or typed equality interface. Converting a value to Object and default struct field
boxing still allocate. This correctness qualification makes no throughput or
allocation-reduction claim. Boxed floating ToString remains unsupported: the native
admission check still rejects a selected load set containing floating boxes together
with virtual Object.ToString. The subsequent [enum adapter qualification](../enum-object/README.md) extends
equality/hash to Int32-backed enums. Enum and broader primitive formatting remain
gaps, not deliberate semantic differences.

Reproduce with a matched development bundle:

```sh
python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/floating-object/NativeFloatingObject.rvnproj \
  --bundle target/floating-box-bundle/bundle \
  --aot target/floating-box-bundle/bundle/bin/neoclr-codegen \
  --output target/floating-box-app
python3 docs/experiments/floating-object/check-native.py \
  target/floating-box-bundle/bundle target/floating-box-app \
  --output docs/experiments/floating-object/native-validation-20261020.json
```
