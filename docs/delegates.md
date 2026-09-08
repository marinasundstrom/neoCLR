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

## Remaining work

Lambdas and closures will build on delegates, following the C# approach of
compiler-generated environment objects holding captured bindings. They will not
introduce a competing callable object model. Capturing a local binding into a shared
environment needs its own compiler/lifetime slice; it is different from silently
copying a receiver during method-group binding.

Generic custom Neo delegate declarations, more arities, variance, multicast operations, open-instance binding, native callbacks and
guest Delegate.Method/Target APIs remain unimplemented. Delegate-valued expressions
support ordinary invocation, including
callback(args), holder.Callback(args) and MakeCallback()(args). Neo binds source methods and bundled static IL methods; the expected signature
selects static overloads. Bound bundled instance methods remain an IL-only path.
Defaults and null delegates are unsupported. Nullable metadata remains a separate
slice. Single-target IL equality compares closed delegate type, closed method and
receiver location; the guest Equals/GetHashCode API is not added here.
