# Selectable memory management

Author direction, 2026-10-18. The immediate implementation is a small named GC
selector. Broader provider composition is a future direction, not a prerequisite
for the concurrent-GC proof of concept.

## Implemented scope

`GarbageCollector` and `--gc mark-sweep` select the existing collector before heap
creation. It is the sole included/default choice. Interpreter workers inherit the
choice; JIT forwards explicit selection to the native backend. Standalone AOT fixes
selection when compiling the image. Unsupported names and incompatible options fail
explicitly. There is no active-heap switching or dynamic provider loader.
See [selection semantics](../runtime-gc.md#explicit-collector-selection-development).

This preserves current managed GC semantics. Like .NET GC configuration, selection
must agree with the execution environment; it does not change managed object APIs.
The author wants the eventual option to include separate implementations, including
non-GC memory managers, without bundling all of them. That broader flexibility is
not implemented by this selector.

## Keep the experiment small

The author subsequently emphasized practical solutions over unnecessary complexity
and explicitly described this work as a proof of concept. Keep the concurrent
candidate isolated, demonstrate useful overlap and test its lifetime behavior against
the synchronous collector. Measure pause time, total CPU and memory before deciding
whether integration is worthwhile. Do not build a general provider framework first.

A future implementation may need different allocation, roots or mutation barriers.
Reject incompatible generated code rather than assuming a selection flag can supply
missing barriers. Preserve dependent roots and native resource cleanup even in an
experiment intended to run applications. Other strategies may have different service
capabilities; those differences need explicit contracts when they are implemented.

[MMTk's VM binding architecture](https://www.mmtk.io/code) offers a useful comparison
for separating runtime integration from collection algorithms. It is an input for
future work, not a decision to adopt its framework. The current
[concurrent investigation](concurrent-gc-evaluation.md) can proceed without it.
