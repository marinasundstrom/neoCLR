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

## Return an outcome from Main

**Development:** a Raven program can return Result directly from its entry point,
including after asynchronous work. Main is static and nongeneric, with either no
parameters or one `string[]` parameter. Exactly one matching Main is required.

| Main return type | Successful process exit |
| --- | --- |
| `()` or `Task<()>` | `0` after completion. |
| `int` or `Task<int>` | The returned integer. |
| `Result<(), E>` or `Task<Result<(), E>>` | `Ok(())` becomes `0`. |
| `Result<int, E>` or `Task<Result<int, E>>` | `Ok(code)` supplies the exit code. |

For each Result form, `Failure(error)` writes the error's normal `ToString()` display
and a newline to **standard error**, then exits with code `1`. A null error writes
an empty line. Normal console output remains on standard output. The operating
system's usual exit-code range/truncation rules still apply.

This complete, tested entry point needs no manual conversion to an exit code:

```raven
module NeoClr.RuntimeTests.EntryPoints

import System.*
import System.Result.*

func Main(args: string[]) -> Result<(), string> {
    if args.Length > 0 { return Failure("sync unit failure") }
    return Ok(())
}
```

The [integrated configuration sample](https://github.com/marinasundstrom/neoCLR/blob/main/samples/extensions/src/ExtensionsDemo/Examples/Extensions/Main.rvn)
uses `async func Main(args: string[]) -> Task<Result<(), ServiceError>>`. It uses `?`
when composing DI services, then awaits configuration observations in `--watch` mode.
The native entry adapter drives the existing task queue and host operations before
observing completion; application code should use `await`. A function returning a
Task directly follows the same startup rules. neoCLR spells the unit task `Task<()>`
(or `Task<unit>`); there is no separate nongeneric .NET Task class.

`E` follows Raven's ordinary error-conversion rules. A user-defined implicit
conversion can translate a lower-level error into the outward error type, and an
error implementing `System.Error` can propagate to `Result<T, Error>`. Neither an
interface nor a converter is required when the existing error type already matches.
These conversions preserve normal `?` semantics; they do not change the exit-code
mapping. Implement `ToString()` for a useful entry-boundary display, or explicitly
format structured diagnostics before returning a string error. Implementing Error
alone does not automatically print its cause chain or capture a stack trace.

Structural error unions work too: `Result<(), FirstError | SecondError>` (and
its Task form) preserves either error alternative. The process adapter prints the
active error's normal display. The same rule applies to `Result<int, string | int>`;
see the [tested union entries](https://github.com/marinasundstrom/neoCLR/tree/main/runtime/System.Runtime/test/ResultEntryPointTests/NeoClr/RuntimeTests/EntryPoints).
This does not imply general-purpose boxing of a whole structural union as Object.

Result failures are recoverable application outcomes. Cancellation, a task that
remains incomplete after registered work drains, and terminal runtime faults still
follow their existing fault behavior. The entry adapter does not add exception
catching or drive application-owned private queues.

[Executable entry-point fixtures](https://github.com/marinasundstrom/neoCLR/tree/main/runtime/System.Runtime/test/ResultEntryPointTests) ·
[Native entry-point contract and validation](https://github.com/marinasundstrom/neoCLR/blob/main/docs/native-result-entrypoints.md)

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
