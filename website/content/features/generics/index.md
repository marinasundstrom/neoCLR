# Generics

Generics let a type or method describe an operation once while retaining the types
chosen by each caller. neoCLR materializes generic definitions with their type
arguments. Reflection on a closed construction exposes the substituted fields,
parameters and results. Open definitions preserve their parameters and constraints.

Like .NET, neoCLR keeps declaration identity separate from generic arguments and
checks constraints when constructing types and methods. Its structural function
types add a different option: a generic argument can describe an entire callable
signature without naming a delegate class.

## Generic types keep their element types

Collections are a familiar example. `ArrayList<string>` exposes string elements;
`ArrayList<int>` exposes integer elements. Their shared definition does not require
callers to store everything as Object and cast it back.

```raven
let values = ArrayList<string>(2)
values.Add("First")
let copy = values.Copy()
copy[0] = "Changed"
```

This excerpt comes from the [generic collection sample](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/raven-target/samples/library-generic-collections.rvn).
Copy creates another sequence; it follows each element's normal value/reference
semantics. Generics do not imply deep copying. See [Collections](../collections/).

User-defined classes, structs, records and unions can also carry type parameters.
Fields and members retain those parameters in the definition, then substitute the
chosen arguments. A reference-type container still has object identity; a value-type
container still copies its fields. That choice is independent of being generic.
`Optional<T>` and `Result<T, E>` use ordinary library-defined generic unions to
retain payload types across their cases; see [Outcomes](../outcomes/).

## Generic methods and inference

A generic function or method has its own type parameters, separate from those of
its containing type. A function can accept and return the same `T`, or transform
one type into another. Raven can infer method type arguments from supported call
arguments; callers can also supply them explicitly. Constraints check whether the
chosen arguments are valid, rather than selecting a different runtime type on the
caller's behalf. Constructor inference and advanced inference cases have their own
compiler limits; an explicit construction such as `ArrayList<string>()` is clear.

Generic arguments are retained in metadata and runtime type identity. Two closed
constructions of the same declaration with different arguments are different types.
An open definition may be inspected without pretending it is an instantiable closed
type. Partially substituted signatures preserve unresolved components.

## Constraints describe what a body may rely on

Raven's neoCLR target carries constraints into native metadata. Reference-type,
value-type, constructor and nominal class/interface bounds restrict admissible
arguments. A nominal bound lets a body use the supported members of that contract.
For example, this declaration is drawn from the [imported constraint tests](https://github.com/marinasundstrom/neoCLR/blob/main/docs/validation/imported-special-constraints/raven/Library.rvn):

```raven
public open class ReferenceBox<T> where T: class {
    private var value: T
    init(value: T) {
        self.value = value
    }
    func Get() -> T => value
}
```

Runtime construction and generic call resolution also validate constraints; metadata
import must preserve them across assembly boundaries. Constraints do not promise
that every .NET generic operation or constraint combination is implemented.

## Variance describes safe interface conversions

Declared `out` and `in` interface parameters support the qualified reference-only
variance profile. A producer can expose a wider reference view, while a consumer
can accept an appropriate narrower view. Value arguments require exact matching;
variance does not silently box values or make writable collections covariant.
Invariant wrappers remain boundaries, including the current `Result` and `Sliceable`
contracts. See the [variance design and validation](https://github.com/marinasundstrom/neoCLR/blob/main/docs/design/interface-variance.md)
for supported combinations and open reflection limits.

## Reflection and execution

Generic definitions retain parameter identities, constraints and declared member
signatures. Closed reflection substitutes the relevant owner and method arguments.
The runtime and native compilation paths materialize the concrete signatures and
storage they need; this is not a promise of the CLR's exact code-sharing strategy
or allocation/performance behavior. Open generic relationship/member queries and
native execution still have documented profile limits.

## Function types as generic arguments

**New development capability:** use matching neoCLR and Raven toolchain revisions.
The initial function-dependent method profile is intended for interop and library
authors; it is not available when Raven targets ordinary .NET.

### Constrain a parameter to a function

`where T: func` requires a structural function type. Different arities and return
types can satisfy the constraint. It does not imply that every function returns
unit or that its parameters can be called directly from an open generic body.

`ReturnType<T>` projects the result type of a function-constrained parameter.
It remains an open type expression until substitution supplies enough information.
A partially constructed function can still contain generic components.

The typed reflection helper uses this capability: `FunctionReflection.Invoke<T>`
accepts a value of `T` and an `object?[]`, and its result is `ReturnType<T>`.
For example, a `(int, int) -> int` argument produces an `int` result. Nullable
reference arguments remain null and value arguments are boxed in the array.
Reflection validation failures use the runtime's existing terminal failure path;
handler faults propagate. Task results receive no special treatment.

### Materialize a whole method signature

A generic definition can declare `func name: T with args`. The name is ordinary;
`T` supplies the complete public signature when the containing type closes. The
implementation binds `args` as an owned `object?[]`, excluding the receiver.

The initial profile supports nonvirtual methods whose signature comes from a
function-constrained parameter of the declaring type. The open body must produce
`ReturnType<T>`; it cannot assume an arbitrary value or unit is valid for every T.
An explicit `default(ReturnType<T>)` is available for libraries that establish
their own result policy.

`MethodInfo.FunctionType: TypeInfo` exposes the authoritative signature type.
An open whole-signature declaration exposes its constrained parameter. Once closed,
it exposes a `FunctionTypeInfo`, and ordinary parameter/result reflection works.
Querying unresolved parameters or results fails rather than inventing an empty
parameter list. This differs from .NET's fixed generic method parameter-list shape.

### Library consumers

The separate experimental `Neo.Signals` extension uses these capabilities to expose
subscription-only `Signal<T>` properties while retaining a `SignalSource<T>` for
publishing. It accepts unit-returning handlers, preserves registration order and
returns disposable subscription handles. No event metadata is needed. Its eventual
inclusion in the runtime has not been decided.

The implementation currently pays for an argument array, boxing where needed and
reflection dispatch. JIT/AOT optimizations and richer signature constraints remain
future work. Completion, reactive errors and Task behavior belong to separate
library policies.

See [function types](../functions/), [introspection](../introspection/) and the
[design and validation record](https://github.com/marinasundstrom/neoCLR/blob/main/docs/design/generic-function-methods.md).

Execution qualification: the signal consumers pass in the interpreter. The native
compiler builds and shares signature specialization, but the signal JIT probe stops
at unsupported boxing of inhabited unit (`Void`). JIT/AOT execution of this library
is not qualified; unit boxing/unboxing support remains follow-up work.
