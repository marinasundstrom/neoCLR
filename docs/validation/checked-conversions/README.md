# Checked conversion SDK qualification

Development slice, 2026-10-16. The metadata SDK now authors all 20 standard
Conv_Ovf_* opcodes, covering signed/unsigned 8-, 16-, 32-, 64-bit and native-width
destinations, with signed or unsigned interpretation of integer sources.
This is prerequisite work for Raven checked contexts. Neither Raven syntax nor
native JIT/AOT conversion lowering is enabled by this slice.

The observable contract follows [.NET checked conversion](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.conv_ovf_i4_un):
convert within range, truncate floating-point values toward zero and report
overflow when the truncated value is unrepresentable. NaN and infinities fault.
Unsigned-source forms reinterpret integer bits; floating sources remain numeric.
NeoCLR uses ArithmeticOverflow rather than a CLR exception at its host boundary.
The [host API reference](../../../api-docs/experimental-metadata.md#checked-numeric-conversions-development-2026-10-16)
describes stack categories and rejection behavior.

## Reproduce

With the repository .NET SDK and built runtime/codegen binaries:

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- \
  --checked-conversions target/checked-conversions-sdk-20261016-v3
python3 docs/validation/checked-conversions/run.py \
  --images target/checked-conversions-sdk-20261016-v3 \
  --output target/checked-conversions-native-20261016-final
python3 -m unittest discover -s scripts -p 'test_native_api_snapshot.py'
```

Output directories must be fresh. The SDK runner also belongs to the normal test
suite; without an output path it executes .NET and validates native emission.

## Evidence and limitations

The matrix contains 852 cases over all 20 opcodes. Sources include Int32, Int64,
IntPtr, UIntPtr, Single and Double. Integer cases cover source extrema,
destination boundaries, adjacent out-of-range values and negative bit patterns.
Floating cases cover fractional truncation, negative fractions, both bounds,
NaN and infinities. Expected integer results use BigInteger to avoid precision
loss in the test oracle. Successful fixtures return 42 only after result
comparison; overflow fixtures must fault instead of returning.

All 852 cases passed on .NET and in the interpreter (442 successful conversions
and 410 expected overflow faults). Four API snapshot checks passed. Empty, Boolean and String
stacks reject on both export paths for each opcode. AOT explicitly rejects one
fixture for each opcode before producing an object file. This documents the
remaining shared JIT/AOT backend gap; it is not a successful compiled-execution
claim. No managed JIT execution is claimed in this slice.

The [results summary](results.json) retains counts and fingerprints. Full commands,
per-image hashes and logs are in the ignored output directory. The test uses the
graph-based binary container writer, which selects the existing schema 3 when
floating literal bits require unsigned 64-bit storage; no new encoding/version is
introduced. No library bundle or Runtime Contract setting is needed.

Validation is on macOS ARM64 with 64-bit pointers; it does not qualify a 32-bit
native backend. Native checked conversion lowering and Raven checked contexts,
selection/fallback and narrowing remain next work. Broader structural work stays parked.
