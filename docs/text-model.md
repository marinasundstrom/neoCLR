# Text model and initial String API

The proposed direction is valid UTF-8 as String's default representation, with
explicit UTF-16 conversion for migration and native interop. The initial explicitly
UTF-8-named String methods are now implemented. This does not settle the String ABI
or a general indexing contract. The current
interpreter uses Rust String internally; host representation alone does not define
the guest platform contract.

The motivation is to make common UTF-8 interchange boundaries inexpensive, rather
than to assume all other runtimes use the same internal representation. UTF-16
remains relevant to .NET and foreign APIs; compatibility conversions are explicit.

Separate the concepts through types and APIs:

- Byte holds an encoded octet, not necessarily a complete character.
- A future System.Text.Rune should hold a validated Unicode scalar value.
- System.Char retains its familiar UTF-16 code-unit meaning for compatibility.
- Text-element APIs should handle grapheme clusters for user-facing operations.

UTF-8 represents a scalar using one to four bytes. A displayed text element can
contain multiple scalars. Byte counts, scalar counts, and text-element counts are
different operations. See [Unicode's encoding FAQ](https://unicode.org/faq/utf_bom.html)
and [.NET's Unicode model](https://learn.microsoft.com/en-us/dotnet/standard/base-types/character-encoding-introduction).

String.Length, general indexing, search offsets, and migration adapters still need explicit
contracts. SliceUtf8 explicitly uses UTF-8 byte offsets, as documented below. Familiar
.NET signatures must not quietly switch from UTF-16 offsets to byte offsets. Byte
counting is now explicit; scalar enumeration and a UTF-16 compatibility view remain
possible extensions.

A valid-text constructor should validate bytes and expose decoding failure as Result.
Raw or invalid data should remain bytes; lossy replacement should be explicitly
requested. UTF-16 conversion also needs a policy for unpaired surrogates, which Char
can represent but Unicode scalar values cannot. Native APIs need explicit encoding,
length, and termination contracts. UTF-8 is not a reason to assume every foreign
function accepts UTF-8 or requires null termination.

Normalization, comparison/culture behavior, grapheme segmentation, and the exact
Rune API remain future decisions. Choosing an encoding alone does not settle them.

## Implemented String members

System.String is a runtime-represented primitive type with ordinary platform-library
methods. The current methods are compiled by the same assembler as application code:

| Member | Contract |
| --- | --- |
| static Concat(String left, String right) -> String | Produce an owned concatenation, preserving all text including embedded NUL |
| instance Equals(String other) -> Boolean | Exact ordinal equality; no culture, case folding, or normalization |
| instance IsEmpty() -> Boolean | True only for the valid empty String |
| instance GetUtf8ByteCount() -> Int32 | Count encoded UTF-8 bytes, not UTF-16 units, scalars, or graphemes |
| instance SliceUtf8(Int32 byteStart, Int32 byteLength) -> Result<String,Error> | Copy a valid UTF-8 byte range into an owned String |

Concat, byte count, and slicing forward through validated InternalCall declarations.
Equals and IsEmpty execute ordinary IL. No new IL opcode, array representation, or
special member dispatch is required. The runtime-service report classifies the three
helpers as StringOperations.

SliceUtf8 accepts an empty range at any code-point boundary, including the end of the
string. Negative arguments or a range beyond the encoded byte length return
Err(Error("ArgumentOutOfRange")). Both endpoints must be code-point boundaries;
otherwise the result is Err(Error("InvalidUtf8Boundary")), including an empty range
inside a multi-byte encoding. Success is Ok(String), including Ok(""). These checks
do not require grapheme boundaries: a combining mark can be sliced independently.

Empty is not absent, and an embedded NUL is ordinary content, not a terminator.
No invalid-text constructor or native string marshalling is added. String values remain
owned interpreter values, and copies/results do not expose shared writable byte storage.
This is not a permanent allocation ABI or a requirement for implicit GC.

A byte count outside Int32 range, concatenation size overflow, or failure of explicit
helper allocation reservation produces a terminal Fault. Range and encoding-boundary
errors are ordinary Result values. Host input validation and execution-limit Faults keep
their existing contracts and stack diagnostics. String memory has no separate guest
quota yet; catastrophic host allocation failure elsewhere is not guaranteed recoverable.

`cargo run -- run examples/strings.neoil` prints Hello, neoCLR!, reports 10 UTF-8 bytes
for "café 🌍", prints the globe slice, and handles an invalid boundary without terminating.
