# Explicit native byte-buffer copies

Development interpreter foundation, 2026-10-22. The public overloads are documented
in [the API reference](../../../api-docs/native-interop.md#native-byte-copies).
This is an explicit copy bridge, not the pinning or AOT-import completion gate.

## Reproduction

Build the source libraries with the native-enabled Raven compiler, install them into
a development bundle, and build the current interpreter. Run the built-in suite:

```sh
python3 scripts/test-runtime-library.py --bundle /absolute/path/to/bundle \
  --output /absolute/path/to/fresh-evidence --runtime target/debug/neoclr \
  --suite native-buffers --mode interpreter
cargo test --lib native_byte_copy
```

The harness builds `runtime/System.Runtime/test/NativeBufferTests/fixture.c`, emits an absolute
NativeImport declaration, discovers the tests through the framework, and records
fixture/runtime/library hashes. Unix is the currently admitted harness profile.
Use CC and SDKROOT to select a matching compiler/SDK when the host defaults disagree;
on the qualified macOS host Xcode clang required the Xcode macOS SDK rather than the
newer CommandLineTools SDK. No toolchain settings are changed globally.

The four built-in tests exercise Memory subwindows through an in-place C import,
UTF-8 bytes without an intermediate array, null pointers with empty slice literals,
and 64 repeated bidirectional transfers with zero managed allocation count increase.
The low-level tests cover preflight failure, capacity, initialization, read-only
output, active native borrows, use after free, offsets and empty null access. Managed
allocation counts do not measure host allocation or throughput.

## Compiler repairs

Raven main 9193bdff1 fixes default pointer typing and emission; the native branch
carries it as fa8ecb50c. All 47 focused pointer/default .NET tests pass. Explicit and
target-typed pointer defaults remain pointers and emit a native-sized zero. The
initial RAV2001 was invalid `0b` fixture syntax, not a slice type-inference defect.

Raven native 21c279632 supports empty Slice/ReadOnlySlice collection expressions
through an adapter capability to lower to the ordinary default memory-view value. The opt-in checks imported native
memory-view metadata or the explicit source-core declaration contract. Other collection
types and .NET emission retain their own construction rules. Built-in empty/null
copy execution is the native consumer regression for both compiler repairs.

## Limits

Only live tracked NativeMemory allocations are accepted. Initialize native output
storage before calling a foreign writer: foreign writes do not update initialization
tracking. An active foreign borrow excludes copy/free, but concurrent managed-buffer
mutation is unsupported. The interpreter stores array bytes as tagged values, so the
transfer still loops over slots. JIT/AOT imports, packed managed storage, genuine
pinning, native-backed Memory ownership and arbitrary foreign-owned pointer views
remain separate work. No release package or cross-platform qualification is claimed.


## Qualified evidence

On macOS ARM64, all four built-in tests pass with compiler 21c279632 and the repaired
interpreter ([saved results](validation.json); full local report in
`target/native-buffer-tests-final-20261022/report.json`). Both low-level
`native_byte_copy` tests pass. The five source libraries were built with 255906ffe;
the consumer needs the subsequent pointer-default and empty-view repairs. The local
bundle's inherited JIT/AOT binaries were not exercised or requalified. API rendering
uses 21c279632; no complete SDK/package compatibility claim is made.

Four API snapshot tests, 24 website tests and the 5,646-page website build pass.
The local preview is rebuilt; publication remains a separate operation.
