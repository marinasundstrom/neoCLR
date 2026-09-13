# Raven primitive storage and character APIs

The bridge admits SByte, Byte, Int16, UInt16, Char, UInt32, Int64, UInt64, Single,
IntPtr and UIntPtr values in addition to the existing Int32/Double/String/Boolean
support. Each newly admitted primitive exposes its existing concrete CompareTo
method. Char also exposes all 16 current classification helpers.

Storage signatures remain exact. The importer now follows the runtime's
[integer storage/stack distinction](integer-types.md): small integers and UInt32
load as Int32; UInt64 loads as Int64. Single and Double share the floating stack
category while storage preserves width. This follows the existing CLR-like runtime
model rather than introducing wrapper objects. Built-in CLI signature codes are
recognized even when Cecil supplies a fallback corlib scope; callable method owners
still pass dependency and signature validation.

The importer admits ldc.i8, ldc.r4 and the existing ordinary conv.i1/u1/i2/u2/i4/u4/
i8/u8/i/u/r4/r8/r.un instructions. It does not add checked conversions or a general
numeric operator compiler. Boolean still uses the runtime's distinct stack value;
the later [Boolean boundary slice](raven-boolean-api.md) projects canonical Raven
literals and Boolean.CompareTo without changing that runtime representation.

## Samples and compiler dependency

The [primitive sample](experiments/raven-target/samples/library-primitives.rvn)
checks signed and unsigned ordering at boundaries, Single comparison, Int64/Char
parameter receivers, native zero comparisons and every Char classifier. Native
nonzero conversion/language support is not established by those zero checks.
Character literals represent UTF-16 code units. Numeric Char casts express surrogate
code units because Raven's current lexer does not accept their Unicode escapes.
The [character API contract](character-classification.md) distinguishes Unicode digit
classification from Int32.Parse's ASCII decimal grammar.

Use Raven experiment commit `26907410f` or later. That isolated compiler correction
admits missing explicit fixed-width integral casts and preserves source signedness
when widening. It was tested on the ordinary CLR (14 focused and 69 overlap tests),
then exercised through this target. The broader feature-suite build was stopped when
its unrelated Raven.Macros dependency stalled; it is not reported as passing.
No implicit-conversion matrix expansion, syntax change or installed tool refresh is
part of that fix. Samples use explicit numeric casts where Raven requires them.

Copy the sample into a fresh [prepared project](experiments/raven-target/README.md)
and use **neoCLR: Run saved project**. The saved-project suite includes it;
`verify_editor.py --primitives` checks all character method completions. Signature
checks distinguish Char metadata from Int32 even though both use Int32 stack values.
General Comparable interface dispatch and arbitrary generic primitive payloads
remain separate coverage work.

## Numeric comparisons after Preview 5

The importer now admits `ceq`, `cgt`, `cgt.un`, `clt`, `clt.un` and numeric
`beq`, `bne.un`, `bgt`, `blt`, `bge`, `ble` branches, including unsigned/unordered
and short branch forms. Operands must share a supported numeric stack category;
this does not add reference ordering or mixed-category implicit conversions.
Both successors still pass the existing control-flow and stack verification.

This closes a CLI importer gap rather than changing Raven or the instruction set.
It reuses the runtime's [integer rules](integer-types.md) and
[floating comparison rules](floating-point.md), based on ECMA-335: opcode
signedness determines integer ordering, unordered floating comparisons preserve
NaN behavior, and signed zero compares equal. The bridge converts neoCLR's Boolean
comparison result to the CLI's canonical Int32 result using its existing Boolean
adapter. This preserves current runtime behavior at the cost of an adapter call.

`verify_queries.py` checks all six source comparison operators in expression and
conditional contexts, with signed boundaries, unsigned high bits, NaN, infinity,
and signed zero. It also runs an ordinary `Where(value => value > 1)` regression.
These checks cover Raven's emitted forms, not every possible hand-authored CLI
program. Native nonzero comparisons remain outside the validated Raven surface.
Published Preview 5 tools do not contain this fix; it is for the next build.

Raven experiment commit `09cf60417` selects unsigned comparison opcodes for
unsigned operands and unordered comparisons when inverting floating `<`/`>` to
implement `>=`/`<=`. Baseline Raven sorted high-bit unsigned values as negative;
its inclusive floating comparisons could accept NaN. The correction is target-neutral
and is tested by executing emitted assemblies on .NET as well as through neoCLR.
It requires rebuilding the experimental compiler; changing the bridge alone cannot
repair incorrect signedness already encoded in IL.

Validation on 2026-09-13: 25 query checks (252 individual numeric expectations),
51 saved-project checks, and five focused Raven code-generation tests passed.
The wider Raven overlap run passed 24 tests and failed the existing
`TryLookup_UlongMixedWithSignedIntegral_IsRejected` binding test; that failure was
reproduced against unchanged compiler code and remains separate work.

### Mixed numeric operator follow-up

The previously recorded `ulong`/signed lookup failure is now corrected in the Raven
experiment (`bc0ec8046`). User-defined binary operator candidates require implicit operand
conversions, following [C# operator applicability](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/expressions#1245-binary-operator-overload-resolution).
Previously, a rejected built-in combination could fall through to .NET numeric
metadata and be admitted through an explicit conversion. Fixing applicability in
Raven avoids a target-specific runtime workaround and preserves explicit casts.

Target regression checks reject mixed `ulong`/`long` addition and comparison before
execution, while `uint`/`int` comparison still promotes to `long`. This does not
expand the implicit conversion table; missing widening conversions such as `short`
to `int` remain follow-up work. The ordinary Raven operator test group passes all
61 checks, including both operand orders and user-defined operator applicability.

All 28 target query checks passed with that compiler, including the new mixed-numeric
acceptance/rejection checks. Previously published tools remain unchanged.
