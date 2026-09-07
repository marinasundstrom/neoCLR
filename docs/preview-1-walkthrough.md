# Preview 1 runnable walkthrough

These programs demonstrate the current source preview: familiar System APIs and
CLI-style calls, arithmetic and branches, alongside real Void, ordinary Result/Option
carriers, explicit native allocation and terminal Faults. They do not require a
high-level compiler or extensive object-oriented programming.

This walkthrough is executable today. It is not a declaration that Preview 1 is
released: [exact-release platform and toolchain validation](preview-1.md) remain open.
The Raven-like pseudocode below explains intent; the linked neoIL files are the
actual programs. No iterator framework, Stream hierarchy or implicit ownership
facility is required by these samples.

## Build once

From the repository root, with the [Rust and native build prerequisites](../README.md#build-and-run-a-sample):

```sh
cargo build --locked
cargo test --locked --test preview_walkthrough
```

The test runs the public CLI to assemble, check, verify and execute each program
both as source and as a JSON artifact. It checks normal output, handled input errors,
EOF, and failing process exits with Fault traces. Additional cases check empty and
single-element array traversal, allocation release and file failures.

The same tests run in the existing CI test step; platform support still requires
successful CI evidence for the exact release commit. The tests use temporary output
paths and the file integration requires no network or external accounts.

## Program menu

Run these commands from the repository root:

| Program | Command | Expected stdout, before the final `=> Void` |
| --- | --- | --- |
| Free function and console | `cargo run --locked -- run examples/hello_functions.neoil` | `Hello, world!` |
| Interactive calculation | `cargo run --locked -- run examples/console_input.neoil` | Prompt, then `42` when you enter `21` |
| Ordinary values and alternatives | `cargo run --locked -- run examples/ordinary_unions.neoil` | `success`, `7`, `failure`, `7`, `Some<Void> is present` |
| Growable list | `cargo run --locked -- run examples/array_list.neoil` | `ArrayList count:`, `5`, `0`, `1`, `4`, `9`, `16` |
| Array loops | `cargo run --locked -- run examples/array_loops.neoil` | `Sum of squares:`, `30` |
| File summary | `cargo run --locked -- run examples/file_summary.neoil` | `File contents:`, `Hello, neoCLR 🌍!`, `UTF-8 bytes:`, `19` |
| Read-only type inspection | `cargo run --locked -- run examples/type_inspection.neoil` | `System.Int32`, `Box`, `1`, `System.Int32`, `Same type` |
| Pointer-backed carrier | `cargo run --locked -- run examples/pointer_union.neoil` | `42`, `7`, `11` |

Comma-separated outputs in the table are separate lines. These programs finish with
process exit status 0, including console input errors that the program handles.
Cargo's build/progress messages are separate from the guest program output.

To exercise the artifact path yourself, choose a new output filename:

```sh
cargo run --locked -- assemble examples/array_loops.neoil array-loops.neo.json
cargo run --locked -- check array-loops.neo.json
cargo run --locked -- verify array-loops.neo.json
cargo run --locked -- run array-loops.neo.json
```

The assembler refuses to overwrite an existing artifact. Reuse the artifact or choose
a new name when repeating these commands. The bundled System library is provided
by the executable; [explicit library builds](../README.md#build-the-runtime-library-explicitly-optional)
are also supported. JSON artifacts are prototype metadata and IL, not .NET assemblies.

## Free functions and inhabited Void

[HelloFunctions](../examples/hello_functions.neoil) implements:

```text
func Message() -> String { return "Hello, world!" }
func Main() -> Void { Console.WriteLine(Message()) }
```

A function does not require a class container. `call Message()` pushes its String
result. WriteLine consumes it and returns the one Void value, which Main returns.
Intermediate calls returning Void use `pop` when their result is discarded.

## Console calculation and expected errors

[ConsoleInput](../examples/console_input.neoil) reads an unsigned decimal number
using Console.ReadByte. Its application helper returns Result<Option<Int32>,Error>:
None represents immediate EOF; an error represents invalid/empty input or a read
failure. Native console failures have the specific ConsoleReadError type, which
this application explicitly converts to a displayable application Error.

| Input | Output after the prompt |
| --- | --- |
| `21` followed by Enter | `42` |
| EOF before any digits | `End of input` |
| Enter with no digits | `Empty input` |
| `x` | `Expected ASCII digits` |
| Ten digits | `At most nine digits` |

The prompt is `Enter an unsigned number (up to nine digits):`. Enter/LF or EOF
terminates a number; CR bytes are ignored to accept redirected CRLF input. Input
is intentionally ASCII-only, unsigned and bounded to nine digits so doubling fits
Int32. This is application IL, not a general ReadLine or parsing service.

To send EOF interactively, use the terminal's EOF convention (commonly Ctrl-D on
Unix, or Ctrl-Z followed by Enter on Windows). The automated test supplies exact
bytes and closes stdin, so it does not depend on terminal behavior.

Case testing and extraction use ordinary member calls. The public runtime API
returns Result and Option with concrete error types; it does not throw exceptions.
The [ordinary union sample](../examples/ordinary_unions.neoil) separately exercises
constructors, distinct cases, Void payloads and independent owned value copies.

## Counted loops and foreach-style traversal

[ArrayLoops](../examples/array_loops.neoil) implements this intent:

```text
func Sum(values: Array<Int32>) -> Int32 {
    var total = 0
    for value in values {
        total = checked(total + value)
    }
    return total
}

func Main() -> Void {
    let values = Array<Int32>.Allocate(5, 0)
    for (var i = 0; i < values.Length; i = i + 1) {
        values[i] = checked(i * i)
    }
    Console.WriteLine("Sum of squares:")
    Console.WriteLine(Sum(values))
    values.Free()
}
```

The first loop initializes `[0, 1, 4, 9, 16]`. Sum borrows the descriptor and returns
30. Both loops lower to an indexed local, a Length getter and conditional branches.
Indexer reads/writes call get_Item/set_Item; checked arithmetic uses add.ovf/mul.ovf.
There is no foreach opcode or enumerator allocation. An overflowing checked sum is
a Fault in this sample, rather than an application-defined recoverable result.

Passing the descriptor copies its pointer and length, so Sum shares the same storage.
It must not free that borrowed storage. Main frees exactly once after the call.
The host checks that the sample's tested traversal variants leave no live allocations.
This explicit lifetime is visible even if a future language supplies scope-based cleanup.

## A bounded file integration

[FileSummary](../examples/file_summary.neoil) calls
System.IO.File.ReadAllText(path, 4096), which returns
Result<String,System.IO.FileReadError>. On success it prints the text and its UTF-8
byte count. On NotFound it prints `File not found`; other cases use the error's
ToString method. Recoverable file errors finish normally with Void.

The fixture [summary.txt](../examples/data/summary.txt) contains exactly
`Hello, neoCLR 🌍!` without a trailing newline. Its 19-byte size is not a character
count. Paths are resolved from the process working directory, so run from the
repository root. The program reads only; it does not modify the fixture.

Report(String path) is an ordinary free function. To explore other paths in the CLI,
change Main's ldstr operand and assemble/run that source. The automated host tests
invoke Report directly with missing, empty, oversized and invalid-UTF-8 files. The
latter two print `FileTooLarge` and `InvalidUtf8`. This shows typed file errors without
introducing a filesystem object model or Stream abstraction.

## Explicit references and manual lifetime

[PointerUnion](../examples/pointer_union.neoil) contrasts a copied Int32 snapshot
(42) with a borrowed carrier that observes mutation through an alias (7), then reads
an error from stack storage (11). TryGetOk copies into caller-provided storage,
TryGetOkPointer supplies a borrowed pointer, and TryGetError handles the other case.
The carrier uses a tag plus Void*. Its caller owns
the payload and releases heap storage exactly once. See the
[pointer contract](pointer-carriers.md) for casts, lifetimes and native-layout limits.

This borrowed carrier has different copy semantics from the current ordinary System
carriers, whose temporary System.Value storage owns a host value tree. System.Value
is [scheduled for retirement](value-storage.md#retirement-decision), not a permanent
arbitrary-value abstraction. The pointer sample does not yet supply a native storage
replacement for every String or nested carrier payload.

## Faults terminate execution

```sh
cargo run --locked -- run examples/fault_trace.neoil
cargo run --locked -- run examples/array_bounds.neoil
```

Both commands intentionally exit with status 1 and print a Fault to stderr, with no
successful `=> Void` result. FaultTrace reports `Demonstration fault` and guest frames
in the order Fail, Work, Main. ArrayBounds tries index 2 of a two-element buffer and
reports the failing System.Array operation and Main. Each frame includes a logical
IL instruction location; source-line mapping is not implemented.

The sample test asserts the diagnostic contract rather than freezing metadata row
numbers that may change as the preview evolves. Faults cannot be caught in guest IL.
An embedding host receives a Fault for that execution; its application need not exit.
No guest cleanup handler runs, although execution teardown reclaims its tracked native
buffers. Native code and host allocation failures are outside any universal containment
claim. See [Fault traces](stack-traces.md) and [memory semantics](heap-and-pointers.md).

## Read-only type inspection

[TypeInspection](../examples/type_inspection.neoil) gets a type token, describes a
Box<Int32> value through System.TypeOf<T>.Of, reads its name and generic argument,
and compares it with Box<int>. Identity comes from loaded metadata, not names alone.
The helper describes declared T; it introduces no dynamic object dispatch or payload
erasure. See [the descriptor contract](type-inspection.md) for scope and lifetime.

## A growable collection without interfaces

[ArrayList<T>](array-list.md) lives in System.Collections. Allocate, Add, Count, Capacity,
an indexer and Free are implemented in platform IL. Copies share an explicit state
pointer, so growth through one alias is visible to the others. Free once after all
borrowers finish. The current element subset requires native layout; String and current
System.Result payloads are not yet supported. A future List<T> interface is separate.
