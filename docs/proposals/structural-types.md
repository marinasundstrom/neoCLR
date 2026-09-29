# neoCLR Structural Types, Function Types, and Callable Objects

## 1. Summary

neoCLR should support **structural types as first-class runtime types** alongside nominal types.

A structural type does not require a nominal type declaration in metadata and does not require a metadata token identifying its type or members. Instead, its structure, members, assignability rules, interface implementations, and runtime behavior are provided by the neoCLR type system.

Examples of structural types may include:

```text
T[]
(T1, T2)
A | B
A & B
(A, B) -> R
```

Structural types may nevertheless:

- implement nominal interfaces;
- expose synthesized members;
- participate in generic constraints;
- participate in introspection;
- optionally inherit from or be assignable to `Object`;
- have runtime-defined operations that lower directly to neoIL instructions;
- have different physical representations without changing their logical type model.

Being structural does **not** imply being outside the normal type system.

The central principle is:

> Nominal types describe themselves through metadata. Structural types are described by the type system.

Whether a type is nominal or structural is separate from whether it implements interfaces, exposes members, or derives from `Object`.

---

# 2. Nominal and Structural Types

neoCLR distinguishes between two broad categories of types.

## 2.1 Nominal types

Nominal types have declared identity.

Examples include:

```text
class
struct
interface
enum
named union
```

Their declarations and members are represented in metadata and can normally be identified using metadata tokens.

Two nominal types are different because they have different identities, even if their structures happen to be identical.

For example:

```raven
class Point {
    X: int
    Y: int
}

class Coordinate {
    X: int
    Y: int
}
```

`Point` and `Coordinate` remain distinct types.

---

## 2.2 Structural types

Structural types are constructed by the type system rather than introduced through nominal declarations.

For example:

```text
int[]
(int, string)
Stream & Seekable
Error | Timeout
(int, string) -> bool
```

Their identity is derived from their structure.

For example:

```text
(int, string)
```

can be identified from:

```text
Tuple(
    int,
    string
)
```

without requiring a generated type such as:

```text
TupleOfIntAndString
```

in metadata.

Likewise:

```text
(Request) -> Response
```

does not require a generated nominal delegate type.

---

# 3. Structural Types Still Participate in the Type System

Structural types must not be treated as second-class types.

They can participate in:

- assignability;
- interface implementation;
- generic arguments;
- generic constraints;
- reflection/introspection;
- member resolution;
- overload resolution;
- pattern matching;
- runtime type checking.

For example, an array could be understood as:

```text
int[]

Structural: yes
Base: Object
Implements:
    Iterable<int>
    Collection<int>
```

while a tuple could be:

```text
(int, string)

Structural: yes
Base: none
Implements:
    Equatable<(int, string)>
```

The fact that both types are structural does not require them to have the same inheritance behavior.

---

# 4. `Object` Is Independent of Structural Typing

neoCLR should not define:

> Every type derives from `Object`.

Nor should it define:

> Structural types cannot derive from `Object`.

Instead, object compatibility is an independent type-system property.

For example:

```text
class Foo          <: Object
int[]              <: Object
(int, string)      </: Object
A | B              </: Object
A & B              </: Object
```

The exact rules remain type-specific.

This allows `Object` to retain meaningful runtime semantics rather than becoming synonymous with "any possible neoCLR value."

If neoCLR eventually requires a true universal type, that concept should be distinct from `Object`.

Conceptually:

```text
Any
 ├── Object-compatible values
 ├── tuples
 ├── unions
 ├── intersections
 └── other runtime values
```

The naming and necessity of such a universal abstraction remain open.

---

# 5. Structural Members

Structural types may expose members even though those members do not exist as metadata declarations.

For example:

```raven
array.Length
tuple.Item1
tuple.Deconstruct(...)
function.Invoke(...)
```

The compiler should not recognize these merely through hard-coded member names.

Instead, the neoCLR type system should report that these members exist.

Conceptually:

```text
MemberInfo
├── MetadataMemberInfo
└── StructuralMemberInfo
```

A metadata member may have:

```text
MetadataToken
```

while a structural member has an identity derived from its structural definition.

For example:

```text
Array.Length
    kind = ArrayLength

Tuple.Item1
    kind = TupleElement(0)

Tuple.Deconstruct
    kind = TupleDeconstruct

Function.Invoke
    kind = FunctionInvoke
```

This allows all neoCLR languages to observe the same type model without reproducing Raven-specific conventions.

---

# 6. Structural Members and neoIL

A structural member does not necessarily correspond to a method call.

Member resolution and lowering should remain separate operations.

For example:

```raven
array.Length
```

may resolve as:

```text
StructuralMember:
    ArrayLength
```

and then lower to an intrinsic neoIL operation:

```text
ldlen
```

or its neoCLR equivalent.

Similarly:

```raven
function(args)
```

may resolve against the synthesized invocation operation of a function type and lower directly to a function-call instruction.

Therefore:

> The type system defines that an operation exists. The compiler/runtime defines how that operation is executed.

This avoids manufacturing nominal methods solely to make operations visible to compilers.

---

# 7. Arrays

Arrays are a strong candidate for structural types.

For example:

```text
int[]
string[]
Customer[]
```

should not require generated nominal array types.

The runtime constructs an array type from its element type and other relevant structural properties.

An array may nevertheless be object-compatible:

```text
T[] <: Object
```

and implement nominal interfaces:

```text
T[] : Iterable<T>
T[] : Collection<T>
```

depending on the final standard library design.

---

# 8. What Happens to `Array<T>`?

neoCLR should avoid introducing a nominal `Array<T>` superclass merely because traditional CLR arrays have a nominal `System.Array` base.

The actual type is:

```text
T[]
```

rather than:

```text
Array<T>
```

There are several possibilities for `Array<T>` if such a name proves useful.

It could represent:

1. a type-system representation of the array structural type;
2. an interface describing array capabilities;
3. no public type at all.

The third option should remain viable.

If interfaces such as:

```text
Iterable<T>
Collection<T>
```

already describe the reusable capabilities of arrays, a nominal `Array<T>` abstraction may be unnecessary.

A nominal type should not be introduced merely to give structural arrays a name.

---

# 9. Array Representation

Making arrays structural also allows the logical array type to be separated from its storage strategy.

neoCLR may eventually support distinctions such as:

```text
heap allocated array
inline array
fixed-size array
stack allocated array
```

without necessarily forcing every representation to become a separate nominal type hierarchy.

For example:

```text
int[4]
```

could potentially describe an array whose size participates in its structure.

The runtime/compiler may then determine whether its storage is:

```text
inline
stack
heap
```

depending on the type and usage.

This should be explored separately from the initial array type model, but structural arrays avoid prematurely tying array semantics to heap allocation.

---

# 10. Tuples

Tuples should also be structural types.

For example:

```text
(int, string)
```

is structurally determined by its element types.

The runtime/type system may synthesize members such as:

```text
Item1: int
Item2: string

Deconstruct(out int, out string)
```

No nominal `Tuple<int, string>` or `ValueTuple<int, string>` declaration is required.

---

# 11. Tuple Element Names

Tuple elements may optionally have names:

```text
(x: int, name: string)
```

The underlying structural identity should generally remain:

```text
(int, string)
```

Names are therefore annotations rather than nominal identity.

Default names may be synthesized:

```text
Item1
Item2
Item3
...
```

Named elements may provide aliases:

```text
x     / Item1
name  / Item2
```

Whether both forms are always exposed is an API/language-design question.

---

# 12. Tuple `Deconstruct`

Tuples should expose a synthesized `Deconstruct` operation.

For:

```text
(int, string)
```

the type system may expose something equivalent to:

```text
Deconstruct(out int, out string)
```

without requiring an actual `MethodDef`.

This allows tuple destructuring to participate in the same member/type system as user-defined destructuring where appropriate.

---

# 13. Function Types

Function types should be structural types.

For example:

```text
(int) -> string
(Request) -> Response
(object, EventArgs) -> ()
```

The signature itself defines the function type.

neoCLR therefore does not require nominal delegate types such as:

```text
Func<int, string>
Action<object, EventArgs>
```

merely to represent signatures.

A function type consists conceptually of:

```text
parameter types
return type
calling semantics where relevant
```

Additional properties such as parameter modifiers may eventually participate in function-type identity.

---

# 14. Function Types and `Object`

Function values should likely be object-compatible.

That is:

```text
(int) -> string <: Object
```

This is particularly useful because function references are runtime values that may need to be:

- stored;
- passed through non-generic infrastructure;
- inspected;
- compared;
- dynamically invoked;
- wrapped by nominal callable types.

However, this does not imply that all structural types must derive from `Object`.

Function object compatibility is a deliberate property of function types.

---

# 15. Function Type Introspection

Function types should have a dedicated introspection representation.

For example:

```text
TypeInfo
├── NominalTypeInfo
├── ArrayTypeInfo
├── TupleTypeInfo
├── UnionTypeInfo
├── IntersectionTypeInfo
└── FunctionTypeInfo
```

A `FunctionTypeInfo` may expose information such as:

```text
Parameters
ReturnType
```

and potentially calling-convention information where necessary.

For:

```text
(int, string) -> bool
```

the runtime can therefore provide a `FunctionTypeInfo` describing that structural signature without requiring metadata for a generated delegate type.

---

# 16. Functions and Function References

neoCLR should distinguish between:

### Function

Executable code or a declared function/method.

### Function type

The structural signature:

```text
(Request) -> Response
```

### Function reference

A first-class runtime value referring to executable code.

A function reference may additionally contain:

- a bound instance;
- closure state;
- captured environment;
- foreign-runtime information.

For example:

```raven
let handler = service.Handle
```

creates a function reference where `service` may already be bound.

The function reference can then be invoked without separately supplying the target.

---

# 17. `Callable`

neoCLR should introduce a non-generic `Callable` interface.

Its primary purpose is to provide a common runtime abstraction for callable values, replacing much of the role served by `System.Delegate`.

Conceptually:

```raven
interface Callable {
    FunctionTypeInfo FunctionType { get }

    object? Invoke(params object?[] arguments)
}
```

The exact argument and return representation remains subject to the universal-value discussion described later.

`Callable` means:

> This value represents something that can be invoked, and its function signature can be inspected at runtime.

---

# 18. Why Non-Generic `Callable` Exists

A non-generic callable abstraction allows heterogeneous callable values to be passed through infrastructure without knowing their signature statically.

For example:

```raven
List<Callable>
```

could contain:

```text
() -> ()
(int) -> string
(Request) -> Response
nominal callable objects
closures
foreign-language callable objects
```

Code can inspect:

```raven
callable.FunctionType
```

before dynamically invoking the callable.

This replaces the useful erased runtime role of `Delegate` without requiring delegate types to represent function signatures.

---

# 19. `Callable<F>`

neoCLR may additionally provide a generic callable contract:

```raven
interface Callable<F> : Callable
    where F : func
{
}
```

where `F` must be a function type.

For example:

```text
Callable<(Request) -> Response>
```

means:

> This value can be invoked using the `(Request) -> Response` signature.

This gives neoCLR both:

```text
Callable
```

for erased/runtime callable handling and:

```text
Callable<F>
```

for statically known callable contracts.

---

# 20. Generic Constraints on Function Types

neoCLR needs a way to constrain a generic type parameter to function types.

A possible Raven syntax is:

```raven
where F : func
```

For example:

```raven
class FunctionList<F>
    where F : func
{
}
```

This states that `F` is not an arbitrary type.

It must be a structural function type such as:

```text
() -> ()
(int) -> string
(object, EventArgs) -> ()
```

The exact spelling of `func` remains a language-level design question, but the underlying neoCLR constraint should exist independently of Raven syntax.

---

# 21. Typed Invocation

When a callable's function type is statically known, invocation should remain fully typed.

For example:

```raven
let handler: Callable<(Request) -> Response>

let response = handler(request)
```

or equivalent explicit invocation syntax.

This path should not require:

```text
object[]
```

boxing, runtime parameter checking, or reflective invocation.

The compiler knows `F` and can emit the appropriate direct/indirect invocation operation.

---

# 22. Dynamic Invocation

A `Callable` whose function type is not statically known must still be invocable.

Therefore the non-generic interface should provide an erased invocation facility conceptually equivalent to:

```raven
object? Invoke(params object?[] arguments)
```

For example:

```raven
func Execute(callable: Callable, arguments: object?[]) {
    let result = callable.Invoke(arguments)
}
```

The runtime uses:

```raven
callable.FunctionType
```

to validate and dispatch the invocation.

This is analogous to `Delegate.DynamicInvoke`, but dynamic invocation becomes part of the general neoCLR callable model rather than a special property of delegates.

---

# 23. `Callable.Invoke` vs `MethodInfo.Invoke`

These operations represent different concepts.

A `MethodInfo` represents a description of a method.

Its invocation may therefore require:

```text
method
target
arguments
```

Conceptually:

```raven
methodInfo.Invoke(target, arguments)
```

A function reference is already a callable value.

For:

```raven
let function = person.GetName
```

the `person` instance may already be bound into the function reference.

Invocation therefore becomes:

```raven
function.Invoke(arguments)
```

The distinction is:

> `MethodInfo.Invoke` invokes a described member against a target.

> `Callable.Invoke` invokes an already-constructed callable value.

This separation should remain even if the underlying runtime invocation machinery is shared.

---

# 24. Invocation Errors

Dynamic invocation inherently allows invalid argument lists.

Examples include:

```text
incorrect argument count
incorrect argument type
unsupported conversion
invalid foreign value
```

neoCLR should consider representing these as recoverable errors rather than faults.

For example:

```raven
Result<object?, InvocationError> Invoke(
    params object?[] arguments
)
```

Possible errors might include:

```text
ArgumentCountMismatch
ArgumentTypeMismatch
ConversionFailed
InvocationFailed
```

The precise API should align with neoCLR's broader error model.

An error produced by the invoked function itself should also remain distinguishable from failure to perform the invocation.

---

# 25. `object[]` May Not Be Sufficient

If neoCLR permits types that do not derive from `Object`, then:

```raven
Invoke(params object[] arguments)
```

cannot represent every possible neoCLR function invocation.

For example, if tuples are not object-compatible:

```text
(int, int) </: Object
```

then a tuple cannot necessarily be placed in an `object[]`.

neoCLR may therefore eventually require a universal runtime value representation.

Conceptually:

```raven
Value Invoke(params Value[] arguments)
```

or:

```raven
Result<Value, InvocationError> Invoke(
    params Value[] arguments
)
```

`Value` here is illustrative rather than a proposed final type name.

This question depends on the broader relationship between `Object`, structural values, boxing, and universal runtime values.

The initial implementation may use `object[]` while keeping this limitation explicit.

---

# 26. `Callable<F>` and `Invoke`

`Callable<F>` does not necessarily need to physically declare a typed `Invoke` method.

For example:

```raven
interface Callable<F> : Callable
    where F : func
{
}
```

may be sufficient.

Given:

```text
F = (Request) -> Response
```

the type system knows the invocation signature.

It can synthesize:

```text
Invoke(Request) -> Response
```

or simply allow call syntax:

```raven
callable(request)
```

This avoids needing metadata capable of expressing:

```text
"the parameters and return type of this method are derived from F"
```

as an ordinary generic interface member.

`Callable<F>` therefore acts partly as a type-system-recognized contract.

---

# 27. Nominal Callable Types

Nominal types should be able to explicitly implement a callable signature.

For example:

```raven
class RequestHandler :
    Callable<(Request) -> Response>
{
    func Invoke(request: Request) -> Response {
        ...
    }
}
```

The type system validates that the nominal type supplies a compatible invocation operation.

This enables patterns currently built around delegates while allowing the callable itself to contain arbitrary state and behavior.

---

# 28. Duck-Typed Invocation

There are two useful notions of callability.

### Structural/language callability

A language such as Raven may decide that any type containing an appropriate:

```text
Invoke(...)
```