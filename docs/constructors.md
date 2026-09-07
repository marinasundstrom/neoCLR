# Constructor invocation

Record types can define overloaded instance IL constructors returning real `Void`:

```text
.type Box<T>
    .field private Stored T
    .method public instance .ctor(T value) -> Void
        ldarg value
        newobj Box<T>
        starg this
        ldvoid
        ret
    .end
.end

// Inside a function:
ldc.i4 42
newobj instance Box<Int32>::.ctor(Int32)
```

The invocation consumes the declared arguments and produces a complete `Box<Int32>`
value. The constructor runs in an ordinary guest frame; its hidden receiver occupies
argument zero (`this`) and declared arguments start at one. Parameter/local names are
optional mappings to indices. Constructor references use the normal owner, overload
signature, generic substitution and bound member identity rules.

This first subset initializes the receiver as a whole. For a record with fields,
the receiver slot begins uninitialized. `starg this` installs a complete value of
the owner type. Reading the receiver before that, or returning without initializing
it, is a verifier error and an execution Fault. Explicit ldarga this followed by
initobj T can also initialize the whole receiver when T supports a typed default;
see [managed initialization](managed-initialization.md). The verifier requires initialization
on every incoming path to a read or return. No implicit defaults, nulls or unused
field values are supplied; initobj is an explicit initialization operation. Zero-field records already have a complete empty receiver.
Without initobj, aggregate construction still supplies every field explicitly,
including `ldvoid` for a Void field.

The constructor must return exactly one `Void` and leave no extra evaluation-stack
items. At that return boundary, construction supplies the initialized receiver to
the caller instead of the constructor's `Void`. Fault traces and frame/instruction
limits include the constructor normally. Reachability analysis follows its body.

Only instance IL `.ctor` methods on record types are construction targets. Static,
runtime-implemented and native-import methods are not supported construction targets.
Accessibility applies to the constructor and its owner. A public constructor can
initialize its own private fields; callers need not have direct field access.

Existing `newobj Type` remains aggregate construction, consuming all fields in
declaration order and checking access to every field. Ordinary `call instance
Type::.ctor(...)` still receives an already complete copied receiver and returns
`Void`; it does not write a caller's local back. These remain distinct operations.

The assembler normalizes `newobj instance Type::.ctor(...)` to the additive format-4
JSON instruction `newobj.ctor` with a normal function-reference operand. Explicit
`newobj.ctor instance Type::.ctor(...)` is also accepted. Older readers cannot execute
this new operation. This prototype encoding is not a CLI binary opcode assignment.

Construction creates an ordinary value and implies no heap placement, reference
counting or ownership policy. Ordinary methods support [byref receivers](reference-slots.md),
but construction still uses the whole-value convention above. Field-by-field
initialization, construction into supplied storage, inheritance, and destruction remain future work
in the [construction proposal](construction-and-initialization.md). Recoverable
construction failure can use an ordinary factory returning `Result`; constructor
invocation itself does not introduce exception handling or special union operations.

Run the generic constructor/property demonstration:

```sh
cargo run --locked -- run examples/constructors.neoil
```

It prints `42`, then `Constructed through a public constructor`, and returns `Void`.
The [README](../README.md) documents building, assembling, verifying and running
serialized artifacts as well as source files.
