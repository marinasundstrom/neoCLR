# Shared scheduling integration decision — 28 October 2026

## Decision at the author's wrap-up

Keep the committed correctness fixes enabled in the normal runtime paths:
DNS/multi-address native wrappers publish GC frames; audited owner-host provider
admission covers DNS, file operations and ordinal Contains. Raven's structural
callback overload fix remains on its neoCLR compiler line. None needs an opt-in
runtime switch. Existing published SDK binaries do not update themselves.

Keep **NEOCLR_SHARED_READY_QUEUE opt-in**. Do not change collector or scheduling
defaults. The native storage primitive remains available to owner/worker services,
but this evidence does not qualify general cross-thread managed HTTP Task.Run or
justify making the shared queue candidate the default. No unproven optimization
is enabled merely to close the work. The author asks to wrap up and enable only
improvements warranted as defaults; this is the resulting bounded decision.

The broad HTTP/worker objective is only partially demonstrated: generated callback
barriers and restricted heap ownership have earlier qualification, and real HTTP
now has repeated both-collector comparisons, but the acceptance plan's full AOT,
capacity, cross-thread ownership and execution-context profiling gates remain
unmet. They are follow-up work, not silently passed gates.

## Repeated untraced HTTP comparison

Three rounds, mode order reversed in the middle round, eight persistent clients,
24 first-use endpoints then 1,920 warm requests/process. All **23,328 requests**
pass body/shared-capture/file-content/shutdown checks. Identical 1 MiB heaps,
1024/16384 allocation/publication budgets and separately built native hosts from
`413f1fadd`. Raven neoCLR compiler `91c1edca8`, matching development libraries.
macOS ARM64, uncontrolled OS scheduling and filesystem cache. No CPU affinity or
production resource isolation is claimed. Test runner and downstream Python peer
share the host. This is a closed-loop fixed workload, not sustainable capacity.

| Queue / GC | Ready median (s) | First-use sweep (s) | Warm req/s median (range) | Median trial p99 (ms) |
| --- | ---: | ---: | ---: | ---: |
| baseline-sync | 5.331 | 2.513 | 1541 (1446–1572) | 44.93 |
| shared-sync | 5.428 | 2.517 | 1623 (1448–1751) | 43.74 |
| baseline-live | 5.602 | 4.571 | 1141 (1087–1146) | 48.45 |
| shared-live | 5.750 | 4.550 | 1055 (1048–1257) | 51.16 |

Ready includes process launch, metadata selection/verification and runtime setup.
First-use sweep is the sum of the first 24 checked request latencies, including
lazy compilation and downstream waits. Warm p99 is the median of three per-trial
p99 values, not a pooled percentile. File payloads are tiny; cold/slow storage is
not represented. Raw trial values and per-family tails are retained in summary.json.

Ratios of throughput medians: shared/sync **+5.3%**, shared/concurrent **−7.5%**.
Paired round changes: sync +13.68%, +0.17%, +3.19%; concurrent −3.65%, −7.89%, +10.21%.
The small sample and overlapping ranges do not establish a broadly reliable win.
Concurrent GC is slower in this workload; that does not settle its potential on
larger heaps, longer-lived objects or other application loads. Existing defaults
remain the defensible choice.

## Separate native profiles

Two shared-queue diagnostic runs use 100 cycles/client (19,200 warm requests each),
with five-second macOS sample captures started after the first-use sweep. Both
pass, **38,448 total checked requests**. Their timings are excluded from the table.
The summary tool explicitly rejects sampled/profile results as benchmark inputs.
The retained sampler output contains many unsymbolized JIT frames, so it cannot
attribute managed parsing/serialization methods or prove their relative cost.

Recognized top-of-stack counts include enter_owned (306 sync / 174 concurrent),
stack_roots (232 / 120), thread-local lookup (294 / 574), poll (202 / 224), and
experiment_gc_store (411 concurrent). File syscalls also appear. Counts come from
different captures and are neither CPU percentages nor normalized comparisons.
Waiting harness/marker threads must not be mistaken for hot executing work.
These observations prioritize root publication, store checks and HTTP/native
boundary attribution over speculative queue-policy or task-relationship redesign.

## Reproduce and inspect

Prepare the mixed hosts without --trace, using the retained build.json recipes.
Then run:

```sh
python3 docs/experiments/shared-scheduling/mixed-http.py --build BUILD --output NEW --clients 8 --cycles 10 --repetitions 3
python3 docs/experiments/shared-scheduling/summarize-mixed.py NEW/results.json --output NEW/summary.json
```

For separate macOS native samples add `--native-sample-seconds 5`, use 100 cycles,
and select `--modes shared-sync shared-live`. Sampling is optional and its output
is labeled native-profile. Gzip files retain raw source bytes; the untraced harness
snapshot predates only the optional native-sampling addition. Recorded hashes in
reports preserve that distinction. Published website deployment remains manual.

## Deferred follow-up

Keep the existing shared scheduler/GC relationship predictable. Before any future
default switch, measure offered-load saturation/backpressure, AOT, cold/slow I/O,
multiple applications, full context retention and HTTP phase attribution. Explore
shared DNS/worker completion notification and root/store-path costs only with
explicit ownership proofs and matched benchmarks. No parent/task relationship
metadata, HTTP-specific scheduler, or automatic blocking offload was added.
