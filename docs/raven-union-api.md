# Option and Result APIs from Raven

The source-built bridge projects the existing Option and Result library contracts
through closed generic metadata. It maps admitted payload types recursively rather
than maintaining a separate adapter for each result/error pair. Primitive values,
String, Date/Time/LocalDateTime, error values and nested Option/Result shapes are
supported; arbitrary application types and reference payloads remain separate work.

```raven
let outcome = Result<long, Error>.Ok(42L)
if outcome.IsOk {
    let value = outcome.GetOkCase().Value
}
let failure = Result<long, Error>.Error(Error.FromMessage("Failure"))
let missing = Option<string>(Option.None())
```

The surface includes case constructors and Value access, carrier constructors,
Result.Ok/Error factories, predicates, checked case getters, TryGet,
TryGetOutput/TryGetResidual and FromResidual. Raven's TryGetValue metadata methods
project the runtime TryGet overloads for patterns. The synthetic carrier Value
property is a compiler recognition hook, not a callable runtime API.

Case getters fault on the wrong variant. Conditional output addresses become
readable only on the successful branch. A default carrier is not a valid case;
construct an explicit None, Some, Ok or Error. Empty cases and cases with valid
default payloads can be initialized; a string or message payload is not invented.
Named Void remains a legitimate generic payload, separate from CLI void returns.

This reuses the [target contract comparison](raven-target-contracts.md) and
[runtime error contracts](runtime-error-contracts.md). As with CLR generics,
member signatures are closed against their declaring type and validated before
execution. The adapter remains a bounded experiment, not a general CLR loader.
Result/Option error flow and guarded outputs preserve neoCLR's existing contracts;
no runtime opcode or exception behavior changes are introduced.

Execution coverage is in [the union sample](experiments/raven-target/samples/library-unions.rvn)
and the saved-project checks. Installed SDK/extension packages must be refreshed
before their metadata includes this surface.

## Writable case payloads

The target projects the original public case Value field/property pair as a read/write
Value property. Assigning `case.Value` mutates that case value's storage; a carrier
previously constructed from it retains its copy. This preserves the existing runtime
API rather than silently making payloads immutable during metadata projection.
The [case-payload sample](experiments/raven-target/samples/library-case-payloads.rvn)
covers Ok, Error and Some updates and carrier-copy independence.

## Composing outcomes after Preview 8

The development library adds [Option and Result operators](raven-outcome-operators.md),
including Map, Then, recovery, branch actions and explicit iterable conversion.
The guide lists the full bounded port and Raven.Core compatibility differences.
