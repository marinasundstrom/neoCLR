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
element byref returns and public persistent Memory APIs remain unimplemented.
Array range callers now share storage and must use ToArray to retain copy
semantics. ArrayList still returns owned range copies pending a separate growth
and invalidation contract. No native-performance claim follows from these tests.
