# Dynamic member requirements

Development API in `System.Diagnostics.CodeAnalysis`, owned by System.Runtime.

- `DynamicallyAccessedMembersAttribute(memberTypes: DynamicallyAccessedMemberTypes)`
  declares required members on a `TypeInfo` parameter. Read-only `MemberTypes`
  returns those flags. The attribute constructor stores metadata; runtime execution
  of the constructor is not the retention mechanism.
- `DynamicallyAccessedMemberTypes` uses standard .NET flag values. Current native
  analysis accepts `None` (0), `PublicParameterlessConstructor` (1),
  `PublicConstructors` (3), `PublicProperties` (512), and their combinations.
  Other exposed values are reserved for unsupported member categories and fail
  native analysis when requested, including `All`.

See the generated [attribute reference](api/System/Diagnostics/CodeAnalysis/DynamicallyAccessedMembersAttribute/index.html)
and [flag reference](api/System/Diagnostics/CodeAnalysis/DynamicallyAccessedMemberTypes/index.html).
[Supported flow, diagnostics and annotated runtime APIs](https://github.com/marinasundstrom/neoCLR/blob/main/docs/dynamically-accessed-members.md)
describe the current boundaries. Unknown-flow warnings mean retention has not been
proven. Generic parameters, returns, fields, receiver annotations and string
resolution remain unsupported; project retention files remain available.

Nongeneric class/interface implementations should repeat the same parameter
requirements as their interface declarations. The native backend reports `DAM2005`
on mismatches, including missing annotations; it does not silently strengthen the
interface contract or retain extra implementation bodies. Nongeneric reference-class
overrides are checked against their nearest base declaration with `DAM2006`, including
abstract declarations. Generic mappings, generic ancestry and value-type overrides
remain outside these checks.
