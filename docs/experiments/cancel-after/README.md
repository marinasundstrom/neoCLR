# CancelAfter behavior fixture

Four named NeoClr.Testing cases cover resetting the countdown and deferred zero,
disabling/reenabling it, disposal suppressing pending cancellation, and manual
cancellation removing the countdown without duplicate notification. Tests.rvn keeps
the original ordering, durations, timer-backed waits, registrations and disposal.
Callbacks deliberately signal promises or count notifications; ordinary sequencing
uses await.

Main explicitly awaits the cases and registers completed-result reference adapters
with TestSuite. ConsoleReporter prints outcomes and Main returns report.ExitCode.
Recoverable assertion failures are named and permit remaining independent cases
to run. Automatic async discovery remains unsupported.

The separate Faults project retains terminal-failure subprocess probes for negative
and excessive durations, negative duration after cancellation, and scheduling or
disabling after disposal. Python keeps native building, process limits and fault
checks; the behavior run requires an exact four-pass framework transcript and empty
stderr. Interpreter mode is selected explicitly because the runtime defaults to JIT.

```sh
python3 scripts/validate-timers.py --suite cancel-after \
  --bundle <matching-bundle> --runtime <matching-bundle>/bin/neoclr \
  --aot <matching-bundle>/bin/neoclr-codegen --output target/cancel-after-tests
```

[Migration evidence](../../validation/testing-migration-20261101/README.md#cancelafter)
records four behavior cases and five fault probes in both interpreter and AOT,
plus an isolated wrong-notification-count control. JIT coverage is not added.
The existing [cancellation API comparison](../../http-client-design.md#shared-native-deadline-checkpoint--2026-09-24)
and [timer/lifetime design](../../design/http-client-lifetime.md) remain applicable;
this changes no production cancellation or timer contract.
