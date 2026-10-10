# JsonObject map contract probe

The attributed JSON DOM suite now tests safe map lookup, ordinal names, explicit
JsonNull versus absence, concrete/interface indexers, empty views, insertion-order
snapshots, node identity, pair deconstruction and serializer precedence.

`Missing.rvn` separately reads an absent field through JsonObject (no arguments)
or Map (`map`). Each must terminate with `JsonObject field not found` in native and
interpreted execution. A terminal Fault cannot be caught by the Raven test runner.
The [16-case framework evidence](../../../runtime/raven/tests/json-map-validation.json)
records successful native macOS ARM64 and interpreted runs.
The [fault evidence](validation.json) records all four executions and input hashes.

```sh
python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/json-map/Missing.rvnproj \
  --bundle /path/to/matching/bundle --output target/json-map-missing
python3 scripts/test-runtime-library.py --suite json-dom \
  --bundle /path/to/matching/bundle --output target/json-map-tests
```

Use matching rebuilt libraries. The class exposes a read-only Map view; Add retains
its typed duplicate/quota errors and no MutableMap setter is added here.
