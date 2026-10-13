# Native GC handle admission

The native collector now validates allocation headers and locates exact payload
bases in one heap walk. Once both requested bases have matched, it validates the
remaining headers without further pointer comparisons. Previously admission
validated the heap and then walked it again for each base. Allocation inspection,
resource registration, host-root creation/replacement, callback admission and owned
suspended-root admission share the helper. Handle reads/releases are unchanged.

This preserves full trailing-corruption rejection, null-value roots, exact-base
requirements, context isolation and failure outputs. It adds no persistent index,
metadata layout, allocation or collector policy. The walk remains linear. Compared
with CLR handle tables, this is a refinement of neoCLR's existing bounded native
host-root contract, not a new managed GCHandle API or constant-time handle admission.
See the [runtime GC design](../../docs/runtime-gc.md).

## Reproduction and scope

```
python3 benchmarks/gc-handles/benchmark.py --baseline cf342fb8c --output NEW_DIRECTORY
```

The controller compiles baseline and current collectors with identical root helpers
and a C workload. One warmup precedes five alternating pairs per case. Heap creation
is excluded; each timed iteration creates, reads and releases one strong or owned
host root. Each process verifies its read value and complete final reclamation.
All heaps use 16-byte payloads. Values are at the start, middle or end of the heap;
controls place both owner and value first, or use a strong root without an owner
(`ownerPosition: -1`). This measures host-root API overhead, not collection time,
managed scheduling, concurrency or HTTP throughput. Other consumers of the shared
helper receive correctness coverage, not separate speedup claims.

`macos-20261013.json` retains all samples, commands and input hashes. Local macOS
ARM64 late-base cases at 256/4,096 objects improve 1.76–2.59×. Early-base controls
are 1.00–1.03× (rounded); 16-object cases are 1.00–1.40×. These are paired medians,
not a universal non-regression guarantee. Earlier local prototypes exposed an
early-base slowdown; the final loop stops membership comparisons once satisfied.

The sanitized native GC/root gate passes all 21 executions. The added regression
covers corruption after an otherwise-valid match across all shared admission paths,
unchanged outputs and registrations on failure, interior/foreign/free pointers,
null roots, successful callback admission and exactly-once resource cleanup.
Platform qualification is pending. Optional `benchmark_gc_handles` dispatch in
`native-network.yml` runs the same matrix and GC/root contracts on macOS/Windows.

The relocated native package test also passes with the selected Xcode 26.2 SDK.
An initial invocation picked the incompatible Command Line Tools 27 SDK; the
explicit `SDKROOT` rerun resolves that local toolchain selection issue.
`http-aot-20261013.json` retains a matched fixed-guest AOT comparison (five pairs
after warmup, 768 verified responses). Plain/JSON medians remain within 0.3% of
baseline. It relinks current native helpers against retained rooted-helper guest
objects; it is not a fresh compiler or JIT qualification, nor an HTTP speedup claim.
