# Function definition identities and call binding

Implemented subset: module-local function identities, identity-qualified references,
and call binding before generic specialization. [Type definition rows and closed keys](type-identities.md)
are also implemented; type lookup remains name-based. The bootstrap linker still
combines an application with System and [explicitly supplied dependencies](module-sets.md). General module
versioning and independently scoped type namespaces are not implemented.

## Definition identity

Each assembled function definition carries `definition: { module, index }`. The index
is its zero-based row in that source module's function table, including free functions,
static/instance methods, constructors, and native declarations. Declaration order
assigns the rows; names and parameter/local aliases do not determine those numbers.

The identity survives linking and generic specialization. Appending System's methods
to an application's linked table does not renumber System's identities. A linked
array position is an implementation index, not the definition's module-local row.
Verifier reports include both the definition identity and the linked function index.

This is not a permanent identifier across rebuilds. Reordering definitions changes
rows; module names are currently identifiers, not version/content hashes. External
artifact versioning and provenance need a future module-identity contract. Do not
cache these rows across unrelated builds or assume same-name replacement modules
are compatible merely because a row exists.

## Call references

FunctionRef retains name, owner, instance/static kind, and parameter signature, and
adds an optional `definition` identity. Ordinary source calls can remain symbolic:

```text
call Choice<Int32>::Forward(Int32)
```

When overload selection is ambiguous after substitution, an explicit identity selects
the intended definition:

```text
call Choice<Int32>::Describe(Int32) @ Example:0
call Choice<Int32>::Describe(Int32) @ Example:1
```

The operand still states the effective parameter types at the call site. The selected
row must match the name, supplied owner, instance kind, and substituted parameter signature.
Unknown identities and incompatible signatures fail; an explicit identity never
falls back to name lookup. Return types remain on definitions and are not overload
selectors. The suffix uses an unsigned 32-bit index and an exact module name.

The same suffix is accepted on attribute constructor references. Declaration headers
do not accept a suffix: their identities come from their module rows. Labels and
parameter/local/field aliases retain their existing independent index mappings.

## Binding before specialization

The linker first validates the combined module, resolves every IL call in its declaring
context, and records the selected definition on the linked call operand. This includes
unused methods and open generic bodies. Method specialization substitutes the owner
and parameter types but preserves that identity.

For example, Choice<T> may declare Describe(T) at row 0 and Describe(Int32) at row 1.
A symbolic call to Describe(T) within Forward(T) selects row 0 in the open context.
Calling Forward on Choice<Int32> therefore still calls row 0. It does not repeat name
selection against two now-identical Describe(Int32) signatures. A direct symbolic
call to Choice<Int32>::Describe(Int32) remains ambiguous; supply the definition when
that source signature alone cannot select it.

This establishes reference binding for the interpreter and verifier, and provides a
basis for native compilation to retain the same target. It does not add generic
method parameters, conversion-based overload resolution, virtual dispatch, or a
public function-handle/invocation API.

## Validation, compatibility, and scope

Source assembly writes definition identities. Existing serialized definitions without
identities remain accepted; link-time normalization derives their module-local rows.
Supplied definition identities must agree with the containing module and row before
linking. Duplicate linked identities are rejected. Application/library normalization
happens separately so the combined table cannot accidentally redefine their origins.

Symbolic call binding changes the linked execution/verification copy only. It does
not mutate the caller's Module or rewrite the source artifact. Serialized call operands
can carry explicit identities; missing identities retain symbolic resolution until
linking. Older readers that reject unknown metadata fields will reject newly emitted
identity-bearing definitions/references; no binary CLI token encoding is assigned here.

Signature guards catch mismatched references but cannot establish compatibility with
a different same-name artifact whose row/signature happens to match. Revision-qualified
module identities remain necessary before persistent cross-build handles or caches.
Type resolution and duplicate-signature rules retain the prototype's global namespace
limits. Entry selection is still name-based and parameterless.

`examples/member-identities.neoil` demonstrates an implicitly bound generic call and
an explicitly selected colliding overload. It prints generic declaration followed by
integer declaration. Tests also cover System row preservation, legacy metadata,
malformed identities, signature guards, and attribute constructor references.
