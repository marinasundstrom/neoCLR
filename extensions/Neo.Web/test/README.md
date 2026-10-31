# Neo.Web framework tests

`Neo.Web.Tests` is a separate executable test project using `NeoClr.Testing` and
referencing the matching development bundle's `Neo.Web` assembly. It verifies
public framework behavior: builder ownership, route registration, generic binding,
closed typed results, function/typed middleware ordering, lazy scoped activation,
request scopes across await, response status and shutdown.
The exhaustive `HttpResult` match is compiled against the imported framework API.

Run it with the repository's `scripts/validate-neo-web.py` harness. HTTP integration
checks use real loopback connections; result-contract checks need no listener.
Sample application behavior belongs separately in
[`samples/web/test/WebDemo.Tests`](../../../samples/web/test/WebDemo.Tests).
No separate Neo.Web testing framework is introduced.

The binding follow-up adds registration rejection for conflicting/multiple body
sources, nonactivating service probing (including a keyed-only negative control),
framework-parameter identity, and header replacement/injection rejection. The
external WebDemo HTTP consumer additionally covers query conversion/UTF-8 errors,
JSON DTO/scalar bodies, 400/415 failures and the application response-cache example.
