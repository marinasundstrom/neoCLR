# Raven error-value APIs

The target now exposes the existing public constructors, predicates, checked case
accessors and ToString methods for:

- FileReadError, FileWriteError and ConsoleReadError
- Utf8SliceError, Int32ParseError and IntegerDivisionError
- InvalidRangeError, InvalidDateError, InvalidTimeError and OverflowError
- EnvironmentError's ToString (it has no declared constructor)

Nested error cases retain their ordinary value types and parameterless constructors.
For example:

```swift
import System.*
let missing = IO.FileReadError(IO.FileReadError.NotFound())
let sameCase = missing.GetNotFound()
System.Console.WriteLine(missing.ToString())
```

`GetNotFound` faults when called on a different case; use IsNotFound before checked
extraction when the case is uncertain. Empty error/case values have valid defaults.
An error carrier with case storage must be initialized with a
real value before use. The importer does not invent a default case or message.

These are value errors, not an Exception hierarchy. They preserve the
[existing error contracts](runtime-error-contracts.md) and Result-based API policy.
They do not introduce exception-driven recovery. Text descriptions remain library
presentation, not stable discriminants: file-read and file-write labels currently
have differences which the example deliberately preserves. The projection does not
claim general nested error-union pattern lowering; predicates and checked accessors
are the admitted runtime contract here.

## Trying and checking

The [error API sample](experiments/raven-target/samples/library-errors.rvn) constructs
all 23 nested cases, round-trips their checked accessors and prints descriptions,
then exercises simple errors and ordinary string messages. Its
[expected output](experiments/raven-target/samples/library-errors.expected.txt) is
checked by the saved-project suite. Use a fresh
[prepared target project](experiments/raven-target/README.md).

```sh
python3 docs/experiments/raven-target/verify_error_values.py /tmp/PROBE/editor/Demo.rvnproj \
  --raven /path/to/Raven --runtime /path/to/neoclr
```

This checks a wrong-case runtime fault and rejection of an uninitialized carrier
before executable output is produced. `verify_editor.py --errors` checks nested cases,
and accessors. The shared catalog supplies declarations and checked
bindings. The development library removes the legacy System.Error wrapper and
requires matching rebuilt references; installed Preview 8 bundles are unchanged.
