# Boxed enum equality and hashing

## Native adapter — 2026-10-20

.NET's [Enum.Equals](https://learn.microsoft.com/en-us/dotnet/api/system.enum.equals?view=net-10.0)
requires the same enum type and underlying value; aliases compare equal, and a
boxed integer is not an enum. [Enum.GetHashCode](https://learn.microsoft.com/en-us/dotnet/api/system.enum.gethashcode?view=net-10.0)
provides the corresponding value hash. neoCLR retains these semantics for its
existing Int32-backed enums. Other underlying types remain a platform gap.

The interpreter already implements this contract. Native dispatch uses the existing verified enum metadata and box inventory,
without adding an enum-specific report or storage representation. It checks exact type tags
before loading and comparing the Int32 payload. Hashing reuses the underlying
integer hash. This uses the existing copied box layout, allocation and GC rooting;
it adds no extra payload wrapper, interface or ABI. There is no allocation or
throughput improvement claim. Ordinary and closed generic struct defaults can
now delegate enum fields through these Object slots.

No Raven compiler, Runtime Contract or temporary CLI encoding changes are needed.
The native runtime/backend owns this repair; compiler-projected Enum ancestry is
unchanged. `EquatableTo` remains an explicit contract. Hash values are not a
persistent serialization contract.

At this checkpoint native enum formatting remained unsupported. The subsequent
[formatting qualification](../enum-display/README.md) removes the rejection below. A selected load set combining enum
boxes with virtual Object.ToString is rejected, including calls on unrelated types,
until value-aware formatting is implemented. This conservative admission rule
prevents the existing default display path from silently returning an enum type
name. This is a limitation, not a new enum formatting contract.

## Validation

`enum_boxes_preserve_nominal_equality_and_hashing` covers zero, positive and negative
values, unnamed values and both Int32 endpoints; wrong enum/integer/null operands;
and rejection of unsupported formatting. The minimal native fixture uses only
direct enum boxes. `Main.rvn` uses the normal GC/stack-budget profile and covers
aliases, copied boxes, exact type tests/unboxing, ordinary/generic fields, dictionary
keys and retention across explicit GC. `check.py` runs the same source artifact in
the interpreter, JIT and AOT and records executable/compiler/library hashes.

Qualification results are recorded alongside this consumer. Public signatures are
unchanged, so the Object manual reference is updated without regenerating the
reference assembly.

The final Raven consumer passes interpreter, macOS ARM64 JIT and AOT; see
[validation-20261020.json](validation-20261020.json) for commands and artifact hashes.
The matched development compiler and class library are unchanged from the preceding
floating-object qualification. This is not replacement-release qualification.

```sh
python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/enum-object/EnumObject.rvnproj \
  --bundle target/enum-object-final-bundle/bundle \
  --aot target/enum-object-final-bundle/bundle/bin/neoclr-codegen \
  --output target/enum-object-final-app
python3 docs/experiments/enum-object/check.py \
  target/enum-object-final-bundle/bundle target/enum-object-final-app \
  --output docs/experiments/enum-object/validation-20261020.json
```

The focused enum, floating-box and ordinary-struct native regressions all pass;
see [native-tests-20261020.txt](native-tests-20261020.txt). Run these tests with the
Xcode 26.2 SDK on this host; the default macOS 27 beta SDK is incompatible with its
linker. Initial executable-loader stalls occurred before the Rust harness started;
the recorded results are completed reruns, not inferred passes.
