# neoIL assembler reference, format 3

The assembler converts readable neoIL into the JSON module format understood by
the interpreter. This is a prototype assembler, not an ECMA-335 `ilasm` replacement.
Instruction spellings are familiar where possible; operand encoding and some
stack effects intentionally differ.

```text
.module HelloWorld
.entry Main
.function Main() -> Void
    ldstr "Hello, world!"
    call System.Console.WriteLine(string)
    ret
.end
```

Directives and opcodes are case-sensitive. Blank lines and whole-line `;` comments
are ignored. Inline comments are not supported. Strings use JSON quoting and
escapes. Identifiers contain ASCII letters, digits, underscores, and dots.

- `.module Name` occurs exactly once. Executables declare one `.entry FunctionName`;
  library modules omit it.
- `.type Name` begins a type (known System primitives use runtime representation); `.field Name Type` declares ordered fields;
  `.end` closes the type. `.method static/instance Name(...) -> Type` declares
  a nested method with its own `.end`; see [type system](type-system.md).
- `.function Name(T0, ..., Tn) -> Type` begins a free function; parameter types
  appear inline and are addressed by `ldarg` index. `.local Type` declares local
  slots before instructions/labels; `.end` closes the function.
- Legacy `.function Name -> Type` with leading `.param Type` directives remains
  accepted. Without `.param`, this declares a parameterless function. An inline
  parameter list cannot be combined with `.param`.
- `.methodimpl InternalCall` marks a runtime-provided function with no IL body or
  locals. It lowers to `impl_flags: 4096`; ordinary functions default to zero.
  Unknown flags and missing or mismatched native bindings are rejected.
- `.pinvoke "library" "entry_point" cdecl` declares a native C-ABI import with no
  body or locals. It is separate from InternalCall and supports scalar/pointer
  signatures. Calls remain ordinary signature-based calls. See [native interop](native-interop.md).
- `Label:` identifies the following instruction. Branches name labels in the same
  function; labels can be forward references. A label past the last instruction
  cannot be a branch target.
- Types include `Void`, `SByte`, `Byte`, `Int16`, `UInt16`, `Char`, `Int32`,
  `UInt32`, `Int64`, `UInt64`, `Single`, `Double`, `IntPtr`, `UIntPtr`, `Boolean`, `String`, `Error`, a record name,
  `Option<T>`, `Result<T,E>`, `Ref<T>`, or `Ptr<T>` (also spelled `T*`). Spaces inside generic signatures are allowed.
  Primitive aliases `int8`, `uint8`, `int16`, `uint16`, `char`, `uint32`,
  `int64`, `uint64`, `float32`/`single`, `float64`/`double`, `void`, `int32`/`int`, `nint`, `nuint`, `boolean`/`bool`, and `string`
  and fully qualified names such as `System.Int32` normalize to canonical types.
  There is a nesting limit of 32 in assembly type expressions.

`.entry Main` selects the parameterless `Main()` overload, independently of
declaration order. The entry function may return any supported type.
The CLI displays the return value in a diagnostic Rust-style representation.

## Parameter and local names

Use `Type name` consistently in declarations:

```text
.method static Parse(string value) -> Result<Int32,Error>
    ldarg value
    call neoCLR.Runtime.ParseInt32(string)
    ret
.end

.function Main() -> Result<Void,Error>
    .local Point point
    ...
.end
```

Names are optional: `Parse(string)` and `.local Point` remain valid. Named and
unnamed slots can be mixed. Legacy `.param string value` is also accepted with
headers that omit an inline parameter list. Names are unquoted; the former `name: Type` syntax is no longer accepted.

`ldarg value`, `starg value`, `ldloc point`, and `stloc point` resolve to numeric indices during
assembly. Numeric operands remain valid even when slots have names. Calls continue
using types only, such as `call Parse(string)`; names do not participate in overload
identity. Field operands accept an index or a qualified alias such as `Point::X`.

Names are case-sensitive ASCII identifiers beginning with a letter or underscore,
followed by letters, digits, or underscores. Parameter and local name scopes are
separate; each rejects duplicates. Unknown names are assembly errors. Declarations
must precede instructions and labels.

In an instance method, `ldarg this` denotes receiver index zero; an explicitly named
parameter resolves to its declared index plus one. `this` is reserved in instance
parameter declarations. It does not add a parameter to the signature. See the
[named-slot sample](../examples/names.neoil).

## Call signatures and overloads

Every call specifies its ordered parameter types, including empty parentheses for
zero parameters:

```text
call System.Console.WriteLine(string)
call System.Int32.Divide(int32, int32)
call Initialize()
call Handle(Result<Option<Void>, Error>, Ref<Point>)
```

Overloads are identified by name plus exact ordered parameter types. Arity, type,
and parameter order all matter; return type alone cannot distinguish overloads.
There is no implicit conversion or runtime overload selection. After selecting a
target, the interpreter checks the actual argument values against that signature.
Omitting parentheses or naming a nonexistent overload is an error.

Definitions use the same parameter types as calls, with optional names:

```text
.function Describe(int32) -> string
    ldstr "integer overload"
    ret
.end
```

Multiple definitions can share a name if their parameter signatures differ; two
definitions of the same signature are rejected. Return-only overloads are a current
prototype limitation; richer assembly-level signatures remain a design requirement. See
[the overload sample](../examples/overloads.neoil).

Instance calls use `call instance Owner::Member(...)`; static methods use
`call Owner::Member(...)`. Dotted qualified-name shorthand is still accepted.
An explicit owner is encoded separately in the call reference. Instance calls
consume a receiver before the declared parameters; `ldarg 0` is the receiver and
`ldarg 1` is the first declared parameter. Receivers are read-only snapshots in this
prototype. Static and instance overloads are distinct.

## Instructions

In this table the rightmost item is the top of the evaluation stack. Every
instruction consumes its operands unless stated otherwise. `T`, `E`, and `U`
denote actual runtime types. Integer storage has CLI-style truncation and stack
normalization; other checks use exact type equality. See [integer storage](integer-types.md).

| Instruction | Stack effect | Meaning |
| --- | --- | --- |
| `ldc.i4 n` | `→ Int32` | Signed decimal 32-bit literal |
| `ldc.r4 x`, `ldc.r8 x` | `→ F` | Binary32/64 constant, loaded into internal floating-point category |
| `ldc.i8 n` | `→ Int64` | Signed decimal 64-bit literal |
| `ldc.bool true/false` | `→ Boolean` | Boolean literal |
| `ldstr "text"` | `→ String` | String literal |
| `ldvoid` | `→ Void` | The one Void value |
| `ldarg i` | `→ T` | Copy argument at zero-based index |
| `starg i` | `T →` | Replace typed argument in the current frame |
| `ldloc i` | `→ T` | Copy initialized local |
| `stloc i` | `T →` | Replace typed local |
| `dup` | `T → T,T` | Copy value; references preserve identity |
| `pop` | `T →` | Discard value |
| `add`, `sub`, `mul` | `N,N → N` | Wrapping integer arithmetic |
| `add.ovf`, `sub.ovf`, `mul.ovf` | `N,N → N` | Signed checked integer arithmetic; overflow Fault |
| `and`, `or`, `xor` | `N,N → N` | Bitwise logic |
| `not` | `N → N` | Bitwise complement |
| `neg` | `N → N` | Wrapping two's-complement negation |
| `shl`, `shr`, `shr.un` | `N,Int32 or Native → N` | Left, arithmetic-right, or logical-right shift; count masked to width |
| `rem`, `rem.un` | `N,N → N` | Signed/unsigned remainder; zero divisor Faults |
| `div` | `N,N → N` | Signed quotient truncated toward zero; zero/overflow Fault |
| `add.ovf.un`, `sub.ovf.un`, `mul.ovf.un` | `N,N → N` | Unsigned checked arithmetic; overflow Fault |
| `div.un` | `N,N → N` | Unsigned quotient; zero Fault |
| `clt.un` | `N,N → Boolean` | Unsigned comparison |
| `conv.i` | `Integer or Ptr<T> → IntPtr` | Native signed conversion |
| `conv.u` | `Integer or Ptr<T> → UIntPtr` | Native unsigned conversion |
| `conv.ovf.i1/u1/i2/u2/i4/u4/i8/u8/i/u` | `Integer or F → destination stack category` | Check signed source against destination range; overflow Fault |
| `conv.ovf.i1/u1/i2/u2/i4/u4/i8/u8/i/u.un` | `Integer or F → destination stack category` | Same check with unsigned integer source interpretation |
| `conv.r4`, `conv.r8` | `Integer or F → F` | Convert/round to binary32 or binary64 |
| `conv.r.un` | `Integer → F` | Convert unsigned integer interpretation |
| `ckfinite` | `F → F` | Fault on NaN or infinity |
| `cgt`, `cgt.un` | `N,N or F,F → Boolean` | Greater-than; unsigned integer or unordered floating comparison for .un |
| `ldind.r4`, `ldind.r8` | `Ptr<Single or Double> → F` | Load matching floating-point storage |
| `stind.r4`, `stind.r8` | `Ptr<Single or Double>,F →` | Store matching floating-point width |
| `conv.i1/u1/i2/u2/u4` | `Integer → Int32` | Truncate then sign/zero-extend to stack width |
| `conv.i8/u8` | `Integer → Int64` | Signed/unsigned widening or retain 64 bits |
| `conv.i4` | `Integer → Int32` | Retain low 32 bits |
| `ptr.fromint T` | `IntPtr or UIntPtr → Ptr<T>` | Interpret native address bits |
| `ceq` | `T,T → Boolean` | Structural equality; reference identity |
| `clt` | `N,N → Boolean` | Signed left operand less than right |
| `br Label` | `→` | Unconditional branch |
| `brtrue Label` | `condition →` | Branch when true, nonzero, or non-null |
| `brfalse Label` | `condition →` | Branch when false, zero, or null |
| `beq Label`, `bne.un Label` | `T,T →` | Branch on equality or inequality (including unordered floats) |
| `bgt Label`, `blt Label`, `bge Label`, `ble Label` | `N,N →` | Signed integer or ordered floating comparison branch |
| `bgt.un Label`, `blt.un Label`, `bge.un Label`, `ble.un Label` | `N,N →` | Unsigned integer or unordered floating comparison branch |
| `switch (Label, ...)` | `Int32 →` | Branch by zero-based index; otherwise fall through |
| `call Name(T0, …, Tn)` | `P0,…,Pn → R` | Call declared IL or InternalCall function |
| `ret` | `R → caller` | Return exactly one value; no extra stack items |
| `newobj Name` | `F0,…,Fn → Name` | Construct frame-owned record in field declaration order |
| `ldfld i` | `Record → T` | Copy field |
| `stfld i` | `Record,T → Record` | Produce updated record value |
| `sizeof T` | `→ Int32` | Byte size of supported native layout |
| `alignof T` | `→ Int32` | Native layout alignment |
| `localloc` | `integer → Ptr<Byte>` | Allocate uninitialized frame-local bytes; stack must otherwise be empty |
| `heap.alloc T` | `Integer → Ptr<T>` | Allocate uninitialized storage for count elements |
| `heap.free` | `Ptr<T> → Void` | Free allocation base; null is a no-op |
| `ptr.null T` | `→ Ptr<T>` | Actual null address |
| `ptr.cast T` | `Ptr<U> → Ptr<T>` | Reinterpret target type, preserving address |
| `ptr.add` | `Ptr<T>,Int32 or IntPtr → Ptr<T>` | Signed byte offset; checked prototype bounds |
| `ldflda i` | `Ptr<Record> → Ptr<T>` | Address field at zero-based index |
| `unaligned. n` | `→` | Prefix a supported memory access with alignment 1, 2, or 4 |
| `ldobj T` | `Ptr<T> → T` | Copy initialized value from native storage |
| `stobj T` | `Ptr<T>,T →` | Copy value into native storage |
| `initobj T` | `Ptr<T> →` | Zero a supported native layout without a constructor |
| `cpobj T` | `Ptr<T>,Ptr<T> →` | Copy initialized value from source to destination |
| `initblk` | `Ptr<T>,Int32,integer →` | Fill a byte range with the low byte of the value |
| `cpblk` | `Ptr<T>,Ptr<U>,integer →` | Copy a byte range and initialization state (overlap supported) |
| `ldind.i4` | `Ptr<Int32 or UInt32> → Int32` | Indirect 32-bit load |
| `stind.i4` | `Ptr<Int32 or UInt32>,Int32 →` | Indirect 32-bit store |
| `ldind.i1/u1/i2/u2/u4` | `Ptr<Integer> → Int32` | Load indicated width with signed/unsigned interpretation |
| `ldind.i8` | `Ptr<Int64 or UInt64> → Int64` | Load 64 bits |
| `ldind.i` | `Ptr<Native> → Native` | Load native integer |
| `stind.i1/i2` | `Ptr<Integer>,Int32 →` | Truncate into byte/short storage |
| `stind.i8` | `Ptr<Int64 or UInt64>,Int64 →` | Store 64 bits |
| `stind.i` | `Ptr<Native>,Native →` | Store native integer |
| `heap.new` | `T → Ref<T>` | Explicitly allocate shared identity |
| `heap.load` | `Ref<T> → T` | Copy heap contents |
| `heap.store` | `Ref<T>,T → Void` | Replace heap contents |
| `some` | `T → Option<T>` | Construct Some |
| `none T` | `→ Option<T>` | Construct None with explicit element type |
| `ok E` | `T → Result<T,E>` | Construct Ok; operand specifies error type |
| `err T` | `E → Result<T,E>` | Construct Err; operand specifies success type |
| `is.case Case` | `Union → Boolean` | Test Some/None or Ok/Err; invalid case family Faults |
| `ldcase Case` | `Union → Payload` | Extract matching case; mismatch Faults; None yields Void |
| `error "code"` | `→ Error` | Construct bootstrap error value |
| `fault "message"` | `→ termination` | End guest execution |

`F` is represented by binary64 internally. add/sub/mul/div/rem, neg, ceq, clt, and
clt.un also accept floating-point operands; .un comparisons test unordered values.
Floating division by zero produces infinity/NaN, not the integer Fault. Integer
conversions accept F with truncation; see [floating-point rules](floating-point.md)
for rounding and the explicit unchecked out-of-range policy. All 20 checked integer
conversion forms are described in [checked conversions](checked-conversions.md). Float constants serialize as
`arg: {"bits": unsignedInteger}` to preserve non-finite values in JSON.

Here `N` is Int32, Int64, IntPtr, or UIntPtr; binary operands currently require matching
types. Signedness comes from the opcode, not the type name. This subset does not yet
implement full CIL evaluation-stack normalization or mixed-width arithmetic. See
[native integers](native-integers.md) for conversions and address tracking.

Use `dup; is.case …; brtrue …` to preserve a union for extraction on a matching
branch. There is no implicit error propagation or unsafe successful-case assumption.

## Runtime library contracts

These are ordinary functions compiled from [System.neoil](../runtime/System.neoil).

| Symbol | Parameters | Return |
| --- | --- | --- |
| `System.Console.WriteLine` | `String` | `Void` |
| `System.Console.WriteLine` | `Int32` | `Void` |
| `System.Int32.ToString` | instance receiver `Int32`, no parameters | `String` |
| `System.Int32.Parse` | `String` | `Result<Int32,Error>` |
| `System.Int32.Divide` | `Int32, Int32` | `Result<Int32,Error>` |
| `System.Math.Abs` | `Int32` | `Result<Int32,Error>` |

The library uses three host primitives: `neoCLR.Runtime.WriteLine(string)`,
`neoCLR.Runtime.Int32ToString(int32)`, and `neoCLR.Runtime.ParseInt32(string)`.
They have explicit `.methodimpl InternalCall` declarations in the System module. Duplicate library signatures are rejected at link
time; other overloads of the same names are allowed. Console output is buffered
until successful execution. See [library design](runtime-library.md).

## Serialized metadata

The assembler emits a versioned JSON object containing `format`, `name`, `entry`,
`types`, and `functions`. Functions carry `name`, `parameters`, `returns`, `locals`,
and `body`, plus `impl_flags`, `owner`, and `instance`. Free functions have no
owner; methods have a validated declaring type. Type definitions carry `representation`. Instruction objects use `op` and optional
`arg`. Primitive types serialize as strings; constructed types serialize as tagged
objects, for example `{"Option":"Void"}`, `{"Ref":{"Named":"Point"}}`, and
`{"Result":["Void","Error"]}`. Unknown fields and opcode names are rejected.

The loader checks declarations, duplicate names, signatures, indices, branch
targets, and symbol existence even in uncalled functions. It does not yet prove
control-flow stack consistency, all return paths, or definite initialization.
Metadata validation errors do not currently retain assembler source locations;
syntax errors carry line numbers and execution Faults carry function/PC locations.

Call targets store as structured references rather than untyped names:

```json
{"op":"call","arg":{"name":"System.Console.WriteLine","owner":{"Named":"System.Console"},"instance":false,"parameters":["String"]}}
```

The parameter array is required, including `[]` for zero arguments. Return types
remain on function definitions/runtime binding contracts. Format 3 adds declaring-type and instance-call semantics. Older modules and System
libraries must be reassembled. Pointer signatures use `{"Ptr":"Int32"}` and imply
no ownership or automatic memory management. Pointer instructions are additive
format-3 opcodes; older runtimes reject them as unknown instructions. See
[heap and pointers](heap-and-pointers.md) for layout, native-address representation,
checked interpreter restrictions, and the remaining native pointer capabilities.

Optional `parameter_names` and `local_names` arrays preserve declaration names
separately from type signatures. Entries are strings or null for unnamed slots.
Parameter names exclude the implicit receiver. A nonempty name array must have the
same length as its corresponding type array; an omitted or empty array means all
slots are unnamed. For example:

```json
{"parameters":["String"],"parameter_names":["value"],"locals":["Int32"],"local_names":["count"]}
```

This is an additive format-3 metadata extension. Earlier format-3 artifacts without
names still load; serialized opcode operands are unchanged. The eventual binary
backend can map parameter names to parameter metadata and local names to appropriate
local/debug metadata without putting names into instruction operands.

See [memory operations](heap-and-pointers.md#copying-and-initializing-memory) for block counts, alignment, pointer tracking, and zero-length behavior.

See [frame-local allocation](heap-and-pointers.md#frame-local-allocation) for `localloc` lifetime, alignment, and limits.


## Switch tables and conditional branches

`switch (Zero, One, Other)` consumes one Int32 and selects the corresponding
zero-based target. Targets can repeat and refer forward or backward within the
current function. An out-of-range index falls through to the next instruction;
negative Int32 values are interpreted as unsigned for selection and therefore
fall through for ordinary tables. `switch ()` is valid and consumes its index
without branching. Older evaluation-stack entries are preserved.

Parentheses are required, labels are separated by commas, and trailing commas are
rejected. Undefined labels receive source-line diagnostics. The JSON prototype
encodes `switch` with an `arg` array of absolute instruction indices, checked at
load time even for unreachable instructions. This follows the existing branch
representation. A future CIL writer must translate the table to relative byte
offsets from the instruction after the switch, as specified in
[Microsoft's switch reference](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.switch).

`brfalse Label` consumes a condition and branches for false or zero; `brtrue`
branches for true or nonzero. Supported conditions are Boolean, Int32, Int64,
IntPtr, UIntPtr, native pointers, and the prototype Ref arena references. Small
integer and unsigned storage types normalize to their usual integer stack category
before branching. Every current Ref is non-null, including arena index zero.

Pointer conditions test only the numeric address. A non-null pointer can be foreign,
one-past-end, or stale and still test true. This instruction neither dereferences it
nor establishes that it is safe to access. Both paths consume exactly one condition
and preserve older stack entries. These zero/null tests follow the familiar
[CLR conditional branch contract](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.brfalse).

String, records, Error, unions, Void, and floating-point values are not conditions.
neoCLR does not infer reference semantics for value types based on their .NET names,
or infer truth from string length or Option/Result cases. Use `is.case` for unions
and explicit comparisons for floats. Managed byrefs and short-form branch aliases
remain pending. Tables do not add implicit default branches, union destructuring,
or a static stack verifier. Stack types and instruction budgets remain checked during
execution. See `examples/control-flow.neoil` for pointer testing, an integer-controlled
loop, and multi-way dispatch.


## Argument assignment

`starg index` or `starg name` consumes one value and replaces an argument slot in
the current invocation. It emits a numeric argument index, with the same name
resolution as `ldarg`. In instance methods, `this` is slot zero and explicit
parameters start at one. Under neoCLR's existing value receiver semantics,
`starg this` replaces the callee's receiver value; it does not mutate the caller's
record. A free/static function has no implicit `this` slot.

Stores use the declared argument type. Small integer values truncate to their
storage width, Single rounds to binary32, and subsequent `ldarg` normalizes back
to the evaluation-stack category. Other values require the matching storage type.
These rules follow the [CLR argument-store contract](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.starg).
A Void argument can be replaced with the real Void value, consistent with neoCLR's
inhabited Void type.

Replacing a pointer argument changes the pointer slot, not the pointed-to storage,
and never frees or retains the allocation. Replacing an ordinary argument does not
write back into a caller's local or argument. Pointee writes still use the separate
memory instructions. Invalid indices are rejected during validation, including in
unreachable code; bad types and stack underflow Fault at the executing instruction.
The `starg.s` spelling is supported; argument address-taking (`ldarga`) remains pending.

The `examples/arguments.neoil` sample sums down by updating its `count` argument,
then demonstrates that the caller's local retains its original value. It prints
6 followed by 3.


## Comparison branches

Comparison branches pop the right operand, then the left, and compare left against
right. They push no result and preserve any older stack entries on both the taken
and fallthrough paths. Every opcode accepts a label and retains its own opcode in
the temporary metadata, with an absolute instruction-index target. Forward and
backward targets are validated exactly like other branches. Short forms remain pending.

`beq` and `bne.un` use the same exact-type value equality as the existing `ceq`:
values compare by value and references by identity. Matching-target pointers compare
addresses without dereferencing. This also retains the prototype's structural equality
for records/unions and value equality for strings; it does not introduce CLR object
reference semantics for those values. NaN is unequal to every floating value, including
itself, so `beq` falls through and `bne.un` branches. Positive and negative zero compare
equal.

The ordered branches support matching Int32, Int64, IntPtr, UIntPtr, or internal
floating-point operands. Opcode spelling determines integer signedness. Plain `bgt`,
`blt`, `bge`, and `ble` use signed integer comparisons and reject unordered floating
comparisons (fall through for NaN). Their `.un` forms compare integers as unsigned
and branch when either floating operand is NaN. In particular, floating `bge` is
not simply an inverted ordered less-than test: it must also reject NaN. This follows
the CLR contracts for [bge.un](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.bge_un)
and [ble](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.ble).

Mixed numeric stack categories require explicit conversions. Pointer ordering and
managed-byref comparisons remain unsupported; convert a pointer to a native integer
explicitly if address ordering is intended. Unsupported operand types and stack
underflow Fault at execution. Run `examples/comparison-branches.neoil` for checks of
all ten opcodes, including unsigned integer and unordered floating cases.


## Compact constant and slot spellings

The assembler accepts these CIL spellings as aliases for existing instructions:

| Source spelling | Canonical instruction | Operand rule |
| --- | --- | --- |
| `ldc.i4.m1`, `ldc.i4.0` through `ldc.i4.8` | `ldc.i4` | No operand; value is in the opcode |
| `ldc.i4.s n` | `ldc.i4 n` | Signed decimal value from -128 to 127 |
| `ldarg.0` through `ldarg.3` | `ldarg index` | No operand |
| `ldloc.0` through `ldloc.3` | `ldloc index` | No operand |
| `stloc.0` through `stloc.3` | `stloc index` | No operand |
| `ldarg.s`, `starg.s`, `ldloc.s`, `stloc.s` | Corresponding full instruction | Index or name resolving to index 0 through 255 |

The limits match the CLI's signed-byte constant and unsigned-byte slot operands;
see [ldc.i4.s](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.ldc_i4_s)
and [ldarg.s](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.ldarg_s).
A slot must also exist in the signature or local table. Instance argument indices
still include `this` at zero. There is no `starg.0` opcode; use `starg.s 0` or
`starg.s this` in an instance method.

Each spelling emits exactly one canonical instruction in JSON. Source spellings
and byte-width hints are not retained, and this does not produce a compact binary
file yet. A future CIL writer will choose encodings from the canonical instructions.
Short branch forms still require binary offset handling and remain pending.
Labels, instruction budgets, and stack behavior are identical to the full spelling.
The `examples/compact.neoil` sample prints 42 using compact constants and slot access.


## Identifier mappings and field aliases

Indices are the normalized addresses for parameters, locals, and record fields in
this prototype. Identifiers are optional assembly conveniences, resolved in their
own context. Parameter names are unique within the parameter table of a function;
local names are unique within its separate local table; field names are unique
within their declaring type. The same spelling can appear independently in each
scope. These are assembly/metadata names and need not correspond to identifiers in
a higher-level source language. They do not participate in runtime slot lookup.

`ldfld Point::X`, `stfld Point::X`, and `ldflda Point::X` resolve X's declaration
index in Point. The owner is mandatory for named field operands, so no evaluation
stack type inference is required. Types declared later in the same module are
supported. Unknown owners/fields and malformed aliases produce source-line errors.
Names are case-sensitive. Numeric operands remain valid.

The emitted operand is just the field index, identical to authoring that index
directly. The qualifier selects the mapping table during assembly; it adds no runtime
receiver-type assertion. The interpreter applies the index to the actual record
operand, using its existing bounds and storage checks. This is the temporary
prototype's index representation, not yet a CLI FieldDef/MemberRef token. General
external-module field resolution remains pending. Parameter/local name tables and
field declaration names may remain in metadata for tools; instruction execution
uses indices exclusively.

Record declarations accept `.pack n` and `.size n`; see [sequential layout controls](heap-and-pointers.md#sequential-packing-and-size-controls) for alignment, minimum-size, and native-access rules.

See [unaligned access](heap-and-pointers.md#unaligned-memory-access) for supported prefix targets and control-flow restrictions.

Generic record headers use `.type Pair<T, U>` (or indexed unnamed parameters such
as `!0`); field signatures resolve names to indexed type parameters. See
[generic metadata](generic-metadata.md) for supported references and current execution limits.

Unions will follow an [ordinary type convention](unions-and-enums.md), expressed
through metadata and member calls. No `.union`/`.variant` directives or general
union opcodes are implemented. Existing Option/Result instructions remain bootstrap
facilities until library types can replace them.
