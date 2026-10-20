# Persistent memory region Object contracts — 2026-10-20

Memory and ReadOnlyMemory Object equality now compares backing owner identity,
offset and length, independently of mutable contents and access restriction. Equal
regions have matching process-local hashes. Default ownerless memory hashes to zero;
an empty window retaining an owner is distinct from default and from other offsets.
Mutable/readonly equality does not change exact type tests or unboxing.

This follows the [.NET Memory contract](https://github.com/dotnet/runtime/blob/main/src/libraries/System.Private.CoreLib/src/System/Memory.cs):
Object equality accepts either writable or readonly memory of the same element type,
while hashing uses owner identity and the region, not content equality. Borrowed views
remain ineligible for boxing. These are semantic properties of the view representation,
not new ordinary-struct defaults or content-comparison operators.

Implementation reuses existing box allocation, checked query/unboxing, view payload
storage and GC rooting. The runtime excludes intrinsic view declarations from generic
fieldwise defaults; their fieldless source projection is not an empty user struct.
Native boxes store the existing owner/offset/length payload. The current nonmoving
native collector traces object words, retaining the backing owner; a moving collector
will need stable owner hash identity, as with other reference hashes. No new managed
header, element copy, host service or ABI is introduced.

[Baseline](baseline-20261020.json): the interpreter rejects the expected cross-access
region equality assertion and native compilation rejects Memory boxing.
[Final source](Main.rvn) passes interpreter/JIT/AOT:
[validation](validation-20261020.json). Cases include direct inherited and boxed calls,
matching hashes, dictionary keys, contents mutation, differing windows/owners, defaults,
empty end windows, wrong types, exact unboxing and a box retaining the sole array owner
across collection. ToString remains ordinary diagnostic type display, not UTF-8 decoding
or content serialization. [Borrowed rejection](borrowed-rejection-20261020.json) preserves
the compiler's Slice-to-Object rejection.

Native support remains bounded to the existing byte-view profile. Other native element
types require their own storage lowering; this change does not silently admit them.
The interpreter compares element types as well as regions. No public typed EquatableTo
contract is added: current inherited Object calls box, including direct syntax. A typed
nonboxing API is a further library/compiler integration task. No allocation reduction
or throughput claim follows from enabling these Object contracts.

```sh
python3 scripts/build-native-project.py --profile console \
  --project docs/experiments/memory-object/MemoryObject.rvnproj \
  --bundle target/memory-object-bundle/bundle \
  --aot target/memory-object-bundle/bundle/bin/neoclr-codegen \
  --output target/memory-object-app
python3 docs/experiments/memory-object/check.py \
  target/memory-object-bundle/bundle target/memory-object-app \
  --output docs/experiments/memory-object/validation-20261020.json
```
