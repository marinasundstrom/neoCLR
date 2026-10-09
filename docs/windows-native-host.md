# Windows native console host prerequisites

Development experiment, 2026-10-09. The Windows scalar/literal-console backend is
qualified; managed Windows code generation remains rejected. This document records
host requirements before widening that profile. It does not introduce a public API.

## Memory ownership: qualified private probe

The existing macOS console host owns a zeroed 1 MiB buffer for one synchronous
entry, then collects and frees it. The independent Windows
[heap adapter](../tools/native/windows-host-memory.c) reserves an address range,
commits the middle payload read/write and leaves a reserved page at each end.
It accepts logical capacities from 1 byte through 1 MiB, rounds commitment to whole
pages, and exposes the exact logical capacity separately. It rejects zero,
oversize and replacement of an already-owned heap without modifying the record.
Release clears ownership only after successful OS release; releasing empty storage
is harmless. The owner must be zero-initialized, must not be copied or mutated,
and must be used without concurrent access. This is a private trusted-host contract.

.NET 10's [NativeMemory.AllocZeroed](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.interopservices.nativememory.alloczeroed?view=net-10.0)
is the unmanaged zeroed-allocation baseline, not the managed GC. Reusing `calloc`
would match today's console host with less code. This experiment instead uses
[VirtualAlloc](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualalloc)
for explicit reservation/commit boundaries and hardware-inaccessible perimeter
pages. New committed memory is zeroed. [VirtualFree](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualfree)
releases the original reservation with size zero and `MEM_RELEASE`. Primary
sources reviewed 2026-10-09; .NET API view is 10.0, Win32 contracts are documented
platform APIs rather than a pinned runtime implementation.

The benefit is an observable host allocation boundary; costs are page rounding,
extra reserved address space, OS calls and Windows-specific code. No speed claim
is made. Guards protect the committed region, not individual objects or the
logical end inside a rounded page. They do not replace GC bounds checks, root
tracking, allocation descriptors or guest memory-fault handling. Unlike the .NET
zero-size allocation API, this private bounded adapter rejects zero. Allocation
failure is a host setup failure, not yet a mapped guest fault.

The focused [Windows workflow](../.github/workflows/windows-native-host.yml) builds
with MSVC `/W4 /WX /O2 /std:c11 /MT`. Its C consumer tests 12 allocation/release
lifecycles at one byte, page size, page size plus one and 1 MiB, including zeroing,
alignment, page states, real access violations at both guard boundaries, overwrite
rejection, release and repeated empty release. It also rejects zero, oversize,
SIZE_MAX and null owners. Test-only SEH observes access violations; it establishes
no generated-code SEH/unwind contract. OS allocation failure injection remains open. Collector integration is tracked
separately below. Reports retain the source revision, input hashes, native
binary, build/execution logs and output hashes.

**Windows evidence:** [run 37952512363](https://github.com/marinasundstrom/neoCLR/actions/runs/37952512363)
at `9c3c586e1e4277be5068a4f45fa5033807c92c18` passes all 12 lifecycles on Windows
Server 2022 x64 with warnings treated as errors. All nine downloaded artifact hashes
match; the three source hashes match the recorded Git revision with Windows checkout
CRLF line endings. See the [retained report](windows-host-memory-validation.json).
That run qualifies allocation and perimeter protection only. The subsequent
collector evidence is below; neither run qualifies guest memory faults or native
stack safety.

## Collector integration: working Windows consumer

The private [collector host](../tools/native/windows-gc-host.c) now owns the guarded
heap and existing `neoclr_aot_context`. The creating thread must remain alive and
all raw collector operations remain thread-affine. Close checks the creating thread
and `neoclr_gc_entry_check_v1` before releasing storage, so registered host roots or
published guest frames prevent teardown. Render fault diagnostics before close.
Unregistered native pointers remain the trusted caller's responsibility. This does
not enforce ownership for future suspended activations or allow carrier migration.

Reuse the existing [nonmoving GC comparison and contracts](experiments/aot-console/native-gc.md)
rather than introduce a separate Windows collector. Relative to .NET GC roots,
this remains a bounded explicit root protocol, with conservative object scanning
and stable addresses; it does not offer the CLR's general managed execution model.
The new wrapper adds lifecycle enforcement at host teardown, not a new collector
algorithm. Its thread-affinity cost is explicit and provisional for scheduler work.

The Windows build enables MSVC's `/experimental:c11atomics` for the existing
monotonic host-handle allocator. Microsoft's [C11 support announcement](https://devblogs.microsoft.com/visualstudio/visual-studio-2022-17-5-released/)
describes the opt-in lock-free implementation (reviewed 2026-10-09). Keep that
compiler requirement visible; validation must exercise handles on two native
threads without reusing IDs. No atomicity is inferred for the context itself.

The executable acceptance allocates a cyclic graph plus garbage, keeps the graph
alive through a host handle, transfers ownership to a published frame, and
reclaims it after the frame leaves. It rejects premature/foreign-thread close,
checks TLS handle isolation, fills the heap to exhaustion with unchanged output
on failure, and collects/releases all storage. The existing C collector contract
consumer also runs on Windows, covering interior/fault roots, initialized array
slots, reuse and malformed-descriptor recovery. MSVC C4200 suppression is scoped
to the C flexible-array text descriptor; static assertions preserve its 8-byte
header and byte offset.

**Windows execution passed (2026-10-09):**
[run 37953377713](https://github.com/marinasundstrom/neoCLR/actions/runs/37953377713)
at `8ac8040b967fc6f70bd793d37bfdd043e7a98d5e` builds and runs `host-collector.exe`
with `/W4 /WX /O2`. It prints:

```text
Windows collector: rooted graph, frame handoff, thread isolation, exhaustion and cleanup passed
```

The existing `collector-contract.exe` also exits zero, and all 12 guarded-heap
lifecycles still pass. All 24 downloaded artifact hashes and 13 input hashes match
the recorded revision (with Windows checkout CRLF where applicable). The
[retained report](windows-collector-validation.json) records commands, outcomes,
source identities and the earlier strict-build failure. Two focused macOS collector
and host-root tests pass after the shared-header compatibility change. No collector
algorithm or Raven compiler changes were needed. These are working native C host
consumers; managed Windows code generation remains separately gated.

## Stack protection: bounded Windows host implementation

.NET's [EnsureSufficientExecutionStack](https://learn.microsoft.com/en-us/dotnet/api/system.runtime.compilerservices.runtimehelpers.ensuresufficientexecutionstack?view=net-10.0)
checks whether sufficient stack remains and can throw InsufficientExecutionStackException.
neoCLR's existing native experiment instead uses a status result, a 64 KiB maximum
machine frame and a 256 KiB remaining-stack reserve for matched adapters and fault
return. Those macOS measurements do not qualify Windows.

The private [Windows stack helper](../tools/native/windows-native-stack.c) uses
[GetCurrentThreadStackLimits](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getcurrentthreadstacklimits)
to validate the current native stack region and
[VirtualQuery](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualquery)
to obtain its allocation base. It reads the address of the current return slot via
[_AddressOfReturnAddress](https://learn.microsoft.com/en-us/cpp/intrinsics/addressofreturnaddress?view=msvc-170),
not its contents. That address is just below the caller's position after the helper
returns. Invalid bounds, unexpected memory state or a fiber return status 9.
Bounds are refreshed on every call, with no TLS cache or activation ownership claim.

Windows [thread stacks](https://learn.microsoft.com/en-us/windows/win32/procthread/thread-stack-size)
commit pages as needed within a reservation. Checking only currently committed
space would reject normal fresh stacks; this helper budgets from the allocation
base and leaves 256 KiB plus one bottom guard-page margin. This tests address-space
headroom, not whether future commitment will succeed under OS memory pressure.
The helper itself requires a usable host stack. Arbitrary native reentry, manually
switched stacks, modified stack guarantees and foreign code are outside its scope.
Primary sources reviewed 2026-10-09; API behavior is not a pinned implementation claim.

Compared with .NET's managed sufficient-stack check, this private helper returns a
status and depends on a bounded caller/adapter contract. Reusing the existing
status and reserve avoids inventing a second guest fault; OS queries add overhead
and conservatively retain stack capacity. A cached threshold is deferred until
ownership and measurements justify it. Neither this helper nor a depth count
alone proves arbitrary generated frames safe.

The Windows C consumer creates 128 KiB, 512 KiB and 1 MiB reserved worker stacks.
The small stack must reject entry without changing output. The others descend
through non-elided 16 KiB buffers, publish GC frames, stop with status 9, collect
while a root is live, and return normally with every frame removed. It verifies
headroom again and reclaims the object. A converted fiber must be rejected and
normal-thread checks must work after conversion back. The build retains compiler
stack probing; it does not recover from an OS stack overflow. The independent
probe passes [run 37954770377](https://github.com/marinasundstrom/neoCLR/actions/runs/37954770377)
at `db2fd66b`, stopping at depths 15 and 46. The collector host now calls this
check before allocating: small stacks and fibers return status 9 with the owner
record unchanged. Integrated admission execution is pending.

Windows managed code generation still needs final-frame and outgoing-call bounds,
Cranelift page-probing checks, guard placement and actual generated-code fault
return tests. MSVC C-frame success does not prove those properties or close the
x64 unwind/SEH/native stack-walking gap. See the existing
[macOS stack experiment](experiments/aot-console/native-stack.md). Windows managed
profiles remain rejected until that separate integration is qualified.

## Suspension and scheduling ownership

Memory is owned by a host execution scope, not OS thread identity. The record has
no TLS dependency, but neither this adapter nor the existing GC roots implement
activation migration. The eventual host must retain the heap, roots and code
generation while any activation or callback is live, including while suspended.
Shutdown/cancellation must first settle those owners and only then release memory.
The raw allocator does not dynamically check this release precondition. The
collector wrapper now rejects teardown while registered handles or published
frames are live and from foreign threads. It does not account for unregistered
suspended work; that requires explicit activation ownership before migration.

Do not cache thread stack bounds as an activation's permanent stack contract.
Stackful activations need explicit stack ownership and bounds; stackless activations
need rooted continuation state. Either choice must preserve borrowed references,
first-fault propagation and exactly-once cleanup. Reuse the
[suspension validation scenario](runtime-scheduling-design.md#co-evolution-with-native-foundations--2026-10-09)
before adding migration or green threads. Current root TLS, callbacks and stack
adapters remain replacement boundaries, not public scheduling policy.

Next: execute the Windows host stack consumer, then qualify generated-code stack
probing and managed lowering against the working collector host. Keep Windows managed profiles and project kits rejected until
those separate requirements have executable evidence.
