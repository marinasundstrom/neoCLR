# Reference and value contract families — 2026-10-20

The [consumer](Main.rvn) checks ArrayList/array/StringBuilder owner identity,
stable hashes across mutation and explicit GC, immutable builder snapshots,
CancellationToken source identity through shared struct defaults, and Uri/Path
value equality, hashes and boxed display. It passes interpreter, JIT and AOT:
[execution evidence](validation-20261020.json).

The [baseline native build](baseline-20261020.json) failed to link timer cancellation.
Synchronous CancellationTokenSource.Cancel/Dispose can retain Timer calls even
without an async method. The project builder now selects the existing scheduler
host when timer bindings are selected. It only drains a default task queue when
the compiler actually emitted that drain function. This reuses invocation ownership
and cleanup; it does not manufacture a Task or Promise for synchronous operations.
[Final build inputs](native-build-20261020.json) identify the exact artifacts.

This consumer is representative evidence for the [public type review](../../design/class-library-soundness-20261020.md#whole-library-contract-pass),
not an exhaustive test of every method or timing behavior. There is no throughput
or allocation-count claim. Identity hashes need only remain stable for the same live
object; different objects may collide.

```sh
python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/type-contracts/TypeContracts.rvnproj \
  --bundle target/value-display-bundle/bundle \
  --aot target/value-display-bundle/bundle/bin/neoclr-codegen \
  --output target/type-contracts-final-app
python3 docs/experiments/type-contracts/check.py \
  target/value-display-bundle/bundle target/type-contracts-final-app \
  --output docs/experiments/type-contracts/validation-20261020.json
```
