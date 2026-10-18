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

JIT/AOT ABI, native GC maps, optimizations, borrowed generic anti-constraints,
element byref returns remain unimplemented.
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
Native JIT/AOT and actual stream/codec/crypto signature migrations remain pending.

Persistent compiler integration: Raven `c173a723c`. All five source libraries and
the production native API renderer build successfully; four native snapshot
checks pass. The generated reference includes both persistent types and members.
