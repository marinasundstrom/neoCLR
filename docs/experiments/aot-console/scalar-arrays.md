# Native scalar arrays — bounded development support

The sample survey found primitive arrays as the first AOT rejection in 39 consumers.
This slice admits ordinary and reserved arrays of Int32, UInt32, Int64, UInt64, SByte,
Int16, UInt16, Boolean, Void, IntPtr and UIntPtr. Byte arrays retain their existing
compact representation and I/O bindings. No Raven syntax, public library API or CLI
bridge encoding changes. Interpreter behavior remains the reference contract.

## Contract and layout

`newarr` initializes every element to its scalar default. `array.reserve` leaves each
slot unreadable until stored. Indexed stores normalize small integers and publish the
initialization marker after the complete value; indexed loads return the existing CIL
evaluation-stack category. Signed and unsigned array identities remain distinct even
when they use identical machine storage. Passing/copying an array reference preserves
aliasing; stores change that shared array. Null and bounds checks retain their existing
fault codes/messages and output remains unpublished on failure.

Private storage uses a 24-byte header, one eight-byte lane per element and one marker
byte per element. It reuses the checked snapshot load/store path, with a new private
allocation adapter. Length is limited to 65,536 and the explicit host byte capacity.
The allocator does not collect and publishes only on success. The array reference is
a GC root; its numeric payload is atomic (`NEOCLR_GC_BYTES`), so pointer-shaped numbers
do not retain other allocations. Header tags and padded lanes are private, not stable
native interop ABI. Images and adapters must be rebuilt together.

Element borrows, jagged arrays, interface/nominal array views, floating-point arrays,
Char arrays and arbitrary generic array arguments remain outside this slice. Admission
rejects unsupported operations instead of treating their storage as bytes. Scalar
array null/identity tests and length are admitted; broader Object/Array APIs depend on
separate library/dispatch support. This does not promise every previously blocked sample
now runs: selected follow-ups record the next rejection.

## .NET comparison and tradeoff

Reuse the [managed array](../../managed-arrays.md) and
[checked reservation](../../reserved-array-capacity.md) research. Reference aliasing,
default scalar initialization, copied scalar reads, invariant element types and faulting
bounds are existing neoCLR contracts aligned with the .NET ergonomic baseline.
Reserved unreadable capacity is an explicit neoCLR contract, not a claim that CLR
arrays normally expose uninitialized values. This implementation adapts those semantics
to the existing private native layout, rather than adding a separate AOT API.

Reusing padded lanes and marker checks is simple and keeps the reservation rules
consistent, but costs more space than packed Boolean/small-integer arrays and adds a
check even for default-initialized arrays. Packing or removing redundant checks is a
later optimization requiring measurement. No speed or memory advantage is claimed.
The interpreter already implements these operations and needs no semantic change.

## Validation

The compiled IL test exercises all eleven element types in seven modes (77 mode/type
comparisons): default values, reserved writes, full-width and signed/narrow values,
100 discarded arrays while a live array survives collections, uninitialized reads,
negative/end indexes, null arrays, negative lengths and the array cap. Native results
and fault diagnostics match the interpreter. Sanitized hosts check result atomicity,
frame cleanup, final empty heap and a buffer canary. Mismatched signed/unsigned array
loads and element borrows reject compilation.

The allocator kernel runs with/without GC, checks markers/zeroing, invalid and exhausted
allocation without output/cursor mutation, and proves a pointer-shaped UInt64 payload
does not keep its referenced allocation alive. All eleven native GC kernel tests pass;
two existing compiled record-array tests pass after sharing their load/store lowering.

Run focused checks with the configured macOS SDK:

```sh
cargo test --manifest-path tools/aot-poc/Cargo.toml --test console native_scalar_arrays
cargo test --manifest-path tools/aot-poc/Cargo.toml --test console native_record_arrays
cargo test --manifest-path tools/aot-poc/Cargo.toml --test native_gc
```

[Sample follow-up](../aot-sample-assessment/scalar-array-followup.json) records real Raven
consumers rebuilt from unchanged sources and run through both execution modes. Each
mode uses the same emitted artifact within that case; the rebuilt PE bytes are not
claimed identical to the earlier survey's files. `library-arrays` now passes, including
returned arrays, shared aliases, mutation through a function argument and length.
