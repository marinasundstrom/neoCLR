# Neo.Web generic handler qualification — 2026-10-31

Development work following foundation commit `6228d0be9`, on macOS ARM64.
Matching unified Raven main compiler commits: `8e9bb96ec` (structural contract)
and `9cd1c94ae` (imported module function bindings).
This is not a published release or Windows qualification.

The bounded API uses `MapGet<T: func>` / `MapPost<T: func>`, registration-time
signature inspection, cached route/service bindings and per-request checked
invocation. Results use the standard closed `HttpResult<T>` union; the first HTTP
adapter supports string payloads and their task form. WebDemo exposes ordinary
handler functions and has a separate NeoClr.Testing project. Framework HTTP behavior
remains in `extensions/Neo.Web/test`.

Logs below preserve output with trailing whitespace normalized.

## Focused checks

- `compiler-tests.log`: 28 passing semantic/portable-lowering tests, including
  structural member enumeration, binding, completion, named-handler inference,
  .NET isolation and primitive captured receiver calls.
- `runtime-tests.log`: 37 passing function/reference/construction tests, including
  transitive library type initialization without granting host access.
- `metadata-tests.log`: function constraints survive authored instance-method
  references, generic construction and native round trips; nonfunction arguments,
  invalid ordinals, conflicting contracts and CLI erasure reject.

## Execution qualification

| Consumer | Interpreter | JIT | Native/AOT |
| --- | --- | --- | --- |
| Neo.Web framework tests | 4/5 | 5/5 | 5/5 |
| Direct WebDemo handler tests | 4/4 | 4/4 | 4/4 |
| Independent Python HTTP peer | 12/12 | 12/12 | 12/12 |

The interpreter's in-process framework HTTP test fails with transport `TimedOut`.
Its middleware test passes, including scope lifetime and joined shutdown. The
independent interpreter HTTP peer passes all requests, observes 12 middleware
completion logs and clean shutdown. This is bounded qualification, not an all-green
execution matrix; the in-process timeout remains an unresolved gap.

`results.json` retains the original campaign commands and tool hashes, plus the
follow-up native sample and handler-test build inputs/artifacts. The initial campaign
stopped at a native handler-test build failure caused by retaining unused inspection
metadata. The reachability fix resolves that failure: both native and JIT handler
suites pass. Follow-up JIT framework and native HTTP sample checks also pass with
the updated backend. The original native framework and JIT/interpreter HTTP peer
results remain applicable; no web implementation changed in that backend fix.

The development bundle reuses core libraries composed during the compiler working
build; this is not a claim that every library was rebuilt from Raven `9cd1c94ae`.
The compiler, Neo.Web and shared sample logic use the imported-function fix. Exact
build inputs and artifact hashes take precedence over the bundle directory name.
Native/JIT hosts use the matching 2,048-type budget. Imported function values are
currently limited to nongeneric module-level functions.

Additional focused checks: 18 reflection metadata tests, two inspection-retention
tests, three native-budget tests and seven dynamic result tests pass. The native
API snapshot uses renderer `9cd1c94ae`; four snapshot checks pass. The local website
build validates 6,895 pages. Publication was not performed.

## Middleware contract

`Use` composes fixed `(context, next)` function values once at startup. The first
registered middleware is outermost. Await next with the supplied context at most
once, or return to short-circuit. One request scope spans the entire outermost
task, including work after next; shutdown joins that work. Framework tests cover
ordering, short-circuiting, typed failure, cancellation, default routing and joined
shutdown. WebDemo supplies request logging middleware. No continuation retention,
reentrant next invocation, branch builder or streaming contract is introduced.

## Explicit boundaries

The union uses `HttpResult<string>.Ok(_)`, `.Created(_)`, `.BadRequest` and
`.NotFound` in exhaustive matches. Nongeneric sealed roots with generic nested
cases were probed but not implemented: native admission and existential generic
case-pattern emission need separate work. No manual carrier was introduced.

Query/body binding, serializers for other payload types, arbitrary Task adapters,
streaming and optimized generated invocation remain future work.
Reflection and binding plans are cached at registration, but request arrays,
boxing, validation and DI resolution remain. These checks are correctness evidence,
not interpreter/JIT/AOT throughput benchmarks.

Native function inspection currently retains descriptor factories for bindings
whose target and shape are already closed before specialization. Bindings created
only by specializing an open generic body are not yet a general native inspection
contract. Interpreter inspection uses the loaded bound function directly. The
reference handlers use the supported closed binding form. Generated invocation,
more precise descriptor retention and cross-mode benchmarks remain later work.
