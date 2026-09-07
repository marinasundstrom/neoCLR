# Floating-point fundamentals

System.Single and System.Double are canonical primitives with float32/single and
float64/double aliases. Their native storage is IEEE binary32 (4 bytes) and binary64
(8 bytes), using the host's f32/f64 alignment and byte order. They can appear in
records, locals, parameters, returns, and constructed signatures. Their full BCL
member APIs and formatting/parsing support remain to be implemented.

## Storage and evaluation

This interpreter chooses binary64 for the CLI internal floating-point category F.
It represents that stack category using Value::Double. Single values widen on load;
Single storage boundaries round to binary32. This applies to locals, fields, native
storage, call arguments, and return values. conv.r4 explicitly rounds to binary32
before placing the value back in F; conv.r8 selects binary64. A Single expression
may carry more precision between stores or explicit conversions.

This follows the storage/internal-representation distinction in
[ECMA-335](https://ecma-international.org/publications-and-standards/standards/ecma-335/).
Binary64 is this prototype's explicit implementation choice, not a promise of identical
intermediate rounding to every .NET backend. Inferred generic/union constructors
currently see Double for F values, like the existing narrow-integer normalization
limitation; explicit typed generic construction remains future work.

ldc.r4 and ldc.r8 accept decimal/exponent literals and NaN, inf, and -inf. The assembler
rounds constants to the requested width. Temporary JSON metadata stores IEEE bits,
not JSON floating-point numbers, so special constants and signed zero round-trip:

```json
{"op":"ldc.r4","arg":{"bits":1065353216}}
```

That constant is Single 1.0. ldc.r8 uses a 64-bit unsigned bits field. These are
additive format-4 instruction operands, not a new proposed binary CLI encoding.
NaN payloads are preserved in serialized constants; payload propagation through
arithmetic or precision changes is not guaranteed.

## Arithmetic and comparisons

add/sub/mul/div/rem and neg accept F operands. Arithmetic overflow yields infinity;
floating-point division by zero produces infinity or NaN, not an integer-style
Fault. Remainder uses truncating division semantics, with the dividend's sign for
nonzero finite results; it is not the IEEE nearest-quotient remainder operation.
Signed zero and NaN propagate according to the host IEEE operations. No control of
rounding modes, floating-point status flags, or subnormal modes is exposed.

ceq treats NaN as unequal and positive/negative zero as equal. clt/cgt are false for
unordered operands; clt.un/cgt.un are true if either operand is NaN, or the indicated
ordered relation holds. The new cgt/cgt.un also support existing integer categories.
Comparison results remain the prototype's Boolean values.

ckfinite preserves a finite F value and otherwise terminates with an instruction-
located Fault. It is the explicit finite-result check; ordinary arithmetic does not
implicitly perform it. Unsigned integer division/remainder, checked integer arithmetic,
and bitwise/shift instructions do not accept F. Mixed integer/F arithmetic requires
an explicit conversion.

## Conversions and memory access

conv.r4/r8 convert signed integer stack values or F values. Integer-to-Single
conversion rounds directly to binary32 to avoid an intermediate binary64 double-
rounding error. conv.r.un interprets integer bits as unsigned and converts to F;
it rejects a floating-point operand. Values larger than the mantissa's exact integer
range may round, as expected for floating-point conversion.

Existing integer conversion instructions now accept F, truncating toward zero.
For unchecked conversion results that CLI leaves unspecified, this interpreter
explicitly chooses destination-range saturation and NaN-to-zero. This includes the
small integer destination ranges before stack widening. This policy is not a claim
of bit-for-bit agreement with every .NET implementation's out-of-range result. See
[conv.i4](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.conv_i4).
Checked conversion instructions now diagnose these cases; see
[checked conversions](checked-conversions.md).

ldind.r4/r8 and stind.r4/r8 access matching Single/Double pointers. ldobj/stobj also
support both types. Single stores round to binary32; loads widen to F. Existing
initialization, type, null, bounds, alignment, and lifetime checks apply. Explicit
pointer casts permit interpretation of the stored IEEE bits as integer storage.

See the [floating-point sample](../examples/floating.neoil). Decimal, Half, math
library coverage, full comparison/branch opcode coverage, full native marshalling, and the
remaining verifier work are separate follow-ups. This slice changes no allocation
or ownership policy.
