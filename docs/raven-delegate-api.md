# Func and collection predicates from Raven

The target surface exposes the five existing System.Func delegate types, from
Func<TResult> to Func<T1,T2,T3,T4,TResult>. The bounded importer binds static
application functions and supports ordinary callback invocation syntax. It checks
the target signature and accessibility before emitting the runtime's existing
`delegate.bind` instruction. Function pointers are not exposed to guest code.

```raven
func IsAnswer(value: int) -> bool { return value == 42 }
let values = ArrayList<int>()
values.Add(42)
let found = values.Find(IsAnswer)
```

ArrayList.FindIndex, Exists and Find now use the shared runtime implementations.
Find returns Option<T>; no-match is None and FindIndex returns -1. The adapted
collection library preserves its normal-path iterator disposal using no-result
Dispose calls. A callback fault remains terminal; this slice does not introduce
fault-path finally/defer cleanup.

For completion callbacks use the target's named `Func<Void>`:

```raven
func Finished() { Console.WriteLine("Done") }
let callback: Func<Void> = Finished
callback()
```

The runtime delegate contract returns its unit value; adapters bridge a no-result
application target into that contract and discard the unit at Raven's no-result
invocation boundary. Generic Void return signatures are distinguished from a literal
CLI void return before substituting generic arguments. `Func<()>` currently attempts
to combine a synthesized Raven Unit type with target metadata and is not supported
by this experiment. It is not a substitute for the named Void spelling here.

[The executable sample](experiments/raven-target/samples/library-delegates.rvn)
covers all five arities, completion and present/missing predicates. The source
compiler requires Raven experiment commits `5f274c063` (retain constructor metadata
proxies) and `8e0f6cb7d` (respect emitted void signatures in callback bridges).
Both fixes have compiler-owned regressions; the latter also reproduces and fixes an
ordinary CLR InvalidProgramException. Installed tools must be refreshed separately.

This reuses the [delegate contract and .NET comparison](delegate-contract.md).
The familiar constructor/Invoke metadata is projected onto the existing runtime
binding model, without adding opcodes. Static targets demonstrate this API slice;
capturing lambdas, bound application-instance callbacks, multicast behavior and
arbitrary function-pointer operations are not claimed.

## Array callbacks

`Array.ForEach<T>(T[], Func<T,Void>)` now uses the existing runtime algorithm with
ordinary managed Int32 and String arrays. Raven accepts both inferred and explicit
method type arguments and wraps matching static functions. The callback runs once
per element in index order; a fault terminates execution. The adapted generic
method returns no result, while its Func<T,Void> call still consumes the runtime
unit result. No Action family or new opcode is introduced.

[The array sample](experiments/raven-target/samples/library-array-callbacks.rvn)
exercises both element shapes and an empty array with no callback invocations.
Raven experiment commit `de872fa34` emits empty arrays without a host Array.Empty
dependency (ordinary CLR compilation retains its existing optimization).
Raven experiment commit `11e9964f2` preserves target
MethodSpec signatures for closed imported generic methods; the importer still
admits individual supported API contracts rather than arbitrary generic methods.
