# Timer behavior fixture

Six named NeoClr.Testing tests preserve the original ten reported checkpoints.
The five synchronous phases remain one combined test so the one-shot timer's
start/stop/dispose history and the repeat/reschedule sequence stay intact. Five
separate tests cover async repeat serialization, restart without overlap, disposal
while a callback is active, already-cancelled callbacks and pending cancellation.

Tests.rvn retains timer durations, promise barriers, pauses and explicit disposal.
Callback helpers record sticky validity flags for excess invocations or overlap;
framework assertions check those flags after the relevant lifecycle completes.
The callbacks deliberately exercise Timer notification and async serialization.

Main explicitly awaits tests in their original order, registers bound completed
results with TestSuite, writes ConsoleReporter output and returns report.ExitCode.
Recoverable assertion failures are reported by name and allow remaining independent
cases to run. Terminal failures remain terminal. Automatic async discovery is not
added. Python requires an exact six-pass transcript and empty stderr.

Faults.rvn remains a separate subprocess fixture: negative/excessive duration,
zero/negative interval and starting a disposed timer must terminate. The existing
interpreter/AOT scope and process/instruction budgets are unchanged.

```sh
python3 scripts/validate-timers.py --suite timers \
  --bundle <matching-bundle> --runtime <matching-bundle>/bin/neoclr \
  --aot <matching-bundle>/bin/neoclr-codegen --output target/timer-tests
```

See [migration evidence](../../validation/testing-migration-20261101/README.md#timers)
and the existing [timer design and API comparison](../../design/http-client-lifetime.md).
This changes no Timer API or scheduling contract and makes no performance claim.
