# Managed constructors and chaining

Implemented 2026-09-08, before the reflection hierarchy migration. This extends the
[whole-value constructor subset](constructors.md) with managed receiver construction.
A constructor chain initializes one complete concrete owner; base calls do not
construct a separate base value or slice the derived value.

## Neo

```swift
abstract record Counter(Value: int) {
    init(value: int) { this.Value = value }
    readonly abstract func Read() -> int
}
record OffsetCounter(Offset: int): Counter {
    init(value: int, offset: int): base(value) {
        this.Offset = offset
    }
    readonly override func Read() -> int { return this.Value + this.Offset }
}
```

An explicit init replaces the record's positional aggregate call signature in Neo.
The parentheses on the record declaration still declare fields. The initializer's
parameters supply its call signature; field names are not implicitly assigned.
OffsetCounter(40, 2) produces a value; new OffsetCounter(40, 2) constructs that value
and places it on the managed heap using Neo's existing lowering.

The small compiler supports one explicit initializer per record, and a derived init
requires an explicit : base(...) whose base declares init. Arguments evaluate left
to right before entering the base constructor. There is no implicit Object
constructor. Source overload declarations and : this(...) are deferred; IL supports
overloaded constructors and same-type delegation now.

Records without init retain aggregate construction, including inherited positional
fields. Constructor chaining does not make constructors the exclusive way to create
a value: the existing aggregate/default operations retain their own contracts.

## IL and storage

```text
.type abstract Counter
.field Value Int32
.method instance byref .ctor(Int32 value) -> Void
    ldarg this
    ldarg value
    stfld Counter::Value
    pop
    ldvoid
    ret
.end
.end

.type OffsetCounter
.extends Counter
.field Offset Int32
.method instance byref .ctor(Int32 value, Int32 offset) -> Void
    ldarg this
    ldarg value
    call instance Counter::.ctor(Int32)
    pop
    ldarg this
    ldarg offset
    stfld OffsetCounter::Offset
    pop
    ldvoid
    ret
.end
.end
```

newobj instance OffsetCounter::.ctor(Int32,Int32) consumes the declared arguments
and produces a complete OffsetCounter value. The runtime creates unpublished storage
for all inherited and own fields and passes a restricted managed receiver. Fields
begin uninitialized, not null or implicitly zeroed. Abstract bases can initialize
their portion of this storage but cannot be allocated as complete abstract objects.

call to a managed constructor is legal only from an active managed constructor,
using its own receiver and targeting the same type or its direct base. The runtime
projects that receiver internally; no user cast of an uninitialized value is needed.
At most one delegation call executes in each constructor activation. A derived
constructor must complete its base chain before field access. Same-type delegation
can complete the initialization obligations for its caller. Cyclic delegation faults.

Constructors may assign their own fields through managed stfld, or ldflda/stobj
where the field has an addressable type. Managed stfld consumes receiver and value
and produces Void, matching this prototype's explicit Void result convention. The
existing value-receiver stfld still produces an updated record value. Direct field
storage supports reference fields without introducing nested managed references.
Readonly views and declaring-field visibility remain enforced.

Field reads require initialization; inherited fields can be read after base
completion, subject to visibility. Constructors cannot obtain writable addresses
of inherited fields or replace the complete receiver. All own fields and the base
chain must be initialized at every successful return. This includes fields whose
type is Void and explicit managed-reference fields.

## Publication, verification and lifetime

A construction receiver or its interior field address cannot be stored, returned,
used as an out argument or passed to an ordinary method. This restriction lasts
until the outermost constructor returns, even after all fields have been assigned.
It includes virtual/interface calls and runtime identity/reflection queries on this.
Ordinary functions may compute initializer values; initialized fields can hold
independent managed heap references. Their targets remain ordinary usable objects.

The verifier tracks direct receiver/field capabilities, definite field assignment,
base completion and possible delegation across control-flow merges. It rejects
capability storage/ordinary access and repeated delegation, including loops that
could execute a base call again. Runtime checks independently enforce initialization,
owner-only writes, call eligibility, publication and cycles without verification.
Cycle rejection currently occurs at execution, not through a global constructor
graph proof. This is not a general-purpose partial-initialization alias analysis.

Unpublished storage is an explicit GC root and participates in array budgets.
A fault discards the incomplete receiver; this adds no automatic resource cleanup
or guest exception unwinding. Constructor and base frames appear in normal traces
and debugger stepping. The debugger displays <construction storage> under the
owning frame's locals as a diagnostic entry, not an addressable local index;
receiver references identify frame#.construction.

## .NET comparison and tradeoffs

Primary sources consulted 2026-09-08:

- [C# specification §15.11.2–4](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/classes#15112-constructor-initializers)
  defines base/this initializers, cycle rejection and constructor execution order.
  C# performs declared instance field initializers before a base constructor call;
  Neo currently has no such field-initializer feature and runs explicit derived
  field assignments after base completion. Do not equate these ordering rules.
- [.NET 10 stfld](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.stfld?view=net-10.0)
  supports addressed field mutation and leaves no result. neoCLR retains its explicit
  Void result convention and its older record-value update form.
- [CLI Partition III, call and newobj](https://download.microsoft.com/download/7/3/3/733ad403-90b2-4064-a81e-01035a7fe13c/ms%20partition%20iii.pdf)
  provides the opcode baseline. neoCLR reuses these instructions and the existing
  byref receiver metadata; no new opcode or mandatory Object base is introduced.

Copying a separately constructed base would complicate owner identity and prevent
a consistent initialization contract. Adopting default-filled CLR objects and
allowing ordinary calls during construction would expose states neoCLR's non-null,
value-default model does not promise. Restricted unpublished storage keeps those
states internal and centralizes checks across languages. Costs include field-state
tracking and more restrictive constructor bodies: use argument/static helpers
instead of instance helper calls. Placement construction, source overloads, richer
initialization capabilities and cleanup on failure remain future work.

The [comparison probe](experiments/constructor-chaining-dotnet/Program.cs) pins SDK
10.0.100/net10.0 and checks base, derived and delegating body order plus final virtual
dispatch. On the pinned SDK it printed base, derived, delegating, then 42. It does
not establish performance, CoreCLR layout or Neo frame lifetimes.

## Run and compatibility

```sh
cargo run --locked -- run examples/source/constructor-chaining.neo --gc-stats
cargo run --locked -- verify examples/source/constructor-chaining.neo
cargo test --locked --test constructor_chaining --test constructors
(cd docs/experiments/constructor-chaining-dotnet && dotnet run)
```

The Neo sample returns 42. Tests include source/JSON round trips, generic abstract
bases, delegation, missing/conditional initialization, GC pressure, readonly field
stores, escape rejection, ordinary-call rejection, debugger storage and legacy
constructor behavior.

Old root record constructors with value receivers retain whole-value initialization.
They cannot serve as base constructors; derived constructors require managed byref
receivers. New byref constructor artifacts and managed stfld operands need this
runtime revision. No metadata fields or serialized opcode names were added.
