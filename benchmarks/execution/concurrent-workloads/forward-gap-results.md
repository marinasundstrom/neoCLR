# Remaining costs after forwarded stack elision

2026-10-25. Refresh the whole-request profiles with the qualified forwarded-stack
objects before selecting another optimization. No runtime/default behavior changes.

Six final macOS wall-stack profiles pass 49,248 plaintext/JSON responses across
sync, guarded-sync and concurrent modes. All profile builds match their separately
measured forward-elision controls by compiler options and SHA-256 of compiler,
managed objects and direct source inputs. Reported input hashes must still match
current files. The profile summary also checks response success, completed epochs,
clean index/store shutdown and provenance of its input measurement. Four negative
controls reject changed options, changed input hashes, failed responses and missing
measurement pairs; the positive control passes. Original whole-gap output remains
byte-identical with the tool's default options.

The first six profiles also passed 49,248 responses, but retained the previous
profile recipe's direct-seed option. That option is absent from the forward-elision
performance controls. Preserve those initial reports separately; rerun without the
option and use only the matching `forward-profile-v2` cohort below. This is why
build identity is now checked rather than assumed from nearby experiment names.

## Current evidence

Uninstrumented CPU medians reuse the balanced forward-elision measurements; no
profiled timing is used as performance evidence:

| Workload | Sync CPU seconds | Guarded-sync | Concurrent | Guarded share of excess |
| --- | ---: | ---: | ---: | ---: |
| Plaintext | 0.405503 | 0.591754 | 0.693703 | 64.6% |
| JSON | 0.669853 | 0.928042 | 1.085720 | 62.1% |

The share is `(guarded - sync) / (concurrent - sync)`, a descriptive difference of
medians. It is not an isolated barrier cost: generated code, collection schedules
and live sets also differ. Nevertheless, most excess persists without concurrent
marking. Marker-only optimization cannot account for all of it.

| Top-frame observations | Plain sync | Plain guarded | Plain live | JSON sync | JSON guarded | JSON live |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| TLS resolution | 82 | 111 | 91 | 79 | 136 | 76 |
| Scalar store | — | 101 | 66 | — | 95 | 58 |
| Owned frame entry | 93 | 60 | 46 | 74 | 60 | 42 |
| Stack roots | 62 | 53 | 29 | 66 | 43 | 25 |

These are all-thread wall observations, including waits, not CPU percentages.
Dashes mean absent from the reported table, not zero measured cost. Live condition
waits contribute 493/468 observations; do not interpret them as busy CPU. TLS
caller accounting reconciles with zero unattributed observations. Store-associated
TLS remains visible, as do native stack checks and frame entry. Short profiles do
not establish a statistically reliable before/after change in any symbol.

## Next implementation target and integration boundary

After rejected acquisition-borrowing experiments and the smaller gain from removing
many outside-arena guards, investigate the common frame-entry path next. In
`runtime/native/gc/root-probe.c`, `enter_owned` validates every storage descriptor,
then initializes the frame; `stack_roots` already reuses the admitted ancestor proof.
Measure descriptor counts and entry frequency before changing representation or
validation lifetime. Dynamic addresses belong to each activation: an image-level
cache cannot prove them valid. Reusing immutable layout facts must preserve current
slot/root values, native rejection semantics, frame lifetime and owner boundaries.
This builds on the existing CLR/root-map research and changes no .NET API contract.

A separate integration requirement remains: `concurrent-live/generate.py` and
`automatic.py` explicitly reject `NEOCLR_WORKER_GC`. The worker collector contract
in `runtime/native/gc/native-gc.h` requires all participants paused for the whole
collection and does not enumerate foreign TLS resources. The pool's heap gate also
coordinates readers, mutators and compiled participants. Concurrent marking of one
owned heap does not qualify shared-worker collection. Keep the rejection until
participant-root capture, mutation logging, allocation/reclamation ownership and
worker lifecycle are integrated and exercised with crossing references and failures.
Do not treat per-thread state consolidation as permission to transfer owned handles.

No allocation or pause improvement is claimed. Shared-worker, migration, broader
pause and concurrent-default acceptance remain open; synchronous GC remains faster.

## Reproduction

Run `summarize-cpu.py` on both final raw profile reports, then:

```sh
python3 benchmarks/execution/concurrent-workloads/summarize-whole-gap.py \
  --root target/concurrent-workloads \
  --measurement-prefix forward-elision-http \
  --profile-prefix forward-profile-v2 --mode-suffix inline-index \
  --verify-profile-builds --output target/concurrent-workloads/forward-gap-summary.json
```

`forward-profile-v2-commands.json.gz` contains exact invocations. Raw measurement
reports, hashed stacks and summaries share that prefix. Initial mismatched reports
use `forward-profile` without `v2`; they support no matched attribution claim.
`forward-gap-summary.json` records input hashes and control CPU medians;
`forward-gap-negative.json` and its compressed driver record the negative checks.
The prior forward-elision measurement archives supply uninstrumented controls.
Retained source/object paths and the local macOS sampling tool are required.
