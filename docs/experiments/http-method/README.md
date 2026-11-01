# HTTP method values and wire spelling

Development validation for `System.Web.Http.HttpMethod`. The framework suite in
[`runtime/System.Web/test/HttpMethodValuesTests`](../../../runtime/System.Web/test/HttpMethodValuesTests/NeoClr/RuntimeTests/Tests.rvn)
checks predefined values, parsing, custom construction, equality/hash behavior and
request factories. [Its evidence](../../../runtime/System.Web/test/HttpMethodValuesTests/validation.json)
records eight passing cases in both interpreted and macOS ARM64 native execution.

`Client.rvn` and `Server.rvn` exchange the custom token `m-SEARCH` with independent
Python peers. The client fixture also isolates invalid constructor faults and the
unsupported CONNECT guard. [Wire evidence](validation.json) records six cases in
each mode. The shared Windows workflow runs the same fixtures; no Windows result
is claimed by this local report.

Reproduce against a matching native development bundle:

```sh
python3 scripts/test-runtime-library.py --suite http-method-values --bundle PATH --output target/method-values
python3 scripts/validate-http-method.py --bundle PATH --output target/method-wire
```

See [the implemented contract](../../http-client-design.md#httpmethod-implementation--development-2026-10-11)
for the .NET comparison, source compatibility change and protocol limits. This is a
library API; no new compiler capability or metadata encoding is introduced.


## NeoClr.Testing migration — 2026-11-01

The client, CONNECT rejection and server request checks now return TestFailure from
three named tests. Main explicitly awaits the selected lifecycle and uses TestSuite,
ConsoleReporter and report.ExitCode through a completed-result reference adapter.
The server handler records its request assertions, echoes the response and exposes
the result after ServeOne settles; a handler mismatch becomes a named framework
failure. Listener/port setup failures remain terminal. Async/parameterized discovery
is not added; the host supplies the client URL and the server announces its port
before reporting the test result.

Python still validates exact custom request spelling, Content-Length and body bytes,
and response X-Method/body bytes using independent peers. Invalid constructors
remain subprocess fault checks. Successful guest runs now require the exact named
framework transcript, successful exit and empty stderr. Interpreter execution is
selected explicitly, preserving instruction limits despite the runtime's JIT default.
The existing interpreter/AOT scope and process budgets remain unchanged.

Use explicit matching runtime/codegen paths with the reproduction command above:
`--runtime PATH/bin/neoclr --aot PATH/bin/neoclr-codegen`.
[Migration evidence](../../validation/testing-migration-20261101/README.md#custom-http-methods)
records both modes and a server callback negative control with an independently
validated wire response. The historical validation.json remains unchanged.
