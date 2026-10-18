# Direct incoming request construction — 2026-10-18

## Fresh baseline

Rebuilt all four managed libraries from current source at 4014257ba, using
the qualified Raven compiler 478e0dd85 (compiler-primitive-4), current release
backend and current native sources. Recompiled Server.rvn and rebuilt Go.
This replaces the older managed guest identified in the concurrent-GC review.
Build manifests include source/tool/artifact hashes and commands.

The baseline uses paced synchronous GC, the existing 32 MiB native heap and
unchanged Serve callback fixture. The initial 10-second 300/sec smoke completes
all 3,000 measured requests; it verifies the rebuilt fixture, not sustained capacity.
Matched baseline/candidate/Go load measurements follow in a separate slice.

Rebuild with scripts/build-native-class-library.py and the shared
../task-completion-20261018/build.py, using the retained final-load/native link
recipe. Evidence contains both library build provenance and guest build commands.
No public HTTP API or GC policy changes in this baseline slice.

## Direct incoming construction

`FromIncoming` now validates the separated authority and origin-form target, then
constructs HttpRequest directly. It does not concatenate a scheme/authority/target,
allocate Uri, split URI components, or parse the resulting URL again. Host and
Target remain strings. Target and authorities without a port reuse existing owned
strings; explicit ports extract the hostname. No pooled-buffer views escape.

The decoder remains responsible for origin-form admission, framing, headers and
body ownership. The incoming constructor preserves the old ASCII/percent grammar,
URI error precedence, 4096-byte parser bound, 1024-byte combined URL-equivalent
bound, 253-byte hostname bound, ports and malformed Host suffix handling. The
limits count the implied scheme length without constructing it. The byte grammar
intentionally matches the existing Uri parser; future grammar changes must update
the differential cases too. Two temporary field encodings remain; this is not
zero-copy parsing or a complete elimination of per-request allocation.

[ASP.NET Core HttpRequest](https://learn.microsoft.com/en-us/dotnet/api/microsoft.aspnetcore.http.httprequest?view=aspnetcore-10.0)
exposes Host, Path, QueryString and Scheme separately, with URL-combining extension
methods. That supports keeping reconstruction optional. This slice preserves
neoCLR's existing raw Target string and separate hostname/Port rather than changing
them to ASP.NET's HostString/decoded Path contracts. It changes internal work, not
the public API, HTTP feature support or outbound URL/Uri overloads. The matching documentation reference is regenerated and remains
binary-identical; only its source fingerprint changes.

The public-wire Probe deliberately uses Accept to inspect parser outcomes. Its
50 cases compare exact success fields, error cases/messages, response framing,
body and content type against the fresh pre-change AOT oracle. All input is
fragmented. The performance fixture continues to use Serve and a queued await.
Plain HTTP is exercised here; TLS transport itself is unchanged and is not
requalified by these wire cases.

All 50 cases pass in interpreter, JIT and AOT against the fresh baseline AOT
oracle. The first probe build exposed an unrelated SocketError.ToString backend
restriction through HttpError.ToString; the final probe explicitly matches tested
error payloads. An initial interpreter attempt used the stale release executable
and failed enum metadata admission before listening. The qualified primitive/enum
runtime passes all cases. Both failures are retained, not treated as passes.

Reproduce with the shared guest builder and Probe.rvn, then verify.py --build
BUILD --library LIBRARY --runtime RUNTIME --backend CODEGEN --host JIT_HOST
--output REPORT. Establish the old-cohort oracle with --modes aot, then add
--oracle REPORT for the candidate's three-mode comparison. Tool and module hashes
are retained in validation-inputs.json and the build manifests.

## Allocation impact

Separate diagnostic runs process 200 measured +20 warm-up +one stop exchange,
including startup/shutdown in all allocation counts. Zero dropped caller records.

| Counter | Fresh baseline | Direct incoming request | Change |
| --- | ---: | ---: | ---: |
| Allocations | 58,914 | 54,715 | −7.13% |
| Allocated payload bytes | 3,212,315 | 3,092,539 | −3.73% |
| GC idle calls | 18,543 | 17,302 | −6.69% |

The allocation difference is 4,199, equivalent to 19 per exchange. The candidate
compiled Serve closure contains no Uri methods; the baseline does. These counts
are against the freshly rebuilt managed cohort, not the older concurrent-GC
experiment guest. [Counters and compiled Uri functions](evidence/allocation-summary.json),
[baseline callers](evidence/baseline-callers.json.gz),
[candidate callers](evidence/candidate-callers.json.gz). Instrumented timing is
not used as speed evidence. Idle-call reduction is not a measurement of GC CPU.

## Offered-load measurements

Same loopback host, unchanged ten-byte response and queued Task.Run await,
16 server lanes, 64 client in-flight slots, new connection per request. Both neoCLR
builds use paced synchronous GC and the 32 MiB heap bound; Go uses GOMAXPROCS=1,
GOGC=100 with GOMEMLIMIT unset. Each point has a fresh server and warm-up.
No builds or profilers run during timing. Requests are scheduled at a fixed offered
rate; arrivals that cannot get an in-flight slot remain failures, not discarded
work. Completed/sec counts completions inside the window, excluding later drain.
A provisional pass requires all arrivals successful, offered-latency p99 ≤250 ms
and dispatch-lag p99 ≤5 ms. This is an experimental criterion, not a service SLO.

### Initial 30-second matrix

Two rounds reverse both rate and runtime order. The author subsequently reported
side activities; their exact interval is unknown. Retain every point as potentially
contended evidence. The separate initial smoke also had 51.9 ms p99 dispatch lag:
all 3,000 responses were correct, but that smoke did not pass the timing criterion.

| Build / round | Offered/sec | Successful / offered | Completed/sec in window | CPU ms/completion | Offered p99 ms | Dispatch lag p99 ms | Pass |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| [baseline 1](evidence/load-baseline-1-300.json) | 300 | 9,000 / 9,000 | 299.97 | 1.996 | 5.84 | 0.72 | yes |
| [baseline 1](evidence/load-baseline-1-400.json) | 400 | 11,932 / 12,000 | 397.70 | 2.044 | 172.35 | 1.04 | no |
| [baseline 2](evidence/load-baseline-2-300.json) | 300 | 9,000 / 9,000 | 300.00 | 2.097 | 42.59 | 3.10 | yes |
| [baseline 2](evidence/load-baseline-2-400.json) | 400 | 12,000 / 12,000 | 399.93 | 1.955 | 8.41 | 0.46 | yes |
| [candidate 1](evidence/load-candidate-1-300.json) | 300 | 9,000 / 9,000 | 299.97 | 1.916 | 7.56 | 0.87 | yes |
| [candidate 1](evidence/load-candidate-1-400.json) | 400 | 12,000 / 12,000 | 399.97 | 1.823 | 7.36 | 0.65 | yes |
| [candidate 2](evidence/load-candidate-2-300.json) | 300 | 8,929 / 9,000 | 297.57 | 2.224 | 226.00 | 93.96 | no |
| [candidate 2](evidence/load-candidate-2-400.json) | 400 | 11,952 / 12,000 | 398.37 | 1.865 | 89.24 | 3.29 | no |
| [go 1](evidence/load-go-1-300.json) | 300 | 9,000 / 9,000 | 300.00 | 0.282 | 5.03 | 1.64 | yes |
| [go 1](evidence/load-go-1-400.json) | 400 | 12,000 / 12,000 | 400.00 | 0.188 | 251.35 | 243.81 | no |
| [go 2](evidence/load-go-2-300.json) | 300 | 9,000 / 9,000 | 300.00 | 0.210 | 8.52 | 4.40 | yes |
| [go 2](evidence/load-go-2-400.json) | 400 | 12,000 / 12,000 | 400.00 | 0.160 | 3.78 | 1.12 | yes |

### Bounded 60-second confirmation at 300/sec

The author reported side activities during the first confirmation baseline, so
that point may overlap them and must not be used as clean speedup evidence. The
remaining points ran after the report; this is not independent proof of host
exclusivity. The confirmation was limited to two reversed-order rounds, without
selecting only successful runs or searching indefinitely for a favorable result.

| Build / round | Offered/sec | Successful / offered | Completed/sec in window | CPU ms/completion | Offered p99 ms | Dispatch lag p99 ms | Pass |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| [baseline 1](evidence/confirmation-baseline-1-300.json) | 300 | 17,832 / 18,000 | 297.18 | 2.025 | 282.41 | 27.91 | no |
| [baseline 2](evidence/confirmation-baseline-2-300.json) | 300 | 18,000 / 18,000 | 300.00 | 1.944 | 5.37 | 0.44 | yes |
| [candidate 1](evidence/confirmation-candidate-1-300.json) | 300 | 17,972 / 18,000 | 299.53 | 1.812 | 19.32 | 0.68 | no |
| [candidate 2](evidence/confirmation-candidate-2-300.json) | 300 | 17,973 / 18,000 | 299.55 | 1.820 | 88.23 | 1.47 | no |
| [go 1](evidence/confirmation-go-1-300.json) | 300 | 18,000 / 18,000 | 300.00 | 0.202 | 4.47 | 1.21 | yes |
| [go 2](evidence/confirmation-go-2-300.json) | 300 | 18,000 / 18,000 | 300.00 | 0.204 | 5.43 | 1.85 | yes |

The final reversed pair uses **6.38% less CPU/completion** (1.820 versus 1.944 ms),
but the candidate misses 27 arrivals versus none for the baseline, with p99
88.23 versus 5.37 ms. The other candidate confirmation also misses 28 arrivals.
Go completes all arrivals in both confirmations; final-pair candidate CPU is
8.93 times Go's 0.204 ms/completion. This is a local workload comparison, not a
maximum-capacity measurement or a general runtime ranking.

Average CPU improvement does not imply lower spikes: in the final pair, the
largest roughly half-second process CPU interval is **0.872 core** for the
candidate versus **0.631 core** for the baseline (Go 0.079). These coarse sampled
peaks are not instantaneous CPU measurements. Peak sampled RSS is 12,816 KiB
candidate, 12,912 KiB baseline and 15,456 KiB Go; the small neoCLR difference
is not a broad memory-capacity claim. All runs validate admitted response bytes,
clean shutdown and zero server/client completion accounting difference.

**Conclusion:** fewer allocations and lower CPU in the final matched comparison,
with no demonstrated sustained throughput gain. The candidate is not qualified
as a reliable 300/sec operating point by these longer runs. Tail latency and
admission losses remain unresolved; the post-report misses must not automatically
be dismissed as side activity, nor assumed to be GC. The next investigation should
correlate request admission/dispatch and native scheduler/GC timestamps before
changing another policy. The optimization preserves functional behavior; its
observed admission/latency tradeoff remains an explicit performance concern.

[Initial summary](evidence/load-summary.json),
[confirmation summary](evidence/confirmation-summary.json), and
[activity/interpretation notes](evidence/measurement-notes.json) retain every point.
Each linked trial has a lossless .json.gz report with raw request records and
checksums. Comparison manifests retain commands, environment and input hashes.
Reproduce with compare.py --baseline-build BASELINE --candidate-build CANDIDATE
--go-build GO --rates 300 400 --duration 30 --output OUTPUT; the confirmation uses
--rates 300 --duration 60. summarize.py OUTPUT prints the tables' source values.
