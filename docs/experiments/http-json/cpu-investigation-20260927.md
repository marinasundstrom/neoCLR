# HTTP sample CPU investigation — 2026-09-27

The author asked to run the installed sample server, investigate high CPU and decide
whether optimization is needed. **Steady idle is cheap; request processing merits a
bounded optimization.** Separately, two older Raven language servers were each
using approximately one CPU core even while the sample server was not running.

## Measured behavior

[Exact results and artifact hashes](cpu-investigation-20260927.json) describe the
installed release runner from neoCLR 03947b07, Raven b7bc6838d and the matching SDK.
No compiler rebuild, runtime change, timeout relaxation or quota change was made.
The machine had background load; these are diagnostics, not throughput certification.
CPU seconds measure process work, while elapsed seconds also include waiting.

| Phase | CPU cost | Observation |
| --- | --- | --- |
| Startup to listening | 4.35 seconds | 4.81 seconds elapsed; assembly, linking and verification are repeated at launch |
| Ten seconds idle before requests | 0.04 seconds | Approximately 0.4% of one core |
| Ten seconds idle after requests | 0.09 seconds | Approximately 0.9% of one core |
| Small GET, both runs | 0.91–1.22 seconds/request | Returns 200 and the report |
| Small POST, both runs | 1.36–1.65 seconds/request | Returns 201 and accepted:true |
| Raw request with 128-byte headers | 0.73–0.86 seconds | Returns 200 |
| Raw request with 1,024-byte headers | 3.21–3.49 seconds | Returns 200 |
| Raw request with 2,049-byte headers | 4.26–4.44 seconds | Connection closes without a response; sample exits 1 |

A shorter repeat measured 0.5% before requests and 3% immediately after them over
two-second windows. Those observations are retained: the latter can include final
completion/cleanup after the client receives its response. The ten-second windows
and stack profile provide stronger evidence about steady waiting. In the idle
profile, 2,282 of 2,296 main-thread samples are under Wake::park/condition-variable
waiting. The scheduler's existing 10 ms park is not a one-core busy loop.
Both successful request batches finish with exit 0; the first reports zero live
managed objects after seven requests. The over-limit experiment deliberately records
failure, not a passing supported request.

## Where active CPU goes

The request profile's main thread has 3,256 samples. Its collapsed top-of-stack
counts include 421 in Module::type_definition, 346 in vm::resolve, 389/176 in mutex
lock/unlock, and 230/154/151 in shared-heap, slot and frame array accounting. These
are sampling observations, not separately timed phases or additive speedup estimates.

Source inspection explains the repeated work:

- `Module::type_definition` scans definitions by name and generic arity.
- `vm::resolve` scans functions and constructs/substitutes matching signatures.
- Once arrays are in use, the interpreter checks frame/heap array usage at each
  instruction boundary. Existing slot summaries avoid repeated payload traversal,
  but storage and accounting still incur synchronization and aggregation costs.
- HTTP request decoding processes each header byte through managed methods. More
  bytes amplify interpreter overhead even for the same route and response.

See the [earlier slot-accounting investigation](slot-budget-summary.md) for the
existing quota contract and .NET/CLR comparison. A native sampling profile also
contains default-value construction, allocations and field/type checks; this is
not evidence that one lookup cache will remove all latency.

## Recommended sequence

1. **Keep the sample alive after a rejected request.** The previous live server
   had already exited with Request header limit exceeded. Both controlled 2,049-byte
   requests reproduce the fatal path. The decoder's 2,048-byte bound is explicit;
   the sample propagates an Accept failure into System.Fault. Do not merely raise
   the bound: handle an invalid connection without terminating the serving loop,
   and specify whether the API can return an HTTP error response before closing.
   Actual headers from the previous browser request were not captured.
2. **Index or cache runtime metadata resolution in a bounded slice.** Start with
   immutable loaded-program type/function lookup and measure the same fixtures
   before/after. Keys must preserve module/revision identity, generic arguments,
   overload and receiver semantics; retain access checks and malformed-metadata
   rejection. The benefit would be avoiding repeated scans of thousands of methods;
   costs are memory, cache ownership and invalidation rules. This is a proposal,
   not an implemented speedup. Like CLR loading/JIT resolution, aim to avoid
   repeatedly resolving an unchanged call during execution; it does not imply JIT
   compilation or CLR performance parity.
3. **Then reassess quota/slot synchronization.** Preserve instruction-boundary
   enforcement, shared-object safety, alias mutation and GC behavior. Reducing
   checks or removing locks without an ownership/accounting proof is not an
   acceptable optimization. A broader accounting redesign has greater risk than
   the initial lookup work and should follow a fresh profile.
4. **Treat startup separately.** Loading/verification costs several CPU seconds per
   launch but does not explain steady server CPU. Consider a reusable loaded image
   later, preserving validation and artifact identity. No scheduling, green-thread,
   HTTP protocol or API expansion is justified by this profile alone.

This is material work for a usable POC, not a request for general optimization:
spending about one CPU second serving a tiny local GET and several seconds parsing
1 KB of headers is enough to justify a focused runtime follow-up. It is not yet an
agreed numeric release threshold or a controlled performance comparison with .NET.

## Separate editor finding

At the start, PIDs 37889 and 38069 were older cached Raven.LanguageServer instances,
roughly six days and twenty hours old, each near 100% CPU. The latter's native sample
reports a 10.1 GB physical footprint (12.8 GB peak), with managed frames unresolved
and GC work visible. This predates the fresh server run and is independent of its
idle behavior. It does not establish a source-level Raven bug or attribute the issue
to the newly installed extension. Capture managed stacks/allocation evidence in a
separate Raven investigation; restart the affected old editor sessions as mitigation.
Neither process was terminated or modified during this investigation.

## Reproduce

The scripts start and stop only their own server and require macOS ps/sample. Use
fresh output directories and run serially; the header check asserts the currently
observed fatal path and must be updated when that behavior is repaired.

```sh
python3 docs/experiments/http-json/measure-server-cpu.py \
  --bundle "$HOME/.neoclr/experiments/development-20260927-async" \
  --output /tmp/neoclr-http-cpu
python3 docs/experiments/http-json/measure-server-headers.py \
  --bundle "$HOME/.neoclr/experiments/development-20260927-async" \
  --output /tmp/neoclr-http-headers
```

Raw profiles and logs remain in target/http-cpu-investigation on the measured host.
No full test suite, website build or public API snapshot refresh was needed.
