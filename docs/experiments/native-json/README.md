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

## Representative document checks

`Main.rvn` now runs two nested bulletin documents through both `JsonValue` and
`Bulletin`/`Report` objects. Each path deserializes, serializes and deserializes again.
Object checks assert all fields of both nested reports; DOM checks compare a known
canonical document after each round trip, so a stable but incorrect transformation
cannot satisfy the test. One input includes surrounding whitespace.

| Case | Coverage |
| --- | --- |
| Weather bulletin | Nested objects, UTF-8 Café, positive/zero integers, true/false |
| Second bulletin | Unicode 雪, Int32.MinValue, nested object field preservation |
| Mixed DOM document | Integer arrays, true/false/null array, empty array/object, nested decimal token |
| Rejected documents | Trailing array comma, wrong nested property type, earlier flat validation cases |

The mixed DOM document is deliberately not mapped to a typed object: nullable and
decimal object mapping are outside the current mapper contract. Typed Int32/Boolean/String/Report arrays now have value checks, including an empty
Report array, plus integer/object array serialization. Lists/sequences, string-keyed
maps and native jagged arrays still require their own acceptance cases.
Exit codes 20–25 identify document corpus failures; success keeps the original
single-line output. The Windows action uses this same source.
