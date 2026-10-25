# Standard suite baseline — 2026-10-25

Apple M1, 8 logical CPUs, 16 GiB RAM, macOS ARM64. This is the first retained
standard-length benchmark baseline for future **same-collector** comparisons.
Current results are not evidence of a general concurrent advantage. Sync remains
current default; concurrent remains the intended general-purpose default.

The successful full run contains 82,240 checked HTTP responses and 60 standalone
invocations (five rounds per collector/sample). The separate diagnostic run has
12,480 HTTP responses and 36 standalone invocations. All outputs, allocation equality
for standalone, quiescent epochs and index cleanup pass. Profile histograms and
begin/finish/collection summaries pass their validators. Native compiler options
preserve the previously qualified immutable-byref and direct/forwarded stack proofs.

| Non-HTTP sample | Sync CPU ms | Concurrent CPU ms | Managed allocations |
| --- | ---: | ---: | ---: |
| Compute | 62.072 | 62.110 | 13 |
| Reusable buffer | 17.430 | 104.809 | 14 |
| Steady | 96.161 | 161.898 | 1,048,589 |
| Bursts | 53.496 | 85.913 | 524,301 |
| Retained window | 218.273 | 346.483 | 1,049,615 |
| Mixed phases | 107.662 | 166.866 | 1,048,589 |

These are per-invocation medians, excluding process startup and compilation. Work
counts differ across samples; compare within a row. Short-lived object-heavy cases
still cost substantially more under the concurrent candidate, as does the retained
window. Compute is effectively flat. The reusable-buffer outlier allocates only
14 objects and completes no concurrent epoch.

One separate reusable-buffer diagnostic run records 8,388,671 managed stores,
8,388,658 cached store lookups and 13 slow lookups, versus begin CPU 31,291 ns and
joined marker CPU 1,042 ns. This localizes the next investigation toward repeated
runtime store admission rather than assuming allocation rate or marker CPU alone
explains the gap. Inclusive/nested phase costs must not be added twice.

The HTTP pair gives plain sync/concurrent 5,139/4,076 requests/s and JSON
3,130/2,730 requests/s. CPU totals, allocations, memory peaks, per-round variation,
commands and input hashes are in the raw reports. The collectors use independent
policies documented in the [procedure](../README.md). Earlier improvements against
older concurrent settings are a separate comparison, not a claim to beat sync.

`benchmark-run.json` is the normalized benchmark index. `profile-run.json` identifies
the diagnostic index; its hardware was recorded by the surrounding same-host run,
before the runner's automatic hardware-field addition, and it is not a benchmark
comparison input. Child report hashes refer to the original uncompressed bytes;
the corresponding gzip archives preserve them. Absolute target paths in raw commands
identify the original environment, not portable installed dependencies.

`pilot-*` preserves the successful initial short-loop experiments. Those lengths
are superseded and must not be mixed with the standard cases. `samples.json` pins
the final workload definition. The standard [comparison tool](../compare.py)
rejects changed definitions, diagnostic timing and hardware/work mismatches; it
compares each collector with itself and reports policy changes explicitly.

The [website results](https://github.com/marinasundstrom/neoCLR/blob/main/website/content/benchmarks/gc/index.md)
and chart are generated from `benchmark-run.json` by `site-results.py`, not manually
transcribed. The tests cover same-collector matching and invalid comparison inputs.
