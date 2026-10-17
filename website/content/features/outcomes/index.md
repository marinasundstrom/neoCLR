# Option, Result and propagation

Option represents absence; Result represents recoverable failure. Raven patterns extract case values, and ? propagates an outcome to the caller.

<a id="handling"></a>

## Error and optional-result handling in Raven

A Result or Option does not require a match at every call. Use `?` when the enclosing function should propagate an error. Use a pattern binding for a value that may be absent. A match is useful when the individual cases need different processing or when the application decides how to report an error.

**Development Console example:** `Console.ReadLine()` returns `Result<Option<string>, TextReadError>`. The question mark propagates a read failure. The Some pattern binds the input, while the else branch handles EOF and returns. An empty line is still Some containing an empty string.

```raven
{{CONSOLE_PROPAGATION_SAMPLE}}
```

The current compiler spells this linear guard `let Some(input) = ... else`; its return makes input available to the statements that follow. The enclosing Result return type makes error propagation explicit. The complete sample handles the final error at the entry point.

### Use if let for a success branch

```raven
{{CONSOLE_IF_LET_SAMPLE}}
```

Here input is scoped to the success block. Both forms combine propagation with pattern binding, and both are tested with successful input, EOF, an empty line and invalid UTF-8. These techniques apply to other APIs with compatible Result and Option contracts. [Console API and stream contracts](../../docs/console.html) · [Console feature](../console/) · [Runnable samples](../../samples/console-streams.zip)

<a id="example"></a>

## Option, Result and propagation in Raven
```raven
{{RAVEN_SAMPLE}}
```

Normalize(-42) produces Ok(42). The minimum Int32 value cannot be made positive, so the sample propagates OverflowError. It prints Continued and 42 for the first call, then Overflow propagated for the second.

[Complete executable sample →](../../samples/library-propagation.rvn) · [VS Code setup →](../../try/#development)

<a id="operators"></a>

## Composing outcomes
Use matching references and runtime libraries for these operators.

Map transforms a successful value. Then chains an operation that already returns an outcome. OrElse supplies a fallback only when needed. Patterns remain useful when you want to handle each case directly.

```raven
{{OUTCOME_OPERATORS_SAMPLE}}
```

This example prints 42, 43, 7, Read: Unavailable, 9 and 42 on separate lines. The explicit Then callback signature works around a current compiler inference limitation; the other callbacks use inferred types. The recovery callback only runs for Error. ToIterable explicitly converts a successful value to a one-element collection; None and Error become empty collections.

[Complete executable sample →](../../samples/library-outcome-operators.rvn) · [Expected output →](../../samples/library-outcome-operators.expected.txt) · [Preview setup →](../../try/#development)

<a id="error-payloads"></a>

## Errors are ordinary values
Use a string for a simple message or a dedicated error type when callers need to distinguish cases. Result.Error is the union case; it does not require an error base class.

With imported Result cases and an expected type, write `let failure: Result<int, string> = Error("Unavailable")`, as in the tested example above.

<a id="adhoc-errors"></a>

## Preserve several error types

**Development:** an ad-hoc error union makes the possible error types visible in
the function signature. It avoids a shared base class, interface, or implicit
converters written only to forward an existing error. This tested configuration
loader preserves either the file-read error or the numeric parse error:

```raven
{{ADHOC_ERROR_UNION_SAMPLE}}
```

`?` returns immediately on either failure. After matching `Error(let error)`, use
`error is FileReadError` or `error is NumberParseError` to distinguish the listed
alternatives and inspect their cases. An unrelated error cannot propagate into
this union. The final `Ok(config)` constructs the successful Result.

This is an alternative to propagating one particular error type or converting
errors into a nominal union. Dedicated error types and nominal unions remain
preferable for many public APIs: they give errors a domain name and meaningful
cases, and keep the contract independent of lower-level implementation choices.
Use a converter when it performs that intentional domain mapping.

HTTP JSON client helpers return `HttpError | JsonError`, composing transport and
serialization errors directly. Match the typed alternatives after extracting Error;
HttpError and JsonError retain their nominal cases. See [HTTP JSON APIs](../../docs/json.html).

Native structural unions support value and reference alternatives. Supported
native layouts retain references and store scalar payloads inline without a nominal
carrier. Exact alternative identity matters: equal enum bits do not make different
enum types interchangeable. Further structural work is parked; open-generic
compositions and wider backend coverage remain incomplete. Consult the detailed
contract for supported combinations.

[Union API and limits](../../docs/unions.html) ·
[Tested configuration loader and validation](https://github.com/marinasundstrom/neoCLR/tree/main/docs/experiments/adhoc-error-unions)

<a id="operator-list"></a>

## The current operator set
| Operation | Option | Result |
| --- | --- | --- |
| Transform a value | Map | Map |
| Chain an outcome | Then | Then |
| Keep a matching value | Filter | — |
| Transform an error | — | MapError |
| Recover with an outcome | OrElse (None) | OrElse (Error) |
| Extract with a fallback | UnwrapOr, UnwrapOrElse | UnwrapOr, UnwrapOrElse |
| Handle both branches | Match | Match |
| Run a branch action | Tap, TapNone | Tap, TapError |
| Convert to zero/one elements | ToIterable | ToIterable |
| Convert absence to an error | OkOr (value or factory), MapResult, ThenResult | — |
| Remove one nested layer | Flatten | — |

Callbacks run immediately on the matching branch. Tap methods return the original outcome. UnwrapOr takes an already evaluated fallback; UnwrapOrElse calls a parameterless factory only for None or Error. Use Result.OrElse or Match if the error itself is needed. ToIterable allocates a small collection and discards an Error payload, so use patterns when the error needs handling.

Raven.Core users will recognize these operators: neoCLR calls Where **Filter** and ToEnumerable **ToIterable**. Throwing unwrap helpers, generic-default helpers, nullable adapters, JSON integration and context-error support are outside this slice. Collection queries have their own [.NET operator mapping](../collections/#dotnet-mapping).

<a id="limits"></a>

## Behavior and limits

Option and Result are standard Raven unions. Development references use `UnionValue` (formerly `IUnion`); `Value` returns the boxed active case, and `HasValue` reports whether a case is active. Explicit `None` is active. A default carrier is inactive and should not be used to represent absence. Current development `TryGetValue` clears its output to the payload default when the case does not match, matching Raven and native emission; older bridge snapshots preserved it. Prefer patterns and propagation in application code.
Unlike exception-based APIs in .NET, recoverable outcomes are in the return type. This makes handling explicit and changes caller code. Runtime faults still exist; Result does not convert every failure into a recoverable case.

[Detailed contract and comparisons →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/raven-union-api.md)

<a id="direction"></a>

## Planned work and open questions

Keep patterns and propagation consistent across the library. DNS, sockets and HTTP combine Task with Result, with cooperative cancellation and explicit cleanup. Broader async disposal remains future work.

Terminal failures carry [runtime-assigned fault codes](../../docs/faults.html). Explicit guest faults use UserFault; guest code cannot choose a code. These remain separate from recoverable Result errors and Task cancellation.

[Related proposals and open questions →](../../proposals/#async)

<a id="feedback"></a>

## Questions and contributions

Questions, examples and documentation corrections are welcome. See [how to contribute](../../#feedback).

Report issues with a small program, the toolchain version, expected behavior and observed output. API proposals should identify the missing operation or contract.

[Discuss on GitHub ↗](https://github.com/marinasundstrom/neoCLR/issues)

## Printing the current call stack

**Development:** `System.Diagnostics.StackTrace()` captures the active guest call
stack at construction. `ToString()` prints readable method names and instruction
indexes, and retains that snapshot after the original calls return or GC runs.
It supports interpreter, JIT and native AOT execution; current qualification is on
macOS ARM64. This is useful for seeing how execution reached a point in your code.

Capture is bounded to 64 frames and 64 KiB with explicit truncation. Source lines,
structured frame access and suspended async caller history remain incomplete.
Faults remain terminal; capturing a stack does not introduce recovery.
[StackTrace API and example](../../docs/stack-trace.html).
