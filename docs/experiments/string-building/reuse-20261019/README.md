# StringBuilder reuse — 2026-10-19

The same 64-iteration consumer appends combining fragments, snapshots, appends
nonempty/empty lines, snapshots again and clears for reuse. It also checks exact
multibyte quota and empty-builder behavior. The permanent consumer is
[Reuse.rvn](../Reuse.rvn), with [project](../Reuse.rvnproj). The measurement used a
temporary Probe project containing the same executable statements; retained build
reports identify source paths, compiler/library hashes and native link commands.

| Native macOS ARM64 counter | Before | After |
| --- | ---: | ---: |
| Managed allocations | 651 | 137 |
| Reclaimed allocations at cleanup | 651 | 137 |
| Collections | 4796 | 3960 |

That is 514 fewer managed allocations (78.96%) for this workload. The diagnostic
host reports the existing GC counters after final collection; it does not modify
guest code. Counts include the complete process workload, not just builder arrays.
Collections use the console host's existing policy and are not throughput results.
No wall-time, total host allocation, HTTP or concurrent-GC improvement is claimed.

AppendLine now stores text and LF separately after quota/capacity preflight.
ToString replaces retained fragments with its immutable snapshot without shrinking
the backing array; Clear replaces used slots with empty text and keeps capacity.
This avoids temporary line text, repeated one-slot arrays and subsequent regrowth.
The tradeoff is retaining the largest allocated reference array until the builder
is collected; previous text references are cleared. Mutable UTF-8 staging, typed
formatting and view ingestion remain separate work.

The reuse consumer passes interpreter/JIT/AOT. The native diagnostic executable
passes with identical output and all tracked allocations reclaimed. API snapshots
are refreshed from the matching five-library rebuild with Raven 2d22055dc.
This development bundle is not a release candidate; String work remains required.

The existing Build, Limit, LineLimit and InvalidLimit consumers also pass in all
three modes (12 executions). Failure checks assert the intended quota/constructor
message as well as exit status, rather than accepting any runtime failure. Together
with Reuse this is 15 passing contract executions. See contracts.json.
