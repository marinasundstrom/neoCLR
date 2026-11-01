# Filesystem-watcher behavior tests

Three named NeoClr.Testing cases cover defaults/disposal, typed setup errors and
physical notifications through create, atomic replacement, stop, restart and disposal.
The physical case retains subscription-removal, descendant filtering and callback
payload checks, with the existing two-second settling intervals and acceptance of
rescan notification for atomic replacement. Assertions and host phase order are
unchanged by the reporting migration.

Main runs the cases in their original order, explicitly awaits the physical
lifecycle, registers completed-result reference adapters with TestSuite, writes
ConsoleReporter output and returns report.ExitCode. Recoverable failures are named
and allow remaining independent cases to run; terminal faults remain terminal.
Automatic async/parameterized discovery is not added.

Python retains real filesystem writes triggered by READY markers, OBS diagnostics,
isolated directories and the native invocation-ownership/teardown harness. The
framework transcript is checked separately from those host-protocol lines. Guest
stderr must be empty; only the exact existing DAM2004 JIT compiler warning is
permitted and recorded separately. JIT uses explicit matching modules/helper/codegen.

```sh
python3 scripts/validate-filewatch.py --bundle <matching-bundle> \
  --runtime <matching-bundle>/bin/neoclr --aot <matching-bundle>/bin/neoclr-codegen \
  --output target/filewatch-tests
```

[Migration evidence](../../../../docs/validation/testing-migration-20261101/README.md#filesystem-watcher)
records all three cases in interpreter/JIT/AOT, the native ownership harness and a
wrong-defaults control that still completes the physical lifecycle. Existing
[watcher design](../../../../docs/file-system-watcher.md) and provider limitations
remain applicable. This changes no production watcher contract or OS support claim.
