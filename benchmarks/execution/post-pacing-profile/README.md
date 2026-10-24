# Post-pacing worker profile — 2026-10-24

Capture the qualified four-owner candidate from `cb6bccb27` after the publication
budget change, separately from uninstrumented capacity. The exact managed object
is rebuilt and byte-compared by the existing stack analyzer before symbol mapping.
Raw build, capture, commands, sample and mapped summary are retained here.
`profile.py --directory target/publication-budget` reproduces the capture from the
prior capacity artifacts. This is macOS ARM64 wall-clock sampling under load,
including blocked threads; percentages are **not CPU shares** or throughput gains.

Of 10,696 worker self samples, the leading runtime entries are:

| Symbol | Samples |
|---|---:|
| TLS address resolver | 1,476 |
| Owned frame entry | 1,063 |
| Stack-root publication | 636 |
| Indexed collection | 612 |
| Exact-base validation | 532 |
| GC frame leave | 331 |
| sigprocmask | 314 |
| Allocation | 270 |

The coordinator spends all 2,674 samples waiting on a condition variable. This
cohort does not support a main-thread serialization explanation for the residual
cost. Earlier profiles had roughly 1,739/10,940 worker samples in collection;
the lower post-pacing prominence is consistent with the independent trigger counts,
but it is not a per-helper speedup measurement.

Keep idle collection for now: scheduler acquisition invokes it on IDLE/PENDING
before waiting or ending its drain, where it can reclaim abandoned native resources
and parked stacks. Suppressing it requires a separate cleanup/progress contract.
Investigate remaining frame/TLS work and address-proof misses. The bounded
[cache experiment](../base-cache-size/README.md) measures one of those costs;
image admission, precise tracing, metadata graphs and generational GC remain open.
