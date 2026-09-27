# Option and Result APIs from Raven

The Raven library declares `Option<T>` and `Result<T,E>` with standard Raven
`union` syntax, following Raven.Core. `Some(T)` and `None` represent presence and
absence; `Ok(T)` and `Error(E)` represent success and recoverable failure.

```raven
import System.*
import System.Result.*
import System.Option.*

let outcome: Result<long, string> = Ok(42L)
if outcome is Ok(let value) {
    Console.WriteLine(value)
}
let failure: Result<long, string> = Error("Failure")
let missing: Option<string> = None
```

Use patterns to extract payloads and `?` to propagate absence or failure.
[Composition operators](raven-outcome-operators.md) include Map, Then, recovery,
branch actions and explicit iterable conversion.

## Generated union contract

Both carriers implement `System.Runtime.CompilerServices.IUnion`. Their `Value`
property returns the boxed active **case**, not its payload. `HasValue` distinguishes
an active case from a default carrier. An explicitly constructed `None` is active;
`default(Option<T>)` is inactive and is not a substitute for `None`. An inactive
carrier has `HasValue == false`, `Value == null`, and no matching case.

Generated `TryGetValue(out case)` overloads return true for the requested active
case. A mismatch returns false and preserves the destination. For a direct call,
qualify an explicit payload-case type with its closed carrier, for example
`Option<int>.Some<int>`; ordinary patterns infer this association. Case types remain
distinct when payload types coincide, including `Result<T,T>`. `Some<Void>` and
`Ok<Void>` carry a real unit value; `Some<Void>` differs from `None`.

`TryGetOutput`, `TryGetResidual` and `FromResidual` implement the ordinary Raven
propagation contract. The first two initialize their outputs to defaults and
report whether the corresponding success or residual case was present. Their
output behavior differs from conditional case extraction with `TryGetValue`.

The previous manual carrier predicates, checked case getters and `TryGet` aliases
are removed from the Raven API. Rebuild applications with matching references and
runtime libraries. Prefer case construction and patterns over representation APIs.
The historical Neo bootstrap profile retains its separate manual carriers until
that profile is retired or explicitly migrated.

## Compiler and runtime boundary

This reuses the [target contract comparison](raven-target-contracts.md) and
[runtime error contracts](runtime-error-contracts.md). Like CLR generic metadata,
member signatures are closed against the declaring type and checked before
execution. Raven's companion and case attributes associate source cases at the
bridge boundary; the VM executes ordinary fields, calls, boxing and interfaces.
No union opcode, VM case registry or exception translation is introduced.

The benefit is one source and metadata contract for generated unions. The cost is
a provisional importer that follows Raven's emitted shape and requires coordinated
compiler/reference/runtime updates. This is bounded target support, not a general
CLR loader. See the [focused contract sample](experiments/raven-target/samples/library-union-contract.rvn)
for the release validation cases.

## Task outcomes

`System.Tasks.TaskOutcome<T>` uses the same generated union contract, with
`Completed(T)` and `Cancelled`. Both are active cases; the default is inactive.
`Value` boxes the case, not its payload. A completed `Result.Error` remains a
completed task, distinct from cancellation. TaskOutcome has no Propagatable
implementation; task/await cancellation follows the task builder contract.

The focused contract fixture uses an explicit closed-case cast to inspect a boxed
Completed payload. In this compiler snapshot, a fully qualified Completed pattern
over Object can retain an open payload type, while qualifying it through the closed
carrier can fail to match the boxed case. Ordinary patterns over TaskOutcome itself
work. This compiler binding limitation is not a missing IUnion implementation.

Native reflection snapshots also return Options for declaring types and property
accessors. Their materialization boundary validates and constructs the selected
profile's closed storage shape: discriminator plus Some/None records for Raven,
erased case storage for the separate Neo bootstrap. This is a library interop
contract, not a general runtime union representation or CLR metadata requirement.
