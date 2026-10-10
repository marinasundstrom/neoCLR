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
from the pinned development compiler and runs the same projects. The earlier
nested-object/vector/node and collection slices now pass Windows; the
[verified JSON/collection regression](../native-collections/windows-json-map-validation.json)
records the matching compiler run. macOS native and interpreter collection round trips pass. This development probe is not a published capability or benchmark.

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
Report array, plus integer/object array serialization. Native jagged arrays still require their own acceptance cases; collection cases
are described below.
Exit codes 20–25 identify document corpus failures; success keeps the original
single-line output. The Windows action uses this same source.

## Explicit node properties (development)

NodeEnvelope combines JsonValue, JsonObject and JsonValue[] with normal model
construction. Nodes embed their JSON content directly; the checks cover Unicode,
exact number-token spelling and round trips. Root Deserialize<JsonValue> accepts
every kind, while concrete node declarations reject mismatches before model code.
JsonNull is explicit data; Object properties do not infer nodes. Embedded depth
overflow and cycles still fail whole-document limits. Exit code 26 identifies this
corpus; it passes macOS interpreter/native execution with the rebuilt libraries.


## Built-in collections (development)

Exit code 27 identifies the collection corpus. It exercises List<int> mutation,
serialization-only ArrayList<bool>/HashMap<string, bool> construction,
a Sequence<Report> model property, Map<string, JsonValue> dynamic content,
empty ArrayList/HashMap, MutableMap and nested Map<string, Sequence<int>>.
Repeated round trips preserve values, array order and distinct ordinal keys.
Unicode and a key longer than 31 bytes exercise content hashing without a
temporary fixed-size byte vector. A bad later list/map element must fail before
a faulting earlier model constructor runs. Non-string keys are rejected.

Readers construct ArrayList and HashMap(StringComparer.Ordinal); arbitrary
collection implementations, custom comparer preservation and inferred Object
nodes are excluded. Existing container/depth/document limits apply recursively.
The project driver links matching native Unicode helpers only when selected.
Run a focused consumer with `validate-native-json.py --case json`; the default
continues to run both reflection and JSON projects. The validator builds the current
interpreter for matching private service support.

The public DOM/document portion also has focused tests in the
[Raven JSON DOM suite](../../../runtime/raven/tests/json-dom/JsonDom.rvn).
Typed object mapping and reflection retention checks remain in this acceptance
consumer. The framework migration does not imply those checks have moved yet.
