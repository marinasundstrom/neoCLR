# neoIL assembler reference, format 2

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
- `.type Name` begins a record; `.field Name Type` declares ordered fields;
  `.end` closes the type.
- `.function Name(T0, ..., Tn) -> Type` begins a free function; parameter types
  appear inline and are addressed by `ldarg` index. `.local Type` declares local
  slots before instructions/labels; `.end` closes the function.
- Legacy `.function Name -> Type` with leading `.param Type` directives remains
  accepted. Without `.param`, this declares a parameterless function. An inline
  parameter list cannot be combined with `.param`.
- `.methodimpl InternalCall` marks a runtime-provided function with no IL body or
  locals. It lowers to `impl_flags: 4096`; ordinary functions default to zero.
  Unknown flags and missing or mismatched native bindings are rejected.
- `Label:` identifies the following instruction. Branches name labels in the same
  function; labels can be forward references. A label past the last instruction
  cannot be a branch target.
- Types are `Void`, `Int32`, `Boolean`, `String`, `Error`, a record name,
  `Option<T>`, `Result<T,E>`, or `Ref<T>`. Spaces inside generic signatures are allowed.
  Primitive aliases `void`, `int32`/`int`, `boolean`/`bool`, and `string`
  normalize to the corresponding canonical types in every type context.
  There is a nesting limit of 32 in assembly type expressions.

`.entry Main` selects the parameterless `Main()` overload, independently of
declaration order. The entry function may return any supported type.
The CLI displays the return value in a diagnostic Rust-style representation.

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

Definitions use the same parameter type list as calls:

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

## Instructions

In this table the rightmost item is the top of the evaluation stack. Every
instruction consumes its operands unless stated otherwise. `T`, `E`, and `U`
denote actual runtime types; all checks use exact type equality.

| Instruction | Stack effect | Meaning |
| --- | --- | --- |
| `ldc.i4 n` | `→ Int32` | Signed decimal 32-bit literal |
| `ldc.bool true/false` | `→ Boolean` | Boolean literal |
| `ldstr "text"` | `→ String` | String literal |
| `ldvoid` | `→ Void` | The one Void value |
| `ldarg i` | `→ T` | Copy argument at zero-based index |
| `ldloc i` | `→ T` | Copy initialized local |
| `stloc i` | `T →` | Replace typed local |
| `dup` | `T → T,T` | Copy value; references preserve identity |
| `pop` | `T →` | Discard value |
| `add`, `sub`, `mul` | `Int32,Int32 → Int32` | Wrapping arithmetic |
| `add.ovf`, `sub.ovf`, `mul.ovf` | `Int32,Int32 → Int32` | Checked arithmetic; overflow Fault |
| `div` | `Int32,Int32 → Int32` | Signed quotient truncated toward zero; zero/overflow Fault |
| `ceq` | `T,T → Boolean` | Structural equality; reference identity |
| `clt` | `Int32,Int32 → Boolean` | Left operand less than right |
| `br Label` | `→` | Unconditional branch |
| `brtrue Label` | `Boolean →` | Branch when true |
| `call Name(T0, …, Tn)` | `P0,…,Pn → R` | Call declared IL or InternalCall function |
| `ret` | `R → caller` | Return exactly one value; no extra stack items |
| `newobj Name` | `F0,…,Fn → Name` | Construct frame-owned record in field declaration order |
| `ldfld i` | `Record → T` | Copy field |
| `stfld i` | `Record,T → Record` | Produce updated record value |
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

Use `dup; is.case …; brtrue …` to preserve a union for extraction on a matching
branch. There is no implicit error propagation or unsafe successful-case assumption.

## Runtime library contracts

These are ordinary functions compiled from [System.neoil](../runtime/System.neoil).

| Symbol | Parameters | Return |
| --- | --- | --- |
| `System.Console.WriteLine` | `String` | `Void` |
| `System.Console.WriteLine` | `Int32` | `Void` |
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
and `body`, plus `impl_flags` for the implementation kind; no owning class is needed. Instruction objects use `op` and optional
`arg`. Primitive types serialize as strings; constructed types serialize as tagged
objects, for example `{"Option":"Void"}`, `{"Ref":{"Named":"Point"}}`, and
`{"Result":["Void","Error"]}`. Unknown fields and opcode names are rejected.

The loader checks declarations, duplicate names, signatures, indices, branch
targets, and symbol existence even in uncalled functions. It does not yet prove
control-flow stack consistency, all return paths, or definite initialization.
Metadata validation errors do not currently retain assembler source locations;
syntax errors carry line numbers and execution Faults carry function/PC locations.

Format 2 stores call targets as structured references rather than untyped names:

```json
{"op":"call","arg":{"name":"System.Console.WriteLine","parameters":["String"]}}
```

The parameter array is required, including `[]` for zero arguments. Return types
remain on function definitions/runtime binding contracts. The format version was bumped
because this changes the call operand schema. Format 1 JSON modules must be
reassembled from source after updating calls with explicit signatures.
