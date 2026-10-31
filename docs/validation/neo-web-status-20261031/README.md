# HttpResult status projection — 2026-10-31

Development follow-up to `410aec015`, using unified Raven `9cd1c94ae` on macOS ARM64.

`HttpResult<T>.StatusCode` is a read-only int member with an exhaustive case match:
Ok/Created/BadRequest/NotFound project 200/201/400/404. It works independently of the
payload type and hosting. The HTTP writer consumes the property; direct WebDemo
handler tests assert it alongside typed cases/payloads. Framework tests exercise
imported closed values and an open generic helper. No mutable status override,
new result alternative, serializer or endpoint-specific result subset is introduced.

## Execution and API checks

- Framework: native and JIT 5/5; final interpreter 5/5 in two runs.
- Direct WebDemo tests: native, JIT and interpreter 4/4.
- Independent Python HTTP consumer: 12/12 native requests, including all four
  result statuses and unchanged bodies, with middleware completion and clean stop.
- 99 focused runtime tests pass: slots 12, shared heap 12, managed arrays 15,
  reference arrays 15, async await 35 and runtime GC 10.
- Native API rendering includes the union member page with `val StatusCode: int`
  and its XML summary. Four snapshot tests pass; aggregate source fingerprints
  are refreshed. Website validation checks 6,897 pages, retaining 131 documented
  legacy pages and existing missing-summary warnings. No publication occurred.

[results.json](results.json) retains commands, input hashes, all failed and successful
runs, and component build provenance. Core libraries are reused from the prior
qualified working-build bundle; Neo.Web is freshly composed with the matching
compiler. The native reference's base provenance remains explicit, with the new
Neo.Web composition recorded in componentBuilds.

## Interpreter accounting follow-up

The first candidate interpreter HTTP run failed while website work was active;
an isolated rerun also failed. Both passed the direct status-property checks and
middleware test. The previous framework assembly still passed using the identical
heap-cache interpreter, so competing work alone did not explain the observation.
No failure was removed or converted into an expected pass.

The prior sampling evidence already identified frame accounting as the remaining
major cost after heap totals were cached. Reuse complete totals for stable frame
argument/local slots, including parked activations; their alias writes invalidate
the owning frame through the same numeric-summary mechanism as heap slots.
Evaluation stacks and separate construction storage are still measured each time.
Current limits and collection-before-rejection remain unchanged, with no timeout,
instruction budget, scheduling or HTTP policy changes.

The new regression mutates a caller-owned string array through a callee alias after
its summary has been filled and verifies the byte quota still rejects growth. The
other tests cover copies, tighter limits, nested aliases, native replacement,
byte-write epochs, parked async payloads, GC roots and collection under pressure.
Numeric caches retain no managed values. Costs include one invalidation flag per
frame and additional host metadata; this is not a claim of universal runtime gains.

The exact same candidate framework assembly passes with the new interpreter in
30.93 and 28.35 seconds, versus the earlier failed 76.05/58.13-second runs. Failed
runs stop HTTP assertions early, so those totals are not a throughput comparison.
The earlier framework control passed in 65.84 seconds. These local observations
qualify this consumer; they do not guarantee arbitrary-load latency or close every
historical interpreter HTTP timeout.

Reuses the [.NET/CLR array and quota comparison](../../managed-arrays.md) and the
[prior heap-cache investigation](../neo-web-timeout-20261031/README.md). The separate
[result design](../../design/neo-web.md#variant-enforcement-and-aspnet-core-familiarity-2026-10-31)
compares existing ASP.NET Core typed alternatives with the intended Raven closed
variant/exhaustiveness model; broader endpoint result sets remain future work.
