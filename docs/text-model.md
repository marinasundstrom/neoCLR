# Text model: proposed UTF-8 direction

The proposed direction is valid UTF-8 as String's default representation, with
explicit UTF-16 conversion for migration and native interop. This proposal is not
a settled String ABI, indexing contract, or implementation milestone. The current
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

String.Length, indexing, slicing, search offsets, and migration adapters need explicit
contracts before implementation. Familiar .NET signatures must not quietly switch
from UTF-16 offsets to byte offsets. Explicit byte-length and scalar-enumeration APIs
are candidates; a UTF-16 compatibility view is another option. None is selected yet.

A valid-text constructor should validate bytes and expose decoding failure as Result.
Raw or invalid data should remain bytes; lossy replacement should be explicitly
requested. UTF-16 conversion also needs a policy for unpaired surrogates, which Char
can represent but Unicode scalar values cannot. Native APIs need explicit encoding,
length, and termination contracts. UTF-8 is not a reason to assume every foreign
function accepts UTF-8 or requires null termination.

Normalization, comparison/culture behavior, grapheme segmentation, and the exact
Rune API remain future decisions. Choosing an encoding alone does not settle them.
