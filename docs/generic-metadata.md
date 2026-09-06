# Generic metadata foundation

This slice implements type parameter references, constructed type references, generic
record field signatures, validation, substitution, and closed generic record values.
Static and instance IL methods on generic types are also implemented. Library-defined
Option/Result remain pending; no reflection facility is introduced.

```text
.type Pair<Left, !1>
    .field First Left
    .field Second !1
    .field Nested Option<Left>
.end
```

Type parameters have zero-based indices. Named entries are optional aliases; `!1`
in the declaration above declares an unnamed parameter at index one. In fields,
`Left` and `!0` encode the same parameter reference. Named entries must be unique
within the declaration and obey slot-name syntax. Primitive type spellings are
reserved and cannot be parameter aliases. Parameter names do not change type identity.
Use explicit indices to avoid depending on names from a higher-level language.

Metadata adds `TypeParameter(index)` and `Constructed { definition, arguments }`
signatures. Definitions carry an optional `generic_parameters` name table, whose
length is their arity. Ordinary definitions omit this table. This prototype still
uses one type definition per canonical name; same-name definitions of different
arities are not supported. Type parameter indices use unsigned 16-bit values.

Closed references such as `Pair<Int32, String>` require a known generic definition
and exactly the declared number of arguments. Bare generic names are not closed
references. Type parameters may appear within their declaring type's fields and methods,
including under pointers, current builtin wrappers, and other constructed types.
Free-function signatures and locals must be closed. Method-level generic parameters,
constraints, and variance remain pending. Validation checks metadata as well as source, including nested arity and
parameter context; type nesting is limited to 32 levels.

The Rust embedding helper `Module::instantiated_fields` resolves the field signatures
of a closed record type by substituting arguments for parameter indices. Substitution
preserves nested types and pointer layers without expanding referenced definitions,
so a field such as `Node<T>*` does not cause recursive expansion. Void is accepted
as an ordinary type argument. The original definition remains unchanged. The lower
level `Type::substitute_type_parameters` helper substitutes a supplied argument list
and rejects missing parameter indices. These are implementation APIs, not guest
reflection facilities.

## Closed generic record values

`newobj Pair<Int32, String>` consumes the substituted field types in declaration
order and produces one value with the complete closed type identity. The same rules
apply to non-generic records. Small integer and floating-point fields use the normal
storage conversions; `ldfld` restores their evaluation-stack representation.
`stfld` returns an updated copy and checks the substituted field's storage type.
Nested generic records, zero-field generic records, and Void fields are supported.
No native allocation or ownership operation is implied by construction.

Field aliases such as `Pair<Int32, String>::First` resolve to indices, just as for
non-generic owners. The owner must be a valid record type (closed outside generic methods), but is only an
assembly mapping: no owner assertion is retained in the normalized instruction.
Numeric field operations act on the actual record value. Closed generic values can
be used in locals, parameters, return values, and free-function overload signatures;
`Pair<Int32, String>` and `Pair<String, Int32>` remain distinct types.

The interpreter's Object value now stores a Type rather than a definition-name
string. This is a Rust embedding API change. The serialized `newobj` operand retains
the legacy name string for non-generic records; constructed operands use structural
Constructed signatures, including indexed argument order. Legacy modules still load.
Older readers reject the new structured operands rather than erasing type arguments.

`examples/generic-values.neoil` demonstrates construction, field aliases, independent
copies, and a function accepting a closed generic value. The earlier
`examples/generic-metadata.neoil` demonstrates closed generic pointer signatures.
Native layouts and allocation now support closed generic records whose substituted
fields have supported layouts. Method-level generics remain unsupported. A recursive
pointer signature does not expand its pointee or acquire a lifetime/ownership policy.

Option, Result, Ref, and Ptr retain their bootstrap signature encodings for now.
They cannot be redeclared as generic definitions under those reserved short names.
Ordinary generic record construction and members on generic types are implemented.
Marker custom attributes are implemented. Typed access/storage support
will then enable the [union convention](unions-and-enums.md), before migrating
Option and Result into the System library. No union-specific type category or
instructions are planned.
Existing modules without the new optional metadata continue to load; old readers
will reject the new signature variants instead of interpreting them as old types.

## Methods on generic types

Declare ordinary `.method static` or `.method instance` members inside `.type Box<T>`.
The declaring type supplies the parameter context: T and !0 normalize identically
in parameter/return signatures, locals, and typed instruction operands. Metadata
keeps the definition owner as Box and its signatures open. Call operands instead
identify a constructed owner, such as `call Box<Int32>::Create(Int32)` or
`call instance Box<Int32>::Get()`. Inside another generic method the owner/arguments
can reference that method's declaring type parameters, e.g. `Box<!0>::Create(!0)`.
Bare generic owners and omitted generic call owners are rejected.

The interpreter substitutes the owner's arguments into the method, including nested
call signatures, construction, pointer, and memory operands. Each frame executes a
closed method copy; stored definitions remain open and unchanged. Instance argument
zero (`this`) has the exact closed owner type. Receivers remain values: returning an
updated receiver does not mutate the caller's original. Existing frame/instruction
limits apply to recursion. The prototype does not yet cache specializations.

Overload resolution uses the closed owner, static/instance kind, and substituted
parameter types. If substitution makes two overloads identical, the call is rejected
as ambiguous rather than selecting declaration order. The current call representation
does not yet carry a separate definition token to disambiguate those overloads.
Return types do not distinguish overloads. Generic owners cannot currently declare
InternalCall or P/Invoke methods, and an open method cannot be the module entry point.

Validation checks every method's open signatures, indices, branch targets, and call
references, including unused methods. At execution, substituted types must be closed
and satisfy nesting limits. Native layout operations involving a type parameter are
checked when executed with its concrete argument: `sizeof !0` works for Int32 and
Faults for String, which has no native layout. This is not a generic-constraint or
full stack-verification implementation. See `examples/generic-methods.neoil` for
creation, extraction, and updates using methods on Box<T>.

## Native layouts for closed generic records

`sizeof`, `alignof`, `heap.alloc`, `ldobj`, `stobj`, `cpobj`, and `initobj` accept
closed generic records whose fields support native storage. `ldflda` produces a
pointer to the substituted field type; qualified aliases still normalize to indices.
Packing and minimum size apply to each closed instantiation using the existing
sequential layout rules. `Box<Byte>` and `Box<Int32>` can therefore have different
sizes and alignment. Loads reconstruct the exact closed type, including nested
records and zero-sized Void fields. Allocation remains separate from construction.

Layout recursion tracks closed type identities. Finite nesting such as
`Box<Box<Int32>>` is supported; direct/mutual recursion by value is rejected.
Expanding generic recursion is bounded by signature nesting, layout depth, and
complexity limits. Recursive pointers have native pointer size and do not expand
their pointees. String, Error, bootstrap unions, and Ref still lack native layouts,
so a closed generic record containing one by value cannot be allocated natively.
This does not add a native ABI for passing records by value through P/Invoke.

Typed loads/stores preserve initialization checks, packing/alignment rules, and
pointer provenance. A pointer copied through a generic record does not extend the
pointee's lifetime. `initobj` uses the supported fields' existing zero representations;
it does not run a constructor. See `examples/generic-memory.neoil` for allocation,
field addresses, and a size query executed through a method on a generic type.
