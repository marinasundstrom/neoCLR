# Deferred async state storage

Development regression, 2026-09-25. A class-based async method saves an application
Result containing a standard error union, suspends on a pending Promise, and reads
the saved error after managed garbage collection. The importer must mark only
application reference types implementing the core IAsyncStateMachine contract with
`.field deferred`; ordinary classes retain constructor initialization checks.

```sh
python3 docs/experiments/deferred-async/verify.py \
  --toolchain-root /absolute/path/to/matching/development/toolchain \
  --runner target/release/examples/measure_async
```

The fixture uses TaskQueue only to drive the current transitional builder contract.
It does not propose that queue as the future public scheduler. See
[the storage decision](../../value-storage.md#deferred-class-fields--development).
The HTTP JSON sample exercises the same fix with actual socket suspension.

Local validation: 12 focused Rust constructor/storage tests pass. The pending-await
consumer prints `Retained error`, with 357 allocations, eight collections and zero
final live objects. This covers the current heap-state mode; it does not claim a
new value-state initialization contract or cross-platform release validation.
