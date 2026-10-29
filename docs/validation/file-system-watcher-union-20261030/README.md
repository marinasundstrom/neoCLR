# Watcher union callbacks — 2026-10-30

Author-directed replacement of the separate callbacks/EventArgs hierarchy with
Subscribe and a standard Raven FileSystemNotification union. Direct callbacks are
now the notification convention; library event helpers remain future work.

## Focused evidence

- Rebuilt System.Runtime with the frozen compiler recorded by hash in results.json.
  The isolated source copy excludes the concurrent Descriptors/FunctionReflection
  edits, using their HEAD versions. No OS provider or runtime binding changes.
- Native macOS arm64 consumer: setup/defaults, callback removal, recursive filtering,
  creation, atomic replacement, stop/restart/disposal and union payload matching pass.
  Native invocation ownership/teardown harness passes.
- Interpreter: the same consumer passes with the current local executable, frozen
  by hash for the run. The first attempt used the older bundled interpreter from
  before the previous slice's queue fix and reproduced its known dropped-change
  failure. The final receipt uses the corrected executable; no interpreter source
  changes are part of this slice. The local executable may include concurrent work;
  its binary identity is recorded rather than claimed as a pristine source build.
- The migrated configuration sample prints before/after values, preserves settings
  after malformed JSON, handles RescanRequired and exits successfully.
- Negative compilation rejects calling the internal FileSystemChange constructor
  (RAV0500), and rejects Renamed with an error payload instead of FileSystemRename
  (RAV1501). These preserve constructor visibility and case payload typing.
- Aggregate reference regenerated through ordinary Raven source-union projection;
  native API render and declaration links checked, including Subscribe, the union
  and rename paths. Staged-source fingerprint check excludes unrelated concurrent
  source edits and untracked Signals work. No broad website build was needed.

Windows remains an execution qualification gate. The provider/transport are unchanged;
reuse the prior slice's OS queue and ownership unit evidence rather than rerunning it.
Native codegen reaches all five notification cases; actual macOS ambiguous renames
arrive as RescanRequired, so this receipt does not claim OS-paired rename delivery.

Reproduce with a matching runtime/compiler/class-library bundle:

```sh
python3 scripts/validate-filewatch.py --bundle /path/to/bundle \
  --runtime /path/to/current/neoclr --aot /path/to/neoclr-codegen \
  --output target/watch-union-check
python3 scripts/build-api-docs.py --check
```

The negative inputs were `FileSystemChange("/root/", "../escape")` and
`FileSystemNotification.Renamed(FileSystemWatcherError.IoFailure)` in separate
library functions importing System.IO.*. Diagnostics are retained in results.json.
