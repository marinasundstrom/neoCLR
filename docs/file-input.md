# Bounded UTF-8 file input

`System.IO.File.ReadAllText(String path, Int32 maxBytes) -> System.Result<String,Error>`
provides the first file integration. Its ordinary platform-IL method calls the declared
`neoCLR.Runtime.ReadAllText` InternalCall. Runtime-service analysis reports `FileInput`;
and `ValueStorage`; this report describes dependencies, not an access-control mechanism.
The host returns an explicitly erased String or Error; platform IL constructs ordinary
System.Result.Ok<String> or System.Result.Error<Error> cases. ReadAllTextTyped is removed.

The method reads a regular file synchronously under the host process's filesystem
permissions. Relative paths resolve against the process working directory. Symlinks
are followed. Paths use the platform's normal String-to-path conversion; non-UTF-8
native path names cannot be expressed by this API. No sandbox or configurable host
provider is implemented in this slice.

`maxBytes` is a nonnegative limit on input bytes. The reader checks actual bytes read,
including a one-byte overflow probe, rather than trusting the initial file size. It
uses an 8 KiB scratch buffer and grows retained input only within the limit. This is
not a total process-memory limit: execution and value copying have their usual costs.
An empty file succeeds with a zero limit. Partial contents are not returned on failure.
The file handle is released on success, Error, or Fault. Concurrent writers can change
what is observed; this operation does not promise an atomic filesystem snapshot.

UTF-8 decoding is strict. BOMs, embedded NULs, whitespace, and line endings are preserved;
there is no encoding detection, trimming, or newline conversion. The initial Int32.Parse
helper expects the entire text to be an integer, so the numeric fixture has no trailing
newline.

| Error message | Meaning |
| --- | --- |
| ArgumentOutOfRange | Negative maxBytes; checked before the path |
| InvalidPath | Empty/NUL-containing path or host InvalidInput error |
| FileNotFound | Host reports a missing path |
| AccessDenied | Host reports permission denied |
| NotRegularFile | Opened handle metadata identifies a non-regular file |
| FileReadFailed | Other open, metadata, or read failure |
| FileTooLarge | Read encounters a byte beyond maxBytes |
| InvalidUtf8 | Complete bounded input is not valid UTF-8 |

These are bootstrap message values, not a finalized structured I/O error taxonomy.
Host error classification can differ by OS, especially for directories. Allocation
reservation failure produces a terminal Fault. Byte overflow is reported before text
decoding. Interrupted reads retry.

The operation is blocking. Instruction limits and cooperative cancellation are checked
by the interpreter around instructions; they cannot interrupt an in-progress open or
read. Checking regular-file metadata after open does not prevent a special path from
blocking during open. Use ordinary files for this prototype. Timeouts, cancellable I/O,
and embedding-controlled providers remain future work.

## Run the demonstration

From the repository root:

```sh
cargo run --locked -- verify examples/file_input.neoil
cargo run --locked -- run examples/file_input.neoil
```

Expected output:

```text
42
InvalidInt32
File input handled
=> Void
```

The sample reads `examples/data/number.txt`, doubles its parsed value, then handles
invalid content from `examples/data/not_a_number.txt`. Its `ReadNumber` and `Report`
functions also accept other paths through the hosting API. Missing-file results follow
the same reporting path. Arithmetic uses ordinary wrapping `mul` semantics.

The assembled form uses the same commands documented in the README:

```sh
cargo run --locked -- assemble examples/file_input.neoil file_input.neo.json
cargo run --locked -- verify file_input.neo.json
cargo run --locked -- run file_input.neo.json
```

The data files remain external and paths remain relative to the working directory,
not the assembled artifact. No streams, sockets, async machinery, or new IL instructions
are required for this example.
