# Neo.Signals

Experimental library-based signals for neoCLR. This is a separate extension assembly;
whether it should become part of the runtime remains an open decision.
It requires the matching development Raven `codex/neoclr` compiler and neoCLR runtime.

A publisher owns `SignalSource<T>` and exposes its stable `Signal<T>` view through
an ordinary read-only property. Consumers can subscribe through the view; only the
source can publish. There are no event metadata rows or nominal delegate types.

```raven
class Publisher {
    private val source = SignalSource<(int, string?) -> unit>()
    val Changed: Signal<(int, string?) -> unit> => source.Signal
    func Change(value: int, label: string?) { source.Emit(value, label) }
}

let publisher = Publisher()
let subscription = publisher.Changed.Subscribe((value, label) => {
    Console.WriteLine(value)
})
publisher.Change(42, null)
subscription.Dispose()
```

Import `System.*` and `Neo.Signals.*`, and reference `Neo.Signals.rvnproj` plus
`System.Runtime.rvnproj`. See the [executable sample](../../samples/signals/).

## Contract

- Handlers must return `unit`. Construction with another result type calls `Panic`.
- Delivery is synchronous and follows registration order. An empty source does nothing.
- Duplicate registrations are independent. Each returned `Disposable` removes
  exactly its own registration; disposal is idempotent.
- Dispatch uses a registration snapshot. Adding or removing handlers during delivery
  affects subsequent dispatches, including a nested dispatch started afterwards.
- A handler fault propagates immediately and stops that delivery. There is no error
  callback, completion state, Task unwrapping, scheduler or Rx operator layer.
- Concurrent calls require external synchronization. Registrations retain their
  function objects and captures until removed and absent from active snapshots.

`Emit` uses the general `func Emit: T with args` capability. Its public parameters
materialize from `T`; its implementation receives an owned `object?[]` and calls
`FunctionReflection.Invoke<T>`. Boxing, argument-array allocation and reflection
are intentional costs of this initial interop implementation. No allocation or
throughput advantage over .NET events is claimed. `+=`/`-=` sugar is not included.

## Build

```sh
dotnet /path/to/rvnc.dll neoclr --project extensions/Neo.Signals/src/Neo.Signals/Neo.Signals.rvnproj
dotnet /path/to/rvnc.dll neoclr --project samples/signals/src/SignalsDemo/SignalsDemo.rvnproj
```

Run the sample with the matching native System seed, System.Runtime and Neo.Signals
assemblies. Runtime/compiler integration status and supported execution modes are
recorded in [generic function methods](../../docs/design/generic-function-methods.md).

Execution qualification: the signal consumers pass in the interpreter. The native
compiler builds and shares signature specialization, but the signal JIT probe stops
at unsupported boxing of inhabited unit (`Void`). JIT/AOT execution of this library
is not qualified; unit boxing/unboxing support remains follow-up work.
