# Error handling

Raven represents recoverable failures as values. Result makes success and failure
visible in a function's return type; unions describe the alternatives, patterns
handle them, and `?` propagates a failure to the caller. These are the foundations
of error handling in neoCLR's APIs.

## Describe outcomes with unions

`Result<T, E>` is a union with two cases: `Ok(T)` carries the successful value and
`Failure(E)` carries the failure. `Optional<T>` uses `Some(T)` and `None` when absence
is enough and there is no error to explain.

The error type can itself be a union. FileReadError, for example, distinguishes
NotFound, AccessDenied, InvalidUtf8 and other file-reading failures. Named cases
let callers decide how to recover without parsing message strings. A domain API
can define its own union, including payloads that callers need to understand a
failure.

Errors can also be ordinary classes, structs or strings. Neither Result nor
propagation requires an error interface or a common base class. The author of an
API chooses the contract it exposes. [Optional and Result](../outcomes/) covers the
containers and their composition operators.

## Propagate when the caller should decide

Use `?` when a function cannot handle a failure meaningfully:

```raven
{{RAVEN_SAMPLE}}
```

Math.Abs returns `Result<int, OverflowError>`. On success, `?` extracts the amount
and execution continues. On failure, it returns the error from Normalize immediately;
“Continued” is not printed. The return type tells callers what can go wrong.

Propagation forwards an error through a supported conversion. It does not invent
a domain error, attach context or capture a stack. Runtime faults are separate:
recoverable error values do not introduce exception catching or unwinding.

## Handle an outcome where you can act on it

Patterns distinguish the cases and extract their payloads. This caller handles
the result from Normalize:

```raven
{{ERROR_HANDLING_CALLER_SAMPLE}}
```

For an error union with several cases, match the error payload too. A missing file
might use a default configuration, while access denied should be reported to the
user. Recovery belongs at a boundary that has enough information to make that
choice; functions in between can propagate.

## Choose the outward error contract

| What your caller needs | Approach |
| --- | --- |
| The same failure contract | Propagate the existing error with `?`. |
| Errors expressed in your API's domain | Define a domain error, often a named union; translate with `MapError` or a pattern match. |
| Several existing error types | Expose an ad-hoc union and handle its alternatives explicitly. |

**Development:** this configuration loader preserves file-reading and parsing
errors as separate alternatives:

```raven
{{ADHOC_ERROR_UNION_SAMPLE}}
```

The caller first extracts the Result error, then distinguishes FileReadError from
NumberParseError and handles that type's cases. Ad-hoc unions are useful for
composition; a named domain union can give a public API a more stable contract
that is independent of lower-level implementation choices.

## Optional diagnostic conventions

Once the error contract is chosen, authors may add a common diagnostic view.
**Development:** `System.Error` provides a message, an optional cause and an optional
existing stack trace. FileReadError, NumberParseError and StreamError are the first
platform adopters. Classes, structs and nominal unions can opt in; an ad-hoc union
does not automatically acquire this interface.

A boundary returning `Result<T, Error>` can propagate any participating
error. This gives callers a less specific signature, while preserving the concrete
runtime value for type checks and diagnostics. With `System.*` imported, the
interface can be written as `Error`: the Result case is named `Failure`, so importing
`System.Result.*` introduces no collision:

```raven
{{ERROR_INTERFACE_BOUNDARY_SAMPLE}}
```

`ContextError<E>` is an optional decorator for a failure that needs an operation
message. WithContext wraps only the failure channel and preserves the typed cause:

```raven
{{ERROR_CONTEXT_PROPAGATION_SAMPLE}}
```

Here `context.Cause` remains a FileReadError; through Error, Cause is an
`Optional<Error>`. A lazy message factory runs only on failure. `FormatError()` renders
the cause chain and at most one existing trace when the application reports the error.
These conventions add diagnostics familiar from exceptions through composition;
they do not change the rules for unions, recovery or propagation.

## Capture stacks deliberately

Capture is **off by default**. Pass `captureStackTrace: true` to WithContext or the
ContextError constructor for one boundary. An application can instead enable
`ErrorDiagnostics.ConfigureStackTraceCapture(true)` at startup, optionally through
a method marked `[Conditional("DEBUG")]`. Debug builds do not silently opt every
error in. Explicit per-call settings override the startup policy.

Capture records where context was attached and reuses existing snapshots. It cannot
recover an earlier failure's stack. Current traces contain guest method names and IL
offsets; source file/line lookup from debug symbols is deferred. Reading or formatting
a stored trace never captures another stack.

[Error and context API, including conditional startup](../../docs/error-context.html) ·
[StackTrace API](../../docs/stack-trace.html) ·
[Outcome operators and propagation](../outcomes/)
