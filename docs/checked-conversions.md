# Checked numeric conversions

The interpreter supports conv.ovf.i1/u1/i2/u2/i4/u4/i8/u8/i/u and the .un variant of
each instruction. These are the familiar CLI checked integer conversion forms. The
intentional neoCLR difference is failure handling: overflow terminates with a Fault
and instruction location rather than raising a catchable exception.

## Source interpretation and destination range

The i/u immediately after conv.ovf selects the destination's signedness; the optional
.un suffix selects unsigned interpretation of the source integer bits. Without .un,
integer inputs are interpreted as signed at their stack width, including the existing
UIntPtr stack representation. With .un, negative-looking bit patterns can represent
large positive values. Source interpretation is not inferred from a declared local
storage type after stack normalization.

For example, Int32 -1 converted by conv.ovf.i8 becomes -1. With conv.ovf.i8.un it
becomes 4294967295. Int64 -1 fails conv.ovf.u8 but succeeds with conv.ovf.u8.un,
preserving the all-ones bits in the Int64 stack category.

| Destination suffix | Checked range | Stack result |
| --- | --- | --- |
| i1 / u1 | Signed / unsigned 8-bit | Int32 |
| i2 / u2 | Signed / unsigned 16-bit | Int32 |
| i4 / u4 | Signed / unsigned 32-bit | Int32 |
| i8 / u8 | Signed / unsigned 64-bit | Int64 |
| i / u | Signed / unsigned host native width | IntPtr / UIntPtr |

Successful conversions do not saturate or wrap. Integer checks preserve exact source
values, including 64-bit values beyond binary64's exact-integer range. Narrow results
widen to their normal stack categories; callers can store them in Byte, Int16, UInt64,
or another matching storage signature using the existing storage rules.

## Floating-point inputs

F values truncate toward zero, then undergo a destination-range check. Both signed-
and unsigned-source forms use the floating value, with no unsigned reinterpretation
of its IEEE bits. NaN, positive infinity, negative infinity, and out-of-range truncated
values Fault. For example, 255.9 converts to Byte 255; 256 fails. -0.9 truncates to zero
and therefore fits Byte. This follows the [CLI checked-conversion model](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.conv_ovf_u1_un).

The implementation uses exact power-of-two bounds with an exclusive upper limit.
Comparing against a rounded floating representation of Int64.MaxValue or
UInt64.MaxValue would incorrectly admit 2^63 or 2^64. These cases and the nearest
representable values below those limits have regression tests.

These checks differ deliberately from the interpreter's unchecked float-to-integer
conversion policy, which saturates out-of-range values and maps NaN to zero. Use a
checked instruction when those inputs must be diagnosed. Resource or input errors
that should be recoverable still need a Result-returning API that validates before
executing a faulting instruction; Fault does not introduce exception handling.

Only numeric stack categories are accepted. Boolean, pointers, strings, and aggregate
values are rejected. Convert an address to a native integer explicitly before checking
numeric range. These instructions neither validate pointer lifetime nor change ownership.

See the [sample](../examples/checked-conversions.neoil),
[integer storage rules](integer-types.md), and [floating-point rules](floating-point.md).
