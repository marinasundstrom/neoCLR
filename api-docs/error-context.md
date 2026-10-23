# Error context (development)

`System.Error` is an opt-in contract for recoverable failure diagnostics. Platform
failure types follow this convention incrementally; applications may implement it,
but `Result<T,E>`, `Option<T>` and `?` do not require it. This development slice
adopts it for `FileReadError`, `NumberParseError` and `StreamError`. Other existing
platform failure types are not yet migrated.

ContextError is an error decorator: it adds operation context, causal chaining and
optional stack diagnostics familiar from .NET exceptions, without exception
inheritance or stack unwinding. An API author still decides which domain type,
decorated type, interface or ad-hoc union to expose. Classes, structs and unions
can all represent errors; the interface does not require a union representation.

## Error

| Property | Contract |
| --- | --- |
| `Message: string` | Human-readable description of this layer; use typed patterns for classification. |
| `Cause: Option<Error>` | `Some` of the underlying failure, or `None` for a leaf. |
| `StackTrace: Option<System.Diagnostics.StackTrace>` | Existing snapshot or `None`; reading it never captures. |

Implement all three getter contracts. Native Raven currently requires explicit
absent implementations returning `None`; default interface bodies remain a compiler
emission gap. The optional values are ordinary Option cases, not inactive defaults.
Messages and diagnostic chains may contain application data; choose what to report
at the application boundary.

## ContextError<E: Error>

An ordinary reference class with two constructors:

- `(message: string, cause: E)` uses the application capture policy, initially off.
- `(message: string, cause: E, captureStackTrace: bool)` explicitly overrides that policy.

`Message` is the operation description; `Cause: E` retains the original typed value.
Through `Error`, Cause is `Some(cause)`. An immediate cause's existing trace is reused.
When capture is requested, a bounded search first reuses a deeper existing trace;
otherwise construction captures the current guest stack. Instances have reference
identity, not record value equality. Use `Message` or `FormatError` for display.

Capture records the attachment site, not a reconstructed original-failure site.
Current StackTrace limits include 64 frames, method/IL locations and no suspended
async callers or source lines. Snapshot formatting never recaptures. Source file/line
lookup from debug symbols is explicitly deferred; Debug and Release currently have
the same trace-detail contract.

## ErrorDiagnostics

`CaptureStackTraces: bool` reports the application policy, initially false.
`ConfigureStackTraceCapture(enabled: bool)` changes it for subsequent ordinary
context construction. Configure at startup before concurrent work begins. Existing
snapshots and explicit per-call overrides are unaffected. Leaf errors never capture.

An application can opt into the familiar Debug convention:

```raven
[System.Diagnostics.Conditional("DEBUG")]
func EnableDevelopmentErrorDiagnostics() {
    System.ErrorDiagnostics.ConfigureStackTraceCapture(true)
}

EnableDevelopmentErrorDiagnostics()
```

The caller must define `DEBUG`; a Release caller without it omits the configuration
call. This works with precompiled libraries: it is application configuration, not a
conditional embedded in the library's constructors. An application may choose another
symbol or call the configuration method directly. Both configurations start with
capture off unless the application opts in. Context still allocates a wrapper, and
value errors may box when viewed through Error; disabled capture avoids stack walks
and snapshot allocation. Formatting remains explicit and walks the cause chain.

## System.Diagnostics.ConditionalAttribute

`ConditionalAttribute(conditionString: string)` exposes `ConditionString: string`.
It may be applied repeatedly to methods or attribute classes. Any defined condition
retains the method call or attribute application; otherwise the compiler omits it.
Conditional methods return unit, cannot have out parameters or implement interface
members, and cannot become delegates. Omitted calls still undergo type checking,
but neither receivers nor arguments execute. The method remains in metadata.
The shared compiler contract is documented in Raven's `docs/compiler/conditional-methods.md`.

Debug symbols and optimization are separate from capture policy. In .NET, available
[PDB mappings](https://learn.microsoft.com/en-us/dotnet/core/diagnostics/symbols)
provide source locations; Release can also distribute symbols. Optimizations can
change the available frames. This change does not implement native PDB lookup.

## Extensions

`ErrorExtensions<E: Error>.WithContext(message)` and
`WithContext(message, captureStackTrace)` return `ContextError<E>`.

`ResultErrorContextExtensions<T,E: Error>` provides the same overloads for
`Result<T,E>`, returning `Result<T,ContextError<E>>`. Two further overloads accept
`message: () -> string`, with or without the capture Boolean. The factory runs once
on failure and never on success. Success values pass through unchanged. Failures
in user-supplied message factories remain ordinary terminal faults; this API does
not introduce exception handling.

`ErrorFormattingExtensions.FormatError()` formats the outer message followed by
`Caused by:` lines and at most one existing stack trace. Traversal stops after 64
layers and prints a truncation marker if more remain, bounding cyclic custom chains.
This bounds traversal, not message/trace text size. Output is diagnostic text, not a
stable serialization format. The formatter does not capture a stack.

## Propagation and compatibility

A contextual wrapper changes the residual type. A function returning
`Result<T,ContextError<FileReadError>>` can propagate a contextual file failure;
a function returning `Result<T,FileReadError>` cannot. Interface conversion does not
wrap, `?` does not attach information, and `MapError` remains explicit domain mapping.
`Result<T,System.Error>` is an explicit common boundary: a concrete Error can
propagate through its ordinary interface conversion, retaining its runtime value
while widening the static signature. Explicit context wrappers can propagate the
same way. This does not automatically wrap or capture.
Structural union propagation does not establish that a union satisfies the Error
generic constraint. Use per-alternative typed wrappers or explicitly project to
Error at a reporting boundary until that separate contract is qualified.

Use the [executable contract suite](https://github.com/marinasundstrom/neoCLR/blob/main/runtime/raven/tests/error-context/Tests.rvn)
for examples of typed causes, lazy construction, propagation and trace reuse.
