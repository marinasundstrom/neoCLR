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

Use `name: Type` consistently in declarations:

```text
.method static Parse(value: string) -> Result<Int32,Error>
    ldarg value
    call neoCLR.Runtime.ParseInt32(string)
    ret
.end

.function Main() -> Result<Void,Error>
    .local point: Point
    ...
.end
```

Names are optional: `Parse(string)` and `.local Point` remain valid. Named and
unnamed slots can be mixed. Legacy `.param value: string` is also accepted with
headers that omit an inline parameter list. Type-first quoted names are not syntax.

`ldarg value`, `ldloc point`, and `stloc point` resolve to numeric indices during
assembly. Numeric operands remain valid even when slots have names. Calls continue
using types only, such as `call Parse(string)`; names do not participate in overload
identity. Field operands remain numeric in this slice.

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
| `brtrue Label` | `Boolean →` | Branch when true |
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
