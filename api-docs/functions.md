# Function shapes and objects

A Function type describes a callable shape. A
Function object is an instance of that shape, holding a checked method target and
any bound receiver or captured environment. This page documents the structural
family; “Function” is a documentation label, not a nominal type declaration.

Function types are another data-composition family: their structure is the callable
signature rather than a nominal declaration. They retain their own Object and
invocation contract; that does not impose reference semantics on tuples or unions.
See [all structural families](structural-types.md).

## Members

| Member | Contract |
| --- | --- |
| [Invoke(P0, …): R](#invoke) | Calls the bound target with this shape's parameters and result. |
| [Function: MethodInfo](#bound-target-property-development) | Read-only property describing the closed bound target. |
| [ToString(): String](#object-members-development) | Module-qualified target name and closed signature. |
| [Equals(Object?): bool](#object-members-development) | Compares shape, closed target and receiver identity. |
| [GetHashCode(): int](#object-members-development) | Hash consistent with value equality. |

GetMethods exposes Invoke, get_Function and the three synthesized Object overrides.
GetProperties exposes Function. Object's GetType reports the actual signature even
through an Object view. Applicable extension members are defined by their extension
containers; they do not give the shape a nominal name.

## Shape

Raven uses `(P0, P1, ...) -> R` (or `() -> R` without parameters). Ordered
parameter types and result type determine the shape. The native model also
preserves readonly/reference/output contracts and the no-result calling convention.
Matching display text alone does not establish identity: component types retain
their loaded identities. There is no named function type facility in this slice.

```raven
let first: (int) -> int = AddOne
let second: (int) -> int = Double
```

Both objects have the same structural shape while referring to different methods.
Raven's current importer admits zero through four input parameters through CLI
Func/Action metadata. Those carriers are compiler transport; they do not produce
nominal delegate declarations in the runtime. Source unit results use the inhabited
unit, including a generic result instantiated with unit. Binding a no-result
method inserts the necessary unit-result adapter.

## Invoke

For shape `(P0, ..., Pn) -> R`, `Invoke(arg0: P0, ..., argn: Pn) -> R` calls the
bound method and returns its result. Ordinary call syntax, such as `first(41)`,
uses the same operation. Arguments must match the parameter contracts, and binding
an incompatible method is rejected during compilation or artifact validation.
The object retains its receiver/environment across collection. Frame-bound
references cannot escape through a retained Function object.

Structural types can have members and extension members. For example:

```raven
public extension FunctionExtensions for (int) -> int {
    func Twice(value: int) -> int {
        return self(self(value))
    }
}
```

This adds an extension to the shape without giving the type a nominal name.
The [migration fixture](https://github.com/marinasundstrom/neoCLR/tree/main/docs/experiments/function-types)
contains the complete source examples and tracks their current validation.

## Introspection and limits

`typeof((int) -> int)` returns common TypeInfo. IsNominalType is false; the
descriptor has no NominalTypeInfo or MemberInfo view. DisplayName is diagnostic
shape text. Member queries remain part of TypeInfo; absence of a declaration name
does not prohibit members or extensions. FunctionTypeInfo exposes Parameters and ReturnType directly. The
[structural family browser](structural-types.md) includes this authored member
reference alongside the generated metadata reference.

The transport carrier types are explicitly excluded from generated type pages;
member signatures retain their matching CLI documentation IDs. Function equality compares shape, closed method and retained receiver identity.
Copies share captures; they do not copy a receiver. Default Function slots contain
null and Invoke faults with NullReference. Constructors must initialize their
Function fields before publication. Function objects inherit Object, support Object views and cast-back, and preserve
their precise signature through GetType. Separate bindings have separate reference
identity even when value-equal; copies retain reference identity.
There is no multicast combination or native function-pointer interop contract.
Use FunctionComparer and FunctionEqualityComparer to adapt callable values to
comparison contracts. The legacy `.delegate`, `delegate.bind` and serialized
Delegate encodings are not accepted by this runtime contract.

Common TypeInfo member queries include the synthesized public instance `Invoke`.
`TypeInfo.IsFunctionType` identifies its structural signature descriptor. Narrow to
`FunctionTypeInfo` for direct `Parameters` and `ReturnType` access. Its
`InvokeMethod` property provides the member view of the same signature, so callers
can choose either path. No generalized function-info interface is introduced. Repeated queries
compare equal but need not share wrapper identity. Parameter names are empty and
positions and modes come from the signature.

Invoke and its parameters have no declaration module or metadata token: their
optional properties return None, as does MethodInfo.DefinitionIndex. DeclaringType
returns Some(the Function signature), and custom attributes are empty. Ordinary
members expose optional metadata too; consumers must pattern-match it.
This descriptive method does not support MethodReflectionExtensions.Invoke yet;
use ordinary typed Function invocation to execute the bound target.

## Future nominal function types

A future nominal function type may inherit an eligible structural Function shape.
Two nominal function types sharing that shape would remain distinct and could not
directly convert to each other solely on signature equality. Non-nominal types are
not generally inheritable; permitted kinds need explicit rules. This proposal does
not make Array, Tuple, Union or Intersection inheritable. Conversion/rebinding rules
remain future design work.

## Bound target property (development)

Every Function signature synthesizes a read-only public instance property
`Function: MethodInfo`. Read it on a Function object to inspect the actual bound
method/function. The signature's `FunctionTypeInfo.InvokeMethod` continues to
represent `Invoke`; it is not the bound target. `GetProperties()` exposes Function,
and `GetMethods()` includes its `get_Function` accessor as well as Invoke.
All synthetic methods have absent module, token and definition-index metadata.
The synthetic property's legacy integer DefinitionIndex is -1 and is not a
metadata-table index; its module/token are absent. Dynamic execution of these
synthetic descriptors returns UnboundMetadata; use the typed property or invocation.

The target MethodInfo has its available module/token/definition information,
closed parameter and result types, and a declaring type when the target has one.
Necessary importer adapters can expose generated targets without source module/token
metadata; those optional properties return None. Module functions have no DeclaringType. Their custom attributes and dynamic
reflection invocation are not supported by the current type-based services.
The descriptor describes the method independently of a bound receiver.
Repeated reads return equal descriptors, without promising wrapper identity.
Null Function property access raises NullReference.

Same signature means same Function type. Function objects are value bindings:
the same closed target and receiver/capture identity compare equal, regardless
of when the bindings were created. Different targets, generic instantiations or
receivers differ. Equal computed results do not imply equal Function objects.
A future common FunctionInfo interface is deferred; Function returns MethodInfo now.

## Object members (development)

Function types have Object as their BaseType while IsNominalType remains false.
They synthesize virtual overrides of ToString(), Equals(Object?) and GetHashCode(),
also discoverable through GetMethods. Typed and Object calls agree: Equals compares
closed target and receiver identity, equal values hash equally, and ToString displays
the module-qualified target name with its closed signature. GetType on an Object
view returns the actual Function signature. Type tests and casts back require the
exact compatible shape; a different signature does not become callable by casting.
Object.ReferenceEquals compares binding creation identity, not value equality.

ToString does not invoke user code, inspect receiver state or serialize captures.
Lambdas may display compiler-generated target names. The diagnostic spelling may
evolve and is not an equality key. Imported source
names are retained independently of lowered implementation names. Null member access
raises NullReference. This Object inheritance contract applies to Functions;
structural identity alone does not imply the same behavior for other type families.

## Provisional dynamic invocation primitive

On `codex/function-kind-constraint`, an existing function reference can be invoked
through `neoCLR.Runtime.InvokeFunction(function, arguments)` with a non-null Object
argument array. Exact scalar boxes are unboxed; references use checked casts. Results
are boxed/reference-preserving, with null for no-result calls. This is a native
primitive and sample-local Raven facade, not a permanent class-library API.
Interpreter/JIT/AOT qualification and restrictions are recorded in the
[dynamic invocation fixture](../docs/validation/dynamic-function-invocation/README.md).
This does not turn a MethodInfo descriptor into a bound function or add multicast policy.


The native reflection slice now recovers the exact selected function shape through
an Object view's GetType. Raven `typeof((int) -> int)` emits a structural native type
token; it does not acquire a nominal Func identity. The retained sample checks
IsFunctionType, IsNominalType=false and type equality before dynamically forwarding
a callback as both an argument and a result. Interpreter/JIT/AOT return 42. This
qualification covers type identity and dynamic invocation; it does not claim new
AOT support for every MethodInfo snapshot/query or generic reflection operation.


Raven native function values can also be tested and captured using ordinary type
patterns, including generic signatures: `if obj is () -> T { }` and
`() -> T callback => callback()` in a match arm. Null and incompatible signatures
fail the test; a successful capture remains callable with its original receiver.
See the retained source consumer above. These are language operations over existing
function metadata, not additional reflection API members.
