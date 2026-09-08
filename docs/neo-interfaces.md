# Interfaces in Neo

Neo can declare an interface, implement it on a record, and explicitly project a
managed reference to that interface. Interface names use ordinary descriptive names,
without an `I` prefix, following Neo and neoCLR convention.

```swift
interface Counter {
    func Add(amount: int) -> ()
    func Read() -> int
}

record SimpleCounter(Value: int): Counter {
    func Add(amount: int) -> () {
        this.Value = this.Value + amount
    }
    func Read() -> int {
        return this.Value
    }
}

func Increment(counter: Counter&) -> int {
    counter.Add(2)
    return counter.Read()
}

func Main() -> int {
    var local = SimpleCounter(40)
    let view: Counter& = &local
    return Increment(view)
}
```

## Projection and virtual dispatch

`as Counter&` selects the declared interface contract. Its operand must already be
a managed reference: `&local as Counter&` or `shared as Counter&`, where `shared`
is a `SimpleCounter&`. Conformance is explicit in the record declaration; methods
with matching names alone do not constitute conformance. Parameter and return types
must match exactly. A record can list several interfaces separated by commas.
Projecting an existing Counter& to Counter& forwards the same view.

Once an operand is a concrete managed reference, projection to an implemented
interface reference is implicit when a parameter, annotated binding, return type
or contextually typed match arm requires it:

```swift
let concrete = new SimpleCounter(40)
let view: Counter& = concrete
let result = Increment(concrete)
```

A frame-owned value still needs explicit address formation: `Increment(&local)`.
A bare `local` remains a value; the compiler does not implicitly address or box it.
The same Concrete& can project to any interface explicitly implemented by its type.
The runtime's IL remains explicit: the compiler emits `interface.borrow` at these
conversion points, followed by virtual dispatch through `callvirt`.

The compiler emits the existing `.interface`, `.implements`, `interface.borrow`
and `callvirt` mechanisms. Projection does not copy the record, allocate a box or
change the object's lifetime. A Counter& uses the original concrete storage for
dispatch. Interface declarations have no storage fields or mandatory Object base.

`as` binds below unary address formation and above multiplication. Parentheses can
make a projected call explicit: `(&local as Counter&).Read()`. This is a statically
checked interface projection, not a C# nullable `as` operation or a general dynamic
cast. Failed conformance is a compilation error. Implicit conversion is limited to
concrete-reference-to-interface-reference projection; interface-to-concrete
downcasts and cross-interface casts are not supported in this subset.

## Instance methods and receivers

Record bodies contain `func` instance methods. In this initial source subset every
method has an implicit managed `this: Concrete&` receiver and lowers to
`.method instance byref`; interface slots use the matching receiver contract.
`this.Value` reads automatically and assignment writes through to the original
record. Methods can use other methods through `this`, accept reference parameters,
and return references subject to the runtime's normal lifetime checks.

Calling a concrete method such as `local.Add(2)` also uses the original location.
Owned locals require `var` for these writable receivers, including read-only method
bodies; an existing managed reference can be held in a `let` binding. Receiver modes
for read-only or explicitly copying methods are future work. `this` cannot be
redeclared or shadowed. The runtime still supports value receivers independently
of this deliberately smaller source surface.

## Lifetime

A view into a frame-owned record cannot escape that frame, even if execution skips
verification. A view into a caller's record can be forwarded back. A heap-backed
interface reference roots its entire receiver allocation through garbage collection:

```swift
func MakeCounter() -> Counter& {
    let owner = new SimpleCounter(40)
    return owner
}
```

No manual dereferencing or invalidation is needed. An interface reference is used
for dispatch, not for loading an abstract interface as a value. The existing
conservative block-local address restrictions also apply to projections and
source method receiver formation.

## Run and inspect

```sh
cargo run --locked -- run examples/source/interfaces.neo --gc-stats
cargo run --locked -- verify examples/source/interfaces.neo
cargo run --locked -- assemble examples/source/interfaces.neo /tmp/interfaces.neo.json
cargo run --locked -- run /tmp/interfaces.neo.json
```

`assemble` refuses to overwrite an existing output; choose a fresh path when rerunning.
The example prints `3`, `3`, `42`, returns 42 and reclaims its one heap allocation.
Its two `3` outputs show mutation observed through two views of the same local.
[Tests](../tests/neo_interfaces.rs) cover multiple implementations, reference
parameters/results, heap retention under collection pressure, invalid contracts,
current-frame escapes and the CLI artifact round trip.

The bounded slice supports source-declared interfaces and records only. Generic
interface declarations, adapting bundled System interfaces, properties, overloads,
static/default interface members, variance, native interface
ABI and general dynamic casts remain outside the Neo compiler subset. Existing
runtime capabilities are described in [interfaces](interfaces.md). No new opcode
or artifact format revision is needed.

## Inherited interfaces

Interfaces may declare comma-separated bases (`interface Counter: Readable`). Neo
projects derived references to bases and resolves inherited methods without manual
dereferencing. See [interface inheritance](interface-inheritance.md) for diamond,
ambiguity, runtime validation and run instructions.
