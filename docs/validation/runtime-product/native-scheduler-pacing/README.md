# Native scheduler idle and integrated pacing qualification

Slices: `e5009853a` scheduler idle integration, `f547c875b` opt-in runtime pacing,
and `ffc518bdb` async-pump ownership refresh. Stress remains the installed-host
default; the interpreter's independent allocation-pressure policy is unchanged.

[Hosted run 38455945430](https://github.com/marinasundstrom/neoCLR/actions/runs/38455945430)
passes at `f547c875b36359335e9e7b945aa0f78bcf6ab1d2` on macOS ARM64 and Windows x64:
33 GC/root executions, seven socket cases (including stress/paced schedulers),
package checks and integrated closed-workload pacing benchmarks. The macOS
attachment check also passes. Reports record source hashes and exact commands.

Local compiled constructor/callback execution passes non-GC, stress-GC and paced-GC
variants against interpreter output; the compiled socket callback consumer passes.
The pressure tests cover large allocations, exhaustion, root-only reachability
changes, foreign/stale boundaries, unexpected collection epochs and owned stacks.
Actual scheduler tests retain pending timer callbacks and reclaim abandoned owners
exactly once without overwriting the caller's delivery ticket.

The follow-up caught an async-pump bug: waiting/ready counts taken before acquire
became stale when idle collection expired a body. The new retained-stack regression
failed at its pump assertion before the fix, then passed with 321 entered bodies,
160 resumed and 161 abandoned, with zero reserved bytes after shutdown. All 35
local GC/root executions pass including retained-stack and idle-drain tests. The
macOS-only carrier is explicitly gated; it is not Windows carrier qualification.
The local report predates a final addition of native-stack.c to input hashes.

Initial local fixture/tooling corrections: retain a callback reused by the test
across idle collection, release that test root before asserting reclamation, use
the explicit Xcode SDK for the direct carrier compile, and use the gate rather than
a recipe-dependent validator without its required recipe. No failure is counted
as passing evidence. Whitespace/line endings in retained reports are normalized.

See [integrated benchmark results](../../../../benchmarks/http-gc-pacing/README.md#integrated-native-policy-qualification--2026-10-13).
Measured results describe a closed native workload, not HTTP or all managed code.
Default promotion requires managed HTTP/runtime-async and sustained-load checks.
Repeated pending polls currently force full collection; avoiding that cost needs
an audited reachability-change signal, not an unqualified idle timeout heuristic.

The follow-up [run 38456400922](https://github.com/marinasundstrom/neoCLR/actions/runs/38456400922)
passes Windows x64 GC/root and socket/package gates at
`ffc518bdbf139998bcd1313969f233f021a262d3`. Its hosted macOS job remained queued
when saved. Local macOS validates the carrier fix and all 35 GC/root executions;
run 38455945430 supplies completed hosted macOS qualification of the shared policy.
The follow-up workflow as a whole is not reported as complete.

## Managed workload follow-up

The [managed HTTP qualifier](../../../../benchmarks/http-gc-pacing/README.md#managed-http-qualification)
rebuilds current Raven plain-text and JSON applications, then uses identical native
objects for stress and paced hosts. The initial JSON attempt lacked reflection
roots; the harness now obtains them using the existing native-sample discovery
procedure. The failed attempt is not passing evidence.

Runtime-await qualification required refreshing an obsolete bundle recipe and
awaiting the current asynchronous `Console.ReadLine` API in the client sample.
The first fresh run passed all AOT cases and the first seven JIT cases, then hit
the server-readiness timeout while JIT compilation was still running. Startup and
peer acceptance now allow the same bounded compilation window as the other JIT
invocations. These are qualification-tool/sample corrections, not GC fixes.

[Windows managed HTTP run 38458148179](https://github.com/marinasundstrom/neoCLR/actions/runs/38458148179)
passes at `2a22f1f460842960a0dedd84b8f1b6759f06f978`; the retained run metadata
and benchmark reports record the platform and commands. Both macOS and Windows
validate 640 responses and full reclamation. Windows short sequential medians
improve 2.20× plain and 2.64× JSON; local timings overlapped compiler work and
remain diagnostic context rather than isolated performance evidence.

The fresh [paced runtime-await report](runtime-await-paced-macos-20261013.json)
passes all 24 JIT/AOT cases on macOS ARM64: language, async cleanup, operation
lifetime, stress, stack reuse, terminal fault, HTTP client/server, timeout,
cancellation, protocol recovery and HTTP continuation fault. The same run also
passes the interpreter lifetime control and four same-heap entries for language,
cleanup and stack-reuse consumers. All recorded input hashes still matched after
the run. The [source recipe](runtime-await-source-macos-20261013.json) independently
passes the combined HTTP interpreter consumer. These reports use current libraries
and application runtime-await lowering; referenced libraries retain their existing
lowering. They do not qualify Windows native suspension or a general default flip.

The corrected run completes after the stale sample and readiness-window fixes
committed in `d63cf9dd8`. No further collector or scheduler change was needed for
these managed workload checks. Sustained/concurrent load, library/provider
integration and platform-wide runtime-await defaults remain separate gates.
