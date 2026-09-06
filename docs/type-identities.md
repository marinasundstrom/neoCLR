# Type definition rows and closed signature identities

Type definitions now carry `definition: { module, index }` metadata. Each index is
a zero-based row in the source module's type table. Type rows and function rows are
independent tables, represented by distinct Rust types (`TypeDefId` and `MemberId`).
There is no value-type/reference-type distinction in either identity.

The assembler assigns rows in declaration order. The loader checks supplied rows
against their source module and table position; omitted identities in legacy JSON
are derived on the linked copy. Linking preserves the source rows when appending
System definitions. Duplicate linked identities are rejected. This adds a metadata
field that older strict readers will reject; no binary token encoding is assigned.

## Closed signatures

The Rust API `resolve_type_identity(module, ty)` resolves a closed signature using
the bundled System library. `resolve_type_identity_with_library(module, library, ty)`
uses a supplied System module. Both follow execution's metadata validation/linking
rules without executing code or modifying inputs. Resolving a System module directly
uses that module's definitions. Each call currently prepares a fresh resolution copy;
this is not a cached loader context or a permanent hosting ABI.

The returned `TypeIdentity` supports equality and hashing:

- Ordinary and primitive types identify a definition row with no generic arguments.
- Constructed types identify a definition row plus ordered, recursively resolved
  argument identities. `Box<Int32>` and `Box<String>` share a definition but have
  different closed identities; nested constructions retain every argument.
- Pointer signatures retain their pointee identity. The bootstrap `Ref`, `Option`,
  and `Result` signatures retain distinct structural wrapper identities until their
  library migration. This does not introduce a general union type category.

For example, given an application that declares `Box<T>`:

```rust
let signature = neoclr::assembler::parse_type("Box<Int32>")?;
let identity = neoclr::resolve_type_identity(&module, &signature)?;
```

Primitive authoring aliases such as `int32` and `System.Int32` normalize to the same
System definition. Evaluation-stack normalization does not merge identities:
Byte and Int32, for example, remain different types. Void is an ordinary resolved
definition and can appear as a generic argument or pointee.

Resolution requires a closed, canonical signature with matching generic arity and
the shared signature nesting limit. Bare generic definitions, unresolved parameters,
unknown definitions, and malformed primitive encodings fail with a Fault. Identity
resolution does not expand record fields or require an available native layout;
having an identity does not prove a type is allocatable or valid for a native ABI.

## Boundaries

These are identities within a particular set of module artifacts. Rebuilding or
reordering a definition table can change rows. Same-name replacement modules are
not proven compatible by matching rows or names. Module revision identity remains
necessary before persistent caching or cross-build handles.

Type operands still use the prototype's globally unique names. This slice does not
add module-qualified type syntax, imports, same-name types from multiple modules,
or replace interpreter value/layout keys with resolved identities. It provides the
definition metadata and structural keys needed for that future resolution work.
Function references retain their separate [binding rules](member-identities.md).
