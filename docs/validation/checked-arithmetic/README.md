# Checked integer arithmetic authoring

Development qualification, 2026-10-16. This is a metadata SDK and native backend
prerequisite for Raven checked contexts, not an implementation of those contexts.

The SDK exposes six .NET-style opcodes: Add_Ovf, Add_Ovf_Un, Sub_Ovf,
Sub_Ovf_Un, Mul_Ovf and Mul_Ovf_Un. CLI output uses the standard overflow-checking
instructions; native output uses existing add/sub/mul.ovf[.un] operations. No new
container version, runtime instruction, Runtime Contract setting or temporary
bridge encoding is introduced. Signedness comes from the opcode. Inputs must
have matching integer evaluation categories; small integers require promotion
and checked narrowing remains separate work.

.NET/CLR is the reference: successful operations preserve the evaluation width
and overflow raises OverflowException. NeoCLR reports its ArithmeticOverflow
fault (native status 2), preserving the host result slot. See Microsoft's
[Add_Ovf documentation](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.add_ovf)
and the [SDK API contract](../../../api-docs/experimental-metadata.md#checked-integer-arithmetic-development-2026-10-16).

The native value profile previously admitted only Int32 checked arithmetic.
It now validates matching Int32, Int64 and native-width operands. Shared codegen
uses width-preserving signed/unsigned overflow operations rather than widening
through Int64 and narrowing to Int32. This fixes 64-bit and pointer-sized
operations in both JIT and AOT without changing unchecked arithmetic.

## Reproduce

Run on macOS ARM64 with .NET and the repository's Rust toolchain. Native JIT uses
the matching library bundle from the function-reflection qualification:
target/function-reflection-library-validated-20261016. The SDK fixtures themselves
have no library dependency. Output directories must be fresh.

```sh
cargo build -p neoclr -p neoclr-codegen
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- \
  --checked-arithmetic target/checked-arithmetic-sdk-20261016-final
python3 docs/validation/checked-arithmetic/run.py \
  --images target/checked-arithmetic-sdk-20261016-final \
  --bundle target/function-reflection-library-validated-20261016 \
  --output target/checked-arithmetic-native-20261016-final
cargo test -p neoclr-codegen --test values checked_wide_and_native_arithmetic
cargo test -p neoclr-codegen --test scalar native_arithmetic_faults_match_interpreter_and_preserve_output
python3 -m unittest discover -s scripts -p 'test_native_api_snapshot.py'
```

On the validation machine, the system clang/ld does not understand the newly
installed macOS 27 command-line SDK. The Rust host tests use
SDKROOT=/Library/Developer/CommandLineTools/SDKs/MacOSX26.5.sdk; the matrix runner
selects the active Xcode SDK explicitly through xcrun. This is a host linker
configuration issue, not a guest arithmetic failure.

## Results

All checks above passed: 108 .NET/interpreter/AOT cases, 18 managed JIT cases,
36 wide/native Rust cases, the existing scalar arithmetic fault regression and
four API snapshot checks. Invalid SDK stacks were rejected on both export paths.

## Coverage and limits

The SDK matrix includes all six operations across Int32, UInt32, Int64, UInt64,
IntPtr and UIntPtr, each with success, overflow and signedness-sensitive bit
patterns: 108 .NET executions and native images. It also rejects underflow,
mixed widths, floating-point and boolean stacks for both export targets.

The native runner checks all 108 images with the interpreter and linked AOT.
Managed JIT checks 18 representative images, covering every operation and
every signature category with all three scenarios. AOT hosts use ABI v2; managed
JIT uses ABI v4 with fault-state/root/heap cleanup checks. Successful images
return 42; overflow leaves the host result sentinel unchanged. Additional Rust
coverage checks 36 wide/native-width success and overflow cases, and preserves
the existing scalar arithmetic fault regression.

The retained [results summary](results.json) identifies binaries, SDK and bundle by SHA-256.
Full commands and logs remain in the ignored output directory; run.py reproduces
them. This qualification is for 64-bit ARM64; it does not establish a 32-bit
native backend. No performance claim is made. SDK checked numeric conversions,
Raven checked expression/block contexts, overload selection/fallback and checked
narrowing are still follow-up work. Structural type work remains parked.
