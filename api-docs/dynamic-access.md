# Dynamic member requirements

Development API in `System.Diagnostics.CodeAnalysis`, owned by System.Runtime.

- `DynamicallyAccessedMembersAttribute(memberTypes: DynamicallyAccessedMemberTypes)`
  declares required members on a `TypeInfo` parameter, return value (`[return: ...]`), field, property or TypeInfo method receiver. Read-only `MemberTypes`
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
proven. Generic parameters and string
resolution remain unsupported; project retention files remain available.

Nongeneric class/interface implementations should repeat the same parameter and return
requirements as their interface declarations. The native backend reports `DAM2005`
on mismatches, including missing annotations; it does not silently strengthen the
interface contract or retain extra implementation bodies. Nongeneric reference-class
overrides are checked against their nearest base declaration with `DAM2006`, including
abstract declarations and closed generic base classes of nongeneric derived classes.
Generic derived classes, generic methods/interface mappings and value-type overrides
remain outside these checks. This does not add generic-parameter annotations.

Annotated returns validate the producer and propagate declared requirements to callers.
Unknown or insufficient returned values produce DAM2001; known type tokens infer roots.

Field writes and property setter values must satisfy their declared requirements;
reads and getter returns carry them to consumers. Direct field-backed getters also
propagate the property contract to storage. Indirect field addresses warn with
DAM2007. HTTP JSON TypeInfo overloads declare the same flags as JsonSerializer.

A method-level annotation on an instance method of TypeInfo or its implementations
applies to the receiver. Static methods and unrelated instance owners reject this
target with DAM1001. Parameterless GetConstructors/GetProperties declare public
constructor/property receiver requirements; corresponding implementations match.
