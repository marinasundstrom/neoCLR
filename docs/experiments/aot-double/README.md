# Native Double instruction slice (development, 2026-10-08)

The ARM64 AOT value backend now admits binary64 literals, parameters, locals,
returns, inline record fields and existing typed object load/store/initialization.
It lowers add/sub/mul/div, equality, ordered/unordered relational comparisons and
comparison branches. Private call/storage lanes retain F64, including mixed and
nested records; Double bits are not GC roots. This is an experimental private ABI,
not a new public native calling convention.

The unchanged Raven Math constants sample now executes interpreted, with sanitized
native adapters, and as a standalone binary depending only on macOS libSystem.
[Execution evidence](../../../benchmarks/native-web/math-double-validation.json)
records matching output and successful HTTP server admission. Compilation still uses
an explicit CLI Core.dll bootstrap. No full bootstrap or production qualification
claim follows from this slice.

## CLI behavior and implementation choice

This fills an AOT implementation gap; it changes neither Raven nor metadata encoding.
Reuse the [floating-point contract](../../floating-point.md): binary64 evaluation,
IEEE infinities/NaN on floating division by zero, signed-zero preservation and no
promise about arithmetic NaN payload propagation. Integer arithmetic keeps its
existing checked/fault behavior. No fast-math or fused-operation rewrite is introduced.

[CLI cgt](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.cgt?view=net-10.0)
is false for unordered operands, while
[cgt.un](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.cgt_un?view=net-10.0)
accepts unordered operands (retrieved 2026-10-08). The same distinction applies to
corresponding relational branches. Cranelift 0.121.2's ARM64 `condcode_from_floatcc`
leaves unordered relational codes unimplemented. Lowering uses the complement of
an ordered relation, e.g. `cgt.un(a,b) = !(a <= b)`, preserving NaN semantics without
upgrading the backend or introducing helper calls. This adds a Boolean inversion;
no performance improvement is claimed or benchmark needed for this correctness slice.

[Focused check results](validation.json).

## Validation and limits

`tools/aot-poc/tests/values.rs` compares 40 native/interpreted programs: arithmetic,
special values, comparison instructions, unordered branches, nested mixed-lane
record calls, and default/borrowed Double storage. It also checks that remainder,
negation and floating conversion gaps reject before producing an object.
The GC layout control confirms Double is atomic, while a borrow still retains an owner.
`dotnet run --project docs/experiments/aot-double/dotnet` independently exercises 38
matching arithmetic/comparison/branch controls with Reflection.Emit on .NET 10 ARM64.

Reproduce the consumer with `benchmarks/native-web/verify_callbacks.py --case MathConstants`
and the usual `--compiler`, `--runtime`, `--aot`, `--bundle`, `--output` arguments.
Use matching compiler/runtime bundles with assembly-level constants. The earlier
admission-only report remains historical evidence; this execution report supersedes
its Double-literal blocker.

Single, floating conversions, negation, remainder, ckfinite, floating arrays/boxing,
indirect floating opcodes, Math service bindings and formatting remain unsupported
in AOT. Supported typed ldobj/stobj are distinct from the still unsupported ldind.r8/
stind.r8 spellings. This is a bounded release baseline, not complete floating CLI coverage.
The next priorities remain shared introspection faults and bootstrap qualification;
expand floating coverage when a release consumer requires it.
