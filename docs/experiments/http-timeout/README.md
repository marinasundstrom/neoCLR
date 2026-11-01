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
host-controlled network validation remain a separate suite, described below. Python keeps builds,
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


## Real-socket timeout fixture

Network.rvn now contains two named NeoClr.Testing cases: reporting HttpError.TimedOut
for a two-second request timeout, and accepting HTTP 200 after a delayed response
with the timeout disabled. NetworkMain explicitly awaits the selected case and
reports its completed Result through TestSuite and ConsoleReporter, returning the
report exit code. The URL and case selection remain host-supplied; automatic
async/parameterized discovery is not added. Client disposal remains before result
assertions.

The independent Python peer still accepts and reads the request, withholds a response
for the timeout case or delays success by 350 ms, and observes EOF on the socket.
It requires socket closure in addition to the exact framework transcript and empty
stderr. Interpreter selection is explicit. Peer/process timeouts and the existing
interpreter/AOT scope remain unchanged.

```sh
python3 scripts/validate-http-timeout-network.py --bundle <matching-bundle> \
  --runtime <matching-bundle>/bin/neoclr --aot <matching-bundle>/bin/neoclr-codegen \
  --output target/http-timeout-network-tests
```

[Network migration evidence](../../validation/testing-migration-20261101/README.md#real-http-timeout-sockets)
records four successful exchanges and a wrong-status assertion control that still
closes its socket. Historical network-validation.json is unchanged.
