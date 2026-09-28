# Function callbacks (formerly delegates)

Development after Preview 11 replaces neoCLR delegates with structural Function
types and their objects. A shape describes ordered parameter/result contracts;
the object holds a checked method binding and any retained receiver or environment.
Different methods with the same signature use the same Function type.
See the [Function API reference](../api-docs/functions.md) and
[design and compatibility notes](function-types.md).

Raven callbacks use `(P0, ...) -> R`. Native artifacts use `fn<P0,...,R>` and
`function.bind`; `.delegate`, `delegate.bind`, the serialized Delegate representation
and named callable declaration admission are removed. Rebuild older applications
with matching compiler, reference, library and runtime artifacts. CLI Func/Action
metadata remains importer transport, not nominal neoCLR types.

The archived Neo frontend uses `fn<P0,...,R>` directly:

```swift
func Identity<T>(value: T) -> T { return value }
func Main() -> int {
    let callback: fn<int,int> = Identity<int>
    return callback.Invoke(42)
}
```

Function objects preserve checked access, generic targets, virtual/interface
selection, shared captures and heap retention. Frame receivers and unpublished
constructor receivers cannot escape through a binding. Default slots contain null;
invocation faults with NullReference. Constructors must initialize Function fields
before publication. Equality compares shape, closed method and retained receiver
identity. Function shapes inherit Object while remaining structural. Object views
retain the shape, support casts back, and dispatch value equality/hash and qualified
target ToString. Copies retain reference identity; separate equal bindings are
reference-distinct. Function: MethodInfo describes the bound target. No multicast or native function-pointer contract is added.
Named function types remain possible future work.

Examples: [method groups](../examples/source/function-objects.neo),
[callbacks](../examples/source/func-callbacks.neo), and
[Raven Function/async consumers](experiments/function-types/README.md).
The prior [CLR comparison](delegate-contract.md) is retained as design history.

The sections below preserve earlier decisions and observations; the development
contract above supersedes their delegate-specific implementation status.

## Function-type review reopened (2026-09-13)

**Superseded direction, 2026-09-28:** the author now selects Function types and
Function objects to replace and remove delegates, with a nominal/structural
introspection split. See the [design and migration plan](function-types.md).
Named function types may follow later; their identity rules remain open. The development replacement is now implemented. The text below preserves the earlier position.

The author asked whether delegates could instead be modeled as function types, without
selecting that direction. Keep the implemented [Raven callback/closure path](raven-delegates-lambdas.md)
as the baseline during investigation. The earlier Neo/address-mode description above
is historical and does not change the current ordinary type-category direction.

Compare a language function type lowered to existing delegates, a structural callable
signature backed by managed runtime values, and retaining nominal delegates as the
public contract. Language syntax alone need not require a new runtime type. Structural
function types could reduce named delegate boilerplate, but would introduce decisions
about type identity, overloads, generic variance, metadata and cross-language conversion.
A low-level function pointer does not by itself provide a GC-retained closure environment.

Require ordinary method-group conversion, lambda inference, invocation syntax, bound
instance/virtual methods and escaping captures to remain ergonomic. Compare equality,
nullability, ref/out parameters and no-payload Void results. Decide multicast/events
separately rather than assuming every function value needs an invocation list. Check
capture lifetime, reflection, debugger presentation and existing compiler emission before
selecting an alternative. No delegate replacement or new callable opcode is approved.

Primary baseline consulted 2026-09-13: [C# delegates](https://learn.microsoft.com/en-us/dotnet/csharp/programming-guide/delegates/)
already provide typed callbacks, method binding and lambda conversion. A new model
must improve a concrete use beyond renaming this existing capability.


## Delegate fields during class construction — 2026-09-15

A class constructor can assign a delegate field before returning. Allocation reserves
that field as a typed uninitialized slot because this preview has no default/null
delegate value. Reading it before assignment faults, and the outer constructor cannot
return with an uninitialized field. Other fields retain their existing defaults.
Normal delegate defaults remain unsupported; this does not make an invalid callback
callable or introduce nullable delegates.

This repairs a construction gap rather than changing the [delegate contract](delegate-contract.md):
previously allocation tried to construct a default delegate as a record and faulted
before the constructor ran. Compared with .NET's null reference field default, the
preview retains its existing checked delegate-capability rule, adding a constructor
completion check. Nullable delegates remain a separate decision. The runtime tests
cover assignment/invocation, missing assignment and an early read; all 25 delegate
checks pass, including rejection of ordinary delegate default initialization.
