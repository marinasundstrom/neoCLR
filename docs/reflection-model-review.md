# Introspection and reflection: planned contract review

Status: roadmap investigation, recorded 2026-09-14. No new descriptor hierarchy or
reflection execution capability is selected by this document.

The author asked that neoCLR identify its needs before expanding the reflection
model, particularly before repeating an overlapping `Type`/`TypeInfo` split. The
current `System.Type` and member descriptors remain the starting point. Small,
well-defined facts such as `Type.IsValueType` can be added independently.

## Baseline and problem

.NET exposes both `Type` and `TypeInfo`. Microsoft's documentation describes the
latter's role in the .NET Framework 4.5 / Windows Store reflection subset, the
reference-versus-definition distinction, and declared-member enumeration. This
history supplies requirements to examine; it is not evidence that every separation
is a mistake. See [TypeInfo documentation](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.typeinfo?view=net-10.0).

Inspection does not necessarily require execution. A debugger or documentation tool
can need member signatures without constructing objects or invoking methods. A
serializer can need both inspection and controlled access. AOT makes retained
metadata and available code important: .NET Native AOT requires trimming and does
not support runtime code generation. See [Native AOT limitations](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/).
Sources consulted 2026-09-14; implementation details require pinned-source research
when a concrete design is proposed.

## Questions and alternatives

- Define the minimum inspection contract: identity, type categories, signatures,
  generic arguments, inheritance, declared versus inherited members and attributes.
- Identify execution capabilities separately: construction, invocation and field or
  property access. Determine where access checks and validation belong.
- Compare one coherent descriptor API, descriptors with optional capability
  interfaces/services, and separate offline-metadata and live-runtime descriptors.
  A single API is easier to discover but can promise operations unavailable on some
  targets. Separate capabilities can express availability but increase API concepts.
- Decide whether inspecting an unloaded definition requires loading it, and how
  offline metadata identity relates to an instantiated runtime type.
- Specify the difference between an absent member, metadata omitted by a deployment,
  and an unsupported operation. Evaluate Result/Option against the existing fault
  policy; do not silently report an empty collection for unavailable metadata.
- Evaluate metadata retention and executable-code requirements for interpreted,
  future JIT/AOT and constrained targets. Portability should not require every target
  to support dynamic code generation or all reflection operations.

The review also serves the [common platform contract plan](runtime-api-plan.md#common-platform-contract-and-target-capabilities-2026-09-14):
identify which inspection guarantees belong to the core and which execution or
metadata capabilities may vary by target. These boundaries remain open.

## Exit criteria and scope

Use a type browser, a debugger and a small serializer as distinct requirements.
Prototype declared/inherited discovery, value/reference classification, closed
generics and an unavailable-capability case. Document frontend, metadata, runtime
and library responsibilities, migration costs and unresolved questions before
expanding the public contract. Research other platforms and .NET API feedback as
required by the [design process](design-research.md).

Do not add `TypeInfo` or a parallel hierarchy speculatively. This review does not
remove current APIs, commit to an interface split, implement Native AOT, or delay the
bounded preview additions.
