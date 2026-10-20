# Console

Read a line, write text, or work with the standard streams. Recoverable input errors and end-of-input are separate outcomes.

Console is a static class in System. Standard streams use UTF-8.

<a id="input"></a>

## Reading input with Result and Option

ReadLine returns Result<Option&lt;string&gt;, TextReadError>> directly. The current host console is synchronous. The question mark propagates a read error to the caller. The Some pattern binds the line; the else branch handles end-of-input and returns. An empty line is a successful value containing an empty string.

```raven
{{CONSOLE_PROPAGATION_SAMPLE}}
```

The enclosing Result<…> return type makes propagation explicit. The complete application handles the final error at its entry point. There is no need to match both unions at every call. See [error and optional-result handling in Raven](../outcomes/#handling) for the equivalent if let form and when a match is useful.

[Download the tested samples](../../samples/console-streams.zip) · [Console API guide](../../docs/console.html)

<a id="streams"></a>

## Text and byte streams

Console.In provides a SyncTextReader; Console.Out and Console.Error provide SyncTextWriter instances. OpenStandardInput, OpenStandardOutput and OpenStandardError expose SyncInputStream or SyncOutputStream for byte operations. These interfaces live in System.IO and can also be used with files.

Write and WriteLine are convenient for ordinary output. Use the text writers when a write or flush failure should be returned as a Result. Console.In preserves one shared cursor; output properties create wrappers. Closing a wrapper does not close the process channel.

Async stream wrappers remain separate: StreamReader and StreamWriter accept their
async byte capabilities; SyncStreamReader and SyncStreamWriter are the explicit
synchronous escape hatch. All use Result-returning Create factories.

[Console members](../../docs/api/System.Console.html) · [TextReader](../../docs/api/System.IO.TextReader.html) · [TextWriter](../../docs/api/System.IO.TextWriter.html)

<a id="limits"></a>

## Behavior and limits

The shape is familiar from .NET Console: text properties and standard byte-stream factories. The contracts differ: input uses typed Result and Option values, text is strict UTF-8, and operations have explicit bounds. .NET ReadLine returns null at EOF; this API returns None.

Console operations return directly and can block; they do not manufacture Tasks. ReadLine recognizes LF and CRLF, preserves a lone CR as data, and accepts at most 65,536 UTF-8 bytes per line. The overload accepts a smaller bound. Writers emit LF without a BOM and allow at most 65,536 encoded bytes per call, including a newline. SyncStreamWriter completes writes directly; callers flush explicitly. Encoding selection, console redirection setters and terminal controls are not implemented.

Embedding hosts opt into the byte-output hooks. Older hosts, including the current worker and debugger adapters, support legacy WriteLine but return an I/O error for the new output streams. See the [API guide](../../docs/console.html) for ownership and host behavior.

<a id="direction"></a>

## Future work

Separate text interfaces describe the provider’s actual execution capabilities. Nonblocking console host integration remains future work. The current line rules and bounds are provisional; examples and failure cases will guide their evolution.

Reports with input bytes, expected output and the toolchain revision help us evaluate the contract. See [how to contribute](../../#feedback).

## API reference

[Console](xref:System.Console) · [TextReader](xref:System.IO.SyncTextReader) · [TextWriter](xref:System.IO.SyncTextWriter)


## Command-line arguments

The development Raven toolchain accepts a no-result entry with a string array:

```raven
import System.*
import System.Time.*

func Main(arguments: string[]) {
    for argument in arguments {
        Console.WriteLine(argument)
    }
}
```

The array contains application arguments, excluding the executable name, as in .NET.
Without arguments it is empty. Environment.GetCommandLineArgs() includes the
executable name. Main() without parameters is also supported.

## Displaying values

WriteLine overloads accept Boolean, Char and integer types without
boxing. WriteLine(object?) is the fallback: it calls the object's ToString override,
or writes an empty line for null. Integer text uses invariant decimal digits.
Floating-point numeric formatting and format providers remain future work.


Development after Preview 16 also supports Object display of boxed Int64, Boolean
and closed value records, including generic record carriers. Value overrides run
against the boxed copy with GC roots retained. Records without an override use the
type-name default consistently in interpreter, JIT and AOT. This closes native backend gaps;
installed Preview 16 binaries retain their earlier limits. Floating-point Object formatting remains unsupported. Development JSON record
serialization is documented separately in the [JSON guide](/docs/json.html).
