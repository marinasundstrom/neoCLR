# Structural tuple migration (development)

Author direction, 2026-10-14: continue with structural tuples and remove nominal
tuple types. This changes the earlier choice of `System.Tuple<T...>` as the neoCLR
representation. It does not direct changes to ordinary Raven/.NET ValueTuple output.

## Contract and rationale

The target is a value type whose identity is its ordered sequence of element type
identities, without a Tuple TypeDef, generated class name or generic carrier.
Position, duplicates and nesting matter: `(int, string)` differs from `(string, int)`,
`(int, int)` keeps both elements, and a nested tuple is not flattened. A one-element
metadata shape remains distinct from its element; this does not introduce source
syntax for single-element tuples. Empty source `()` retains the existing unit rule.

The assistant's migration defaults preserve mutable positional elements and value
copying, with label aliases excluded from type identity. Nominal interfaces still
require declared conformance; tuple shape does not implicitly implement arbitrary
interfaces. Synthesized positional access and deconstruction are runtime operations,
not proof of nominal interface conformance. Boxing, equality, interface contracts,
layout and tuple descriptor factories need explicit implementation and validation.

The [C# tuple reference](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/value-tuples)
(checked 2026-10-14) documents mutable value tuples, positional access and names
ignored in assignment/equality. The existing [tuple research](tuples.md) records the
ValueTuple library implementation. Preserve those useful value semantics while
replacing the nominal representation by author direction. The benefit is direct
signature identity and introspection without an arity-specific nominal family; costs
include runtime storage/GC/verifier work, synthesized member handling, backend work
and binary migration. No performance benefit is claimed.
The one-to-256-element bound is an experimental metadata resource limit, not a
new permanent source-language arity rule. Older native readers do not recognize
the new Tuple node; use matching tools rather than expecting binary equivalence
with the existing nominal tuple family.

## Sequenced migration

1. **Native signature foundation (implemented):** `Type::Tuple(Vec<Type>)` survives
   native JSON and schema-3/4 binary envelope roundtrips without nominal declarations.
   Type/method/Self substitution traverses elements and retains the depth budget.
   Scope binding preserves order, duplicates and nesting, with one to 256 elements.
   This is metadata syntax, not yet a resolved runtime tuple descriptor.
2. **Runtime values (next):** checked construction, positional reads/writes,
   deconstruction, value copying, nested/reference element lifetimes and layout.
3. **Introspection and authoring:** native host signature authoring/import, resolved
   tuple identity and `TupleTypeInfo`, synthesized members and generic substitution.
4. **Raven:** carrier-independent tuple symbols and native lowering/import, preserving
   labels and .NET target behavior behind explicit capabilities. No CLI approximation
   should be mistaken for native tuple identity.
5. **Removal:** migrate tuple source/runtime consumers, compiler mappings, generated
   artifacts, API snapshots and examples; remove nominal neoCLR Tuple declarations
   and legacy admission after the replacement consumer passes. Explicit nominal
   constructor uses must migrate; do not silently reinterpret old binaries.

Native executable admission currently rejects tuple signatures, including array
wrappers, with `structural tuple execution is not yet supported`. Enabling
`experimental-compositions` does not authorize tuple execution. Decoding a native
metadata document alone does not validate storage or executable bodies. Old nominal
Tuple signatures remain distinct and usable until stage 5. Existing installed VS
Code kit binaries remain the validated union/intersection snapshot.

## Focused evidence

`tests/structural_tuples.rs` covers declaration-free JSON/binary roundtrips, ordered
signature equality, nested type/method/Self substitution, missing arguments, depth
and arity limits, and explicit executable rejection. Use
`cargo test --test structural_tuples --features experimental-compositions`.
Native runtime tuple execution, guest reflection, Raven lowering and nominal-family
removal are still pending; these checks do not claim those stages are complete.

Validated 2026-10-14: all six structural tuple tests pass with the composition
feature enabled; the existing binary/JSON metadata equivalence test passes;
`cargo check -p neoclr-codegen` and the API snapshot check pass. No runtime tuple
execution is claimed by this metadata-only slice.
