# Source native integer gate — 2026-10-06

System.IntPtr and System.UIntPtr now compile unchanged as NativeIntegers.dll, alongside
two internal runtime-service adapters. A separate consumer imports their native symbols
and executes CompareTo for negative, zero, identical and maximum unsigned values. The
library sources are absent from the consumer command; verification passes and execution
exits 42 with empty stdout. Managed arrays and native pointer access are unchanged.

The harness extends the explicit bootstrap ownership manifest with one owner for both
types. The original released manifest is not changed. The test input assembly is authored
through the metadata API to supply nonzero native integers: Raven's existing native
integer cast policy is deliberately unchanged, so these inputs do not imply that Raven
source now supports such casts. They are test data; the comparison implementation is
the actual class-library source, with no consumer stub or substituted implementation.

## Boundaries and compatibility

- Raven a6ee91610 maps IntPtr/UIntPtr signature facts through native import, portable
  primitive identities and native emission. Source/imported primitive selection uses
  explicit ownership with no fallback. Reflection handles remain in the .NET adapter.
- The metadata API permits canonical SetNativePrimitive(IntPtr/UIntPtr) definitions;
  their native storage is scalar, not an instance record containing itself.
- Pure runtime widening services return Int64/UInt64 with sign/zero preservation. They
  introduce no host-resource capability. This follows the existing CLR native-width
  comparison described in [native integers](../../native-integers.md).
- A general portable-lowering fix, Raven 24c2c4d40, initializes temporary storage for
  default(T).Method(). Its own C# regression checks lowering and ordinary .NET mutation
  semantics. Main lacks this portable layer, so the isolated fix is recorded for that
  layer's integration rather than copying the experimental backend onto main.

## Validation

All 161 C# metadata groups pass, including native primitive designation round trips.
Nine runtime integer tests pass, covering both widening services at host signed and
unsigned extrema, wrong signatures and the existing native integer controls. Eight
focused Raven/.NET tests pass, including the independent default-receiver regression.
The artifact-only source gate uses negative and maximum unsigned inputs and verifies
native generic ComparableTo contracts, scalar receivers and temporary default values.

The full-owned-handle audit drops from 35 to **30 diagnostics** across 188 source inputs.
It still emits no System artifact. Remaining observed diagnostics concern Console
services, associated inference/name errors, NativeAllocation/RuntimeFailure imports,
let-else flow and HTTP task result flow. These are not yet independent root-cause counts.
Console execution is not claimed: its remaining adapters and retained-seed WriteLine
result-convention compatibility must be addressed next.

## Reproduce

Build the metadata C# test project first, then run:

```sh
python3 docs/experiments/extended-cli-metadata/verify_source_native_integers.py \
  --compiler /tmp/width-final-compiler1006/rvnc.dll \
  --compiler-revision befb8c1e1+native-widths \
  --core /tmp/preview12-bootstrap/Core.dll \
  --seed /tmp/preview12-bootstrap/System.neox \
  --library /tmp/native-poc-libraries1005/Numbers.dll \
  --ownership runtime/raven/native/poc-ownership.json \
  --runtime target/debug/neoclr --output /tmp/native-width-fresh
```

[Execution evidence](source-native-integers-2026-10-06.json) records commands, generated
ownership, dependency and source hashes. The compiler snapshot predates commits
24c2c4d40/a6ee91610 and is labelled with its base plus changes; exact binary hashes are
recorded. Runtime base 051aa759 similarly precedes this widening-service change, whose
source hashes are recorded. [Full audit](native-bootstrap-integers-2026-10-06.json).
These results qualify this development slice on macOS arm64, not a new release or
full bootstrap. The unchanged guest API snapshot passes its consistency check.
