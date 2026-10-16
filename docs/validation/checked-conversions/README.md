# Checked conversion SDK qualification

Development slice, 2026-10-16. The metadata SDK now authors all 20 standard
Conv_Ovf_* opcodes, covering signed/unsigned 8-, 16-, 32-, 64-bit and native-width
destinations, with signed or unsigned interpretation of integer sources.
SDK authoring was introduced in fd355a771. The subsequent native lowering slice
adds shared JIT/AOT implementation; Raven checked contexts remain follow-up work.

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
stacks reject on both export paths for each opcode. The original SDK qualification at fd355a771 recorded AOT rejection for each
opcode. That historical limitation is superseded by the native qualification below.
Use run.py --expect-aot-rejection only when reproducing that earlier backend.

The [results summary](results.json) retains counts and fingerprints. Full commands,
per-image hashes and logs are in the ignored output directory. The test uses the
graph-based binary container writer, which selects the existing schema 3 when
floating literal bits require unsigned 64-bit storage; no new encoding/version is
introduced. No library bundle or Runtime Contract setting is needed.

Validation is on macOS ARM64 with 64-bit pointers; it does not qualify a 32-bit
native backend. Raven checked contexts, selection/fallback and narrowing remain next work. Broader structural work stays parked.

## Native JIT/AOT lowering follow-up

All 20 instructions now use shared numeric lowering. Integer values retain exact
bits through range checks; unsigned-source forms zero-extend Int32 before wider
checks. Floating inputs promote Single to Double, truncate toward zero and compare
against exact power-of-two bounds. Ordered comparisons reject NaN and infinities.
Nontrapping conversions execute only after range validation. This avoids leaking
hardware traps across the guest ABI and does not round Int64/UInt64 limits through
an inexact floating maximum.

Overflow returns ArithmeticOverflow/status 2 through the existing fault path,
preserving the host result slot. Native-width destinations are 64-bit on the
currently supported codegen targets; this qualification remains macOS ARM64.
Windows profile restrictions are unchanged. No runtime metadata encoding, SDK API,
Runtime Contract configuration or class-library bundle change is needed.

Run the native matrix against the exact SDK images above:

```sh
cargo build -p neoclr-codegen
python3 docs/validation/checked-conversions/run-native.py \
  --images target/checked-conversions-sdk-20261016-v3 \
  --output target/checked-conversions-aot-jit-20261016-final \
  --bundle target/function-reflection-library-validated-20261016
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk \
  cargo test -p neoclr-codegen --test values checked_conversions_route_numeric_functions
cargo test -p neoclr-codegen --test values byte_profile_rejects_invalid_conversions_and_distinct_borrows
```

Validation passed: all 852 AOT cases (442 successes and 410 overflow faults),
26 managed JIT cases, six routing/call-propagation cases, the invalid-profile
regression and four API snapshot checks.

The native matrix reuses the established .NET/interpreter oracle rather than
rerunning unchanged frontends. It executes all 852 images in linked AOT, checks
unchanged output on overflow, and samples 26 managed JIT cases covering all 20
opcodes, all source categories, NaN, infinities, fractional truncation and 64-bit
bounds. The ABI v4 JIT host additionally checks fault state and root/heap cleanup;
AOT uses ABI v2. [Native results](native-results.json) retain exact binary and bundle
fingerprints. The separate Rust regression exercises conversion through a call
with no nominal declarations, including faults propagating back to the host.
