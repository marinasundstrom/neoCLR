# Native close and client EOF investigation

2026-10-25. Diagnostic follow-up to the [plaintext comparison](measurement-results.md).
No runtime defaults, GC algorithms, managed APIs or ownership boundaries change.
The goal is to separate transport delays from collector costs before comparing
HTTP capacity. The live one-owner CPU penalty remains an independent target.

## Corrected intervention

The original NoDelay experiment was inactive. `socket-listener.h` included the
original `socket-os.h` first, so its include guard suppressed the generated header.
The first close-trace attempt exposed this mistake by producing no trace files.
Do not use that original NoDelay run as evidence for or against Nagle effects.
Both generators now include the generated header first and require a marker
inside its guard. Two negative compilations remove that include and fail with
`Experimental socket header inactive`.

The corrected NoDelay cohort checks 288,459 responses, using the same managed
objects as the control. Four-owner sync throughput is 13,228, 5,037 and 6,356
requests/s across its three rounds; live is 7,584, 7,483 and 6,952. This active
intervention still has stalls; it does not justify a default change. The earlier
Kestrel NoDelay comparison remains motivation, not evidence of comparable capacity.

## Native close correlation

`close-trace.py` relinks POSIX adapters with per-thread send/close logs. Successful
sends record endpoints, descriptor and byte count; close records entry and return.
A complete cohort matches **367,775 responses** to successful closes, with a maximum
last-send-to-close-return interval of **15.718 ms**. No sent connection is left
unmatched. Trace syscalls and endpoint lookups add overhead: these are correctness
diagnostics, not throughput measurements. Each thread keeps one process-lifetime
trace descriptor; a two-million-event bound aborts rather than silently truncating.

A subsequent cohort reproduces the failure in live/four-owner repeat 2:

| Exact connection event | Observation |
| --- | ---: |
| Server/client loopback ports | 59121 / 65514 |
| Server PID / descriptor | 52827 / 14 |
| Successful final send | 112 bytes (headers and body) |
| Last send to close entry | 33 us |
| Close duration | 11 us, return 0 |
| Client wait after complete expected body | 9.998368 s, timeout |

The endpoint-specific raw trace confirms prompt native close for this failure.
It does not prove FIN delivery, establish a kernel defect, or exclude other causes
in other trials. `lsof` subsequently shows only listeners. That snapshot and the
one-second stack sample occur **after the client has timed out and exited**; they
are not simultaneous observations of the wait. The sample includes scheduler
polling and collection work and is not a profile of loaded request throughput.
The failed cohort is retained as failed, without a successful cohort average.

## Validation and retained evidence

The Go client reads exactly the expected body, then requires EOF with no extra
byte; it now reports aggregate/max EOF wait separately. Tests cover legacy/custom
bodies, wrong and oversized bodies, and delayed EOF. The summary checker accepts a
valid trace and rejects inactive tracing, missing closes, close errors and count
mismatches. The two include-order negative compilations pass their expected failure.

[close-evidence](close-evidence/) contains compressed build manifests, completed and
failed reports, correlation, failure stack sample, validation reports and all four
raw trace files from PID 52827. The complete cohort's large raw traces remain local
under `target/http-close-trace-active`; their SHA-256 hashes and per-trial summaries
are archived. No missing raw trace is presented as independently replayable evidence.

## Reproduction and next work

```sh
python3 benchmarks/execution/concurrent-http-owners/close-trace.py \
  --build target/concurrent-http-owners-plain --output target/close-new
NEOCLR_CLOSE_TRACE="$PWD/target/close-new/events" \
  python3 benchmarks/execution/concurrent-http-owners/measure.py \
  --build target/close-new --go-client target/http-close-client \
  --seconds 3 --rounds 3 --capture-failure --output target/close-new/check.json
python3 benchmarks/execution/concurrent-http-owners/summarize-close.py \
  --trace target/close-new --output target/close-new/summary.json
```

Build the Go client from the current source first. Use fresh output paths. For a
failed cohort, use `analyze-close-failure.py` instead of the complete-cohort summary.
Failure capture uses macOS `sample` and `lsof`; unavailable diagnostics are recorded.
The correlation tool deliberately rejects ambiguous endpoint reuse.

Next profile loaded one-owner sync, guarded-sync and live execution while keeping
transport failure analysis separate. Compare plaintext and JSON with standalone
task, collection and array workloads using existing profiling tools; preserve
allocation counts, native retained memory and CPU alongside throughput. Broader
shared-heap/JIT HTTP, pause qualification and concurrent-default acceptance remain
open. This investigation makes no new performance-improvement claim.
