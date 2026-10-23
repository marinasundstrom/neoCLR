# Post-index profile and rejected stack-bound grouping — 2026-10-23

**The candidate is reverted.** Keep the free-index runtime at `1832f77cf`; neither
this experiment nor the [marker-prefix shortcut](../marker-prefix/README.md) establishes
a further end-to-end gain. Retain the evidence and strengthened regression tests.

## Fresh profile

The source-matched four-owner capture validates 41,308 plaintext responses. Attribution
rebuilds the generated object and requires byte identity before resolving method names.
Among 10,496 worker self samples, root publication accounts for 17.58%, TLS lookup
16.74%, collection 10.13%, owned frame entry 5.79% and strlen 5.75%. Allocation itself
now accounts for 1.91%, corroborating the preceding free-index improvement. These are
wall-clock stack samples, including blocked time, not measured CPU shares. Coordinator
waits are separated. Capture rates are not capacity results.

Of the TLS self samples, 451 have the paced boundary as immediate caller, 338 the
stack-budget check, 296 transient publication and 165 root-head lookup. This suggests
reducing crossings and repeated work rather than inferring that another worker pool
will remove them. The existing [CLR/Go/Java research](../../../../docs/design/allocation-metadata-runtime-comparison.md)
remains applicable; no new threading or GC contract is selected.

## Candidate and validation

Group the two private macOS retained-stack bounds into one thread-local structure.
Preserve every range/reserve check, OS-stack fallback and explicit fiber-switch update;
no OS bounds are cached. The matching benchmark SDK's optimized ARM64 assembly has
one TLS address-resolution site instead of two in each setter/checker (two instead of
four across the file), with the same 16 bytes of state. This static reduction alone
does not establish an HTTP benefit. Windows source remains unchanged.

Extend the existing stack fixture to exercise explicit retained bounds through
recursive limit detection/unwind/reset. Main-thread invalid bounds remain installed
while separate workers validate OS and retained bounds, proving isolation. Add a
non-prefix mutable-plan test that warms the general marker hint, changes its contents,
and must abort on the next publication; retain it in the maintained native-GC runner.
Optimized and UBSan/bounds builds pass on both candidate and restored source: stack
checks plus probe/GC plan-length, short, moved, missing and hint-invalidation cases
(28 recorded commands in each report). No full async/fiber qualification is claimed
for the rejected candidate. Existing async qualification remains unchanged evidence
for the restored runtime; no production runtime changes remain.

## Paired results

Eight uninstrumented trials use the same generated object, flags, context reuse,
owned heaps, pacing and ten-byte EOF response. Only native-stack.c differs. One/four
owners run forward/reverse, with two seconds warmup, ten measured and 32 connections.
CPU per response includes startup/shutdown and warmup/window/drain completions, minus
client CPU. Builds and validation do not overlap measurements.

| Trial | Requests/s | CPU µs/response | p99 ms |
|---|---:|---:|---:|
| baseline-w1 | 2,266.5 | 440.8 | 19.43 |
| baseline-w4 | 6,898.0 | 534.9 | 7.00 |
| candidate-w1 | 2,313.9 | 431.3 | 18.41 |
| candidate-w4 | 5,762.0 | 560.3 | 19.39 |
| candidate-w4 | 6,778.9 | 539.9 | 6.60 |
| candidate-w1 | 2,170.7 | 444.7 | 30.23 |
| baseline-w4 | 6,652.6 | 547.2 | 7.06 |
| baseline-w1 | 2,235.9 | 443.2 | 23.79 |

One-owner results overlap, with essentially unchanged mean CPU cost. Four-owner
candidate results are lower than baseline and have a disturbed first trial; there
is no supported throughput/CPU improvement. Keep all trials and revert the candidate.
These small changes do not replace investigation of compiler-proven unnecessary
publications or typed image-owned metadata. Both require explicit lifetime/effect
contracts and forced-GC/worker validation; no pointer-based validation cache is adopted.
ThinLTO build qualification remains a separate previously measured option.

## Reproduction and evidence

`profile-build.json.gz`, `capture.json.gz`, `sample.txt.gz` and `summary.json.gz`
retain the profile and method mapping provenance. Use the existing stack-profile
capture/analyze runners with the recorded inputs and a new output directory.
`measurement.json.gz` preserves all capacity trials. `rejected.patch.gz` (decompress before applying) recreates the
candidate on the recorded baseline; `measure.py` accepts `--build`, `--object`,
`--client` and `--output` as in the free-index runner. Apply the patch before measuring.
`candidate-validation.json.gz` and `restored-validation.json.gz` record focused checks;
run `validate.py --output target/new-stack-bounds-validation` to repeat. The assembly
report and both assembly files preserve matching-SDK commands and hashes.
Use the recorded Xcode SDK environment. Reports are local macOS ARM64 evidence,
not Windows qualification, Kestrel parity or race-detector coverage.
