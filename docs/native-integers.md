# Native integers and address conversions

System.IntPtr and System.UIntPtr are canonical runtime primitive types with nint
and nuint aliases. They have signed and unsigned native pointer-width representations,
respectively. They carry integer bits, not ownership or allocation identities. Both
can be used in parameters, locals, records, constructed signatures, and native memory
through ldobj/stobj. Their size and alignment follow the execution host.

The System library declares these types. Their wider .NET member API, properties,
and interfaces remain to be implemented; recognizing a primitive does not imply a
complete BCL implementation. No new literal instruction is needed for small constants:
`ldc.i4 42; conv.i` produces a native signed integer.

## Integer instructions

The existing add/sub/mul instructions wrap at the operand width. Signed checked
add.ovf/sub.ovf/mul.ovf and unsigned checked .ovf.un variants Fault on overflow.
Signed div and unsigned div.un Fault on zero; signed minimum divided by -1 also
Faults. clt and clt.un compare signed and unsigned interpretations. Signedness comes
from the opcode even when the signature is UIntPtr. Comparisons produce Boolean.

Int32, IntPtr, and UIntPtr participate in these operations. The interpreter currently
requires both operands to have the same signature type and preserves that type for
arithmetic results. Mixed native signedness and mixed Int32/native arithmetic require
explicit conversions. Full CIL evaluation-stack normalization remains an implementation
gap, not an intended new platform rule.

| Conversion | Implemented behavior |
| --- | --- |
| conv.i | Int32 sign-extends to native width; native integers/pointers preserve native bits, producing IntPtr |
| conv.u | Int32 zero-extends its 32-bit representation; native integers/pointers preserve native bits, producing UIntPtr |
| conv.i4 | Integer values retain their low 32 bits, interpreted as Int32 |
| ptr.fromint T | IntPtr/UIntPtr address bits become Ptr<T> |

These conversions are unchecked; narrowing truncates. Converting a pointer to Int32
requires explicitly passing through a native integer. Float conversions and checked
conversion instructions remain pending. Native arithmetic and conversion vocabulary
follow the [CLI instruction specification](https://ecma-international.org/publications-and-standards/standards/ecma-335/)
and [conv.i documentation](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.conv_i).

## Native pointer boundary

A pointer-to-integer conversion retains address bits and loses diagnostic allocation
identity. Arithmetic on those integers has ordinary integer semantics, without
allocation-bounds checks. `ptr.fromint T` permits any native bit pattern, including
zero, and consults the interpreter's current live native allocations to recover access
tracking where possible. It prefers an exact allocation base over an adjacent
allocation's one-past address. Otherwise it accepts an interior or one-past address.

A nonzero address outside those allocations remains untracked. It can be copied,
compared, cast, stored, or converted back to an integer, but its memory cannot yet
be accessed or freed by guest instructions. Conversion does not make foreign memory
valid. Externally supplied memory access and P/Invoke remain a later slice.

Integer reconstruction does not recover an old lifetime. If a freed address has
been reused, the reconstructed pointer can identify the new live allocation. A
pointer retained directly keeps its original diagnostic identity and still Faults
on use after free. Programs must explicitly maintain allocation lifetimes; integer
addresses neither retain allocations nor prove safety.

heap.alloc accepts a nonnegative Int32, IntPtr, or UIntPtr element count and checks
native multiplication and resource limits. ptr.add accepts Int32 or IntPtr signed
byte offsets and retains its existing checked-allocation behavior. sizeof/alignof
still produce Int32 sizes; current layouts are capped to that range.

See the [executable sample](../examples/native-integers.neoil) and
[heap/pointer contract](heap-and-pointers.md). No ownership policy is introduced.
