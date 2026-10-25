# macOS ARM64 JIT code publication — 2026-10-25

Investigation of the intermittent stack-transfer test exposed a code-publication
problem shared by ordinary and concurrent JIT execution. It is not evidence of a
GC marking or stack-address proof error. Synchronous GC remains the default.

## Evidence and diagnosis

The preserved pre-integration runner passes only 6/10 unoptimized and 7/10
speed-optimized isolated repetitions. Failures include incorrect statuses and
process signals. Diagnostics identify different transfer cases across failures;
IR and emitted assembly were retained locally. Changing diagnostic logging and
code lifetimes can suppress reproduction, so those passing runs alone are not a fix.
The local LLDB itself crashes during target creation; crash records were used to
identify execution within generated code instead. Personal crash-report metadata
is not included in the repository evidence.

The pinned `wasmtime-jit-icache-coherence` 34.0.2 Unix implementation has a no-op
`clear_cache` on macOS ARM64 and explicitly relies on `mprotect`. Cranelift JIT
0.121.2 calls that helper when finalizing memory. A matched test-only explicit
invalidation immediately before execution changes 83/100 passes to 100/100.
The integrated memory-provider comparison changes 80/100 passes with invalidation
disabled to 100/100 with it enabled. Each comparison alternates control/candidate
processes; these are correctness trials, not throughput measurements.

The first comparison can be reproduced by applying `isolated-probe.patch` to
`f27b8f9c8`, rebuilding the release test runner, and running `compare-flush.py`
from the repository root (create its `target/gc-stack-transfer-investigation`
output directory first). `validate.py` exercises the integrated provider and
its test-only negative control.

## Platform contract and implementation

[Apple's cache API documentation](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/sys_icache_invalidate.3.html)
requires preparing newly written instructions before execution. The corresponding
.NET implementation layer is code publication rather than garbage collection:
[CoreCLR 10 PAL context.cpp](https://github.com/dotnet/runtime/blob/v10.0.0/src/coreclr/pal/src/thread/context.cpp)
implements processor-specific instruction-cache maintenance in
`DBG_FlushInstructionCache`, including the compiler cache-clear builtin on macOS.
This fix follows the platform requirement, without changing managed contracts.

The private `jit_memory` factory installs a memory-provider adapter on macOS ARM64.
It records executable allocations and invalidates those ranges after relocation,
before delegating final page protection/publication. It clears the pending ranges
only after successful finalization, and delegates allocation, protection and
reclamation to Cranelift. Data allocations are not treated as instructions.
All neoCLR codegen JIT constructors, including qualification consumers, use this
factory. Other platforms retain the existing provider.

Cost is one pending-range entry per compiled code allocation and cache maintenance
at publication, with no per-request or per-store check. This does not introduce
MAP_JIT, a new executable allocator, code sharing or hardened-runtime support.
[Upstream issue 12076](https://github.com/bytecodealliance/wasmtime/issues/12076)
has superficially similar symptoms but its MAP_JIT explanation is disputed in the
maintainer replies; that report was not treated as proof of this diagnosis.

The test-only `NEOCLR_TEST_SKIP_JIT_ICACHE` switch provides a negative control;
production binaries never read it. Repeated compilation/execution/freeing tests
retain normal code lifetimes, including native calls, fault exits, differing stack
frames and reused executable addresses. No bypass or retry masks failures.

## Validation scope

Final release-runner checks pass all 198 nonignored codegen tests (34 tests require
separate native hosts). The expanded primitive consumer passes its 14 AOT/JIT
commands. Forty repeated publication runs across `none` and `speed` optimization
exercise 7,680 module lifetimes and 30,720 generated calls without failure; disabling
invalidation reproduces a failure in one of 20 matching stress runs. All six
standalone workloads pass 36 JIT invocations with exact outputs and expected
allocation work. These runs overlapped other validation: their timing fields are
not used as performance evidence. Fresh plain/JSON pipelined HTTP passes all
four sync/concurrent JIT cells (320 responses). Production `cargo check` passes
with the negative-control branch compiled out.

Compressed records retain baseline, matched probes and final checks. The full
collector-default goal remains open: this fixes a prerequisite for reliable JIT
qualification, not an HTTP throughput or concurrent-over-sync performance result.
Windows execution and release packages require their normal target-specific gates.
