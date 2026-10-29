# Console and standard streams

System.Console reads standard input and writes standard output and error. Use
Write or WriteLine for output, PeekChar/ReadChar, counted Read or ReadLine for UTF-8 text input, and the
standard stream accessors when code needs a SyncInputStream, SyncOutputStream, SyncTextReader
or SyncTextWriter.

Console is a static class. Development text reads operate on Unicode graphemes.
Development terminal keys are available through Console.OpenKeyInput and ConsoleKeyReader. Colors, cursor movement and nonblocking guest console host I/O remain unimplemented. Development Console text reads and Write are synchronous because the current
standard-console provider makes blocking host calls. No completed Task disguises
that behavior. Async stream wrappers remain the default outside this explicit
synchronous escape hatch.
Legacy Console.WriteLine diagnostics and ReadByte retain their immediate host paths.

| Member | Type or result | Purpose |
| --- | --- | --- |
| `Console.In` | `SyncTextReader` | Shared invocation-owned UTF-8 input cursor |
| `Console.Out` | `SyncTextWriter` | Write UTF-8 standard output |
| `Console.Error` | `SyncTextWriter` | Write UTF-8 standard error |
| `OpenStandardInput()` | `SyncInputStream` | Read raw bytes |
| `OpenStandardOutput()` / `OpenStandardError()` | `SyncOutputStream` | Write raw bytes without text conversion |
| `PeekChar()` / `ReadChar()` | `Result<Optional<Character>, TextReadError>` | Peek/read one grapheme; None at EOF |
| `Read(count)` / `Read(count, maxUtf8Bytes)` | `Result<string, TextReadError>` | Requested grapheme count, shorter at EOF |
| `ReadLine()` | `Result<Optional<string>, TextReadError>` | Read a line with a 65536-byte content bound |
| `ReadLine(maxUtf8Bytes)` | `Result<Optional<string>, TextReadError>` | Read a line with an explicit content bound |

Console.In returns the same reader throughout one invocation. All Console text
methods share its decoder, buffered lookahead and closed state. Closing it makes
later Console text reads return Closed; it never closes process stdin. Out/Error
and the raw stream factories still create fresh non-owning wrappers. Do not mix
raw input access with buffered text reads: raw access bypasses retained lookahead.
Output wrappers have independent closed state; Close never closes process stdout/stderr. No
`SetIn`, `SetOut` or `SetError` redirection API is provided yet; consumers can accept
TextReader/TextWriter or InputStream/OutputStream directly for custom destinations.


## Grapheme peek and counted reads (development)

TextReader and StreamReader also expose `PeekChar()`, `ReadChar()`, `Read(count)`
and `Read(count, maxUtf8Bytes)`, matching the Console convenience methods.
Peek does not advance the **logical text cursor**, but may read and buffer underlying
bytes. Repeated peeks reuse the cached Character. The next read sees the same original
UTF-8 spelling, including combining marks and emoji sequences.

Counted Read waits for the requested number of complete graphemes, or returns a
shorter remainder at EOF. Zero count returns empty without input. Counts and result
byte bounds range from 0 through 65536; default result bound is 65536 bytes. Character
reads also cap a grapheme at 65536 bytes. EOF is None for character reads and empty
text for counted reads. A result-size error retains buffered text for retry; decoding
and I/O failures do not roll back the underlying stream. Invalid bounds consume nothing.

A grapheme ending in an ordinary letter can still acquire a later combining mark.
The reader retains its final cluster until a subsequent boundary, EOF or a definite
LF boundary establishes completion. This can require lookahead and can wait on live
input. It does not split an incomplete UTF-8 sequence or substitute an idle timeout
for Unicode segmentation. Key events use the separate synchronous ConsoleKeyReader API.

Async StreamReader operations reject overlap with TextReadError.Busy; await them
before starting another. Console uses SyncStreamReader directly and can block.
Reentrant reads are rejected as Busy. Genuine async terminal readiness and
cancellation remain separate work. Result, String and decoder allocations remain;
this is not an allocation-free console API.

.NET's [Peek](https://learn.microsoft.com/en-us/dotnet/api/system.io.textreader.peek?view=net-10.0)
is synchronous and uses an integer sentinel. neoCLR uses Optional with its grapheme Character model; StreamReader offers async peek
and SyncStreamReader/Console offer direct peek. Counted reads follow the fill-or-EOF behavior
of [.NET ReadBlockAsync](https://learn.microsoft.com/en-us/dotnet/api/system.io.textreader.readblockasync?view=net-10.0),
returning UTF-8 String rather than filling a UTF-16 character buffer. This preserves
neoCLR text semantics, at the cost of owned result storage and potentially more
lookahead. Character-memory output overloads remain unimplemented.

The [executable mixed-read consumer](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/console-text-reading/Main.rvn)
covers fragmented UTF-8, repeated peek, mixed reads, byte bounds, EOF and pending input.

## Line input

`ReadLine` returns `Some("")` for an empty line and `None` for EOF before any data.
An unterminated final line is returned once. ReadLine removes the exact configured NewLine string, which defaults to LF.
With that default, CR remains data, including immediately before LF. Configure
"\r\n" to consume CRLF as one delimiter. UTF-8 decoding is strict and preserves
a BOM. A malformed line returns InvalidUtf8. ReadLine itself requests no bytes
past a complete delimiter; it first consumes any lookahead retained by character/count reads.
Repeated Console.In access now preserves that same buffered cursor.

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

`TextWriter.Write(text)` and `WriteLine(text)` return `Task<Result<int, StreamError>>`:
the byte count on success, including NewLine for WriteLine. StreamWriter writes UTF-8
without a BOM, handles short writes and reports IoFailure if a stream makes zero
progress. Each call accepts at most 65536 encoded bytes, including NewLine. A failed
write may already have produced output; there is no rollback or atomic-write promise.
Raw OutputStream.Write preserves arbitrary bytes and completes successfully only for the full range.

Convenience Console.Write and Console.WriteLine flush stdout before returning and
turn failures into terminal Faults. Use Console.Out or Console.Error for recoverable
errors. Existing WriteLine(string/int) keeps its prior host line-output path and
bounds; new Write uses the bounded text writer. Calls through different wrappers
are not serialized into whole messages.

Both TextWriter and StreamWriter also expose:

```raven
func Flush() -> Task<Result<unit, StreamError>>
func Close()
```

RavenDoc includes TextWriter.Flush and StreamWriter.Flush in generated reference.
This guide complements the OutputStream/FileOutputStream Flush entries
in the [stream guide](streams.md). Flush returns unit or a typed error, including
Closed after Close. It does not close the destination. StreamWriter is unbuffered;
Close releases owned output without implicitly flushing. Call Flush explicitly to
observe errors. `StreamWriter.Create(output, true)` leaves its output open; the default factory
creates an owning wrapper. StreamReader has the matching input ownership options.

## Propagation and optional input

A combined Result/Optional does not require nested matches. In a function returning a
compatible Result, `?` propagates a read error. Pattern binding then handles the
optional value:

```raven
func ReadInput() -> Result<unit, TextReadError> {
    let result = Console.ReadLine()
    return DescribeInput(result)
}

func DescribeInput(result1: Result<Optional<string>, TextReadError>) -> Result<unit, TextReadError> {
    let Some(input) = result1? else {
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
Their default implementations return Unsupported. A host must implement these
hooks to support Console.Write or stream-based output; line-only hosts can still
support WriteLine.
The current isolated-worker output host and debugger do not implement the byte
output hooks; these calls return IoFailure there. Worker stream forwarding is not supported.

Without a supplied host, Execution.stdout and Execution.stderr retain exact bytes.
Execution.output retains the line records from WriteLine and joined workers;
it is not a complete rendering of the byte channels. With live host I/O, captured
channels are empty. Neither channel is implicitly redirected to the other.

## Comparison with .NET

.NET also exposes Console.In as TextReader and Console.Out/Error as TextWriter,
with separate OpenStandard methods for byte streams. Its ReadLine recognizes CR,
LF and CRLF and uses null for EOF. This development API instead uses Result/Optional,
strict bounded UTF-8, a shared input reader and fresh non-owning output wrappers; it does not yet offer .NET's
redirection, encoding selection or synchronization facilities. The bounds make
resource use explicit but require callers to handle limit failures. Fresh output wrappers
avoid shared closed-state management but allocate on each access. Shared input
retains lookahead and avoids recreating the reader; closing it affects later text reads. These are
provisional tradeoffs, not claims of superiority. Sources reviewed 2026-09-23:
[Console.In](https://learn.microsoft.com/en-us/dotnet/api/system.console.in?view=net-10.0),
[Console.ReadLine](https://learn.microsoft.com/en-us/dotnet/api/system.console.readline?view=net-10.0).

Node exposes separate standard streams too, but output blocking behavior depends on
the destination/platform. neoCLR currently chooses explicitly synchronous host calls;
the Task contract describes possible future completion without claiming that this provider is nonblocking.
[Node process I/O](https://nodejs.org/api/process.html#a-note-on-process-io).

<a id="object-and-scalar-output-development"></a>

## Object and scalar output

WriteLine(object?) calls virtual ToString and writes an empty line for null.
Specific overloads accept Boolean, Character, SByte, Byte, Int16, UInt16, Int32, UInt32,
Int64, UInt64, IntPtr and UIntPtr without boxing. Integers use invariant decimal
text; Boolean writes True/False; Character writes its neoCLR grapheme text. Existing
string and Int32 overloads remain preferred over object.

The object overload preserves the type's current ToString contract. Floating-point
formatting is still deferred: Single/Double do not yet provide numeric Object
formatting. No culture, format-string or IFormatProvider support is introduced.


## Terminal input host foundation (development)

The Rust host API offers a scoped POSIX input transport, now used by the public key reader below. Windows and async scheduler integration remain pending. The grapheme text reads above use the existing stream provider. The host types are documented here manually because
RavenDoc renders class-library metadata, not the Rust embedding crate.

| Rust API | Contract |
| --- | --- |
| `Console::capabilities() -> ConsoleCapabilities` | Non-mutating discovery; custom hosts default to no capabilities. |
| `ConsoleCapabilities` | Boolean `input_is_terminal`, `output_is_terminal`, `error_is_terminal`, `terminal_input`; no encoding or protocol inference. |
| `Console::open_terminal(TerminalOptions) -> io::Result<Box<dyn TerminalInput>>` | Exclusive scoped input; unsupported by default and on redirected Stdio input. |
| `TerminalOptions { capture_interrupts: bool }` | Defaults to false; POSIX true captures all ISIG characters, including Ctrl+C/quit/suspend. |
| `TerminalInput::try_read(&mut self, &mut [u8]) -> io::Result<TerminalRead>` | Nonblocking bytes into caller storage; `Pending`, `Data(usize)` or `End`. Empty buffers fail. |
| `TerminalInput::read(&mut self, &mut [u8], Optional<Duration>) -> io::Result<TerminalRead>` | Synchronous wait: None waits indefinitely, duration returns Pending on timeout, zero tries immediately. POSIX uses OS readiness; other hosts default to Unsupported. Same buffer/closed errors as try_read; unrepresentable deadlines are InvalidInput. |
| `TerminalInput::close(&mut self) -> io::Result<()>` | Restores modes and releases input; retryable on restoration failure, idempotent after success. Drop makes a best-effort attempt. |

`StdioConsole` shares ownership across instances, rejecting concurrent terminal or
legacy byte readers with `WouldBlock`. Raw bytes retain their native encoding.
No String or Promise is created per read. Mode restoration requires orderly cleanup;
process signals/forced exit do not run Drop, and interrupt cancellation remains open.
The [host implementation and validation](https://github.com/marinasundstrom/neoCLR/blob/main/docs/console-io.md#development-terminal-host-transport--2026-10-20)
cover errors, platform limits and a runnable byte diagnostic. Managed signatures and
reference snapshots are unchanged by this host-only foundation.

## Separate text capabilities and factories (development)

| Wrapper | Required stream | Text interface | Operation result |
| --- | --- | --- | --- |
| StreamReader | InputStream | TextReader | Task of Result |
| StreamWriter | OutputStream | TextWriter | Task of Result |
| SyncStreamReader | SyncInputStream | SyncTextReader | Result |
| SyncStreamWriter | SyncOutputStream | SyncTextWriter | Result |

All four expose static Create(stream), Create(stream, leaveOpen),
Create(stream, encoding), and Create(stream, encoding, leaveOpen). The result is
Result of the concrete wrapper and TextReadError (readers) or StreamError (writers).
Constructors are private. UTF-8 and leaveOpen=false are defaults. Creation performs
no reads, writes or asynchronous work; valid providers currently initialize
successfully. Codec and transfer errors are reported by subsequent operations.
A failed creation does not transfer stream ownership.

Both reader interfaces expose PeekChar, ReadChar, Read(count),
Read(count, maxUtf8Bytes), ReadLine(maxUtf8Bytes), ReadToEnd(maxUtf8Bytes), and Close.
Both writer interfaces expose Write, WriteLine, Flush, Finish, and Close. Sync
versions return the same Result payloads directly. Flush does not finalize the
encoder; Finish does. Close does neither. Writers reuse their byte buffer.
Use one buffering wrapper per stream direction: two wrappers do not share lookahead.

There is no SupportedModes property, Task-blocking adapter, or mode-selected
constructor. The capability supplied to Create determines the contract statically.
This differs deliberately from .NET's unified sync/async reader/writer classes:
more types make unsupported execution modes unrepresentable at the wrapper boundary.

Migration: replace constructor calls with Create and handle the Result. Remove
await from Console reads, Write and calls through Console.In/Out/Error. Raw console
reads/writes take borrowed Slice/ReadOnlySlice windows, not array/offset/count.
Async console host integration is not implemented by this change.

## Configurable line delimiters (development)

All four text interfaces and all four stream text wrappers expose
`var NewLine: string { get; set; }`, defaulting to `"\n"`. No enum or constants
class is required. Set the property on a saved Console.In/Out/Error reference to
configure that wrapper; output property accesses create separate wrappers.

Writers append the exact string in WriteLine. An empty string appends nothing;
Write never rewrites newlines already present in text. The existing 65536-byte
UTF-8 input bound includes the appended delimiter, and the selected encoder may
reject characters it cannot represent. Configuration does not perform output.

Readers recognize an exact UTF-8 sequence and exclude it from the returned line.
Matching is ordinal, even though ordinary String equality is canonically aware.
The delimiter can contain multiple Unicode scalars or graphemes and can cross
any byte-read boundary. An empty delimiter reports InvalidRange; more than 65536
UTF-8 bytes reports LimitExceeded before input is consumed. An unterminated final
line includes an incomplete delimiter prefix as data. EOF without data is None;
a complete delimiter at the start returns Some(""). Result-byte bounds exclude
the complete delimiter. A limit or decoding failure does not rewind input.

Operations snapshot the configuration when they begin. Changes affect the next
operation; no mutable borrowed string view crosses suspension. Readers cache an
ordinal delimiter matcher and reuse it until the setting changes. This avoids
quadratic rescanning for overlapping prefixes; it does not eliminate line/String
allocations.

Migration: the former implicit CR removal before LF is gone. Configure `"\r\n"`
when reading CRLF, or use `"\n"` and preserve CR as content. This exact contract
is shared by synchronous and asynchronous wrappers. Console.ReadLine uses the
configuration of the shared Console.In reader. Unicode grapheme Read/PeekChar
and ReadToEnd are unaffected.

.NET exposes a string [TextWriter.NewLine](https://learn.microsoft.com/en-us/dotnet/api/system.io.textwriter.newline?view=net-10.0).
neoCLR extends that configuration to readers as requested by the author. Exact
reader delimiters make framing predictable, at the cost of configuring mixed or
platform-specific input explicitly. There is no automatic newline-detection mode.

### Key decoder and invocation ownership (development)

The internal Raven decoder operates on borrowed UTF-8 and a bounded legacy sequence
profile. The [decoder evidence](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/console-key-decoder/README.md)
is complemented by the public ReadKey tests below. Rust invocation ownership handles
cancellation and cleanup, including bounded synchronous waits. Matching native entry
hosts own a terminal scope and restore it before diagnostics and heap release on
ordinary success/fault returns. Native worker attachment remains pending.

The [native transport and scope checks](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/terminal-input/README.md)
cover raw bytes, restoration, nested contexts and serial JIT host reuse. POSIX
legacy byte reads are unbuffered so libc cannot strand terminal input. ReadKey is
synchronous; this does not establish an async/cancellable public key contract.

## Terminal keys (development)

`Console.OpenKeyInput()` preserves signal handling; `Console.OpenKeyInput(true)`
explicitly captures signal characters, including Ctrl+C. Both return
`Result<ConsoleKeyReader, ConsoleKeyError>`. Use a disposal scope; reads do not echo.
Open before starting text input. Redirected input returns Unavailable. POSIX native
support currently requires the invocation entry thread; Windows support is pending.
Serialize all console input and session operations.

| Member | Contract |
| --- | --- |
| `ConsoleKeyReader.ReadKey()` | `Result<Optional<ConsoleKeyInfo>, ConsoleKeyError>`; blocks, None at EOF |
| `ConsoleKeyReader.Close()` | `Result<unit, ConsoleKeyError>`; restores modes, repeatable; retry after failure |
| `ConsoleKeyReader.Dispose()` | Attempts Close; invocation cleanup retries restoration on failure |
| `ConsoleKeyInfo(Key, Shift, Alt, Control, TextScalar)` | Value constructor; properties retain these arguments |
| `ConsoleKeyInfo.Equals(ConsoleKeyInfo)` / `==` / `!=` | Memberwise value comparison, including the TextScalar case and scalar payload |
| `ConsoleKeyInfo.Equals(Object?)` | Equal boxed key values compare equal; null and other types return false |
| `ConsoleKeyInfo.GetHashCode()` | Equal key values have equal hashes; hashes are not persistent identifiers |
| `Key: ConsoleKey` | Logical letters/digits, Space, Tab, Enter, Backspace, Escape, arrows, Home/End, Insert/Delete, PageUp/PageDown, F1–F12; None for other text |
| `Shift`, `Alt`, `Control: bool` | Reported modifiers, with legacy terminal ambiguities |
| `TextScalar: Optional<int>` | Valid Unicode scalar for decoder-produced text; None for control/navigation; never a grapheme |

`ConsoleKeyError` has Unavailable (unsupported/redirected host), Busy (input already
owned), Closed, ReadFailed, UnsupportedSequence and InvalidUtf8 cases. Malformed
bytes or unsupported sequences are consumed; an unrelated following byte is retained.
Close hands remaining bytes back to the next key reader or Console text/byte input.
After any ordinary input byte read, key acquisition returns Busy for that invocation:
text-to-key handoff remains unsupported to protect buffered decoder/lookahead state.
This also means a raw ReadByte cannot precede a new key session.

The reader owns one reusable byte buffer. ReadKey has no per-key strings, arrays or
Tasks. A Unicode scalar may be only part of a grapheme. Uppercase text alone does not
imply Shift. Legacy terminals cannot distinguish Tab/Ctrl+I or Enter/Ctrl+M;
Escape-prefixed text is interpreted as Alt. Escape syntax has a 30 ms inter-byte
timeout; incomplete UTF-8 waits for more bytes or EOF. Complete sequences are bounded
at 32 bytes. This is a small keyboard foundation, without mouse/paste/key-release
protocol negotiation. Ordinary faults trigger invocation cleanup; forced process
termination cannot guarantee terminal restoration.

[Public API PTY tests and allocation measurements](https://github.com/marinasundstrom/neoCLR/blob/main/docs/experiments/console-read-key/README.md)
are retained in the repository. Native generated type/member pages cover the four
new types; the aggregate reference also includes their signatures.

### Explicit Ctrl+C exit

Capture mode lets the application decide what Ctrl+C means. This example ignores
ordinary keys and exits only on Ctrl+C; its disposal scope restores terminal settings.
It can instead continue, or ask for confirmation, at the `ConsoleKey.C` branch.

```raven
import System.*
import System.Result.*
import System.Optional.*

func Main() {
    let Ok(keys) = Console.OpenKeyInput(true) else {
        Panic("Terminal key input is unavailable")
    }
    use input = keys
    Console.WriteLine("Press Ctrl+C to exit.")
    while true {
        let Ok(Some(key)) = keys.ReadKey() else {
            Panic("Unable to read a key")
        }
        if key.Control {
            match key.Key {
                ConsoleKey.C => return
                _ => { }
            }
        }
    }
}
```

This handles keyboard input while the key session owns the terminal. It does not
install a general process-signal or close-request handler. A broader shutdown-request
API, including cancellation where supported, remains future work.


Development qualification (2026-10-20): key equality and hashing now include the
optional scalar without boxing it. `None`, `Some(0)` and an uninitialized Optional
remain distinct. Typed key operations have a separate allocation diagnostic;
explicitly boxing a key still allocates. See the
[record-union validation](https://github.com/marinasundstrom/neoclr/tree/main/docs/experiments/record-unions).


<a id="key-value-equality"></a>

### ConsoleKeyInfo value equality (development)

The compiler automatically implements `EquatableTo<ConsoleKeyInfo>`. Interface
conversion and `Equals` use the same generated value semantics, including when
ConsoleKeyInfo is imported as a component of another supported record.


| Member | Contract |
| --- | --- |
| `static func ==(left: ConsoleKeyInfo, right: ConsoleKeyInfo) -> bool` | True when Key, Shift, Alt, Control and the TextScalar case/payload are equal. |
| `static func !=(left: ConsoleKeyInfo, right: ConsoleKeyInfo) -> bool` | Negation of equality. |
| `func Equals(other: ConsoleKeyInfo) -> bool` | Same memberwise comparison as `==`. |
| `override func Equals(obj: Object?) -> bool` | True for an equal boxed ConsoleKeyInfo; false for null or another type. |
| `override func GetHashCode() -> int` | Equal keys have equal hashes. Hashes are not persistent identifiers. |

Typed operations compare the optional scalar without boxing it. Explicit boxing
still follows the value-box allocation contract. No normalization or validation of
manually constructed scalar values is performed by equality. Default Optional storage,
None and Some(0) remain different values. These operations introduce no typed error
result. See the validation linked above for execution modes and allocation scope.

The pinned renderer currently gives the two operators the same generated URL. This
manual member reference preserves both contracts while that renderer gap is open.
