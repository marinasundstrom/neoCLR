# Delegates and callbacks

neoCLR implements nominal, single-target delegates. A delegate holds a closed IL
method binding and, for an instance method, a managed heap receiver. Invocation uses
the familiar Invoke member and ordinary call/callvirt instructions. Binding checks
the signature and access once; passing the callback transfers permission to invoke
that binding. Copies share the receiver, without boxing or copying its contents.

## Neo

```swift
delegate Transform(value: int) -> int

func Identity<T>(value: T) -> T { return value }

func Main() -> int {
    let callback: Transform = Identity<int>
    return callback.Invoke(42) // callback(42) is equivalent
}
```

Custom Neo delegate declarations are currently nongeneric. Runtime/IL delegate
declarations can be generic. A matching method group is wrapped automatically when a parameter, binding or return
supplies the delegate type. Explicit construction such as Transform(Identity<int>)
is also supported. Generic target methods require explicit type arguments.
Bound instance methods require an existing managed reference, for example
`Transform(counter.Add)` where counter is Counter&. The runtime rejects a frame
receiver and an unpublished constructor receiver. No implicit receiver copy or
promotion occurs. A static delegate can still take ordinary frame reference arguments.

A delegate is a value under neoCLR's addressing model. A D& binding refers to delegate
storage; Neo reads it automatically for invocation. Fields and arrays can store
delegate values, retaining their targets through GC. The debugger shows the bound
method and target heap identity. Step enters the actual method, Next steps over it,
and faults retain the ordinary caller/source trace.

## One Func family, including void results

The bundled library provides System.Func<TResult> through
System.Func<T1,T2,T3,T4,TResult>: zero to four input parameters, with the result last.

| .NET API intent | neoCLR delegate |
| --- | --- |
| Func<TResult> | System.Func<TResult> |
| Func<T,TResult> | System.Func<T,TResult> |
| Action | System.Func<Void> |
| Action<T> | System.Func<T,Void> |
| Explicit reference argument | System.Func<T&,TResult> |

There is no separate Action family. Void is already an ordinary neoCLR generic
argument and result; a call returning Void produces the existing unit stack value.
This removes duplicate callback families. It is an intentional API difference from
.NET, whose generic construction rejects Void; code porting Action APIs must use
Func with a final Void argument. It does not remove argument-arity distinctions.
See the [contract and .NET comparison](delegate-contract.md).

```swift
func Increment(value: int&) -> () { value = value + 1 }

func Main() -> int {
    var value = 41
    let increment: System.Func<int&, void> = Increment
    increment(&value)
    return value
}
```

Custom declarations preserve out and readonly parameter contracts:
`delegate Setter(out value: int&) -> ()`. A writable Func<int&,void> is not compatible
with an out parameter. Signature matching is exact, with no variance in this slice.

System.Array.ForEach<T>(readonly T[]&, System.Func<T,Void>) visits a managed array in
index order, stopping if the callback faults. It accepts a reference to either a
frame array or a heap array, and passes each element using T's value/reference mode.
Readonly protects array storage; it does not make objects referenced by its elements
deeply immutable. This nongeneric System.Array API is separate from the existing
legacy native buffer descriptor System.Array<T>.

## Run and debug

From the repository root:

```sh
cargo run --locked -- run examples/source/delegates.neo
cargo run --locked -- run examples/source/func-callbacks.neo
cargo run --locked -- debug examples/source/delegates.neo
cargo test --locked --test delegates --test debugger
python3 docs/experiments/delegates-dotnet/verify.py
```

The first sample prints 42 and 41, then returns 42. The Func sample prints 41, 42 and
43, then returns 42. See [debugger commands](debugger.md) for breakpoints, stack and
heap inspection.

## Lambdas and shared captures

```swift
func MakeCounter(start: int) -> System.Func<int> {
    var value = start
    return () => { value = value + 1; return value }
}
```

An expected delegate type supplies lambda parameter and return types:
`x => x + 1`, `(x, y) => x + y`, `(x: int) => x + 1`, and
`() => { statements }`. Optional parameter annotations must match that contract.
Use a typed binding, parameter, return, field, or explicit delegate construction
such as `Transform(x => x + 1)`. A bare `let callback = x => x + 1` has no expected
type and is rejected. This slice does not infer generic arguments from lambda bodies.

Closures share captured bindings with their declaring code and with other closures.
Changing an outer `var` is visible to the closure, and vice versa. A captured `let`
remains immutable under Neo's existing rules. The compiler allocates captured local
and parameter storage on the managed heap from initialization, so even addresses
taken before delegate creation refer to that shared storage. Returned and nested
closures retain it through GC. Range-loop iteration bindings get fresh storage on
each iteration. Captured references still mean references: storing a frame-backed
T& in the capture object faults; heap-backed references and heap-backed `this` work.
The compiler rejects captured out parameters, uninitialized local declarations and
constructor `this`. Nullable capture slots are not introduced.

A noncapturing lambda becomes a static delegate target. Capturing lambdas use one
managed cell per captured binding and a managed environment per lambda evaluation,
with references to the required cells. This is preliminary compiler lowering, with
extra allocation/indirection costs; it adds no runtime opcode or artifact format.
It differs from method-group binding, which requires an already heap-backed receiver
and never implicitly copies or promotes it. Names under `neoCLR.Compiler.` are
reserved for generated helpers. See the [CLR comparison](delegate-contract.md#closure-lowering).

Run the sample (prints 41 and 42, returns 42):

```sh
cargo run --locked -- run examples/source/closures.neo
cargo run --locked -- check examples/source/closures.neo
cargo run --locked -- debug examples/source/closures.neo
```

Use `step` to enter the generated lambda target, `next` to step over its invocation,
and `out` to return to the caller. Source positions point back to the lambda's Neo
source. `stack` and `heap` expose generated environments and Capture<T> cells;
source-level reconstruction of captured locals is not yet implemented.

## Remaining work

Natural lambda-type inference, explicit capture lists, stack-only closures and
capture allocation optimizations remain future work. Delegate equality compares the
generated method and environment identity, not captured values; do not depend on
whether two separate lambda evaluations compare equal.

Generic custom Neo delegate declarations, more arities, variance, multicast operations, open-instance binding, native callbacks and
guest Delegate.Method/Target APIs remain unimplemented. Delegate-valued expressions
support ordinary invocation, including
callback(args), holder.Callback(args) and MakeCallback()(args). Neo binds source methods and bundled static IL methods; the expected signature
selects static overloads. Bound bundled instance methods remain an IL-only path.
Defaults and null delegates are unsupported. Nullable metadata remains a separate
slice. Single-target IL equality compares closed delegate type, closed method and
receiver location; the guest Equals/GetHashCode API is not added here.
