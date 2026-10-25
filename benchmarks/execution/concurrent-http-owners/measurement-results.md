# HTTP owner performance baseline and an unresolved close stall

2026-10-25. Measure synchronous and live GC with one/four isolated AOT application
owners. Defaults remain unchanged. The initial Task.Run fixture is kept separate
from the new minimal constant plaintext handler in `Plain.rvn`, which returns
`Hello, World!` without task scheduling or per-request owner identity formatting.
The same server/control-listener lifetime and 1 MiB heap per owner are retained.

## Measurement contract

Each cohort rotates four mode/owner combinations through three rounds. Every
process warms up for one second, then accepts 32 closed-loop clients for three
seconds, drains requests, and shuts down each owner through its control listener.
Every body/status is checked, every owner's completed count must be positive,
and summed server counts must match client responses including warmup/drain.
Live diagnostics require positive completed epochs for every owner, balanced
start/finish/cancel counts, no active candidate and released allocation indexes.
No compilation or other benchmark job runs during timing.

The new driver reports wall-window throughput, client latency, native managed
allocation counts, observed process RSS high-water and server CPU per response.
Final runs use `wait4` for **server process CPU only**, including all owner/marker
threads, startup, warmup, drain and cleanup; divide by all checked responses over
that lifetime. Client CPU is reported separately. Earlier Python-only task runs
used the child CPU delta, with the server as the only child during each run.
Owner `PROCESS` CPU records are cumulative process snapshots, never summed.
RSS uses the maximum owner-reported process high-water, not a sum of owner RSS.

The initial Python generator used nearly one core and constrained four-owner
capacity. The existing Go comparison client now accepts an optional expected body
while preserving its legacy ten-byte default. Positive default/custom-body tests
and a wrong-body negative pass. It reads through EOF to avoid hiding connection
closure problems. The phase extension reports total/max connect and post-connect
response time, including drain; ordinary latency percentiles retain only responses
completed inside the window. These client delays are not GC pause distributions.
Native GC profiles are retained, but hook timing is not a complete pause histogram.

## Results and limits

Task.Run/Python cohort (`task-cost.json.gz`): 205,805 checked responses. Median
sync/live one-owner throughput is 4,588/2,604 requests/s; four-owner is 5,326/4,510.
CPU per response is 213/399 us for one owner and 332/694 us for four. About 224
managed allocations/response. This workload and client are not the minimal baseline.

Two complete Go/plaintext cohorts (`plain-cost` and `plain-phases`) check **568,621
responses**. The phase cohort gives:

| Mode | Requests/s range | Median CPU/response | Managed allocations/response | Median observed RSS |
| --- | ---: | ---: | ---: | ---: |
| Sync, one owner | 5,017–5,154 | 194 us | 208.03 | 9.47 MB |
| Live, one owner | 2,618–2,650 | 393 us | 208.05 | 11.99 MB |
| Sync, four owners | 5,461–13,086 | 274 us | 208.07 | 19.89 MB |
| Live, four owners | 6,916–7,444 | 514 us | 208.07 | 23.79 MB |

The stable one-owner result shows roughly twice the CPU cost for live collection,
with no meaningful managed-allocation reduction. The small fractional allocation
differences include startup amortization and scheduling; do not claim equality or
a reduction from rounded values. Native allocation traffic is not counted here.
Four-owner throughput is **not stable capacity evidence**: synchronous p99 rises
from about 3 ms to 231–232 ms across rounds. A separate synchronous-only recheck
passes three runs at 12,709–13,169 requests/s and approximately 270 us/response.
All owners receive roughly balanced request counts; uneven owner utilization is
not established as the cause. Four-owner live is faster than the stalled sync
trials but still uses far more CPU; that is not sufficient for default promotion.

Phase timing narrows the stall to **after connection establishment**: in the two
slow synchronous trials, aggregate connect time is only 2.4–2.6 seconds across all
requests, versus about 94 seconds waiting for responses across the 32 concurrent
lanes. Maximum post-connect delays reach 258–272 ms. The scheduler's ordinary
wait cap is 1 ms, not 250 ms. This does not rule out scheduling, OS/network delay,
or connection-close lifetime issues.

An initial plaintext cohort failed during the final warmup; its client error text
was not captured. Preserve it as `plain-incomplete.json.gz`, not a passing report.
The driver now records every client command/stdout/stderr before checking status.

## TCP_NODELAY hypothesis: not a fix

Kestrel [defaults NoDelay to true](https://github.com/dotnet/aspnetcore/blob/main/src/Servers/Kestrel/Transport.Sockets/src/SocketTransportOptions.cs),
whereas this native socket adapter leaves Nagle enabled. `nodelay.py` makes a
separate POSIX-only relink with checked `setsockopt(TCP_NODELAY)` in a generated
socket adapter/header. It changes no checked-in runtime default, managed API or
ownership boundary. The comparison uses identical managed objects. Disabling
Nagle can reduce small-write delay at the cost of more packets; that tradeoff
needs qualification rather than blanket adoption.

The experiment **does not resolve the stalls**. It also fails on the final
four-owner synchronous trial. The newly retained error is specific: HTTP 200,
correct length and complete `Hello, World!` body, with `Connection: close`, followed
by an I/O timeout while waiting for EOF. The server had no diagnostic error before
harness termination. This localizes that failure to finishing the connection,
but does not prove whether managed completion, native socket lifetime or the OS
is responsible. Keep `nodelay-incomplete.json.gz` and do not calculate a successful
cohort median or promote TCP_NODELAY from this evidence.

## Next investigation

Trace successful send completion through `HttpServeOperation.OnSent`/`Cleanup`
to native socket close, and distinguish close-call timing from FIN/EOF delivery.
Capture per-connection timing and OS state on failure before terminating the
server. Preserve the EOF check; accepting a correct body alone would hide the
observed defect. Resolve or characterize this before capacity comparisons with
Kestrel or other servers. Separately, profile the consistent live CPU penalty,
using guarded-sync controls to distinguish mutation/root bookkeeping from marking.
JSON, broader standalone and multi-owner JIT comparisons, full pause distributions,
retention budgets and concurrent-default acceptance remain incomplete.

## Reproduction

```sh
python3 benchmarks/execution/concurrent-http-owners/build.py \
  --runner target/release/deps/neoclr_codegen-12f33f26d2ff4df0 \
  --sample plain --output target/http-plain-new
go build -o target/http-owner-client benchmarks/http-server/primitive-comparison/client.go
python3 benchmarks/execution/concurrent-http-owners/test-client.py \
  --client target/http-owner-client --output target/http-client-tests-new.json
python3 benchmarks/execution/concurrent-http-owners/measure.py \
  --build target/http-plain-new --go-client target/http-owner-client \
  --output target/http-plain-cost-new.json --seconds 3 --rounds 3
python3 benchmarks/execution/concurrent-http-owners/nodelay.py \
  --build target/http-plain-new --output target/http-nodelay-new
```

Use fresh paths. Omit `--go-client` for the Python control. The current Go program requires 32
clients; the Python control supports `--concurrency`. `--owners 4 --modes sync` isolates the recheck.
The historical incomplete reports are intentional; do not discard failures or
compare only favorable rounds. Raw manifests identify exact binaries, sources and
compiler/host commands. Go build on this host used go version go1.26.3 darwin/arm64.
