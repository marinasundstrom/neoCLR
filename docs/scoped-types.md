# Scoped type operands

Type operands can now identify a source module explicitly:

```text
.references (Models)
.local [Models]Box<[System]Int32>* storage
call [Models]Box<Int32>::Create(Int32) @ Models:0
ldfld [Models]Box<Int32>::Value
```

The bracket names a neoCLR module, not a .NET assembly. Existing unqualified syntax
remains supported. Qualifiers apply to a named definition; generic arguments and
pointer suffixes retain their own structural signatures. `[System]int32` normalizes
the primitive spelling to System.Int32. Primitive qualifiers must name System.
Use `Ptr<[Models]Point>` or `[Models]Point*` for pointers; a bootstrap wrapper such as
Option itself is not a named definition that can receive a qualifier.

Scoped signatures serialize as a `Scoped` type operand containing `module`, canonical
`name`, and `arguments`. Empty arguments describe a non-generic type; generic arity
is checked during loading. Scope is supported in fields, parameters, locals, returns,
typed instructions, method-owner references, and attribute constructor references.
It is not accepted in type declaration names or as a qualifier on a generic parameter.

## Binding

Preparation normalizes definition rows in their source modules, checks every scope,
and lowers valid references to the interpreter's current unique-name signatures.
A type found in another module does not satisfy the qualifier: `[Wrong]Box<Int32>`
fails even if Box exists elsewhere. Scoped primitive metadata must use its canonical
name; the assembler expands spelling aliases. Signature nesting, generic arity, and
closedness rules otherwise remain unchanged.

The original source artifacts keep their scoped operands, including after JSON
roundtrip. Binding changes the loaded copy only. Generic specialization preserves
the selected function definition and substitutes type arguments as before. A scoped
method owner and an explicit function row must both agree with the selected method.

Qualifiers do not grant access: [direct reference lists](module-references.md) still
apply after binding. Group assembly checks field-owner scopes before converting aliases
to indices. LoadedProgram type queries accept scoped operands and produce the same
closed identity as a correctly resolved unqualified signature.

## Current limits

The supplied set must still have globally unique type names. Qualifiers check origin
in this slice; they do not yet allow two modules to define the same type name. The
interpreter's stored values and native layouts still use normalized name-based keys.
Supporting duplicate names requires moving those internal keys to scoped definition
identities throughout execution. Optional [revision labels](module-revisions.md) now
distinguish declared artifacts; side-by-side versions remain separate work.

Lower-level helpers that operate directly on raw Module/Type values expect resolved
signatures; prepare a LoadedProgram for execution, verification, and scoped identity
queries. No new opcode, memory ownership policy, or binary CLI token is introduced.
Older readers without the Scoped variant will reject these new operands.

The three-module example uses scoped Box construction, method calls, and field aliases:
`cargo run --example modules` verifies the set and prints 42.
