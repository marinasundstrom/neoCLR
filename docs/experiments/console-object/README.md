# Console object fallback and scalar output

This development sample exercises virtual ToString through both a concrete class
and object, boxed integer/string values, null, existing string/Int32 overloads,
Boolean, grapheme Char, signed/unsigned integer boundaries and native-sized zeros.
The frozen Raven compiler currently lacks integer-to-native-integer source casts;
this fixture uses default native values, not invented conversion syntax.

```sh
python3 docs/experiments/console-object/verify.py \
  --toolchain-root /path/to/matching/bundle \
  --runner target/release/examples/measure_async
```

The verifier checks output, direct typed calls in imported application IL,
absence of boxing in scalar Console implementations and zero final live heap
objects. Use a matching rebuilt CLI as well as the runner: the new private
Int64/UInt64 formatting bindings must be understood during SDK verification.
See [Console contract and .NET comparison](../../console-io.md).

Floating-point numeric formatting is intentionally not claimed; the object
fallback uses whatever ToString the type currently provides.
