# NeoCLR Dynamic Dispatch and Static–Dynamic Interoperability

## 1. Summary

NeoCLR should provide first-class runtime support for **dynamic dispatch** and interoperability between statically and dynamically typed code.

NeoCLR should not introduce a universal dynamic object type.

Instead, `dynamic` should describe **how a value is used**:

> A value marked as `dynamic` retains its ordinary runtime representation and type, but member operations performed through that value are resolved dynamically.

At the metadata level, a dynamic type use can therefore be represented using an ordinary underlying type—normally `System.Object`—together with metadata indicating that this particular type occurrence uses dynamic dispatch.

Conceptually:

```text id="96sgwx"
System.Object
    + DynamicDispatch annotation
```

A NeoCLR-aware language sees:

```text id="td19im"
dynamic
```

while a metadata consumer that does not understand the annotation can still see:

```text id="w9wpce"
System.Object
```

NeoCLR additionally provides VM-level dynamic dispatch instructions. These instructions delegate resolution to language-defined **dynamic dispatchers**.

This allows:

- dynamically typed languages to expose ordinary NeoCLR metadata;
- static and dynamic code to exchange ordinary runtime values;
- statically typed languages to deliberately perform dynamic dispatch;
- dynamically typed languages to consume nominal NeoCLR types;
- languages such as PHP to freely mix statically and dynamically typed declarations;
- Python-, JavaScript-, PHP-, Raven-, and other language object models to retain their own dispatch semantics;
- the runtime and JIT to recognize and optimize dynamic dispatch sites.

The central principle is:

> **NeoCLR standardizes the static–dynamic interoperability boundary and dynamic dispatch mechanism, while languages retain control over their object models and dispatch semantics.**

---

# 2. Motivation

Languages do not divide cleanly into "static languages" and "dynamic languages."

A single language may use both models.

PHP, for example, can define statically identifiable classes and methods while allowing parameters, fields, properties, returns, and variables whose operative type is determined only at runtime.

Likewise, a statically typed language such as Raven may deliberately allow:

```raven id="90ekai"
let value: dynamic = GetValue();
value.Greet();
```

Python may rely much more heavily on runtime dispatch.

JavaScript may base much of its object model around dynamic properties and prototype relationships.

These languages should not require entirely separate runtime universes.

NeoCLR should instead allow static and dynamic typing to coexist within the same metadata, object, and execution model.

---

# 3. Core Principle

Dynamic dispatch should not require converting an object into a special representation.

Given:

```raven id="fptkjj"
let person: Person = GetPerson();
let value: dynamic = person;
```

both variables refer to the same object:

```text id="8x0atq"
          Person object
             ▲   ▲
             │   │
       person│   │value
             │   │
       static│   │dynamic
      dispatch   dispatch
```

No wrapper is required.

No `DynamicObject` needs to be allocated.

No conversion into a special `DynamicType` runtime representation occurs.

The difference exists at the **type-use and operation level**.

---

# 4. `dynamic` Is Not a Runtime Type

NeoCLR should not introduce a universal runtime type such as:

```text id="f5zydw"
System.Dynamic
```

or:

```text id="l4fsqu"
DynamicType
```

to which values must be converted.

Doing so would introduce unnecessary problems:

- ordinary objects would need conversion or wrapping;
- object identity could become complicated;
- dynamic and static APIs would operate over different representations;
- interoperability would require unnecessary adapters;
- language-specific object models would be forced into a common representation.

Instead:

> **`dynamic` is an annotation on a type use indicating dynamic dispatch semantics.**

The runtime value retains its actual runtime type.

---

# 5. `Object` and `dynamic`

`Object` and `dynamic` may share the same underlying runtime representation while expressing different contracts.

Consider:

```raven id="7piy2p"
func Handle(value: Object)
```

versus:

```raven id="4vjzqg"
func Handle(value: dynamic)
```

The first means:

> `value` has the static type `Object`.

The second means:

> `value` uses the ordinary object representation, but operations through this type use are dynamically resolved.

Conceptually:

```text id="a5d6un"
Object

Object [DynamicDispatch]
```

A language may present the latter as:

```text id="n2eebd"
dynamic
```

---

# 6. Metadata Representation

NeoCLR should preserve dynamic semantics in metadata.

A declaration such as:

```raven id="85sp8n"
func Process(value: dynamic) -> dynamic
```

could have an underlying nominal signature equivalent to:

```text id="3m2wyi"
Object Process(Object value)
```

with additional type-use metadata:

```text id="jyt6qy"
Return:
    Object
    DynamicDispatch

Parameter:
    Object
    DynamicDispatch
```

A NeoCLR-aware metadata reader reconstructs:

```text id="8l8thj"
dynamic Process(dynamic value)
```

A legacy metadata reader may still understand:

```text id="8r9o6n"
Object Process(Object value)
```

This aligns with NeoCLR's broader goal of remaining close to the existing .NET metadata model.

---

# 7. Dynamic Is a Type-Use Annotation

The dynamic annotation should attach to a **particular occurrence of a type**, rather than merely to a declaration.

This matters for constructs such as:

```text id="4vebcu"
List<dynamic>

Dictionary<String, dynamic>

Result<dynamic, Error>
```

Conceptually:

```text id="hmph8i"
List<
    Object [DynamicDispatch]
>
```

rather than:

```text id="ldp9bj"
List<Object> [DynamicDispatch]
```

These mean different things.

The metadata model should therefore support annotations over nested type positions.

This is structurally similar to the problem NeoCLR must solve for nullability metadata.

---

# 8. Locations Supporting Dynamic Type Uses

Dynamic type-use information should be representable wherever type information meaningfully occurs, including:

- parameters;
- return types;
- fields;
- properties;
- locals;
- generic arguments;
- array element types;
- tuple elements;
- function signatures;
- potentially other compound type positions.

For example:

```text id="pklxf6"
Dictionary<
    String,
    List<
        Object [DynamicDispatch]
    >
>
```

must be representable without changing the underlying nominal types.

---

# 9. Static and Dynamic Views of the Same Object

The same object may simultaneously be viewed statically and dynamically.

For example:

```raven id="uvf5r2"
let person: Person = GetPerson();
let value: dynamic = person;

person.Greet();
value.Greet();
```

The first call can use ordinary nominal dispatch:

```text id="6ixd30"
callvirt Person::Greet
```

The second uses dynamic dispatch:

```text id="ss8f89"
dyn.invoke.member Greet
```

Both operate against the same runtime object.

This is fundamental to static–dynamic interoperability.

---

# 10. Dynamic Dispatch Instructions

NeoCLR should provide explicit neoIL operations representing dynamic dispatch.

An initial vocabulary should investigate instructions equivalent to:

```text id="0lf79b"
dyn.get.member
dyn.set.member
dyn.remove.member

dyn.invoke.member
dyn.invoke

dyn.get.index
dyn.set.index

dyn.convert
```

Operator dispatch may also be considered.

The exact instruction encoding is outside the scope of this proposal.

The important point is that the VM knows that these operations require dynamic resolution.

---

# 11. Dynamic Invocation

An instruction conceptually resembling:

```text id="rltl8y"
dyn.invoke.member <member-ref>
```

means:

> Dynamically resolve and invoke this member against the target value according to the applicable dynamic dispatch semantics.

It does not mean:

> Look up this method through the target's ordinary nominal method table.

The resulting target may be:

- a NeoCLR method;
- a dynamically stored function;
- a language-level bound method;
- a callable object;
- a generated thunk;
- a native function;
- an interoperability adapter;
- another executable construct.

NeoCLR does not require all languages to share the same concept of "method."

---

# 12. Dynamic Dispatchers

Languages participating in dynamic dispatch provide **Dynamic Dispatchers**.

Conceptually:

```text id="jmyr76"
DynamicDispatcher
    Bind(request) -> DynamicBinding
```

A dispatcher defines how dynamic operations are interpreted according to a particular language or object model.

A dispatcher may implement rules involving:

- nominal member lookup;
- runtime types;
- runtime properties;
- prototype traversal;
- descriptors;
- missing-member handlers;
- callable objects;
- conversions;
- operators;
- indexing;
- language-specific accessibility;
- other object-model behavior.

NeoCLR does not encode these language rules itself.

---

# 13. No Universal DynamicObject

NeoCLR should not require dynamic values to derive from or implement a managed abstraction equivalent to:

```text id="g80p88"
DynamicObject
    TryGetMember(...)
    TrySetMember(...)
    TryInvokeMember(...)
```

This is unnecessary when dynamic dispatch is understood directly by the runtime.

Language implementations remain free to use whatever backing representation best fits their semantics.

Examples could include:

```text id="m1m8kh"
PythonObject
JavaScriptObject
PHP runtime object
native proxy
runtime-generated object
```

The interoperability contract is dynamic dispatch, not inheritance from a universal managed dynamic-object class.

---

# 14. Dispatcher Selection

Dispatcher selection requires two cases to be supported.

## 14.1 Native Dynamic Objects

Some objects belong to a language-specific dynamic object model.

Examples might include:

```text id="l7g1fo"
PythonObject
    → PythonDispatcher

JavaScriptObject
    → JavaScriptDispatcher
```

Such objects may provide a native dispatcher capable of preserving their originating language semantics.

## 14.2 Ordinary NeoCLR Objects

Any ordinary NeoCLR object should also be usable through a `dynamic` type use:

```raven id="4z8sgt"
let person: Person = ...;
let value: dynamic = person;
```

`Person` should not need to carry a language-specific dispatcher merely because a caller chooses to access it dynamically.

Therefore a dynamic call site also requires an applicable **calling/default dispatcher** capable of dynamically interacting with ordinary NeoCLR values.

---

# 15. Native and Calling Dispatchers

A dynamic operation may therefore involve:

```text id="0vdrkd"
Dynamic call site
    CallingDispatcher

Target
    NativeDispatcher?
```

The dispatch protocol determines which semantics apply.

The common cases are:

```text id="f0ugj6"
Target has native dynamic semantics
        │
        ▼
use native dispatcher
```

and:

```text id="rs4j5l"
Target is ordinary NeoCLR object
        │
        ▼
use calling/default dispatcher
```

This permits a language to define how its dynamic code interacts with nominal NeoCLR objects while preserving native semantics for foreign dynamic values.

The exact precedence and delegation protocol should be specified separately.

---

# 16. Dynamic Dispatch Sites

Each dynamic operation occurs at a **Dynamic Dispatch Site**.

Conceptually:

```text id="b5klhq"
DynamicDispatchSite
    Operation
    MemberReference?
    CallingSignature
    CallingDispatcher
    RuntimeState
```

The site represents a particular dynamic operation in executable code and provides a natural location for caching previously resolved bindings.

---

# 17. Metadata for Dynamic Operations

Dynamic instructions should integrate naturally with NeoCLR's .NET-derived metadata model.

Rather than embedding all information directly into an instruction:

```text id="0xzhq1"
dyn.invoke.member "greet", ...
```

the instruction may reference metadata:

```text id="j6kbbn"
dyn.invoke.member <token>
```

where the token identifies something conceptually resembling:

```text id="kpl50r"
DynamicMemberRef
    Name
    Operation
    CallingSignature
    AdditionalInformation
```

This follows the general CLI model in which compact instructions reference richer metadata structures.

---

# 18. Dynamic Signatures

A dynamic call site may have varying amounts of statically known information.

For example:

```text id="0y8ou1"
(String, Int32) -> Object
```

or:

```text id="47dlx6"
(dynamic, dynamic) -> dynamic
```

or:

```text id="50c5hq"
(String, dynamic) -> Person
```

Dynamic dispatch metadata should preserve whatever static information is available rather than erasing the entire signature merely because dispatch is dynamic.

---

# 19. Dynamic Binding

When a dynamic operation requires resolution, the selected dispatcher receives a request conceptually resembling:

```text id="v3x7lh"
DynamicDispatchRequest
    Target
    Operation
    Member
    Arguments
    CallingSignature
    CallSiteInformation
```

The dispatcher returns a binding conceptually resembling:

```text id="m0kjuh"
DynamicBinding
    Target
    Guards
    InvalidationInformation
```

The exact API is deliberately unspecified.

The important property is that the runtime can reuse the resulting target while the dispatcher's assumptions remain valid.

---

# 20. Runtime Optimization

Making dynamic dispatch a VM-level concept allows NeoCLR to optimize dynamic operations.

Potential mechanisms include:

- monomorphic inline caches;
- polymorphic inline caches;
- runtime-type guards;
- shape guards;
- version guards;
- direct target invocation;
- JIT specialization;
- speculative inlining;
- explicit invalidation.

Conceptually:

```text id="pq3gkj"
dynamic call
     │
     ▼
cached binding?
   /       \
 yes        no
  │          │
guards     dispatcher
valid?       │
 │           ▼
 yes      new binding
  │          │
  └────┬─────┘
       ▼
     target
```

Optimization must never change the observable semantics defined by the dispatcher.

---

# 21. Runtime Dynamic Type Information

The static `dynamic` annotation must be distinguished from runtime type information belonging to a dynamic language.

Some dynamic object models have meaningful language-level runtime types.

Python is a clear example.

A Python value might have:

```text id="e50jd3"
NeoCLR runtime TypeInfo:
    PythonObject

Language runtime type:
    Person
```

NeoCLR should permit such languages to expose optional runtime **DynamicTypeInfo**.

However, `DynamicTypeInfo` is not the runtime representation of the static `dynamic` annotation.

They solve different problems.

---

# 22. `dynamic` vs DynamicTypeInfo

These concepts must remain explicitly separate.

### `dynamic`

A static metadata/type-use annotation.

It means:

> Operations through this value are expected to use dynamic resolution where static resolution is unavailable or inappropriate.

### `DynamicTypeInfo`

Optional runtime reflection information supplied by a dynamic object model.

It means:

> This language/object model recognizes this runtime value as belonging to this language-level type.

For example:

```text id="p5w1kk"
Static declaration:
    value : dynamic

Runtime value:
    TypeInfo        = PythonObject
    DynamicTypeInfo = Person
```

But another value may have:

```text id="fxmvwh"
Static declaration:
    value : dynamic

Runtime value:
    TypeInfo        = Person
    DynamicTypeInfo = none
```

Both are valid.

---

# 23. Dynamic Structure

Some object models permit runtime-visible structure that cannot be represented by ordinary nominal metadata.

For example:

```python id="w1qosr"
person.nickname = "Al"
```

may introduce an instance member not present in the object's nominal `TypeInfo`.

NeoCLR should permit dynamic object models to expose such structure through runtime Reflection.

This information belongs to the runtime object, not to the static `dynamic` annotation.

---

# 24. Runtime Shapes

Dynamic-language implementations may use shapes, hidden classes, maps, or similar mechanisms.

For example:

```text id="m4lnqv"
Shape #17
    name -> slot 0
    age  -> slot 1
```

followed by:

```text id="isix56"
Shape #24
    name     -> slot 0
    age      -> slot 1
    nickname -> slot 2
```

Shapes may be useful for:

- storage;
- dispatch;
- caching;
- JIT optimization.

However:

> **Runtime shape is an implementation/optimization concept and is not automatically a language-level type.**

A language may expose structural information without exposing its internal shape identities.

---

# 25. Runtime Reflection

Dynamic runtime information should be exposed through Reflection rather than ordinary metadata introspection.

NeoCLR's existing distinction remains:

```text id="nwy0ax"
System.Introspection
    nominal metadata

System.Runtime.Reflection
    runtime capabilities
```

Dynamic facilities should therefore live under a namespace tentatively resembling:

```text id="hsduwf"
System.Runtime.Reflection.Dynamic
```

Potential concepts include:

```text id="iqvyh6"
DynamicObjectInfo
DynamicTypeInfo
DynamicMemberInfo
DynamicMethodInfo
DynamicPropertyInfo
DynamicReflection
```

The exact API requires a separate proposal.

---

# 26. Instance-Oriented Dynamic Reflection

Dynamic structure may vary between instances.

For example:

```text id="u93ax6"
alice:
    name
    age
    nickname

bob:
    name
    age
```

Both objects may share the same nominal and language-level runtime types.

Therefore:

```text id="47hs8r"
TypeInfo.GetMembers()
```

must not become instance-dependent.

Instead:

```text id="wl8g8j"
TypeInfo
    → nominal metadata

DynamicTypeInfo
    → optional language runtime type

DynamicObjectInfo
    → dynamic information about this particular value
```

---

# 27. Partial Dynamic Reflection

Not every dynamic object can enumerate its possible members.

A proxy might implement:

```text id="w2z42m"
GetMember(name)
    → resolve remotely
```

without knowing all valid names in advance.

Dynamic Reflection must therefore allow partial capability.

An object may support:

```text id="u30pup"
GetMember
InvokeMember
```

without supporting:

```text id="u1x7bs"
EnumerateMembers
```

The API must be able to express:

```text id="vsdv15"
Continuing directly from **§27 Partial Dynamic Reflection**:

unknown
unsupported
runtime-dependent
```

Dynamic Reflection must not manufacture static certainty where the underlying object model provides none.

---

# 28. Dynamic Reflection Capabilities

Dynamic Reflection should therefore be capability-oriented rather than assuming that every dynamic object supports a common structural model.

Potential capabilities include concepts such as:

```text id="z6t2vy"
DynamicMemberProvider
DynamicMemberEnumerator
DynamicMemberMutator
DynamicInvoker
DynamicIndexer
DynamicTypeProvider
```

These names are illustrative rather than normative.

A Python-like object may provide rich type and member information.

A JavaScript-like object may expose properties and prototype-related information.

A remote proxy may support invocation without member enumeration.

An ordinary NeoCLR object accessed dynamically may expose its nominal metadata through an adapter supplied by the relevant dispatcher.

The Reflection API should preserve these differences.

---

# 29. Dynamic Dispatch and Dynamic Reflection Are Separate

Dynamic Reflection must not become the implementation mechanism for dynamic dispatch.

The hot execution path is:

```text id="u5jqw8"
neoIL dynamic instruction
          │
          ▼
dynamic dispatch site
          │
          ▼
selected dispatcher
          │
          ▼
binding
          │
          ▼
execution target
```

Reflection instead provides managed access to runtime information:

```text id="m0bjlf"
System.Runtime.Reflection.Dynamic
          │
          ▼
runtime dynamic information
          │
          ├── DynamicTypeInfo
          ├── DynamicObjectInfo
          └── dynamic member information
```

The two systems may rely on common runtime infrastructure, but Reflection does not sit between a dynamic instruction and its target.

---

# 30. Static-to-Dynamic Transition

A statically known value can always be viewed dynamically where the consuming language permits it.

For example:

```raven id="zrrf9c"
let person: Person = GetPerson();
let value: dynamic = person;
```

This operation does not change the object.

Conceptually:

```text id="zhvttr"
Person
  │
  │ static view
  ├──────────────→ Person
  │
  │ dynamic view
  └──────────────→ Object [DynamicDispatch]
```

The runtime identity remains:

```text id="ubtmcb"
TypeInfo = Person
```

Only the static type-use semantics have changed.

This means dynamic conversion normally requires no allocation, wrapper, or representation change.

---

# 31. Dynamic-to-Static Transition

The reverse transition requires type validation.

For example:

```raven id="ky8h82"
let value: dynamic = GetValue();
let person = value as Person;
```

NeoCLR can determine whether the runtime value satisfies the nominal `Person` type according to the applicable conversion rules.

If the underlying value is already a NeoCLR `Person`, this may be an ordinary runtime type check.

If the value belongs to a foreign dynamic object model, a language may additionally support explicit projection:

```text id="6ilc94"
foreign dynamic value
        │
        ▼
projection / adaptation
        │
        ▼
Person
```

Projection is distinct from dynamic dispatch.

Dynamic dispatch preserves the target's object model.

Projection deliberately presents the value through another type or language model.

---

# 32. Static-to-Dynamic Language Interoperability

A dynamically oriented language should be able to consume ordinary NeoCLR declarations.

Suppose Raven exposes:

```raven id="55kk8h"
class Person {
    Name: String

    func Greet(message: String) -> String
}
```

A PHP- or Python-like language may choose to permit:

```text id="veokug"
person.Name
person.Greet("Hello")
```

through its own dynamic semantics.

The receiving language's dispatcher determines how nominal NeoCLR members participate in its object model.

NeoCLR does not impose one universal rule.

For example, languages may differ over:

- case sensitivity;
- overload selection;
- property versus field treatment;
- accessibility;
- conversions;
- optional arguments;
- extension methods;
- callable members.

These remain dispatcher responsibilities.

---

# 33. Dynamic-to-Static Language Interoperability

A statically typed language may consume metadata containing dynamic type-use annotations.

Suppose PHP exposes:

```php id="45s2cj"
function transform($value) {
    ...
}
```

Its NeoCLR metadata may describe:

```text id="9fgcwf"
dynamic transform(dynamic value)
```

A Raven projection can preserve that contract:

```raven id="1abn8c"
func transform(value: dynamic) -> dynamic
```

rather than degrading it to:

```raven id="e8m4ur"
func transform(value: Object) -> Object
```

The distinction matters because the former explicitly communicates dynamic dispatch semantics to the consuming compiler.

---

# 34. PHP as a Hybrid Model

PHP demonstrates why NeoCLR should model dynamic dispatch independently of language classification.

Consider:

```php id="s2nw92"
class Processor
{
    public $source;

    public function process($value): string
    {
        return $value->getName();
    }
}
```

This can map to ordinary NeoCLR metadata:

```text id="4zom1v"
TypeDef Processor

Field:
    Object [DynamicDispatch] source

MethodDef:
    String process(
        Object [DynamicDispatch] value
    )
```

The `Processor` type itself is ordinary nominal metadata.

Only particular type uses are dynamic.

If PHP instead declares:

```php id="ubogff"
public Person $source;

public function process(Person $value): string
```

the metadata becomes:

```text id="cql0a1"
Field:
    Person source

MethodDef:
    String process(Person value)
```

The same language therefore moves naturally between statically and dynamically typed declarations.

---

# 35. PHP Dispatch

PHP also demonstrates that the choice between ordinary and dynamic dispatch can occur within the same language.

Given:

```php id="3j72au"
function process(Person $person): string
{
    return $person->getName();
}
```

a compiler may be able to emit ordinary nominal dispatch where PHP semantics permit:

```text id="7k83gu"
callvirt Person::getName
```

Given:

```php id="f9b2uw"
function process($person): string
{
    return $person->getName();
}
```

the corresponding operation may instead become:

```text id="dk9w34"
dyn.invoke.member getName
```

NeoCLR therefore does not need to know that PHP is "a dynamic language."

It merely executes the operation selected by the PHP compiler.

---

# 36. Python-Like Models

Python exercises a different part of the design.

A Python implementation may represent:

```python id="v60xl5"
def process(value):
    return value.name
```

using metadata resembling:

```text id="36pb3p"
dynamic process(dynamic value)
```

At runtime, the value may additionally expose language-level information:

```text id="v29h34"
TypeInfo:
    PythonObject

DynamicTypeInfo:
    Person

DynamicObjectInfo:
    instance attributes
```

The Python dispatcher may resolve member access using:

- `__getattribute__`;
- instance attributes;
- class attributes;
- descriptors;
- inheritance;
- `__getattr__`;
- metaclasses;
- other Python semantics.

None of those rules need to be encoded into NeoCLR itself.

---

# 37. JavaScript-Like Models

JavaScript exercises the structural and prototype-oriented side of the design.

A JavaScript value may have:

```text id="rr91jd"
TypeInfo:
    JavaScriptObject

DynamicTypeInfo:
    absent or limited

DynamicObjectInfo:
    own properties
    prototype relationship
```

Its dispatcher may resolve:

```javascript id="8gl6n8"
object.foo
```

using JavaScript property and prototype semantics.

NeoCLR does not require the object to possess a conventional nominal dynamic type.

Likewise, internal implementation concepts such as hidden classes or shapes do not automatically become language-visible types.

---

# 38. Raven Dynamic

Raven can expose dynamic dispatch without becoming a dynamically typed language.

For example:

```raven id="ylu6te"
func PrintName(value: dynamic)
{
    Console.WriteLine(value.Name);
}
```

The signature records:

```text id="78tvr5"
Parameter:
    Object [DynamicDispatch]
```

and the member access becomes a dynamic neoIL operation.

A Raven value can therefore cross explicitly into the dynamic dispatch model without changing its runtime representation.

This also allows Raven to naturally consume APIs originating in PHP, Python, JavaScript, or other languages that expose dynamic type uses.

---

# 39. Dynamic-to-Dynamic Interoperability

The most important cross-language case occurs when dynamically typed code passes values between languages.

Suppose Python creates a value and passes it to PHP.

PHP receives it through:

```text id="j5xyhk"
Object [DynamicDispatch]
```

and performs:

```text id="qkyi1f"
dyn.invoke.member greet
```

If the target exposes native Python dynamic dispatch semantics, NeoCLR can preserve those semantics:

```text id="lvttvq"
PHP call site
      │
      ▼
dyn.invoke.member
      │
      ▼
Python-backed value
      │
      ▼
Python dispatcher
      │
      ▼
Python-defined resolution
```

The PHP compiler does not need to know how Python performs member lookup.

---

# 40. Native Dynamic Semantics

For objects belonging to a language-specific dynamic object model, the default interoperability principle should be:

> **A foreign dynamic value should retain its native dispatch semantics unless the consuming language explicitly projects or adapts it into another object model.**

This avoids defining a supposedly neutral NeoCLR dynamic object language.

NeoCLR standardizes the dispatch protocol.

It does not standardize the meaning of every dynamic operation.

---

# 41. Calling-Language Semantics

Native semantics alone are not sufficient for every operation.

Consider PHP dynamically accessing an ordinary Raven object.

The Raven object has no native dynamic dispatcher.

The PHP dispatcher must therefore determine how PHP dynamic operations map onto ordinary NeoCLR metadata.

Conceptually:

```text id="4tf9dr"
PHP dynamic call
       │
       ▼
ordinary Raven Person
       │
       ▼
PHP calling dispatcher
       │
       ▼
NeoCLR nominal metadata
       │
       ▼
resolved member
```

The calling dispatcher may apply PHP-specific rules for conversions, member visibility, naming, overloads, and invocation.

This is why the model must account for both:

```text id="31dnad"
target native dispatcher

and

calling-language dispatcher
```

rather than assuming that every object inherently owns dynamic semantics.

---

# 42. Dispatcher Selection Protocol

The exact selection protocol requires further design, but an initial model is:

```text id="u81wmy"
dynamic operation
       │
       ▼
Does target expose native dynamic dispatch?
       │
    ┌──┴──┐
   yes    no
    │      │
    ▼      ▼
 native   calling
dispatcher dispatcher
    │      │
    └──┬───┘
       ▼
     binding
```

More advanced interoperation may require cooperation between both dispatchers.

For example:

```text id="tl4n5v"
calling dispatcher
       │
       ▼
foreign target dispatcher
       │
       ▼
binding/projection
```

The protocol should permit this without making every cross-language call require two complete dispatch passes.

---

# 43. Projection Is Explicitly Separate

Projection should remain separate from native dynamic dispatch.

Native dispatch means:

```text id="fz2zrl"
Python object
    +
Python semantics
```

Projection means:

```text id="q4zg8s"
Python object
       │
       ▼
JavaScript projection
       │
       ▼
JavaScript-facing semantics
```

or:

```text id="kz6i9j"
dynamic object
       │
       ▼
structural adapter
       │
       ▼
Raven trait/interface
```

NeoCLR should support projection infrastructure, but projection should not silently redefine what dynamic dispatch means.

---

# 44. Structural Projection

Dynamic Reflection can assist in adapting dynamic objects to static interfaces or traits.

Suppose Raven declares:

```raven id="l7ypqo"
trait Named {
    Name: String
}
```

and receives a dynamic value exposing a compatible `Name`.

NeoCLR should not claim that the object's nominal type implements `Named`.

Instead:

```text id="k0lfyz"
dynamic object
      │
      ▼
dynamic inspection
      │
      ▼
compatibility check
      │
      ▼
projection adapter
      │
      ▼
Named
```

This preserves nominal type correctness while enabling useful structural interoperability.

The exact projection rules belong to the consuming language or projection system.

---

# 45. Dynamic Values in Generic Types

Because `dynamic` is a type-use annotation, it may appear inside generic constructions:

```text id="c8zmls"
List<dynamic>

Result<dynamic, Error>

Dictionary<String, dynamic>
```

Their underlying nominal signatures might be:

```text id="nvwcbj"
List<Object>

Result<Object, Error>

Dictionary<String, Object>
```

with nested dynamic annotations preserved separately.

This means:

```text id="vp11gt"
List<dynamic>
```

and:

```text id="cj1p25"
List<Object>
```

may have the same runtime generic instantiation while preserving different language-level contracts.

This is desirable because dynamic dispatch changes how values are consumed rather than their storage representation.

---

# 46. Generic Runtime Identity

NeoCLR should therefore not create separate generic runtime instantiations merely because one type argument carries the dynamic-dispatch annotation.

For example:

```text id="e8znvf"
List<Object>

List<dynamic>
```

should ordinarily share the same runtime representation and nominal generic identity.

The distinction exists in richer metadata and compiler interpretation.

This follows the same general principle that type-use semantic annotations should not unnecessarily fragment runtime type identity.

---

# 47. Dynamic Locals

Locals may also carry dynamic type-use information.

For example:

```raven id="0fc9ln"
let value: dynamic = person;
```

can use the same underlying local representation as:

```raven id="ygfmtr"
let value: Object = person;
```

while local signature metadata or associated compiler/runtime metadata preserves the dynamic annotation where required.

The compiler then emits different operations depending on the static view through which the value is accessed.

---

# 48. Assignment Compatibility

Because `dynamic` does not imply a separate object representation, assignment from ordinary reference values into a dynamic location should normally require no runtime conversion:

```text id="57x94k"
Person → dynamic
String → dynamic
Object → dynamic
PythonObject → dynamic
```

Value types require the normal NeoCLR representation rules applicable when stored through `Object`, such as whatever boxing/value abstraction NeoCLR ultimately defines.

Dynamic dispatch itself should not introduce an additional boxing layer.

---

# 49. Dynamic-to-Static Assignment

Assigning from `dynamic` to a statically typed location may require runtime validation:

```raven id="2e20hl"
let value: dynamic = ...;
let person: Person = value;
```

The language determines whether this is:

- implicitly permitted;
- explicitly cast;
- a checked conversion;
- rejected without explicit syntax.

NeoCLR provides the necessary runtime type and projection mechanisms but does not prescribe source-language conversion policy.

---

# 50. Failure Model

Dynamic dispatch can fail for ordinary reasons:

- member not found;
- member not invocable;
- incompatible arguments;
- invalid conversion;
- inaccessible member;
- unsupported operation;
- ambiguous resolution.

NeoCLR should not require these failures to become catchable runtime exceptions.

Consistent with NeoCLR's broader error model, the dynamic dispatch protocol should distinguish:

```text id="whlhc5"
dispatch resolution failure
    → language/runtime-defined recoverable outcome

runtime Fault
    → unrecoverable runtime failure
```

Exactly how dynamic dispatch failures surface to source code is language-specific.

A Python dispatcher may expose Python-style runtime errors.

A JavaScript dispatcher may expose JavaScript-style errors.

A Raven dynamic facility may choose explicit `Result`-based semantics where appropriate.

The runtime dispatch mechanism must not unnecessarily dictate this policy.

---

# 51. Dispatcher Failures vs Target Failures

A distinction should also be made between:

```text id="gbd0ra"
Dispatch failure
    Could not resolve or perform the requested operation.

Target result/failure
    Dispatch succeeded and the invoked target produced
    its own result or failure.
```

These are semantically different.

For example:

```text id="2s83lm"
dyn.invoke.member Save
```

may fail because `Save` does not exist.

Alternatively, `Save` may resolve successfully and return:

```text id="sqrb4h"
Result<void, StorageError>
```

The dynamic dispatch infrastructure should preserve that distinction.

---

# 52. Metadata Compatibility

Representing `dynamic` as an annotation over ordinary type uses provides an important compatibility advantage.

A NeoCLR-aware reader may see:

```text id="f6dgig"
func Transform(value: dynamic) -> dynamic
```

while a reader unaware of NeoCLR dynamic annotations can still understand the underlying signature:

```text id="0q2s58"
Object Transform(Object value)
```

Likewise:

```text id="4c60d7"
List<dynamic>
```

can remain structurally readable as:

```text id="v9j9kk"
List<Object>
```

This aligns with NeoCLR's goal of extending the .NET metadata model non-invasively where practical.

---

# 53. Relationship to Nullability Metadata

Dynamic dispatch annotations and nullability annotations have similar metadata requirements.

Both describe properties of a **type occurrence** rather than necessarily introducing a distinct runtime type.

For example:

```text id="7vnkk3"
List<String?>
List<dynamic>
Dictionary<String, dynamic?>
```

requires metadata capable of describing nested semantic information.

NeoCLR should therefore investigate a general mechanism for richer type-use annotations rather than inventing completely separate encoding systems for nullability and dynamic dispatch.

The semantics remain different, but the metadata infrastructure may be shared.

---

# 54. Interoperability Matrix

The model should be validated against at least the following cases:

| Producer | Consumer | Representation / behavior |
|---|---|---|
| Raven nominal value | Raven static | Ordinary static/virtual dispatch |
| Raven nominal value | Raven `dynamic` | Calling dispatcher over nominal object |
| PHP typed value | Raven static | Ordinary NeoCLR nominal contract |
| PHP untyped value | Raven | `Object [DynamicDispatch]` |
| Python dynamic value | Raven `dynamic` | Native Python dynamic dispatch |
| JavaScript value | Raven `dynamic` | Native Java