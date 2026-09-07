# Minimal console I/O

The initial console boundary provides raw byte input and line output. It is a small
embedding contract, not a guest Stream API. Parsing and control flow in the demonstration
are ordinary platform IL; line decoding and a general text reader are deferred.

## Platform methods

`System.Console.ReadByte() -> Result<Option<Byte>,Error>` is a neoCLR bootstrap extension.
Its IL body calls the explicitly declared `neoCLR.Runtime.ConsoleReadByte` InternalCall.
The runtime constructs an owned result with exact Byte storage:

- Ok(Some(byte)): one byte, including zero or values above 127.
- Ok(None): end of input, separate from an empty line or an input error.
- Err(Error("ConsoleUnavailable")): no host console was supplied.
- Err(Error("ConsoleReadFailed")): the host reported an input failure.

Input has no implicit UTF-8 decoding, newline conversion, or -1 EOF sentinel. The sample stores a raw Byte case payload in a Byte local before loading it for
arithmetic. The local load converts it to the Int32 evaluation-stack category.
Existing bootstrap union instruction semantics are unchanged pending their removal.

Existing WriteLine(String) and WriteLine(Int32) retain their Void return type. When a
host console is supplied, output is delivered immediately, before another guest
instruction runs. Host output failure is currently a terminal Fault with a logical
stack trace; the existing signature cannot carry a recoverable Error. An explicit
Result-returning output contract is a future decision. Partial output is possible on
failure and already delivered output survives later Faults or cancellation.

## Hosting and isolation

ExecutionOptions.console accepts an optional Arc<dyn Console>. The Rust Console trait
has two synchronous operations: read_byte and write_line. This boundary introduces no
guest interface dispatch, stream hierarchy, ownership model, or async execution.
Implementers return I/O errors rather than panic, synchronize their own state, and make
successful line writes visible before returning. Host callback panics are not converted
to guest Errors or Faults.

Without a console, existing embedding behavior is preserved: WriteLine collects lines
in Execution.output and input returns ConsoleUnavailable without touching process stdin.
With a console, output goes only to that host and Execution.output stays empty. The
same option applies to entry execution and resolved static/instance invocation.

Cloning options shares the host console and its input position; it does not clone or
reset input. Guest frames and allocations remain fresh for each invocation. Hosts can
supply a new console for independent sessions, and tests can use an in-memory console
without changing process-wide input/output. Concurrent sessions sharing a console may
interleave calls; no whole-session transaction or exclusivity is implied.

The CLI explicitly selects StdioConsole. It reads process stdin as bytes, writes UTF-8
text with LF endings to stdout, and flushes each line before returning. Program output
is no longer delayed until successful completion, and is not printed a second time from
the Execution result. The final `=> value` remains a CLI diagnostic after success.

```rust
let options = neoclr::ExecutionOptions {
    console: Some(std::sync::Arc::new(neoclr::StdioConsole)),
    ..neoclr::ExecutionOptions::default()
};
let execution = program.run(options)?;
```

This is an experimental Rust API addition: explicit ExecutionOptions struct literals
must include console or use a default update. Existing callers passing Limits directly
retain captured output and unavailable input.

Calls are blocking. Cooperative cancellation and instruction limits cannot interrupt
an in-progress host read/write. StdioConsole retries interrupted reads; custom hosts
define their own retry behavior. Runtime-service planning reports ConsoleInput and
ConsoleOutput independently; discovery performs no console I/O and is not authorization.

## Demonstration

From the repository root, build and verify, then run interactively:

```sh
cargo build --locked
cargo run --locked -- verify examples/console_input.neoil
cargo run --locked -- run examples/console_input.neoil
```

Enter `21` followed by Enter. The prompt appears before reading, then the program prints
`42` and returns Void. To test redirected input from a POSIX shell:

```sh
printf '21\n' | ./target/debug/neoclr run examples/console_input.neoil
```

Expected stdout:

```text
Enter an unsigned number (up to nine digits):
42
=> Void
```

ReadNumber in the sample accepts up to nine ASCII digits and doubles the value in Main.
LF terminates input; CR bytes are ignored for CRLF compatibility. EOF after digits
finishes a number, while EOF without digits returns None. An empty line, a nondigit,
or a tenth digit produces an application Error. This is a one-shot demonstration:
on an invalid byte it returns immediately without draining the remaining input.
It is not a general ReadLine or Int32.Parse implementation. Nine digits keep the doubled
result within Int32 range without needing an additional arithmetic error policy.

The sample can be assembled to JSON and run with the same commands as the README's
HelloWorld workflow. Tests cover redirected CLI prompt ordering, byte values, EOF,
input/output failures, host isolation/defaults, shared input position, verification,
and service analysis.

Next text APIs should follow the byte/string primitives needed to implement their logic
in platform code. Stream, socket, buffering-framework, and asynchronous APIs remain
outside this slice.
