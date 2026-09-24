# Console and standard streams

Development API after Preview 9. `System.Console` is a static class. Its common
methods are `Write(string/int)`, `WriteLine()`, `WriteLine(string/int)`, `ReadByte()`
and `ReadLine()`/`ReadLine(maxUtf8Bytes)`. Terminal key handling, colors, cursor
movement, character-at-a-time text input and asynchronous calls are not implemented.

| Member | Type or result | Purpose |
| --- | --- | --- |
| `Console.In` | `TextReader` | Read UTF-8 standard input |
| `Console.Out` | `TextWriter` | Write UTF-8 standard output |
| `Console.Error` | `TextWriter` | Write UTF-8 standard error |
| `OpenStandardInput()` | `InputStream` | Read raw bytes |
| `OpenStandardOutput()` / `OpenStandardError()` | `OutputStream` | Write raw bytes without text conversion |
| `ReadLine()` | `Result<Option<string>, TextReadError>` | Read a line with a 65536-byte content bound |
| `ReadLine(maxUtf8Bytes)` | `Result<Option<string>, TextReadError>` | Read a line with an explicit content bound |

Each property/factory call creates a fresh wrapper. Wrappers share the underlying
channel and its position, but their closed state is independent. Cache a wrapper
locally for repeated use. `Close` never closes process stdin/stdout/stderr. No
`SetIn`, `SetOut` or `SetError` redirection API is provided yet; consumers can accept
TextReader/TextWriter or InputStream/OutputStream directly for custom destinations.

## Line input

`ReadLine` returns `Some("")` for an empty line and `None` for EOF before any data.
An unterminated final line is returned once. LF and CRLF terminate a line; **a lone
CR is data** in this first implementation. UTF-8 decoding is strict and preserves
a BOM. A malformed line returns InvalidUtf8. No input is read past LF, so repeated
Console.In access does not lose buffered input.

StreamReader implements the same bounded `ReadLine` method for any InputStream.
Bounds range from 0 through 65536 bytes, excluding the terminator. Invalid bounds
consume nothing. Reading at the bound may consume a terminator or excess input to
detect an oversized line; errors do not rewind input. Closed readers return Closed.
The current implementation reads one byte at a time and accumulates a whole line;
this is a correctness-oriented first implementation, not a throughput claim.

`ReadByte` retains its distinct Unavailable/ReadFailed errors. Stream-based input
maps host unavailability and read failures to IoFailure. With no supplied host,
embedding cannot read process stdin. Host calls are synchronous and cannot be
interrupted by cancellation while executing.

## Text and byte output

`TextWriter.Write(text)` and `WriteLine(text)` return `Result<int, StreamError>`:
the byte count on success, including LF for WriteLine. StreamWriter writes UTF-8
without a BOM, handles short writes and reports IoFailure if a stream makes zero
progress. Each call accepts at most 65536 encoded bytes, including any LF. A failed
write may already have produced output; there is no rollback or atomic-write promise.
Raw OutputStream.Write preserves arbitrary bytes and may return a short count.

Convenience Console.Write and Console.WriteLine flush stdout before returning and
turn failures into terminal Faults. Use Console.Out or Console.Error for recoverable
errors. Existing WriteLine(string/int) keeps its prior host line-output path and
bounds; new Write uses the bounded text writer. Calls through different wrappers
are not serialized into whole messages.

Both TextWriter and StreamWriter also expose:

```raven
func Flush() -> Result<unit, StreamError>
func Close()
```

RavenDoc includes TextWriter.Flush and StreamWriter.Flush in generated reference.
This guide complements the OutputStream/FileOutputStream Flush entries
in the [stream guide](streams.md). Flush returns unit or a typed error, including
Closed after Close. It does not close the destination. StreamWriter is unbuffered;
Close releases owned output without implicitly flushing. Call Flush explicitly to
observe errors. `StreamWriter(output, true)` leaves its output open; the one-argument
constructor closes it. StreamReader has the matching input ownership options.

## Propagation and optional input

A combined Result/Option does not require nested matches. In a function returning a
compatible Result, `?` propagates a read error. Pattern binding then handles the
optional value:

```raven
func ReadInput() -> Result<unit, TextReadError> {
    let Some(input) = Console.ReadLine()? else {
        Console.WriteLine("No input")
        return Ok(())
    }
    Console.WriteLine("Input is: $input")
    return Ok(())
}
```

The current compiler's linear guard spelling is `let Some(input) = ... else`.
The else branch must exit; afterward input is known to exist. `if let Some(input)`
with a success block is the alternative when input should be scoped to that block.
Both tested forms are included in the source download. EOF takes the else branch;
an empty line is Some(""); an I/O/decoding error propagates without printing “No input”.
The sample entry point handles the final error once. Prefer propagation and bindings
in ordinary flow; use match when it explains the cases or makes a handling policy
explicit. See the [Raven error-handling section](/features/outcomes/#handling).

## Example

The [runnable Console sample](/samples/console-streams.zip) prompts for a name,
reads one bounded UTF-8 line and sends the greeting to stdout and a diagnostic to
stderr. Its contract fixture checks empty input, CRLF, EOF, short writes, byte
ranges and close ownership. See [Console](xref:System.Console),
[TextReader](xref:System.IO.TextReader) and [TextWriter](xref:System.IO.TextWriter)
for member reference pages.

## Embedding

The CLI opts into process standard streams. Rust Console implementations can add
`write_bytes(error, bytes)` and `flush(error)`; `error = true` selects stderr.
Their default implementations return Unsupported, preserving source compatibility
with existing line-only hosts. Those hosts need to implement the new hooks before
using Console.Write or stream-based output. The old WriteLine path is unchanged.
The current isolated-worker output host and debugger do not implement the byte
output hooks; these calls return IoFailure there. No worker stream forwarding is
claimed by this slice.

Without a supplied host, Execution.stdout and Execution.stderr retain exact bytes.
Execution.output retains the legacy line records from WriteLine and joined workers;
it is not a complete rendering of the new byte channels. With live host I/O, captured
channels are empty. Neither channel is implicitly redirected to the other.

## Comparison with .NET

.NET also exposes Console.In as TextReader and Console.Out/Error as TextWriter,
with separate OpenStandard methods for byte streams. Its ReadLine recognizes CR,
LF and CRLF and uses null for EOF. This development API instead uses Result/Option,
strict bounded UTF-8 and fresh non-owning wrappers; it does not yet offer .NET's
redirection, encoding selection or synchronization facilities. The bounds make
resource use explicit but require callers to handle limit failures. Fresh wrappers
avoid shared closed-state management but allocate on each access. These are
provisional tradeoffs, not claims of superiority. Sources reviewed 2026-09-23:
[Console.In](https://learn.microsoft.com/en-us/dotnet/api/system.console.in?view=net-10.0),
[Console.ReadLine](https://learn.microsoft.com/en-us/dotnet/api/system.console.readline?view=net-10.0).

Node exposes separate standard streams too, but output blocking behavior depends on
the destination/platform. neoCLR currently chooses explicitly synchronous host calls;
future scheduling work must not relabel these blocking operations as asynchronous.
[Node process I/O](https://nodejs.org/api/process.html#a-note-on-process-io).
