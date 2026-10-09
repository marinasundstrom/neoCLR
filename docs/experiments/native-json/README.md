# Native JSON round-trip probe — not yet admitted

This is the acceptance consumer for the [native JSON direction](../../native-json-plan.md).
It uses the existing public serializer, properties and a parameterless constructor.
The interpreter succeeds with `{"Name":"Café","Count":3,"Active":true}`.

Use a matching development bundle:

```sh
python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/native-json/Native.rvnproj \
  --bundle /absolute/path/to/development/bundle \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output target/native-json-admission
```

Native compilation currently fails admission and must not publish an executable.
The first failure was RuntimeTypeHandle specialization; after the type-token
foundation, boxing and object-type queries now pass admission. Explicit reflection
roots support parameterless construction; property discovery and checked getter/setter
invocation are still required. Use the console profile for this synchronous probe;
the HTTP profile expects task-pump exports. This probe is not a supported native
website example or a claim that JSON serialization already works in AOT.
