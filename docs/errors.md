# Error values and terminal Faults

Recoverable application failures are data carried by Result<T,TError>. System.Error is
the prototype's message-bearing Error value; it is distinct from a terminal runtime or
system Fault. Guest code handles a Result case explicitly and can continue. It cannot
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

## Initial Error library API

| Member | Contract |
| --- | --- |
| static System.Error.FromMessage(String message) -> Error | Create an owned Error from runtime text |
| instance get_Message() -> String | Return the stored message |
| instance ToString() -> String | Return the message through the library accessor |

These are ordinary methods in System.neoil. [Property metadata](properties.md) now
associates the Message property explicitly with get_Message. FromMessage and get_Message call two validated
InternalCall helpers; ToString calls get_Message through IL. Runtime-service planning
identifies those helpers as ErrorValues. There are no new instructions or type categories.

The existing `error "literal"` instruction and errors returned by parsing, slicing,
and arithmetic library methods use the same Error representation. Empty messages and
embedded NULs remain valid text. Construction and message access preserve UTF-8 content
without normalization or interpretation as a format string. Messages are owned values,
not borrowed interpreter storage or native string pointers.

This is a small bootstrap representation, not the final structured error taxonomy.
Result<T,TError> does not require TError to inherit from System.Error: domain-specific
records can represent structured failures under the supported type contracts. No base
exception class, implicit error conversion, stack capture, or unwinding is introduced.

## Demonstration

```sh
cargo run --locked -- verify examples/errors.neoil
cargo run --locked -- run examples/errors.neoil
```

ReadPositive returns the parse Error unchanged for invalid text. A non-positive integer
produces a new Error with a message constructed from the input. Report explicitly branches
on Ok/Err, printing either the number or the Error's ToString result. Output is:

```text
42
InvalidInt32
Expected a positive number: -1
Execution continued
=> Void
```

The program uses free functions, primitive-backed library methods, and Result values;
its signatures and operations now use ordinary System.Result and wrapper members,
including the [migrated Int32.Parse boundary](int32-parse.md).
It needs no exception handling or extensive object model. Returned Error/Result values
can also be validated and imported into subsequent host invocations as owned data.

Malformed host inputs, execution limits, and failures of explicit helper allocation
reservations remain Faults. Execution Faults preserve the existing logical stack traces;
ordinary Error values have no automatic trace and are not turned into terminal failures
by reading or formatting them. Catastrophic host/native process failures retain their
existing limitations. Rich diagnostics, error codes, localization, and structured payload
conventions can be developed separately.
