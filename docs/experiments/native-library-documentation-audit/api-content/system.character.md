---
uid: T:System.Character
---
## Grapheme identity and migration

`Character` is one Unicode extended grapheme cluster, preserving its original
UTF-8 spelling. It is an immutable value type; equality, hashing and ordering use
canonical Unicode equivalence. An explicit `StringComparison` selects ordinal
comparison when byte identity matters. Storage can still allocate: value semantics
do not promise allocation-free grapheme construction.

Character literals infer this type. Use `Character` for explicit annotations;
neoCLR no longer has a predefined `char` alias. This replaces the development
`System.Char` API and requires matching compiler and library artifacts. Raven on
.NET continues to use `System.Char` and UTF-16 code-unit semantics.

See [the tested text examples](/features/strings/) for iteration and comparison.
