# Process APIs from Raven

Raven's source-built target surface exposes all current Console and Environment
methods. It calls the runtime library's existing Result/Option APIs; declaration
bodies do not perform host I/O.

| API | Return |
| --- | --- |
| Console.WriteLine(string/int) | CLI void |
| Console.ReadByte() | Result<Option<byte>, IO.ConsoleReadError> |
| Environment.GetCommandLineArgs() | string[] |
| Environment.GetCurrentDirectory() | Result<string, EnvironmentError> |
| Environment.GetEnvironmentVariable(string) | Result<Option<string>, EnvironmentError> |

ReadByte returns Some(byte) for a byte, None at EOF, or a concrete error. The
[console sample](experiments/raven-target/samples/library-console.rvn) propagates
failures and distinguishes EOF from byte zero. It reads bytes, not decoded text.

The [environment sample](experiments/raven-target/samples/library-environment.rvn)
uses nested unions to distinguish a missing variable from an existing empty value.
Invalid variable names return EnvironmentError. Existing host limits, live reads
and UTF-8 requirements described in [the environment contract](environment.md)
remain unchanged.

Command-line arguments are a fresh managed string array per call. Assignment
aliases that array; a subsequent API call returns an independent snapshot. The
current native service produces an owned vector, so a bridge adapter copies its
populated elements into an ordinary managed array. The CLI supplies the guest path
at index zero followed by arguments after `--`; it does not leak the host's full
argument list. String-array initialization uses [typed null defaults](string-default-storage.md).

This follows CLR array/reference categories while retaining neoCLR's existing
Result/Option error contracts. It adds no new opcode. The snapshot copy is a
provisional adapter cost; a future native service can produce the managed array
directly. Non-null intrinsic String representation remains an implementation detail.

Run the controlled host checks with a rebuilt runtime and a prepared target project:

```sh
python3 docs/experiments/raven-target/verify_process.py /tmp/probe/editor/Demo.rvnproj \
  --raven "$RAVEN_ROOT" --runtime "$NEOCLR_RUNTIME"
python3 docs/experiments/raven-target/verify_editor.py /tmp/probe/editor \
  --collections --files --process
```

The checks supply environment values, exercise invalid/missing/empty names and
current directory, verify argument-array aliasing and independent calls, and feed
NUL/255/EOF to Console.ReadByte. Host-specific console failure injection remains
covered by runtime tests, not simulated by the Raven CLI example. Installed
SDK/VSIX packages require a later refresh to include the updated declarations and
runtime; source builds and installed builds must not be confused.
