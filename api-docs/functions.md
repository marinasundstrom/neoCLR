# Function shapes and objects

Development after Preview 11. A Function type describes a callable shape. A
Function object is an instance of that shape, holding a checked method target and
any bound receiver or captured environment. This page documents the structural
family; “Function” is a documentation label, not a nominal type declaration.

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
does not prohibit members or extensions. Dedicated parameter/result introspection
and a general RavenDoc structural-family renderer remain open.

The transport carrier types are explicitly excluded from generated type pages;
member signatures retain their matching CLI documentation IDs. Function equality compares shape, closed method and retained receiver identity.
Copies share captures; they do not copy a receiver. Default Function slots contain
null and Invoke faults with NullReference. Constructors must initialize their
Function fields before publication. Ordinary Object conversion and a separate
Function allocation identity are not supported in this slice.
There is no multicast combination or native function-pointer interop contract.
Legacy `.delegate`, `delegate.bind` and serialized Delegate representations are
rejected. Rebuild applications with matching compiler, reference, library and
runtime artifacts. Comparer adapters are now named FunctionComparer and
FunctionEqualityComparer.

Common TypeInfo member queries remain available. The synthetic Invoke operation
is documented on this family page; it has no declaration token and is not returned
by the existing declaration-based GetMethods implementation. General structural
member discovery remains future work.
