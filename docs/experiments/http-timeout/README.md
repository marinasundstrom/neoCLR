# HTTP timeout behavior fixture

Nine named NeoClr.Testing cases preserve the controlled-handler timeout lifecycle:
default timeout and acknowledgement, caller cancellation, client disposal, caller
cancellation after timeout, disabling, completion removing the countdown, independent
concurrent requests, 96 completed sends releasing timer capacity, and successful
handler completion after a cancellation request.

A separate invocation with an argument runs the tenth named test, a real 16-second
request under the default timeout. The Python validator runs that case only in AOT,
preserving the regression check against the former 15-second native host limit.
It keeps the existing 30-second process timeout.

Tests.rvn uses assertions returning TestFailure; async result-check helpers propagate
those results. Cancellation is observed through OnCompleted deliberately, so checking
a cancelled task does not cancel the test. Main explicitly awaits cases in their
original order, registers completed-result reference adapters with TestSuite, writes
ConsoleReporter output and returns report.ExitCode. Automatic async discovery is not
added. Recoverable failures are named and allow later independent cases to run;
terminal runtime faults remain terminal. Invalid fixture-URL construction remains
a terminal setup check.

Faults.rvn retains six independent invalid-operation probes. Network.rvn and its
host-controlled network validation are separate and unchanged. Python keeps builds,
process orchestration and terminal-failure checks, and now requires exact framework
transcripts and empty stderr for both ordinary and long-request behavior runs.

```sh
python3 scripts/validate-timers.py --suite http-timeout \
  --bundle <matching-bundle> --runtime <matching-bundle>/bin/neoclr \
  --aot <matching-bundle>/bin/neoclr-codegen --output target/http-timeout-tests
```

See [migration evidence](../../validation/testing-migration-20261101/README.md#http-timeout)
and the existing [HTTP lifetime/timeout design and .NET comparison](../../design/http-client-lifetime.md).
This changes no HttpClient API, timeout/cancellation semantics or execution-mode scope.
