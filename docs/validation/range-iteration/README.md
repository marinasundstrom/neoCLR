# Signed integer range iteration — 2026-10-15

Raven main cf7d7c52b adds overflow-safe advancement for signed int/long direct loops
and collection/dictionary comprehensions, including async loop state. It is carried
separately to codex/neoclr as `827bca9c41a8d2073b8bb1af717fae5bed8930c0`, whose portable
range lowering uses the same domain guard.

Before adding a nonnegative step, stop when current > MaxValue-step; before adding
a negative step, stop when current < MinValue-step. The subtraction is representable
in each sign branch. This handles minimum-valued steps without negation and does not
need a wider type. End conditions remain exclusive for .. and inclusive for ..=.
The next out-of-domain mathematical value ends enumeration; it does not wrap or fault.

Current progression state stays separate from the body binding. Start, end and step
retain once-only source-order evaluation. Continue reaches one guarded advance;
break skips it. No native opcode, metadata format, Runtime Contract configuration or
library API changes. Existing zero-step and other numeric-domain behavior is unchanged.
Generic Interval, nominal slicing and explicit stepping remain separate
[API work](../../design/slicing-capabilities.md); structural types remain parked.

## Qualification

- Main: 46 focused range/loop/async tests pass on .NET 11, including an integer
  comprehension and pending awaits at signed limits. Both integer widths are covered.
- Native branch: 72 range/loop/async/portable-comprehension tests pass on .NET 11.
  Native compiler build passes. These tests reuse main's semantics without merging
  unrelated branches; native-only comprehension endpoint checks are retained.
- The original consumer (fingerprinted in results.json) checked eight signed-limit scenarios, overshooting and
  minimum-valued steps, continue/break and side-effect evaluation order. It returned
  42 in interpreter, macOS ARM64 AOT and managed JIT, without arena allocation.
  AOT uses undefined/bounds sanitizers; JIT is not sanitizer-instrumented.
- [results.json](results.json) records commands and source/binary fingerprints. The
  existing five-library range bundle and runtime are unchanged. Native execution
  covers synchronous loops; .NET pending-await tests do not establish additional
  native async scheduling behavior. No throughput claim is made.


## Exclusive unit-step optimization — 2026-10-17

Raven's native portable lowerer can omit the advance overflow guard when the
signed Int32/Int64 step is a literal one and the endpoint is exclusive.
`current < limit <= MaxValue` proves the addition representable. Inclusive,
descending, larger and dynamic steps retain their domain checks. Bounds and step
still evaluate once in order, and `continue` uses the same advance as fallthrough.
This preserves the .NET-target observable iteration contract; it changes no
Runtime Contract option, semantic symbol, metadata, public API or runtime opcode.
Ordinary .NET emission is unchanged and broader structural work remains parked.

The expanded [consumer](Main.rvn) checks maximum exclusive bounds, empty/reversed
loops, explicit unit steps, evaluation order and continue/break alongside the
original general-step/inclusive cases. It now prints a success marker and returns
zero, using the bundled host instead of the original custom host. Run it with:

```sh
python3 scripts/validate-native-range-iteration.py --bundle /absolute/bundle --output /new/output
```

[The new report](unit-step-20261017.json) records passing interpreter, JIT and AOT
runs on macOS ARM64 and checks input hashes before/after execution. Unlike the
October 15 run, this uses the normal optimized package host without sanitizers.
The isolated bundle replaces only `sdk/tools/rvnc/Raven.CodeAnalysis.dll` with a
fresh Debug/net11.0 compiler build; runtime, codegen and native libraries remain
byte-identical to the fixed editor candidate. This is a controlled compiler
experiment, not a qualified release archive. Twenty-three portable range compiler
checks pass, including eight new .NET boundary/parity and native-lowering admission
cases. [CPU measurements](../../../benchmarks/execution/unit-step-20261017/README.md)
track the measured effect. A fresh complete SDK/library cohort remains necessary
before release qualification; Windows execution is not established by this run.
