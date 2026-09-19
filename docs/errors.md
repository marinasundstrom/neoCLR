# Error values and terminal Faults

Recoverable application failures are ordinary data carried by Result<T,TError>.
Use a string for a simple message or a domain-specific type for structured failure. Guest code handles a Result case explicitly and can continue. It cannot
catch a Fault or resume the failed execution.

## Platform policy clarified 2026-09-12

.NET semantics are the baseline unless an improvement is justified. Result-based
recoverable error flow is an explicitly chosen library/platform divergence. Operations
whose failure a caller is expected to handle should expose typed Result errors; they
must not turn those ordinary outcomes into terminal faults merely to avoid designing
their error contract.

Faults represent terminal failures of the current execution. They are not guest class
instances, do not derive from System.Exception, and are not a substitute class hierarchy
with a different name. The current Rust `Fault` record carries a message and optional
function, instruction and stack trace to the host; guest code cannot catch it. That host
representation is an implementation detail, not a required guest object model. Result
error payloads remain ordinary types or union cases without an exception base class.

.NET's exception-object/catch model is therefore not a compatibility promise. Existing
exception-centered source must be adapted, and unsupported imported exception handlers
must be rejected rather than silently dropped or converted. Fault containment, resource
cleanup and future async failure boundaries still need explicit contracts; this policy
does not claim they are solved. An external host may have its own exceptions without
making them neoCLR guest objects.

## Ordinary error payloads

The development library retires `System.Error`, the message wrapper from before the
union convention. `Result.Error(...)` is a union case, not that old type. There is
no common error base type and Task treats a Result as an ordinary payload.

```raven
import System.*
import System.Result.*

let failure: Result<int, string> = Error("Unavailable")
```

This is a breaking development change after Preview 8: replace message-wrapper
payloads with `string`, remove `Error.FromMessage` and `.Message`, and rebuild
references, libraries and callers together. Domain-specific error unions remain.
The wrapper's native helpers, host Value/Type variants, ErrorValues service and
legacy `error "literal"` instruction are removed; neoIL uses `ldstr` for text.
Published Preview 8 bundles are unchanged.

Compared with .NET's exception hierarchy, expected failure remains an explicit
Result value without stack capture or catch/unwind semantics. Removing the wrapper
reduces special runtime support and avoids an imported Result case name collision.
The cost is source/metadata migration; a string alone does not classify failures,
so APIs requiring branches should continue to use specific error cases.

## Demonstration

```sh
cargo run --locked -- verify examples/errors.neoil
cargo run --locked -- run examples/errors.neoil
```

ReadPositive formats the typed parse error for invalid text. A non-positive integer
produces a message constructed from the input. Report explicitly branches on the
Result cases, printing either the number or the message. Output is:

```text
42
InvalidFormat
Expected a positive number: -1
Execution continued
=> Void
```

The program uses free functions, primitive-backed library methods, and Result values;
its signatures and operations now use ordinary System.Result and wrapper members,
including the [migrated Int32.Parse boundary](int32-parse.md).
It needs no exception handling or extensive object model. Returned Result values
can also be validated and imported into subsequent host invocations as owned data.

Malformed host inputs, execution limits, and failures of explicit helper allocation
reservations remain Faults. Execution Faults preserve the existing logical stack traces;
ordinary error payloads have no automatic trace and are not turned into terminal failures
by reading or formatting them. Catastrophic host/native process failures retain their
existing limitations. Rich diagnostics, error codes, localization, and structured payload
conventions can be developed separately.
