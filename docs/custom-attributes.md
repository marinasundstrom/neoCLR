# Custom-attribute metadata

Implemented subset: parameterless marker attributes on type and method/function
definitions. This provides metadata for library conventions without special runtime
type categories. Attributes are data; assembling, loading, linking, and running an
annotated method do not instantiate attributes or execute their constructors.

## Assembly and metadata

```text
.type MarkerAttribute
    .method instance .ctor() -> Void
        ldvoid
        ret
    .end
.end

.type Box<T>
    .custom instance MarkerAttribute::.ctor()
    .field Value T
    .method instance Get() -> T
        .custom instance MarkerAttribute::.ctor()
        ldarg this
        ldfld Box<T>::Value
        ret
    .end
.end
```

`.custom` inside a type body attaches to that type, including when placed after a
field declaration. Inside a method or free function it attaches to that function
and must precede instructions and labels. Module, field, parameter, return-value,
and generic-parameter targets are not implemented. Attribute type names are exact;
there is no automatic Attribute suffix lookup.

TypeDef and Function each have an optional `custom_attributes` list. Each entry
contains a `constructor` FunctionRef with explicit owner, instance flag, name, and
parameter signature. The enclosing definition supplies the parent identity.
Empty lists are omitted from serialized modules, so old modules remain unchanged.
Older readers reject nonempty lists rather than silently ignoring their metadata.
There is no binary CLI custom-attribute blob writer yet.

This follows the constructor-reference direction of
[.NET custom-attribute metadata](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.metadata.customattribute?view=net-10.0).
The prototype supports no fixed or named arguments, raw value blobs, or attribute
instantiation. Those must extend this representation deliberately rather than invent
an executable attribute opcode.

## Validation and constructor members

The referenced constructor must resolve on a closed record type, be an instance
member named `.ctor`, take no explicit arguments, and return Void. Missing owners,
open types, wrong arity, static methods, other member names, unsupported arguments,
and unresolved constructors are rejected. Forward references and references into the
linked System library are supported. The definition still undergoes ordinary method
validation even though applying its attribute does not execute its body.

`.ctor` is now an accepted member name. Its declaration must be instance and return
Void; this change supplies constructor identity for metadata. `newobj Type` retains
its existing field-based record construction and does not automatically invoke a
`.ctor`. Explicit ordinary calls to a constructor still follow existing value-receiver
semantics. General constructor initialization, visibility, and verification rules
remain separate work.

Attribute references stay closed even on generic definitions; `Marker<Int32>` can
be used, but `Marker<!0>` cannot. Specializing an annotated method preserves its
attribute metadata without substituting it. No System.Attribute inheritance check
is imposed because the platform does not yet implement that hierarchy. Repeated
attributes are retained in declaration order; AttributeUsage, multiplicity, inherited
attributes, and compiler-specific target restrictions are not implemented.

## Library conventions

The platform-written System library now defines
System.Runtime.CompilerServices.UnionAttribute with an ordinary parameterless
constructor. It carries no VM behavior. A marker alone neither validates a union
member pattern nor changes layout, allocation, copying, or instruction execution.
The full [union convention](unions-and-enums.md) still needs its construction,
typed-access, and storage contracts.

InternalCall implementation flags and P/Invoke import metadata remain their existing
explicit mechanisms; arbitrary custom attributes do not enable native execution.
No guest reflection API is introduced. `examples/attributes.neoil` demonstrates
annotations on a generic type, its method, and a free function, with a constructor
that would Fault if executed, proving the example uses metadata without instantiation.
