# Nested case types and generic union companions

Status: selected design direction. [Name-plus-arity identity](type-arities.md) is
implemented; nested ownership and the companion case migration remain unfinished.
The user clarified that case types
are ordinary nested types and generic unions should use a non-generic companion type.
This supersedes the earlier exploration of selectively capturing outer parameters.
No inheritance relation is implied by either nesting or union membership.

## Separate companion and carrier definitions

Use this Raven-like pseudocode shape (not accepted assembler syntax):

```text
Result.Ok<T>(value: T)
Result.Error<TError>(error: TError)

Result<T, TError>
```

There are two distinct Result definitions:

| Definition | Role |
| --- | --- |
| Result, arity 0 | Non-generic companion owning ordinary nested case types |
| Result.Ok<T> | Generic case nested under the non-generic companion |
| Result.Error<TError> | Generic case nested under the non-generic companion |
| Result<T,TError>, arity 2 | Ordinary generic carrier accepting those closed case types |

The companion introduces no outer generic parameters to capture. Ok declares T and
Error declares TError as their own ordinary generic parameters. Result<T,TError>
constructors refer to Result.Ok<T> and Result.Error<TError> through explicit signature
arguments. Carrier and companion are related by the source/library convention, not
inheritance, automatic runtime conversion or a new union type category.

For a fully qualified name, the intended shape is System.Result.Ok<Int32> alongside
System.Result<Int32,Error>. The case's declaring owner is the non-generic System.Result
companion, not a partially instantiated or closed System.Result<Int32,Error> carrier.
This avoids introducing special selective-capture rules for generic nesting.

A case value can be offered to a compatible carrier constructor, but the destination
carrier's other arguments still require context. Nesting itself neither infers those
arguments nor decides which carriers accept the case. Existing ordinary constructors
and typed queries describe acceptance. This does not create subtype/assignability rules
between the carrier and its case types or between differently constructed carriers.

The example uses the user's Error case spelling. The currently implemented prototype
uses top-level System.Err<T>, System.Ok<T>, System.Some<T> and System.None. Those have
not been renamed or moved. Any public naming change and caller migration must be
explicit when adopting companions; a nested Result.Error case is distinct from the
existing message-bearing System.Error payload type.

## Required ordinary metadata foundation

1. **Type name plus generic arity.** The same scope must support Result with zero
   parameters and Result with two parameters. This foundation is now implemented in
   definition lookup and the affected runtime paths. Distinct metadata
   definition IDs must remain authoritative; a dotted string is not sufficient identity.
2. **Actual nested ownership.** A nested definition needs a declaring-type definition
   reference, distinct from its namespace and display name. Validate owner existence,
   same-module ownership and acyclic containment. Two different declaring types can
   each contain a type with the same simple name and arity.
3. **Local generic contexts.** Case parameters belong to their own definitions. In the
   companion example, Ok's T is its local parameter zero; the carrier's constructor
   signature supplies its own T as the argument when referencing that case. No implicit
   enclosing carrier arguments are appended. General nesting inside generic outer
   types is a separate contract and must not be accidentally settled by this example.
4. **Explicit normalized references.** Assembly spelling should distinguish namespace,
   declaring owner and generic arity. A frontend may offer convenient lookup, but the
   emitted reference must resolve unambiguously before generic substitution. Update
   linking, member owners, verification, type identity and host validation together.

CLI-inspired nesting and arity concepts can remain familiar. Do not copy a legacy
binary spelling into the language as a semantic requirement. Choose the precise neoIL
and prototype JSON encoding during implementation; no new syntax or format is promised
by this design note.

## Access and ordinary union behavior

Use the current access levels. Specify effective visibility through owner chains and
any containing/nested private-member access before implementation; do not derive access
rights from matching name prefixes. An access-modifier redesign is not a prerequisite.

The generic carrier owns its representation and defines its permitted constructor
inputs and read-only queries. The non-generic companion organizes ordinary case types;
it need not contain runtime state or have an instance. No special union instruction,
base type, implicit allocation, or reflection-driven dispatch is required.

Future Map/MapError/AndThen remain ordinary library operations. Companion case types
may make their source construction and inference more convenient, but generic methods
and callable values are still separate prerequisites.

## Implementation order and proof

Same-name definitions with different generic arities are implemented. Next implement
ordinary nested ownership and generic nested definitions under the non-generic companion.
Only then move Option/Result cases and migrate their constructor/query references.
Preserve the currently working top-level carriers during those foundational slices.

The proof must include coexisting Result and Result<T,TError>, nested Ok<T> and
Error<TError>, same-name cases under distinct companions, Void payloads, Result<T,T>,
multiple nesting levels, and invalid owner/arity references. Round trips, overload
binding, module linking, accessibility, type identity, verification and host input must
agree. Keep generic-outer nesting as a separately specified follow-up if it is not
needed by the first companion implementation.

This supports Preview 1's ordinary type and union program (P3). It does not require
a high-level compiler, inheritance or broad reflection support.
