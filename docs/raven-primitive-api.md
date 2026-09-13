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
