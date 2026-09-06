# Initial semantic decisions

These decisions describe the executable prototype. They are revisable design
choices, not a frozen platform specification. The default is .NET/CLR semantics
where we have not intentionally diverged. Incomplete primitive coverage, simplified
metadata, structural equality in this prototype, and restricted parsing are not
authorization to redefine otherwise unchanged CLR behavior permanently.

## Type and storage are separate

A type describes its data, not whether it is a “struct” or “class.” There is no
value/reference flag on a type definition. A record and an integer can both be
held directly or placed in explicitly allocated native storage accessed through
`Ptr<T>`. The legacy `Ref<T>` arena is another separate storage abstraction.

Locals, arguments, and evaluation-stack slots own values. Loading an argument or
local, duplicating a value, and selecting a field copy the value. Copying a record
recursively copies its fields. Embedded `Ref<T>` values copy their identity and
`Ptr<T>` values copy their addresses, not their targets. `stfld` consumes a record and a replacement field and produces an
updated record; it does not mutate some hidden receiver identity.

A call consumes its arguments and transfers them into a new frame. `ret` transfers
one owned value to the caller. Values may leave a frame because they have no
implicit pointer back into that frame. There are no stack-address instructions or
borrows yet. This chooses copying as the initial escape policy; move semantics and
checked borrowing remain future choices.

`heap.new` transfers a value into an execution-owned arena and produces `Ref<T>`.
`heap.load` copies its current contents. `heap.store` replaces its contents with a
value of exactly the same type, visible through all aliases, and produces `Void`.
References cannot be constructed from integers or forged by guest instructions.
They are non-null and identify arena slots. The execution result retains the arena
so a returned reference remains meaningful within that result; indices have no
meaning across executions. This arena exposes no pointer arithmetic or individual
reclamation. Separately, `heap.alloc/free` and pointer instructions operate on native
storage with explicit release; see [heap and pointers](heap-and-pointers.md).

Native stack placement is not established by these semantics. A future backend
may use native stack storage, frame arenas, registers, or scalar replacement while
preserving observable copying, sharing, and lifetime behavior.

## Void, absence, and errors

`Void` has exactly one value, loaded by `ldvoid`. Every successful function call
returns one value, including a function returning `Void`. `ret` requires exactly
one stack value of the declared return type. A caller discards unused results with
`pop`. This is an intentional departure from ordinary CIL void-return stack effects.

`Option<T>` has `None` and `Some(T)` cases. `Option<Void>` therefore has two
observable values. Internally the no-payload `None` case uses the unit `Void`
value, and `ldcase None` yields that value. `Void` is not a bottom/uninhabited type;
if non-returning functions need a type later, that should be a separate `Never`.

`Result<T,E>` has `Ok(T)` and `Err(E)`. There are no exception instructions,
handlers, catch clauses, or guest unwind semantics. Errors are ordinary values;
callers inspect the case, branch, and handle or return them explicitly.
`E` can be any supported type. The bootstrap `Error` type currently stores an
error code string, with `InvalidInt32`, `DivisionByZero`, and `Overflow` produced
by numeric library operations and the parsing primitive. Structured error types remain future library work.

A Fault terminates the entire guest execution and is not catchable by guest code.
Invalid instructions/metadata, invalid types or stack use, failed invariants,
resource-limit exhaustion, and explicit `fault` are terminal. The host embedding
API returns a Rust `Result<Execution, Fault>` to report termination; this is not a
guest recovery mechanism. Execution Faults normally include function and instruction
index. Some load/limit Faults have no instruction location.

There is no guest null value or `ldnull`. Uninitialized local storage is tracked by
the host and faults on read; it does not become a default null. Future nullable
references/native interop must represent actual null-reference semantics explicitly.
General absence in BCL APIs should use `Option`, including for references when
absence is the meaning.

## Numeric behavior and equality

`Int32` is a signed 32-bit integer on every host. The ordinary `add`, `sub`, and
`mul` wrap on overflow, preserving familiar CIL arithmetic. Explicit `add.ovf`,
`sub.ovf`, and `mul.ovf` Fault on overflow, including in release builds; the Fault
replaces an exception because neoCLR has no guest exceptions. Recoverable
arithmetic uses an explicit result-returning library operation. `System.Int32.Divide`
returns an error for zero divisors and the minimum integer divided by -1.

`System.Int32.Parse` accepts the host Rust decimal `i32` grammar: an optional sign
and ASCII decimal digits with no whitespace trimming or culture handling. This is
a deliberately small contract, not .NET Parse compatibility.

`ceq` requires identical types and compares data structurally; references compare
arena identity. `clt` only supports integers. `brtrue` requires `Boolean` rather
than applying integer/reference truthiness.

## Metadata and library shape

A module has a format version, name, entry-function name, type definitions, and
function definitions. Free functions have no owner. Static and instance methods
have a declaring type; primitives resolve to canonical System definitions and can
own members. `.entry Main` selects a parameterless non-instance entry. Ordinary
instance methods currently use read-only receiver snapshots, with the receiver at
argument zero; see [type system](type-system.md) for explicit limitations.

Types have named, ordered fields unless their representation is runtime-known.
Prototype instructions index fields and locals from zero, and assembled branch
labels become absolute instruction indices within one function.

Constructed `Option`, `Result`, and `Ref` signatures are supported; general generic
definitions, constraints, interfaces, virtual dispatch, inheritance, general custom attributes,
assembly references, metadata tokens, and binary tables are not yet implemented.
Future interfaces use names such as `Enumerable<T>` and `Disposable` without an
`I` prefix. This is a naming convention, not a rule forbidding identifiers that
happen to begin with I.

Library organization should retain familiar `System`, `System.Collections`,
`System.IO`, and related areas. APIs with meaningful absence return `Option<T>`;
recoverable failure returns `Result<T,E>`. Runtime async has no representation yet:
no existing opcode is being given a speculative scheduling contract.

## Platform-written library

The System library is assembled from neoIL using the same front end as application
code and executes through guest frames. `System.Int32.Divide` checks exceptional
numeric cases explicitly and returns Errors; ordinary `div` follows signed integer
quotient semantics and Faults on those invalid cases. `System.Math.Abs` implements
its numeric logic in IL and returns `Result` for overflow. Console and parse APIs
wrap the small host primitive surface. This establishes the library boundary without
claiming a full .NET-compatible library or final binary CIL emission.

## Memory-policy independence

Rust's host memory rules do not define the guest VM. Fundamental `Ptr<T>`/`T*`
values are separate from ownership wrappers. Native heap allocation and pointer
access are implemented in a checked interpreter subset. Reference-counted `Ref<T>`
is a deferred candidate for explicit ownership,
but the implementation still uses an arena. General generic metadata and lifetime
operations are prerequisites for a library-defined Ref abstraction. No global GC or
Rust-style borrowing policy is implied. See [memory layers](memory-model.md).
