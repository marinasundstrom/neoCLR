# References: Raven-like pseudocode and neoIL

Current memory milestone: format 5 uses direct heap-backed T&. Ref and heap.load/store
are removed; heap-only references can be stored in fields/erased payloads and returned
to the host for context-bound inspection. See [the current contract](heap-references.md).

The executable [Neo concept language](neo.md) now implements a small subset of these
ideas; see its guide for accepted source. The examples below still explain the broader
platform contracts. They are **pseudocode,
not inputs to an existing compiler or a specification of Raven**. They follow the
project's name-before-type source notation, with familiar reference, output and
explicit-memory notation inspired by C# and Rust. In particular, this is not Rust's
exclusive borrowing model. The corresponding neoIL samples assemble and execute.

`&value` forms a managed reference to a slot. `T&` describes the type of that
reference. Managed-reference reads and writes are automatic in Neo and in the
pseudocode below; the corresponding IL loads/stores remain explicit. Raw pointers
retain their separate low-level access rules. See the
[runtime-to-Neo reference guide](managed-reference-semantics.md).

| Form | Meaning |
| --- | --- |
| Int32 | An ordinary copied value |
| Int32& | Retaining managed reference to an exact Int32 slot |
| Int32* | Raw native pointer with explicit memory/lifetime obligations |
| Int32*& | Managed reference to a slot whose value is a raw Int32 pointer |
| Settable& | Managed interface view over an implementing concrete slot |
| Ref<T> | Historical proposal removed in format 5; use T& for managed references |

The examples below focus on scoped calls. The same CLR-style T&/ByRef feature now
supports reference locals, managed field addresses and checked caller-backed guest
returns; see [slot contracts](reference-slots.md).
Managed heap allocation and heap-backed reference fields are also implemented;
guest destruction remains future work under the [lifecycle direction](lifecycle.md).

## Passing a reference

```text
func Increment(value: Int32&) -> Void {
    value = value + 1
}

var count: Int32 = 41
Increment(&count)
Console.WriteLine(count) // 42
```

The caller's `&count` lowers to `ldloca count`. The parameter contains the reference,
so `ldarg value` alone does not load the integer:

```text
.function Increment(Int32& value) -> Void
    ldarg value
    ldarg value
    ldobj Int32
    ldc.i4 1
    add
    stobj Int32
    ldvoid
    ret
.end
```

The first reference stays on the evaluation stack as the store destination; the
second is consumed by the load. stobj updates the caller's slot immediately. There
is no copy-back at return and no implied allocation or ownership transfer.

An ordinary T& parameter requires initialized storage when the call begins. Two
parameters may alias the same slot; later reads observe earlier writes. The VM
neither requires unique mutable borrowing nor grants a no-alias optimization promise.
Languages can choose stronger restrictions.

## Assigning an output

```text
func Assign(destination: out Int32&, value: Int32) -> Void {
    destination = value
}

var answer: Int32
Assign(out &answer, 42)
Console.WriteLine(answer)
```

`out &answer` spells both the call's output intent and the address operation in this
pseudocode. A compiler could choose a shorter source spelling. Normalized call
operands contain Int32&, while the callee's metadata carries the out contract:

```text
.function Assign(out Int32& destination, Int32 value) -> Void
    ldarg destination
    ldarg value
    stobj Int32
    ldvoid
    ret
.end
```

The caller lowers to ldloca answer, ldc.i4 42, call Assign(Int32&,Int32), then pop
the real Void result. It can now load answer. [The executable sample](../examples/reference_parameters.neoil)
also forwards the output reference through another call.

Out accepts an uninitialized slot, but every normal return must follow an assignment
in that invocation. A preexisting value is not enough. Runtime checks enforce the
contract even without optional verification. Assignments through aliases and nested
calls count; reads through an unassigned out capability fault. Writes before a Fault
are not rolled back, and replacing a value performs no automatic resource release.

## Mutating a receiver

```text
type Counter {
    Value: Int32

    func Set(this: Counter&, value: Int32) -> Void {
        this.Value = value
    }
}

var counter = Counter(Value: 7)
(&counter).Set(42)
```

The explicit `this: Counter&` projects to `.method instance byref Set(Int32 value)`.
The call supplies ldloca counter. The method loads a Counter value with ldobj,
produces its changed value with stfld, then stores that value through the original
reference with stobj. The [reference receiver sample](../examples/reference_receivers.neoil)
contains the complete body.

Ordinary instance methods still receive copies. Neither method syntax nor the
concrete type's declaration silently determines a heap allocation or a reference
receiver. A future language may make this projection more convenient.

## Projecting an interface

```text
interface Settable {
    func Set(this: Self&, value: Int32) -> Void
}

// Counter declares that it implements Settable.
func Reset(target: Settable&) -> Void {
    target.Set(0)
}

Reset((&counter) as Settable&)
```

Self is a pseudocode placeholder for the contract receiver, not a new IL type.
Counter remains the concrete value in its original slot. The interface view exposes
a contract and the information needed for dispatch; it does not create an interface
object, copy the Counter, box it or keep its storage alive.

```text
ldloca counter
interface.borrow Settable
call Reset(Settable&)
pop
```

Inside Reset, ldarg target followed by ldc.i4 0 and
`callvirt instance Settable::Set(Int32)` dispatches to the concrete byref method.
The view is used directly for dispatch; `*target` does not materialize an abstract
interface value. The implementation and contract must agree on receiver mode.
Value-receiver contracts are also supported, with their existing copy semantics.

[The List sample](../examples/interfaces.neoil) passes List<Int32>& to Sum while
keeping allocation and Free on the concrete ArrayList owner. [The Counter sample](../examples/reference_receivers.neoil)
shows inline mutation through a byref interface contract. Native-pointer
InterfaceRef<I> remains a distinct, lower-level view; it cannot supply a managed
byref receiver. See [both interface forms](interfaces.md).

## Trying to extract a union case

A TryGet method often has no meaningful value to write on failure. The conditional
`out(true)` contract requires assignment on success without inventing a default T:

```text
var ok: Result.Ok<Int32>
if result.TryGet(out &ok) {
    Console.WriteLine(ok.Value)
}
```

The library declaration is equivalent to this pseudocode signature:

```text
func TryGet(this: Result<T, E>, destination: out Result.Ok<T>&) -> Boolean
```

The caller lowers its successful extraction path as follows, assuming result is
already initialized:

```text
.local System.Result.Ok<Int32> ok
ldloc result
ldloca ok
call instance System.Result<Int32,String>::TryGet(System.Result.Ok<Int32>&)
brfalse Miss
ldloc ok
call instance System.Result.Ok<Int32>::get_Value()
call System.Console::WriteLine(Int32)
pop
```

The ordinary library method tests the case, stores a typed case value through the
reference and returns true. On mismatch it returns false without changing the
output. This is ordinary call/branch/load/store IL, not a special union instruction.
The extracted case is a copy, not a reference into the carrier's payload storage.
The carrier's existing bootstrap representation is unchanged by output references.

The verifier recognizes direct brtrue/brfalse tests of the call result and marks
known local outputs initialized only on the success edge. Its first implementation
does not propagate that fact through Boolean locals, comparisons or arbitrary alias
merges. Runtime assignment checks remain active regardless. False grants no new
initialization guarantee; a previously initialized output remains usable.

The general convention is `TryGet(out Case& value)`. Overload resolution uses the
case type, including when different variants have the same payload type:

```text
Option<T>.TryGet(out Some<T>& value) -> Boolean
Option<T>.TryGet(out None& value) -> Boolean
Result<T, E>.TryGet(out Ok<T>& value) -> Boolean
Result<T, E>.TryGet(out Error<E>& value) -> Boolean
```

Some, None, Ok and Error above are shorthand for the corresponding companion/nested
case types. The source signature uses `out`; neoIL declarations use `out(true)` to
encode the conditional assignment guarantee. Normalized call signatures contain
only the case-reference types; the callee metadata carries the output contract.

System.Option and System.Result implement these TryGet overloads. [The runnable union sample](../examples/union_try_get.neoil)
prints 42. The [native pointer carrier sample](../examples/pointer_union.neoil) now
uses managed output slots too: copying methods use out(true) T&. Its payload storage
still has an explicit native lifetime; extraction after that storage expires faults.

## What can cross a context boundary today?

Managed references can pass through active calls, be stored in T& locals and be
returned to guest callers when their root belongs to an active outer frame or the managed heap. Managed
ldflda forms field references with the same root lifetime. Returning an address into
the current frame faults, including addresses of nested fields, by-value arguments
and references forwarded through helpers. No automatic local promotion is implied.
Reference-valued fields and erased payloads may store heap-backed references only.
Heap-backed host results support inspection within their owning execution; reference
inputs across executions and native transfers remain unsupported.

Metadata and runtime checks enforce these boundaries; optional verification provides
earlier diagnostics. Interpreter references identify a stable host cell and field
path. Host allocation does not make an ordinary guest local a heap object: every
return validates the actual root against the returning frame. Native backends must
preserve this contract without copying the interpreter's host representation.

Array-element references, runtime block lifetimes, readonly capabilities,
threads and guest destruction remain separate gates. See the
[reference-return sample](../examples/reference_returns.neoil) and
[lifecycle direction](lifecycle.md).
