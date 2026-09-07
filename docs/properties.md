# Property metadata and ordinary accessors

Properties now have explicit metadata on their declaring type. A property describes
its name, static/instance kind, index parameter types, value type, and optional getter
and setter method references. It does not add an instruction, backing field, storage
layout, virtual dispatch rule, or union-specific behavior.

## Assembly syntax

```text
.type Box<T>
    .field Stored T
    .property instance Value() -> T
        .get instance Box<T>::Read()
    .end
    .method instance Read() -> T
        ldarg this
        ldfld Box<T>::Stored
        ret
    .end
.end
```

A property block belongs directly to a type and has its own .end. It accepts one .get
and/or one .set, followed by full method references using the normal signature and
optional member-identity syntax. Accessors can be declared before or after the property.
Names such as Read or get_Value have no implicit association: the metadata establishes
that relationship. Property signatures use type-only index parameters; their callable
accessors carry any parameter names.

An indexed static property can describe existing methods as follows:

```text
.property static Item(Int32*) -> Int32
    .get Slots::Read(Int32*)
    .set Slots::Write(Int32*,Int32)
.end
```

These methods must be declared on Slots. The pointer is an explicit index argument;
it is not implicitly created or owned by the property. The executable indexed-property
test writes through that pointer using existing stobj semantics.

The getter takes exactly the index parameters and returns the property's value type.
The setter takes the index parameters followed by the value and returns real Void.
Get-only and set-only properties are supported. Static/instance kind must agree across
the property and all associated methods. Void remains a valid value type here.

## Validation and identity

Property names must be valid slot identifiers. Within a declaring type, the name,
static/instance kind, and index parameter types uniquely identify a property signature;
return type alone cannot distinguish overloads in this initial subset. Field and method
names occupy their existing independent categories.

The loader validates property types in the declaring generic context, requires at least
one accessor, resolves full accessor signatures, and checks owner, declaring module,
static/instance kind, parameters, and return type. A supplied method identity must match
its signature. Generic parameter references are indexed, and accessor owners name the
same open declaring type, for example Box<!0>. Scoped types and direct module-reference
rules apply to property signatures and accessor references as elsewhere in metadata.

Preparation binds accessors to method identities in the prepared copy, as it does for
calls. Source metadata is not mutated. TypeDef.properties is an optional ordered metadata
list; omitted properties remain an empty list in older artifacts. The JSON format version
remains 3 with this additive field, but older readers that reject unknown fields cannot
read new artifacts containing properties. No CLI binary Property/MethodSemantics table
writer is implemented yet; this models the association in the prototype format.

## Execution and limits

IL continues to call the accessor method directly. There are no get-property or
set-property instructions, property-name lookup at execution time, or synthesized methods.
Property metadata alone does not execute accessors or make them reachability roots.
Compilers can select the accessor and emit an ordinary call; trimming/reflection policies
would need explicit rules if introduced later.

Instance receivers still follow existing copied-value semantics. Associating a setter
with a method does not make a copied receiver mutate caller-owned storage. An explicit
pointer can mutate its target under the existing memory contract; addressed receiver
mutation and construction/initialization remain separate prerequisites. [Method accessibility](accessibility.md) now governs calls to each accessor independently.
Ordinary field access also enforces visibility. Top-level type access is also checked. Automatic properties,
property attributes, and a guest reflection API remain unimplemented.

System.Error.Message and System.Array<T>.Length now explicitly associate their existing
get_Message and get_Length methods. Their implementations and signatures are unchanged.
Properties are one ordinary-type foundation used by carrier/variant unions. Their
storage and invariants are supplied by ordinary fields and methods; no union instructions remain.

## Run the sample

```sh
cargo run --locked -- verify examples/properties.neoil
cargo run --locked -- run examples/properties.neoil
```

Expected output:

```text
42
Properties describe ordinary methods
=> Void
```

Use the README's assemble/run workflow to execute its JSON artifact. The sample uses a
generic Box property and Error.Message, with all reads expressed as ordinary calls.
