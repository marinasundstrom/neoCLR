# Map indexer terminal-fault probe

`Missing.rvn` reads an absent key through HashMap (no arguments), MutableMap
(`mutable`) or Map (`map`). Each must terminate with `HashMap key not found` in
native and interpreted execution. These cases run separately because a terminal
Fault cannot be caught by the Raven test runner.

Positive access, comparer equality, insert/replace, preserved keys, collisions and
growth are tested in `runtime/raven/tests/collection-construction/MapIndexers.rvn`.
Use the matching rebuilt library bundle; old Map metadata has no indexer contract.

The checked [validation](validation.json) records commands, native/interpreter
faults and artifact/source hashes for all six executions. Build the probe with:

```sh
python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/map-indexers/Missing.rvnproj \
  --bundle /path/to/matching/bundle --output target/map-indexer-missing
```

Run `app`, `app mutable` and `app map`; each exits unsuccessfully with the documented
fault. The positive framework gate is `--suite collection-construction`.
