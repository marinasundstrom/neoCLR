# String storage and reference identity investigation

**Exploration, 2026-09-24.** String content equality/hash/display is implemented.
This document investigates the next representation change; shared storage and String
reference identity are not implemented runtime capabilities.

## Scenario and current costs

An application puts the same text in a local, record field and array, passes it as
Object, then casts it back. It should preserve the reference while content equality
remains independent of identity. Today `Value::String(String)` is cloned by slot
reads and other value copies. String-to-Object/interface conversion allocates a
managed wrapper; conversion back reads a copied payload. Object ToString copies
text again. ReferenceEquals deliberately rejects String, even through aliases.

The [executable experiment](experiments/string-storage/README.md) measures current
Value clones and a private Rust `Arc<str>` candidate. With 10,000 clones:

| UTF-8 bytes per value | Current allocations | Cumulative requested bytes | Shared-clone allocations/bytes |
| ---: | ---: | ---: | ---: |
| 0 | 0 | 0 | 0 / 0 |
| 32 | 10,000 | 320,000 | 0 / 0 |
| 4,096 | 10,000 | 40,960,000 | 0 / 0 |
| 65,536 | 10,000 | 655,360,000 | 0 / 0 |

Each clone is immediately dropped. These are allocator requests in a Release build,
not peak live memory, a byte-copy instrument, timing results or an end-to-end speedup.
Construction is outside the measured region: Arc requires a control block and
allocation, and converting an existing owned String to Arc<str> may copy its bytes.
Atomic reference counting also has a per-clone/drop cost. Empty owned strings already
avoid a text allocation. The candidate is not integrated with Value or the VM.

The current-VM cast baseline allocates 1,000 wrappers for 1,000 casts of the same
String local. An eight-object limit produces 125 collections, peak eight and zero
live wrappers at completion. This demonstrates conversion allocation separately from
payload clone requests; it does not measure all bytes allocated by the VM.

The prototype's Weak handle stops upgrading after the final strong owner is dropped.
That checks logical ownership release; the observer itself retains the Arc control
block until it is dropped, so this is not a physical deallocation measurement.

## .NET baseline and layers

Primary sources reviewed 2026-09-24:

- [C# reference types](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/reference-types#the-string-type): String is immutable; references can alias the same object, while string equality compares contents.
- [.NET 10 String source](https://github.com/dotnet/runtime/blob/v10.0.0/src/libraries/System.Private.CoreLib/src/System/String.cs): ToString returns the same String instance; storage is UTF-16 and runtime allocation participates in the CLR heap.

The checked .NET 10 comparison preserves identity across assignment, Object conversion,
casts, record fields, arrays, ToString and a forced collection. It separately constructs
equal text with a different identity. It does not assert interned literal behavior.

The intended neoCLR reference behavior should match those cases while retaining
UTF-8 and grapheme APIs. No language syntax or new public API is needed for these
cases. Runtime representation, tracing/ownership and host boundaries must enforce
them; compiler-only adaptation cannot recover identity once text has been copied.
Interning, normalization, nullable String storage and generic math are separate work.

## Alternatives

| Candidate | Benefit | Costs and unresolved points |
| --- | --- | --- |
| Keep owned inline text and wrappers | No migration; current content behavior works | Repeated payload copies; no coherent String identity across conversions |
| Shared immutable leaf payload, initially behind an internal text handle | Cheap clones and one identity for aliases; a leaf cannot create a reference cycle by itself | Refcount/control-block cost; host lifetime differs from tracing GC; accounting and wrapper identity must use the text owner rather than wrapper allocation |
| Put every String in the managed tracing heap | One reference model and heap-owned identities; direct root tracing | Changes literal/native-result creation and all storage paths; new allocation safe points, heap-count behavior and host/worker transfer rules |

A wrapper cache alone is insufficient: current String clones have already lost
origin identity. Content interning would also merge separately created equal strings
and impose a different allocation policy. Neither is selected as an identity repair.

## Provisional direction

Proceed toward one internal immutable text handle and shared payload, while keeping
String's public meaning unchanged. The measured copy costs justify that next
implementation slice, but do not select Arc<str> as the final runtime allocator.
Prototype evidence is limited to clone cost, alias identity, Rust-container ownership
and release after the last owner. It is not proof of guest tracing-GC integration.

The first integration gate should preserve one text owner across ordinary VM copies,
then across Object/interface views and back to String. Identity must follow that
owner, not a conversion wrapper. Do not enable ReferenceEquals until the round-trip
and lifetime matrix passes. Empty strings need a deliberate policy; interning remains
optional and is not required for assignment identity.

Compare Arc<str> with a handle that adopts an owned String buffer before selecting
construction behavior. Keep the payload immutable and free of managed-reference
backlinks so reference counting does not create cycles. Do not expose native addresses
as guest identities or hashes. A public Rust Value payload change is a host API
migration even when guest metadata is unchanged.

## Migration map and validation gates

- **Slots, arrays and fields:** Value clone/slot reads, erased Result/Option payloads,
  class fields and record components must share a text owner. Check independent
  equal text, aliases, empty strings, embedded NUL, combining sequences and emoji.
- **VM conversions and dispatch:** literals, castclass/isinst, interface views,
  Object ToString and casts back must retain ownership. Current wrapper allocation
  safe points must remain correct until wrappers are removed or reused.
- **GC and accounting:** String is currently a leaf in `gc::trace`; text allocations
  are not separate managed objects. A shared-leaf design must release payloads when
  the last host/frame/heap owner goes away. Verify locals, operand stacks, fields,
  arrays, cycles of containing objects, failures and execution teardown. Track text
  bytes separately rather than implying `heap_objects` bounds them.
- **Native and host boundaries:** constructors, concat/slice, UTF-8 decoding, files,
  console and reflection produce text. Debug snapshots and returned Values retain
  host copies today. Decide whether the new host result owns text beyond execution.
- **Workers:** workers currently transfer owned input/result Strings and enforce a
  result/output byte quota. Keep accounting explicit if bytes become shared; do not
  silently treat sharing as quota-free or share mutable worker state.
- **Compiler/library consumers:** keep String operators, typed Equals and Object
  equality consistent; run record and collection samples. Keep content hashes
  independent of identity. No change to UTF-8/grapheme indexing is implied.

Before claiming a performance improvement, measure construction and destruction,
retained bytes, small-string overhead, representative app allocations and worker
boundaries, not just repeated clone allocation counts. Enable guest String identity
only after .NET-comparison cases and the GC/host-lifetime gates pass.
