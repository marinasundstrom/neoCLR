# Generic constraints

The initial runtime-enforced subset restricts the outermost form of a type argument:

```swift
record Measurement<T>(Value: T) where T: notvoid, notreference

func CopyValue<T>(value: T) -> T where T: notreference {
    return value
}
```

Neo supports these clauses on plain generic records, generic free functions and
static generic methods on nongeneric records/classes. Clauses follow the record field
list or function return type. Repeat `where` for different parameters. Explicit and
inferred method arguments undergo the same runtime checks; source record constructors
still require explicit arguments. The two flag constraints alone enable no new operation on T.
Nominal bounds additionally enable borrowed member calls, described below.

| Constraint | Rejected argument forms | Still allowed |
| --- | --- | --- |
| `notvoid` | Void itself | `Box<Void>`, arrays or native pointers involving Void, subject to their existing validity rules |
| `notreference` | Mutable/readonly managed references and the legacy interface-reference form | Ordinary values, including values containing references; native pointers, subject to existing pointer rules |

These are independent restrictions and may be combined. They do not imply unmanaged
layout, deep copying, immutability, defaultability or heap-only storage. In particular,
`notreference` does not inspect the object's fields. Passing an existing Foo& to a
bare T parameter infers Foo& and can violate the constraint. To deliberately pass a
copy of Foo, first bind a Foo-valued local using the existing value-read projection.
Constraints never silently change inference into a copy or a reference conversion.

## Metadata and enforcement

neoIL attaches constraints to the declaring type or method:

```text
.type Measurement<T>
.constraint T notvoid notreference
.field Value T
.end

.function CopyValue<T>(T value) -> T
.constraint T notreference
ldarg value
ret
.end
```

The directive accepts the parameter name or its zero-based index. Method constraints
refer to method parameters; type constraints refer to owner parameters. Method
constraints must precede instructions and labels. JSON stores a `generic_constraints`
list of `{parameter, kind}` rows (`NotVoid`, `NotReference`, or `{ "TypeBound": type }`) on TypeDef and Function.
Unknown rules, duplicate rows and out-of-range indices are rejected.

Definitions are validated during loading. Concrete constructed signatures and resolved
generic calls check the argument restrictions, including substituted base/interface
signatures. Instantiated field queries check locally known substituted field contracts.
Host function resolution uses the same checks. Serialization, linking and signature
substitution retain the rows; source syntax is not required for enforcement.

Symbolic forwarding is allowed in this initial subset. An unconstrained Forward<T>
may call a constrained function using T, but resolving the concrete constrained call
must succeed before that body executes. An invalid forwarding path can therefore fault
at execution instead of source compilation. This is not a complete proof that every
possible instantiation of a generic body is valid, nor does it authorize removing
runtime checks. Closed violations visible during verification fail earlier.

This is an additive format-5 metadata extension. Unconstrained artifacts omit the new
field. Older readers use strict unknown-field rejection on these definitions and reject
constrained artifacts rather than silently ignoring their contracts. Rebuild tools when
using the new metadata. Neo also reserves `where`, `notvoid` and `notreference` as names.

## .NET comparison and next steps

Microsoft's [constraint guide](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/generics/constraints-on-type-parameters)
(consulted 2026-09-09) describes C# `where` clauses, base/interface constraints and the
class/struct distinction. Its `notnull` constraint produces warnings in a nullable
context. Neo reuses the familiar clause structure, but addressing mode is separate
from nominal type identity, and these two restrictions are runtime contracts.

The benefit is enforceable argument intent across Neo, IL and host calls without
reintroducing nominal reference/value types. The cost is new metadata and checks,
plus a bounded frontend that currently defers some forwarding failures until runtime.
No JIT or allocation improvement is claimed. A compiler-only prohibition was rejected
because it would not protect IL/artifact consumers; a recursive reference-free rule
was rejected because it would express a different object-layout contract.

## Nominal bounds and borrowed member lookup

```swift
func Read<T>(readonly value: T&) -> int where T: Readable {
    return value.Read()
}
func Compare<T>(readonly left: T&, right: T) -> int
where T: System.Comparable<T> {
    return left.CompareTo(right)
}
record Holder<T>(Value: T) where T: Readable
```

A bound names a record (the same type or a base) or an interface (direct or inherited
conformance). At most one record bound is permitted per parameter; multiple distinct
interface bounds can be combined with the flag restrictions. Constructed bounds can
refer to owner/method parameters, for example `.constraint T System.Comparable<T>`.
Their types participate in binding, substitution, module scope/access checks and JSON
round trips. Type-parameter bounds such as `where T: U` are not implemented.

Conformance is independent of addressing mode: `Cell`, `Cell&` and `readonly Cell&`
all satisfy a `Readable` bound when Cell implements it. Native pointers do not.
The bound does not convert or slice the argument, borrow a value, change inference,
or authorize mutation through readonly access. `Holder<Cell&>` retains its reference;
`Holder<Cell>` stores a copy according to Cell's own field contracts.

The first member-lookup projection supports **method invocations on T& receivers**,
including readonly receivers, in generic functions and static methods. Record-bound
methods must themselves use a byref receiver; copying a potentially derived object
into a base value receiver is not introduced by this feature. A stack value
still needs `&value` at the call site; an existing managed reference passes directly.
Neo looks up source/bundled bound methods, including inherited members, and emits
`interface.borrow` or `castclass`, followed by the ordinary `callvirt`/`call`.
These checked views retain the concrete object and use its explicit/default interface
implementation or virtual override. They create no box and preserve lifetime checks.
Ambiguous members from unrelated bounds are rejected; use a narrower helper contract.
Unconstrained members, direct member access on bare T, bound fields/properties and
method-group conversion and constraint-aware closure lowering remain outside this first lookup slice. Generic source-record
methods are still a separate language feature.

The verifier accepts an open parameter's projection only when its declared bound
proves the view. Concrete execution rechecks conformance and the receiver capability.
Symbolic forwarding still has the previously described limitation: this is not a full
constraint-implication verifier. Partial frontend library probes cannot prove contracts
involving missing application definitions; full linking validates those definitions
and repeats concrete checks.

### Constrained reference conversions

A declared bound also permits an existing `T&` to become a base/interface view:

```swift
func Observe<T>(readonly value: T&) -> readonly Readable& where T: Counter {
    let baseView: readonly Counter& = value
    return value
}
```

Here Counter must declare Readable conformance (directly or through an ancestor).
The same conversion works for arguments, local/field assignments and returns.
Source-type `as` projections use the same rule. Bundled generic interface targets,
such as `System.Comparable<T>&`, work through target-typed assignments and arguments.
Converting to an inherited view needs proof from the declared bound; the concrete
argument happening to implement an unrelated interface does not provide that proof.
There are no downcasts, container variance, bare-T borrowing or base-value slicing.
Readonly input can only produce a readonly view. `out` arguments still require exact
output storage; conformance does not make output locations interchangeable.

Once projected, existing base fields and interface properties/methods are available
through that view. The conversion retains the original object and uses the existing
`castclass`/`interface.borrow` instructions, including verifier evidence, readonly
capabilities and runtime lifetime validation. Returning a view into an argument's
storage is valid while the caller keeps that storage alive; returning a view into the
current frame still faults. A managed heap view retains its owner under the existing
GC rules. There is no metadata or instruction change in this slice.

The [C# specification, §10.2.12](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/conversions#10212-implicit-conversions-involving-type-parameters)
(consulted 2026-09-09) provides conversions from constrained type parameters to their
base/interface types, with reference or boxing behavior depending on the argument.
Neo adopts the familiar use of bounds as conversion evidence, but adapts the operation
to an already explicit managed reference. Copying or boxing would lose the stack
object's identity; refusing all conversion would force repetitive adapters around
ordinary APIs. Checked views reuse the runtime's existing capability and lifetime
rules. Their cost is the continuing explicit `T&` API boundary and runtime escape
checks; this is not a performance claim or CLR boxing compatibility.

This is a platform capability, not a conclusion about language usability; see
[the distinction in the type/API guidance](type-design.md#platform-capability-and-language-usability).

See [constrained-views.neo](../examples/source/constrained-views.neo) for base-field
updates, virtual dispatch and reference identity through generic and ordinary APIs.

### Comparison with CLR constrained calls

The shipped [OpCodes.Constrained contract](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.constrained?view=net-10.0)
(consulted 2026-09-09) describes how the prefix selects reference/value receiver handling
for generic calls and can avoid boxing. The CLI metadata specification (ECMA-335, sixth edition, June 2012), Partition II
§22.21, defines GenericParamConstraint rows referring to type bounds; this is the
baseline for keeping bounds in metadata rather than solely in Neo. See
[ECMA-335](https://ecma-international.org/wp-content/uploads/ECMA-335_6th_edition_june_2012.pdf).

NeoCLR reuses nominal bounds and ordinary virtual dispatch. Its explicit T& receiver
already supplies a managed address, so this slice reuses checked reference-view
instructions instead of adding a `constrained.` prefix. The benefit is preserving
existing identity, readonly and lifetime rules with no allocation mechanism. The cost
is a narrower source API: bare T cannot yet select value-versus-reference receiver
handling after substitution. Supporting that later may justify a CLR-like prefix;
silently borrowing, copying, or boxing is not an acceptable substitute. Receiver
adaptation does not itself require boxing: it should select the existing value/address
representation. NeoCLR does not need to reproduce CLR boxing for base/interface
views, since managed references already retain the concrete object in either storage
location. Any future explicit value-erasure/container operation is a separate contract. There is no
claim of JIT performance improvement or full CLI compatibility. Tests exercise stack,
heap, base overrides, inherited interfaces, explicit/default implementations and
rejection from source, metadata, host calls and verification.

The [comparison probe](experiments/generic-bounds-dotnet/Program.cs) was run on
2026-09-09 with SDK 10.0.100, runtime .NET 10.0.0, macOS ARM64. It printed `42`,
`42`, `invalid bound rejected`: a constrained ref receiver mutated its original struct,
a base-constrained call reached the override, and reflection rejected an invalid
constructed generic method. The added conversion probe prints `value=42, interface=43`
and `True`: C# boxes the struct on conversion to an interface while the base-class
conversion preserves class-instance identity. Neo's explicit managed views deliberately
preserve the original object for both stack and heap storage. It does not compare allocations or throughput. Reproduce
with `cd docs/experiments/generic-bounds-dotnet` followed by `dotnet run`; the directory
pins its SDK in global.json.

Constructor constraints, complete implication checking and reflection enumeration
remain future work. `notnull` awaits nullable type metadata and must distinguish
nullable type arguments from uninitialized storage; see the
[nullability groundwork](runtime-groundwork-review.md).

## Run and validate

```sh
cargo run --locked -- run examples/source/generic-constraints.neo
cargo run --locked -- run examples/source/generic-bounds.neo
cargo run --locked -- run examples/source/constrained-views.neo
cargo test --locked --test neo_constrained_views --test generic_bounds --test generic_constraints --test generic_metadata --test neo_generic_records --test neo_generics
```

All three examples print 42 and return 42. Tests cover source/IL contracts, JSON validation,
host resolution, symbolic forwarding, shallow reference restrictions, pointers,
substituted base and field contracts, and malformed or unsupported constraints.
