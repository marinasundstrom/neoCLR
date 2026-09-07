# Runtime error contract review

Review of the six public Result-returning methods in the current System library.
Parse, Divide and Abs now implement the typed contracts below; the remaining rows
are proposed migration targets and still use System.Error. A caller must not have to parse diagnostic text to
decide what to do next.

| Method | Current recoverable outcomes | Proposed error type and cases | Result appropriate? |
| --- | --- | --- | --- |
| System.Int32.Parse(String) | InvalidFormat and Overflow, distinguished structurally | System.Int32ParseError: InvalidFormat, Overflow (implemented) | Yes: parsing caller-supplied text is expected to fail |
| System.Int32.Divide(Int32, Int32) | DivisionByZero, Overflow for minimum Int32 divided by -1 | System.IntegerDivisionError: DivisionByZero, Overflow (implemented) | Yes for this checked helper; the low-level div instruction retains its Fault contract |
| System.Math.Abs(Int32) | Overflow for minimum Int32 | System.OverflowError, an ordinary single-purpose type (implemented) | Yes: preserves the selected checked Abs contract; no union is needed for a single failure kind |
| System.String.SliceUtf8(Int32, Int32) | ArgumentOutOfRange, InvalidUtf8Boundary | System.Text.Utf8SliceError: OutOfRange, InvalidBoundary | Yes: callers can validate externally supplied byte ranges without Faults |
| System.Console.ReadByte() | ConsoleUnavailable, ConsoleReadFailed | System.IO.ConsoleReadError: Unavailable, ReadFailed | Yes: absence of a configured host and failure of a read are distinguishable; EOF remains successful Option.None |
| System.IO.File.ReadAllText(String, Int32) | ArgumentOutOfRange, InvalidPath, FileNotFound, AccessDenied, NotRegularFile, FileReadFailed, FileTooLarge, InvalidUtf8 | System.IO.FileReadError: InvalidLimit, InvalidPath, NotFound, AccessDenied, NotRegularFile, ReadFailed, TooLarge, InvalidUtf8 | Yes: bounded file input can fail for expected environmental, limit and content reasons |

Names are provisional preview choices. Keep the set of cases tied to observable behavior;
there is no need to reproduce an exception hierarchy. Error types do not inherit from
System.Error. Multi-case errors use ordinary non-generic carriers with directly nested
case types; the generic Result carrier uses its existing companion/case convention.
A future language can project those types as unions without new union opcodes.

## Payload and classification rules

Start with cases that let callers branch correctly. Add data when it helps an actual
caller: for example, the byte limit for TooLarge, or the requested start/length for
OutOfRange. The caller already has the original path/text; retaining a full copy is not
required merely to identify the failure. A Message or ToString member is presentation,
not a discriminant. Generic System.Error can remain available for application-defined
messages; these library operations should expose their specific error contracts.

Parse must distinguish malformed text from range overflow at the native parsing
boundary. Empty text and an invalid sign/digit sequence count as InvalidFormat;
valid decimal text outside the supported range counts as Overflow. Tests must define
precedence for mixed-invalid input rather than accidentally promising a host parser's
incidental behavior. The decimal grammar and whitespace policy need not change.

The file host currently maps unclassified I/O failures to FileReadFailed. Keep an explicit
ReadFailed fallback so OS differences do not require new cases for every platform error.
Do not claim that every operating system classifies directories or invalid paths the same
way. An optional native error code can follow when a real use requires it; it should not
be the cross-platform discriminant.

Console EOF is neither an Error nor a Fault. A zero byte is a present byte. A read failure
must not be translated into EOF. Host unavailability remains a recoverable error under
the current embedding contract, allowing applications to choose an alternative input.

Allocation failure, invalid execution contracts, stale pointers and invalid case extraction
remain Faults. Do not wrap Faults into Result merely to make signatures uniform. Conversely,
a failed host operation is not inherently unrecoverable: WriteLine currently makes output
failure a Fault as an explicit preview policy, not a general rule for future I/O APIs.
Output methods continue returning Void in this review; changing that contract is separate.

## Migration order and acceptance

1. Completed: Int32ParseError and migration of Parse, its host boundary, applications, and tests.
   This proves a typed error union and removes the most visible collapsed classification.
2. Completed: OverflowError and IntegerDivisionError for Abs and Divide. Both methods
   construct ordinary Result cases entirely in IL. Divide no longer uses bootstrap Result.
3. Implement Utf8SliceError and migrate the slicing native/library boundary off bootstrap
   Result. Keep byte-range validation distinct from UTF-8 boundary validation.
4. Implement ConsoleReadError and FileReadError. Preserve current EOF, missing-host,
   bounded input, UTF-8, and platform-dependent I/O behavior.

Use the canonical method names, replacing signatures and updating all callers in the same
slice. Do not introduce parallel Typed methods. Native bindings should report explicit
structured outcomes; library IL constructs the public cases. Do not implement the bridge
by comparing human-readable error messages. Each slice must define the native outcome
representation, update service discovery and include success plus every reachable failure
case in its tests. No new host intrinsic is required solely to recognize an error union.

Verify samples from source and assembled artifacts, case identity/extraction, error
formatting, and the separation from Fault traces. Rebuild application and System artifacts
together when signatures change. The source-preview API remains intentionally unstable.
