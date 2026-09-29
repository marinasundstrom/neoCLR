# Union

Current Raven `union` declarations are named, nominal types. Their cases and members
are documented on the individual declaration pages. A future structural union of
types is a separate design direction; it does not rename or reclassify these APIs.

## Current members and cases

- [Option&lt;T&gt;](xref:System.Option`1) documents Some and None.
- [Result&lt;T, E&gt;](xref:System.Result`2) documents Ok and Error.

RavenDoc renders each declaration's cases, properties, methods and applicable
extensions from its metadata. Use those member pages for the concrete API; there
is no universal member list shared by every declared union.

Structural union identity, common-member lookup, conversion and introspection
construction remain future contracts. This page introduces no synthetic runtime
Union declaration and no general right to inherit a union shape.

[All structural families](structural-types.md)
