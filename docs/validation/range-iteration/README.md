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
- [Main.rvn](Main.rvn) checks eight signed-limit scenarios, overshooting and
  minimum-valued steps, continue/break and side-effect evaluation order. It returns
  42 in interpreter, macOS ARM64 AOT and managed JIT, without arena allocation.
  AOT uses undefined/bounds sanitizers; JIT is not sanitizer-instrumented.
- [results.json](results.json) records commands and source/binary fingerprints. The
  existing five-library range bundle and runtime are unchanged. Native execution
  covers synchronous loops; .NET pending-await tests do not establish additional
  native async scheduling behavior. No throughput claim is made.
