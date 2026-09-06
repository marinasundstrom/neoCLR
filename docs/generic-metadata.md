# Generic metadata foundation

This slice implements type parameter references, constructed type references, generic
record field signatures, validation, substitution, and closed generic record values.
Generic members and library-defined Option/Result remain pending; no reflection
facility is introduced.

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
references. Type parameters may appear only within their declaring type's fields,
including under pointers, current builtin wrappers, and other constructed types.
Free-function signatures and locals must be closed. Generic methods, constraints,
variance, constructed method owners, and method bodies on generic definitions remain
pending. Validation checks metadata as well as source, including nested arity and
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
non-generic owners. The owner must be a valid closed record type, but is only an
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
Native generic layouts/allocation, methods on generic definitions, and generic
methods remain unsupported. Layout controls on generic definitions are recorded
and checked structurally; concrete layout is deferred. A recursive pointer signature
does not expand its pointee or acquire a lifetime/ownership policy.

Option, Result, Ref, and Ptr retain their bootstrap signature encodings for now.
They cannot be redeclared as generic definitions under those reserved short names.
Ordinary generic record construction is implemented. Generic members, custom
attributes, and typed access/storage support
will then enable the [union convention](unions-and-enums.md), before migrating
Option and Result into the System library. No union-specific type category or
instructions are planned.
Existing modules without the new optional metadata continue to load; old readers
will reject the new signature variants instead of interpreting them as old types.
