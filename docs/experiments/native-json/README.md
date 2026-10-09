# Native JSON round-trip probe — development

This is the acceptance consumer for the [native JSON direction](../../native-json-plan.md).
It uses the existing public serializer, properties and a parameterless constructor.
The interpreter succeeds with `{"Name":"Café","Count":3,"Active":true}`.

Use a matching development bundle:

```sh
python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/native-json/Native.rvnproj \
  --reflection-roots docs/experiments/native-json/reflection-roots.json \
  --bundle /absolute/path/to/development/bundle \
  --aot tools/aot-poc/target/debug/neoclr-aot-poc \
  --output target/native-json-admission
```

The flat typed round trip now passes on macOS in both execution modes. The probe
also covers invalid syntax, missing/null/wrongly typed values, no constructor execution
for invalid input, escaped UTF-8 text, false and Int32.MinValue. Source identities in
the private roots file must match the compiler artifact. Use the console profile;
the HTTP profile expects task-pump exports.

Run `scripts/validate-native-json.py --output target/my-json-validation --bundle
/absolute/path/to/development/bundle` for standalone native/interpreter parity and
the public introspection consumer. The Windows workflow rebuilds current libraries
from the pinned development compiler and runs the same projects. Windows
qualification and the expanded nested/array/list/sequence/string-keyed-map milestone
remain pending. This development probe is not a published capability or benchmark.
