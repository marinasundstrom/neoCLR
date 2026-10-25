# Initial worker scheduling profile — 2026-10-25

Twelve diagnostic runs (two repetitions, one/four independent owners, sync/guarded/live)
pass 724,344 checked plaintext responses. Each uses a one-second warmup, ten-second
load and five-second macOS wall-stack sample. These are older qualified owner
hosts/policies, connection-close traffic and a Go client; not the new persistent
HTTP matrix or current tuned concurrent configuration. Timings include profiling
overhead and are not capacity evidence.

| Mode | Owners | Median CPU us/response | Median OS involuntary switches/1,000 responses |
| --- | ---: | ---: | ---: |
| sync | 1 | 199.7 | 853.1 |
| sync | 4 | 258.6 | 1723.2 |
| guarded | 1 | 342.3 | 589.0 |
| guarded | 4 | 423.1 | 2954.7 |
| live | 1 | 390.8 | 2978.5 |
| live | 4 | 501.3 | 4847.8 |

Every configuration costs more CPU per response with four owners in this cohort.
Guarded sync retains much of the excess CPU over ordinary sync even without
concurrent marking. This supports investigating runtime admission/coordination
alongside the collector; it does not establish a precise causal decomposition.

Voluntary OS counters report only 0–3 per invocation despite observed waits, so
they cannot establish an absence of blocking. Involuntary counters are OS lifetime
statistics, not managed task switches or direct scheduling-delay measurements.
Normalize by checked responses before comparing runs; more completed work alone
can produce more total switches. Wall-stack sample observations include blocked
threads, especially the supervising main thread; do not sum them as CPU costs.

Archives: `guarded-profile.json.gz` and twelve `*.sample.txt.gz` reports. Source
digests and exact executable paths are retained. Tuned owner profiles are running;
shared-heap wait/queue attribution and higher-count qualification remain open.
