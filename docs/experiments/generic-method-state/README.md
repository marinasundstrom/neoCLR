# Generic method-state groundwork

Development slice, 2026-10-21. See the [.NET contract comparison](../../introspection-design.md#generic-method-state-development-2026-10-21).

Adds MethodInfo.IsGenericMethod, IsGenericMethodDefinition and
ContainsGenericParameters, distinguishing owner openness from method genericity.
Open method signatures use source-owned method parameter identities; name equality
alone does not imply type identity. Native snapshot tokens authenticate descriptors
against source declarations without creating executable generic instantiations.

The built-in `method-state` suite is deliberately opt-in and interpreter-only until
native type-method discovery has an explicit retention policy. This slice does not
claim AOT/JIT GetMethods support, MakeGenericMethod, generic implementation retention,
or reflective method invocation. Existing native module-function snapshot recipes
share the MethodInfo layout. Runtime/backend and class-library versions must match;
Raven compiler 793b71408 is unchanged.

Validation: all seven built-in tests pass on the interpreter (one method-state,
six type-relations). The runtime identity, module snapshot and legacy reflection
regressions pass, as do all three native open-query regressions (including generated
machine code). See contracts.json and the accompanying focused logs. API snapshot
validation passes; the website builds and checks 5,580 pages. These checks do not
qualify native GetMethods. No performance measurement is claimed.
