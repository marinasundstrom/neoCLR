# Raven slice foundation — 2026-10-19

The production implementation is authored in Raven, compiled with the native
source-core contract and executed by the interpreter. The neoIL view declarations
under tests/fixtures are runtime tests, not a second class-library implementation.

The consumer checks Array range -> Slice<int>, mutation visible in the original,
implicit array arguments for writable/readonly parameters, readonly aliases,
nested readonly windows, directional indexing, explicit ToArray copying and
String.Utf8Bytes. It returns 0. Negative fixtures require RAV2811 (ordinary
field), RAV0131 (readonly write) and RAV2812 (slice array element).

Reproduce after building rvnc with NeoClrMetadataProject pointing to this repo:

```sh
dotnet "$RVNC" neoclr --project runtime/raven/projects/System.Runtime/System.Runtime.rvnproj
dotnet "$RVNC" neoclr --project docs/validation/memory-views-20261019/Consumer.rvnproj
printf '.module System\n' > target/slice-System.neoil
target/debug/neoclr run docs/validation/memory-views-20261019/bin/neoclr/SliceConsumer.dll \
  --module runtime/raven/projects/System.Runtime/bin/neoclr/System.Runtime.dll \
  --object-root runtime/raven/projects/System.Runtime/bin/neoclr/System.Runtime.dll \
  --system target/slice-System.neoil --execution interpreter --show-result
```

Compile each reject-*.rvn as a library with --native-core-reference pointing to
that System.Runtime.dll; require the stated diagnostic and no published artifact.
Use fresh output names for one-shot compilation: the compiler refuses overwrites.
Object-root selection at runtime is required for the source-owned Array contract.

Nine Rust executable view contracts and native metadata round-trip/import tests
also pass. A broader Raven test attempt found an additional defensive-copy failure
in PortableDefaultReceiverTests.ReadOnlyFieldReceiverUsesDefensiveCopy (expected
2, observed 3); it remains open and is not counted as a passing check. The focused
ref-like, value-indexer and ordinary .NET range checks pass: 33 tests, zero failures.

At this interpreter-only milestone, JIT/AOT ABI, native GC maps, optimizations,
borrowed generic anti-constraints and element byref returns remained unimplemented.
See the native follow-up below for subsequent bounded support.
Array range callers now share storage and must use ToArray to retain copy
semantics. ArrayList still returns owned range copies pending a separate growth
and invalidation contract. No native-performance claim follows from these tests.

## Persistent source follow-up

MemoryConsumer.rvnproj selects `RavenNeoClrAsyncLibrary=System.Runtime` and the
same native core/ownership contract. Compile it with the same compiler and run
bin/neoclr/MemoryConsumer.dll using the core module, object root and interpreter
flags above. Result: Int32(0). It tests owner escape from the creator frame, two
persistent fields, receiver-relative windows, explicit GC before/after awaited
Task.Run, shared writable/readonly borrowing, array conversion and explicit copying.

reject-memory-conversions.rvn diagnoses both illegal conversions with RAV1503
and publishes no artifact. The prior SliceConsumer still returns 0 against the
extended core. See persistent-validation.json for binary identity and outputs.
The bounded native follow-up below does not yet enable stream/codec/crypto signature migrations.

Persistent compiler integration: Raven `c173a723c`. All five source libraries and
the production native API renderer build successfully; four native snapshot
checks pass. The generated reference includes both persistent types and members.


## Native persistent-byte follow-up

`NativeMemoryConsumer.rvnproj` exercises Memory<byte> and ReadOnlyMemory<byte>
with array conversion, default values, Length/IsEmpty, one- and two-argument slicing,
function arguments/returns and object fields. After explicit GC, checked nested
slicing proves both nonempty owners and an independently owned empty view remain
live. `NativeMemoryBounds.rvnproj` requires IndexOutOfRange for a nonempty window
of a default readonly memory; it exercises slicing without any factory binding.

Run `native-run.py --compiler <matching rvnc.dll> --output <directory>` after
building `neoclr-codegen` and the source-owned System.Runtime library. The harness
runs both fixtures through the interpreter, AOT and JIT; AOT uses UBSan/bounds.
The native host checks success/fault codes, untouched failure output, balanced
root frames and full reclamation. Native evidence is saved as
`native-validation.json`; raw build/admission logs remain in the output directory.
No new Raven API or compiler change is involved. Borrowed views, non-byte
instantiations, arrays/unions of views, boxing and remaining view operations
remain gated. Async native retention is not qualified by these synchronous tests.

The native harness also isolates past-end, Int32.MinValue and Int32.MaxValue
starts for both persistent byte types on interpreter, JIT and AOT. Valid short
windows cover start=end, default empties and nested views after GC. These cases
extend the existing Slice contract; no public signature or snapshot changes.

See [short-slice-validation.json](short-slice-validation.json) for the extended
harness commands, binary/source hashes, fault outputs and log hashes. The prior
native-validation.json remains the evidence for the initial two-argument step.

## Native readonly conversion follow-up

The success consumer now also exercises implicit Memory<byte>-to-ReadOnlyMemory<byte>
conversion at typed locals, arguments and returns. Separately created nonempty
and empty converted windows retain their sole array owners after creator return
and explicit GC, including a converted view stored in an object field. Default
conversion remains empty. Post-GC checked slicing validates live owner storage.
Run `native-run.py --consumer-only --compiler <rvnc.dll> --output <directory>`;
the earlier bounds matrix is unchanged and its evidence is reused. See
[readonly-validation.json](readonly-validation.json) for commands and hashes.
The exact binding tests reject reversed access, wrong element or lifetime kind,
byref/extra/missing arguments and serialization without trusted shape proof.
Public signatures and their generated API snapshot are unchanged.

## Native Range qualification

The consumer now exercises existing Raven range APIs on both persistent byte
views: open ranges, from-end boundaries, nested ranges, Range.All, default-empty
and end-empty windows, and range-created views held in fields through GC. No new
runtime instruction, intrinsic or public member is required.

Run `native-run.py --consumer-only --range-bounds --compiler <rvnc.dll> --output <directory>`.
This adds six isolated invalid-range fixtures (reversed, past-end and excessive
from-end bounds for each type). At that checkpoint the Raven validator produced UserFault,
not the integer Slice service's IndexOutOfRange; that difference was an
error-classification gap, resolved in the follow-up below. Expected native code is 4, output remains untouched,
root frames balance and the heap is fully reclaimable. See
[range-validation.json](range-validation.json) for the commands and hashes.
Earlier integer-boundary and exact-binding evidence remains applicable.

## Consistent range bounds follow-up

The updated harness expects IndexOutOfRange (native code 8) for invalid ranges.
`--range-bounds` now also compiles interpreter-only failures for Slice<byte> and
ReadOnlySlice<byte>, and runs a standalone negative-length Range call without
reference-arena admission in AOT. Managed JIT uses its required GC/context profile. The success consumer verifies that
TryGetOffsetAndLength still returns Error for reversed bounds and negative length.
See [consistent-bounds-validation.json](consistent-bounds-validation.json); earlier
range-validation.json is retained as historical evidence of the corrected behavior.
The source-owned core and matching documentation bundle are rebuilt for this step.
