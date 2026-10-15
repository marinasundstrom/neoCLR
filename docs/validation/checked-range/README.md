# Checked inclusive System.Range conversion — 2026-10-15

Raven compiler commit: `3c1767769e7a8333cf72b0217b96e5bcf6e53f13` on codex/neoclr.

The native Raven compiler converts an inclusive end into the existing exclusive
Index representation: n becomes n+1; ^n becomes ^(n-1). It first constructs and
saves the original Index, preserving negative-distance validation and once-only
source evaluation. The start is evaluated before the end. MaxValue and ^0 are
checked before arithmetic. No receiver length or new library API is required.

The configured RuntimeFailureContract owns terminal native failure. A normal call
propagates faults even when enclosing operands are pending; the compiler emits an
additional impossible-return guard only at empty-stack statement boundaries.
.NET's existing checked arithmetic/exception lowering is unchanged. This native
implementation requires the library's existing failure contract; no opcode or
metadata format changes are involved. Structural work remains parked.

## Evidence

- [Main.rvn](Main.rvn) checks source evaluation order, stored bounds, from-end
  translation, the largest representable start-relative included end, and resolved
  offset/length. The result is 42 in interpreter, macOS ARM64 AOT and managed JIT.
- [Faults.rvn](Faults.rvn) checks included ^0, included Int32.MaxValue, -1 and ^-1.
  All four terminate with the expected user-fault message in the interpreter.
  The [native fault host](fault-host.c) invokes all four cases, verifies fault code,
  unchanged output and exact messages, and checks managed-root cleanup in AOT/JIT.
- [results.json](results.json) and [fault-results.json](fault-results.json) retain
  commands, compiler/runtime provenance and hashes. AOT uses undefined/bounds
  sanitizers. Managed JIT is not sanitizer-instrumented.
- 32 focused range/index/runtime-failure compiler tests pass on .NET 11. Native
  compiler and test builds pass. The existing five-library range bundle is reused.

The argument-driven fault consumer requires native GC and the process-argument
host adapter; initial AOT host setup without those dependencies was rejected.
Its final qualification includes both. Reused interpreter/metadata evidence is
retained when only the native host linking changes. No performance claim is made.

Safe slicing methods can return Result for receiver-relative validation, but cannot
recover faults during argument construction. The planned nominal capability API
must preserve that distinction; see [slicing capabilities](../../design/slicing-capabilities.md).
