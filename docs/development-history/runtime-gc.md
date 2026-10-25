# Runtime and GC development record

This is the detailed engineering record behind the concise release changelog.
Ordinary runtime improvements, synchronous GC work and concurrent-GC experiments
are distinct tracks; an experimental result is not a published capability.

- [Complete dated entries through 2026-10-25](unreleased-through-2026-10-25.md)
  preserve the previous Unreleased history verbatim, including failed/rejected
  experiments and the evidence available at each step.
- [Author decisions and development conversations](../development-timeline.md)
  distinguish requests, proposals, actions and unresolved questions.
- [Current platform roadmap](../platform-roadmap.md) records active priorities.
- [Coherent runtime bottleneck investigation](../design/runtime-bottleneck-investigation.md)
  and [allocation/metadata research](../design/allocation-metadata-runtime-comparison.md)
  connect runtime costs with CoreCLR, HotSpot and Go design comparisons.
- [Standard benchmark/profile suite](../../benchmarks/gc-suite/README.md) retains
  fixed workloads, configuration identities and self-comparison procedures.
- [Worker/scheduling investigation](../../benchmarks/gc-suite/concurrency/README.md)
  separates independent owners, shared heaps, fibers and collector threads.
- [Typed scalar-store evidence](../../benchmarks/gc-suite/matrix/scalar-store-follow-up.md)
  and [HTTP qualification](../../benchmarks/gc-suite/matrix/scalar-store-http.md)
  track the current optimization, controls and limitations.
- [Generational concurrent design input](../design/generational-concurrent-gc-direction.md)
  remains a proposal, not an implemented generational or moving collector.

Keep new fine-grained investigations in their existing evidence/design records,
linked here when a new track starts. Update release-facing changelog themes when
behavior or demonstrated outcomes change; do not recreate a per-experiment diary
there. Preserve prior claims and corrections in their dated records.
