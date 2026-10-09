# Native JSON round-trip probe — not yet admitted

This is the acceptance consumer for the [native JSON direction](../../native-json-plan.md).
It uses the existing public serializer, properties and a parameterless constructor.
The interpreter succeeds with `{"Name":"Café","Count":3,"Active":true}`.

Use a matching development bundle:

```sh
python3 scripts/build-native-project.py --profile http \
  --project docs/experiments/native-json/Native.rvnproj \
  --bundle /absolute/path/to/development/bundle \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output target/native-json-admission
```

Native compilation currently fails admission and must not publish an executable.
The first failure was RuntimeTypeHandle specialization; after the type-token
foundation it advances to unsupported boxing. Type/property discovery and checked
reflection invocation are still required. This probe is not a supported native
website example or a claim that JSON serialization already works in AOT.
