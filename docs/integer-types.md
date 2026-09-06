# Fixed-width integer storage and execution

The primitive set now includes System.SByte, Byte, Int16, UInt16, Char, UInt32,
Int64, and UInt64 alongside the existing Int32 and native integers. Aliases are
int8, uint8, int16, uint16, char, uint32, int64, and uint64. Canonical definitions
live in the platform-written System library; general primitive member APIs remain
incomplete. No primitive representation selects allocation or ownership policy.

## Storage and evaluation-stack categories

| Storage type | Native size | Loaded stack value |
| --- | --- | --- |
| SByte | 1 | Int32, sign-extended |
| Byte | 1 | Int32, zero-extended |
| Int16 | 2 | Int32, sign-extended |
| UInt16 / Char | 2 | Int32, zero-extended |
| Int32 / UInt32 | 4 | Int32, same bits |
| Int64 / UInt64 | 8 | Int64, same bits |

Eight-byte integers use host u64 alignment; other fixed-width integers use alignment
equal to their size. Native memory uses host byte order. Char is one UTF-16 code
unit and permits surrogate code units. This does not choose String's encoding; see
[text direction](text-model.md). It is not a validated Unicode scalar value.

Loads from locals, arguments, fields, and native storage produce the stack category
above. Callee return values are normalized when passed back to a caller. Stores to
locals, parameters, fields, native memory, and return values use declared storage
types: Int32 truncates into one/two-byte slots or preserves bits in UInt32; Int64
preserves bits in UInt64. Stored records retain each field's declared type. The host
Execution result represents the declared return type, including a narrow integer.
Boolean remains the prototype's distinct Boolean stack value.

This follows the CLI separation of storage and evaluation-stack representations;
for example [ldind.i1](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.ldind_i1)
loads a signed byte as Int32. Mixed-width arithmetic still needs explicit conversions.
Native signed/unsigned stack normalization, general generic argument handling, and
Boolean stack normalization are not yet complete. In particular, the current
inferred some/ok/heap.new constructors see normalized stack types; they cannot infer
Byte or UInt64 ownership/union payload types from those bits alone. Typed generic
construction will need explicit signatures rather than guessing storage types.

## Instructions

ldc.i8 accepts signed decimal literals from -9223372036854775808 through
9223372036854775807. To obtain unsigned high-bit patterns, use conversions or store
the corresponding signed bit pattern in UInt64. There is no floating-point step.

conv.i1/u1/i2/u2 truncate and sign/zero-extend into Int32. conv.u4 retains 32 bits.
conv.i8 sign-extends narrower integer stack values; conv.u8 zero-extends their bit
patterns. Both produce Int64 stack values. Existing native conversions now also
accept Int64, truncating when the host native width is smaller. Checked conversions are implemented; see [checked conversions](checked-conversions.md).

Int64 supports the existing wrapping, signed/unsigned checked arithmetic, division,
and comparison instructions. Arithmetic uses the opcode's signedness, not the storage
signature. Overflow Faults only for checked arithmetic (and signed division's
minimum/-1 case); division by zero always Faults. Floating-point arithmetic is now supported; see [floating-point rules](floating-point.md).
Checked conversions are also available; see [checked conversions](checked-conversions.md).

Indirect instructions now cover ldind.i1/u1/i2/u2/i4/u4/i8/i and
stind.i1/i2/i4/i8/i. A typed pointer must name a member of the corresponding storage
family: byte, short (including Char), 32-bit, 64-bit, or native integer. Load opcode
signedness selects sign/zero extension; stores use the destination type and truncate
where appropriate. Native indirect operations currently preserve the declared native
signedness, matching the existing native-integer prototype. Cast a pointer explicitly
to access another family. ldobj/stobj work for all the newly supported storage types.
The existing null, alignment, bounds, initialization, and lifetime checks still apply.

The [sample](../examples/integers.neoil) demonstrates byte truncation, signed versus
unsigned reads, exact 64-bit storage, and explicit cleanup. These operations do not
add reference counting, GC, or automatic resource destruction.


## Bitwise operations, shifts, and remainder

and/or/xor operate on two matching integer stack categories. not complements all
bits, and neg uses wrapping two's-complement negation: negating the signed minimum
returns the same bit pattern without a Fault. This follows the [CLI neg instruction](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.neg).
These operations accept Int32, Int64, IntPtr, and UIntPtr. Boolean remains a separate
prototype category and is not accepted by integer bitwise instructions.

shl shifts in zero bits; shr extends the sign bit; shr.un shifts in zero bits from
the high end. Opcode signedness controls behavior even for UIntPtr values. The count
can be Int32 or either native integer representation; it need not have the value's
width. Int64 counts are not accepted. The result preserves the shifted value's
category, and shifted-out bits are discarded.

The interpreter explicitly masks counts to five bits for Int32, six for Int64,
and the corresponding native width for native integers. Thus shifting an Int32 by
32 acts like shifting by zero, and a count of -1 acts like 31. This is a deterministic
prototype choice for cases the [CLI shift contract leaves unspecified](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.shl),
not a claim that CLI requires masking on every platform. Source compilers that need
a particular portable shift policy should emit it explicitly.

rem computes a remainder with the dividend's sign; rem.un interprets both operands
as unsigned bit patterns. Zero divisors Fault. The signed minimum rem -1 also Faults
in this interpreter, consistently across hosts, rather than allowing a host-language
panic. CLI implementations have historically differed on that boundary; the
[rem documentation](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.rem)
permits an overflow failure on Intel platforms. No catchable exceptions are introduced.

The [bits sample](../examples/bits.neoil) packs and extracts fields and exercises
signed/unsigned remainder and right shifts. Masking, native widths, invalid operand
types, and exceptional arithmetic boundaries are covered by integration tests.
