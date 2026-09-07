# Resolved function invocation

LoadedProgram can resolve a static IL function without an entry point and return a
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
parameter, and return-type accessors expose the resolved contract without mutation.

## Argument and execution contract

This subset accepts primitive input types only: numeric types, Boolean, String, Error,
and inhabited Void. Arguments must be their exact storage Values. A Byte parameter
requires Value::Byte, not Value::Int32; Single requires Value::Single, not Double.
Guest ldarg still performs the existing evaluation-stack normalization, and stores
and returns retain their existing conversions. This distinguishes the typed host
boundary from the normalized evaluation stack used by IL call instructions.

Arity and argument values are checked before executing any instruction. A Void
parameter still requires one Value::Void argument. Records, union values, raw pointers,
and prototype Ref values cannot be supplied as arguments in this slice. Instance
receivers and direct native/InternalCall targets are rejected during resolution; IL
wrappers can call native declarations using the existing instruction contracts.

Each invocation starts fresh frames, allocations, output, and instruction/frame limits.
There is no synthetic guest caller frame or wrapper instruction charge. A Fault does
not change the handle or program; subsequent invocations can proceed. Instruction
Faults retain the actual guest function/instruction context. Argument faults identify
the function without inventing an instruction location.

The result is the existing Execution, with a precise stored return Value. Return types
are not restricted to primitive types: a guest function may produce records, Result,
Option, or pointers using the runtime's implemented operations. Its returned allocations
and native-library retention belong to that Execution. These results are not transferable
guest handles into a later invocation. Persistent state and aggregate input marshalling
need separate lifetime and validity contracts.

Safe invoke disables native imports. The separate unsafe invoke_with_native method
retains the existing native ABI/trust requirements. Foreign code may maintain its own
process-global state. Resolution and invocation do not automatically run the opt-in
typed verifier; callers can verify the program before resolving or invoking.

## Architectural scope

This is a Rust embedding experiment using the shared interpreter call semantics. It
does not establish a C ABI, a universal Value layout, JIT/AOT backend interface, native
exports, persistent session, callback protocol, or memory-management policy. Future
backends must preserve the logical typed argument/result and Fault contracts while
choosing their own calling convention and representation.

`cargo run --example invoke` loads a library with no entry point, prints Hello, world!,
and invokes Double twice to produce Int32(42) and Int32(60).
