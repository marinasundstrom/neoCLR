# Explicit direct module references

A source can opt into direct-reference checking with one module-level directive:

```text
.module Operations
.references (Models, Utilities)
```

The metadata field is `references: ["Models", "Utilities"]`. `Some`/a present array
enables checking, including an empty array. `.references ()` permits references to
the source's own definitions and implicit System only. System can also be listed
explicitly by other modules. Repeated directives, duplicate names, self-references,
and missing referenced modules fail. Referenced modules must be supplied even when
the source does not use them; the loader does not locate or download artifacts.

Omitting the directive leaves the metadata field absent and retains the prototype's
legacy load-set visibility. JSON null has the same legacy meaning. This compatibility
mode is distinct from an explicit empty list. Older strict readers will reject the
new field. No CLI binary reference-table encoding is assigned by this slice.

## Resolution checks

Each source is checked against its own list. An application referencing Operations
does not automatically reference Models because Operations does. Circular lists are
allowed when every module is supplied. System is validated independently and cannot
depend on application or additional-library modules.

Checks cover named/constructed types recursively in fields, signatures, locals, and
typed instruction operands; selected call definitions; attribute constructors; and
entry references. Explicit `@ Module:index` call targets do not bypass the lists.
The assembler also checks qualified field-name aliases before lowering them to indices.
LoadedProgram type queries use the root module's references. Verification still
analyzes all supplied IL bodies in their own declaring contexts.

These are metadata name/reference rules, not a security boundary or a restriction on
every value that may flow through a call. A callee can return a type that its caller
does not explicitly name. Index-only field operations do not encode the author's
qualified field alias, and do not create a separate module reference after assembly.
Foreign library names in P/Invoke declarations follow the existing native import
contract; they are not guest module references.

## Remaining scope

The complete [module set](module-sets.md) remains caller-supplied, and all its modules
are validated, including unused ones. Names still share one namespace: a reference
list does not permit duplicate type names or choose between colliding definitions.
[Scoped type operands](scoped-types.md) check origin while retaining the unique-name
restriction. [Exact revision pins](module-revisions.md) are also supported.
Visibility/accessibility flags,
and side-by-side module versions remain future work.

The example under `examples/modules/` now declares Application → Operations → Models,
with Models using an explicit empty list. System remains implicit throughout.
