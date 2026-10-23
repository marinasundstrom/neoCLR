# Rejected marker-prefix shortcut — 2026-10-23

A fresh profile after free-space indexing points to root publication, TLS and string
validation; see [the capture](../stack-bounds/README.md). Test checking the generated
`requiredSpillLanes` prefix directly before consulting the general marker-search hint.
The candidate still checks string length and current marker bytes, with the original
search fallback for noncanonical or mutated plans. No immutable-metadata contract is
inferred merely from a pointer or compiler output.

**Reverted.** Eight checked plaintext trials do not justify integration. One-owner
baseline throughput is 2,230.9/2,242.5 requests/s, candidate 2,143.7/2,131.7; CPU/response
rises from 447.9/442.4 to 463.8/453.4 µs. Four-owner baseline is 4,595.8/6,553.5 versus
candidate 6,609.2/6,601.3, with CPU 595.6/543.5 versus 556.4/551.7 µs. The first
four-owner baseline is visibly disturbed (p99 33 ms); retain it, but do not credit
its recovery to the candidate. Both one-owner trials regress by roughly 4–5%.

The same application object, GC budgets, isolated owners, native flags and context
reuse apply in both variants; only root-probe.c differs. Each trial has two seconds
warmup, ten measured and 32 connections, forward/reverse at one/four owners. CPU per
response includes startup/shutdown and warmup/drain responses, minus client CPU.
No builds or test suites run during timed trials. These are desktop measurements.

`rejected.patch.gz` (decompress before applying) preserves the candidate and `measurement.json.gz` its full evidence.
Apply the patch to the recorded baseline before running `measure.py` with the same
`--build`, `--object`, `--client` and new `--output` arguments as the free-index runner.
The candidate has wire-level execution evidence, not a completed GC qualification.
The restored runtime passes optimized/sanitized mutable-plan checks, including a new
non-prefix hint invalidation case retained in the maintained suite. See the linked
stack-bounds validation report. No marker shortcut is shipped or selected.
