# Resolved function invocation

LoadedProgram can resolve a static or instance IL function without an entry point and return a
LoadedFunction handle borrowing that program's immutable snapshot:

```rust
let function = program.resolve_function(
    &neoclr::assembler::parse_function_ref("Double(Int32)")?,
)?;
let result = function.invoke(vec![neoclr::Value::Int32(21)], neoclr::Limits::default())?;
```

Resolution accepts symbolic signatures or explicit definition rows, including scoped
generic owners and exact revision selectors. It requires closed signatures and obeys
the root's direct module-reference list. The handle retains the selected specialized
function and its definition identity; repeated invocation does not relink or reselect
that target. Rust borrowing ties the handle to its source LoadedProgram. Identity,
receiver, parameter, and return-type accessors expose the resolved contract without mutation.

## Argument and execution contract

Inputs include primitives (numeric types, Boolean, String, Error, and inhabited Void),
[validated owned records](record-inputs.md), and bootstrap Option/Result values.
Arguments must be their exact storage Values. A Byte parameter
requires Value::Byte, not Value::Int32; Single requires Value::Single, not Double.
Guest ldarg still performs the existing evaluation-stack normalization, and stores
and returns retain their existing conversions. This distinguishes the typed host
boundary from the normalized evaluation stack used by IL call instructions.

Arity and argument values are checked before executing any instruction. A Void
parameter still requires one Value::Void argument. Raw pointers and prototype Ref
values cannot be supplied as arguments or receivers, including inside records or
Option/Result alternatives. See [bootstrap union inputs](union-inputs.md) for case
and payload validation. Direct native/InternalCall targets are rejected during
resolution; IL wrappers can call native declarations using the existing instruction contracts.

Each invocation starts fresh frames, allocations, output, and instruction/frame limits.
There is no synthetic guest caller frame or wrapper instruction charge. A Fault does
not change the handle or program; subsequent invocations can proceed. Instruction
Faults retain the actual guest function/instruction context. Argument faults identify
the function without inventing an instruction location. Execution Faults also preserve
[owned stack snapshots](stack-traces.md); input and resolution faults have no guest trace.

The result is the existing Execution, with a precise stored return Value. Return types
are not restricted to primitive types: a guest function may produce records, Result,
Option, or pointers using the runtime's implemented operations. Its returned allocations
and native-library retention belong to that Execution. These results are not transferable
guest handles into a later invocation. Supported owned results can be imported
as data with validation; pointer/Ref transfer and persistent state need separate contracts.

Safe invoke disables native imports. The separate unsafe invoke_with_native method
retains the existing native ABI/trust requirements. Foreign code may maintain its own
process-global state. Resolution and invocation do not automatically run the opt-in
typed verifier; callers can verify the program before resolving or invoking.

## Instance receivers

Resolve an instance signature and supply its receiver separately:

```rust
let method = program.resolve_function(
    &neoclr::assembler::parse_function_ref("instance Box<Int32>::Get()")?,
)?;
let result = method.invoke_instance(box_value, vec![], neoclr::Limits::default())?;
```

`receiver_type()` returns the specialized owner type for instance methods and None for
static functions. `parameters()` excludes the receiver. Static invoke methods reject
instance targets; invoke_instance methods reject static targets. Receiver validation
uses the same exact owned-input schema as parameters, including scoped tags,
closed generics, and shared schema limits. Faults distinguish the receiver from declared
argument indices, which start at zero. In guest IL, `ldarg 0` / `ldarg this` still reads
the receiver, and declared parameters follow it.

The receiver moves into a fresh execution as an owned value. Clone it first to retain
an independent host copy. Guest field updates and starg operate on guest values and do
not write back to the caller; a method can return an updated record for explicit reuse.
This does not introduce addressed receivers, shared references, automatic allocation,
or automatic constructor invocation. A resolved `.ctor` remains an ordinary instance Void method
called explicitly on a supplied value; no construction invariant is inferred.

Safe invoke_instance disables native imports. Unsafe invoke_instance_with_native has
the same native ABI/trust requirements as invoke_with_native. Both use the ordinary
interpreter frame and instruction budgets with no synthetic wrapper.

`cargo run --example instance_invocation` invokes Box<Int32> methods, preserving the
original Int32(21) while returning an updated copy containing Int32(42).

## Host execution controls

All invoke methods also accept [ExecutionOptions](cancellation.md) in place of Limits
for cooperative cancellation. Input validation still precedes execution, and cancellation
returns a terminal Fault without invalidating the handle or its program.

## Architectural scope

This is a Rust embedding experiment using the shared interpreter call semantics. It
does not establish a C ABI, a universal Value layout, JIT/AOT backend interface, native
exports, persistent session, callback protocol, or memory-management policy. Future
backends must preserve the logical typed argument/result and Fault contracts while
choosing their own calling convention and representation.

`cargo run --example invoke` loads a library with no entry point, prints Hello, world!,
and invokes Double twice to produce Int32(42) and Int32(60).
