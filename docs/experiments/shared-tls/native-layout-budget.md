# Native layouts for managed HTTPS results

Development checkpoint, 2026-10-12. This is compiler/host admission work, not a
new public API or end-to-end HTTPS qualification.

The in-development HTTP error union contains nested ordinary value records. Its
current representation stores a tag plus separate case payload fields; adding TLS
causes the aggregate to exceed 64 flattened lanes. A compiler-generated async
frame that stores the result also exceeds the separate 32-lane object limit.
These are bounded native backend limits, not invalid Raven or native metadata.

Preserve the existing representation and extend the checked layout budget to 128
lanes for inline values, reference payloads and record-array elements. Reference
allocation includes an eight-byte header, giving a 1,032-byte object bound. Keep
function, type, per-method, frame, stack, array-length and total heap budgets
unchanged. Preserve explicit rejection above the new layout bounds and allocation
failure when the invocation heap has insufficient capacity.

The audit covered dynamically sized compiler lane vectors, call-result storage,
field offsets, stack-root descriptors, reference allocation and record-array
allocation/tracing. Object GC scans allocated payload extents; record arrays also
validate lane counts and initialization markers. Update the latter check together
with the allocator so accepted layouts cannot later fail GC descriptor validation.
There is no new pointer bitmap or overlapping union-payload representation.

Relative to .NET/CLR, ordinary nested values remain inline and references retain
identity; these numerical caps are experimental implementation guardrails rather
than language semantics. The eight-byte private lane layout is not a CLI physical
layout or public native ABI. A more compact union representation remains separate
work with metadata, field-addressing and GC implications. The immediate benefit
is admission of ordinary async HTTP results; the cost is allowing larger values,
copies and heap objects, still charged against existing frame/heap budgets.

Rebuild the native compiler and linked host together, including prepared JIT
bundles. Function signatures and storage headers remain unchanged, but older hosts
retain smaller admission checks. This does not qualify mixed compiler/host builds.
No Raven source lowering or native metadata encoding changes are required.

Local macOS ARM64 validation:

- Exact 128-lane inline copy, nested address and call-return behavior agrees with
  the interpreter; cyclic, unknown and oversized values remain rejected.
- A managed reference in lane 65 survives allocation/GC churn, independently of
  an existing borrowed reference. Existing erased-value/fault-root checks pass.
- Record-array kernels pass with and without GC under undefined-behavior/bounds
  sanitizers, including 128-lane elements, final-lane roots, uninitialized elements,
  malformed descriptors, reclamation and buffer canaries.
- Reference allocation passes at 128 lanes, respects canaries, faults with a heap
  one byte too short, and rejects 129 lanes. Twelve selected reference regressions
  pass, alongside the two inline and two root tests and one array-kernel test.

Windows uses the same allocator/collector sources and compiler bounds. Windows
execution remains pending; the local evidence is not a Windows result. Managed
HTTPS execution and any subsequent failures are separate qualification gates.
