# HTTP method values and wire spelling

Development validation for `System.Web.Http.HttpMethod`. The framework suite in
[`runtime/raven/tests/http-method-values`](../../../runtime/raven/tests/http-method-values/Tests.rvn)
checks predefined values, parsing, custom construction, equality/hash behavior and
request factories. [Its evidence](../../../runtime/raven/tests/http-method-values/validation.json)
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
