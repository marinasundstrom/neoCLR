# Raven String helpers on neoCLR

Implemented from 2026-09-13; grapheme text contract added 2026-09-19.
See [text abstraction](design/text-abstraction.md) for representation, migration and costs.
The Raven target projects these runtime APIs:

| Call | Result |
| --- | --- |
| `String.Concat(left, right)` | String |
| `String.CompareOrdinal(left, right)` | Int32 comparison sign |
| `text.Equals(other)` | Boolean |
| `text.ContainsOrdinal(value)` | Boolean |
| `text.StartsWithOrdinal(value)` | Boolean |
| `text.EndsWithOrdinal(value)` | Boolean |
| `text.GetUtf8ByteCount()` | Int32 byte count |
| `text.IsEmpty` | Boolean |
| `text.Length` | Int32 grapheme count |
| `text.GetIterator()` / `for character in text` | Iterator<Char> / grapheme iteration |
| `text.GetScalars()` | Sequence<UInt32> Unicode scalars |
| `text.SliceUtf8(byteStart, byteLength)` | Result<String, Utf8SliceError> |

The [String sample](experiments/raven-target/samples/library-strings.rvn) demonstrates
the first eight calls. `Hello, värld!` occupies 14 UTF-8 bytes. The
[boundary sample](experiments/raven-target/samples/library-string-boundaries.rvn)
checks empty patterns, embedded NUL, distinct normalized spellings, supplementary
characters and UTF-8/scalar ordinal ordering. Use the sign of CompareOrdinal's result,
not its magnitude; the samples use Math.Sign.

The [slicing sample](experiments/raven-target/samples/library-string-slices.rvn)
uses `?` and typed Result matches to handle successful byte-range copies, OutOfRange
and InvalidBoundary. Empty ranges are accepted only at code-point boundaries,
including the end of the string. Range validation precedes boundary validation.

The declared String members above are projected; integer string indexing is deferred. The error carrier
exposes IsOutOfRange and IsInvalidBoundary; [case constructors, checked accessors
and ToString](raven-error-api.md) are also projected. Broader inherited/interface API coverage
is still tracked separately. Inherited metadata such as Object.ToString can appear in
completion but is not admitted by this catalog. Unqualified Contains, Substring,
Join, Format and IsNullOrEmpty are not imported from the host .NET library.

## Semantics and implementation layers

This slice reuses [the existing ordinal contract and .NET comparison](ordinal-text.md)
and [the String model](text-model.md). Non-null text is immutable and valid Unicode. Managed String slots have a
[typed null default](string-default-storage.md), distinct from empty text; operations
requiring text fault on null. Searches are case-sensitive without normalization or culture processing;
empty patterns match. CompareOrdinal uses UTF-8/scalar ordering, while
GetUtf8ByteCount explicitly measures storage bytes. These are the existing library
choices, not new runtime behavior or a claim of general .NET String compatibility.

One catalog generates declaration metadata and selects executable bindings. It uses
the [shared signature checker](raven-signature-projection.md). Raven's ordinary
instance calls pass a String value; generated guest adapters provide the readonly
managed receiver required by the runtime methods where necessary. They retain no
reference and do not mutate the string. The primitive operations still execute the
neoCLR library, not the metadata stubs or host .NET String implementations.

The benefit is ordinary member access in Raven with the existing runtime contracts.
The remaining cost is an explicit receiver adapter and a bounded catalog that must
track the library. This adds no opcode and does not change Raven's default .NET
compiler behavior. Static callvirt and incompatible signatures are rejected.

## Run and verify

Generate fresh declarations with the current `--interfaces` probe and prepare a
workspace using [the editor instructions](experiments/raven-target/VSCODE.md). Use
`library-strings.rvn` as Main.rvn, then run the neoCLR build/run task. Existing local
workspaces retain their older declaration DLL until explicitly refreshed.

`verify_project.py --collections` now runs both samples alongside the existing
collection, propagation and match programs. It also rejects invalid String argument
types and unavailable host methods. `verify_editor.py --collections --files --strings`
checks instance/static completion against the supplied declarations. The
`--signatures` probe includes malformed String call checks.

Slicing reuses a common Result/error catalog with file operations. Extraction
initializes outputs only on success; ignored/inverted tests and uninitialized error
reads are rejected. Propagating Utf8SliceError into FileReadError is also rejected.
The underlying runtime [text contract](text-model.md) is unchanged.

Run the standalone slicing checks with fresh output:

```sh
dotnet run --project docs/experiments/raven-target/Probe.csproj \
  -p:RavenRoot="$RAVEN_ROOT" -p:BuildProjectReferences=false -p:WarningLevel=0 \
  -- --slices /tmp/neoclr-slices-check
python3 docs/experiments/raven-target/verify_slices.py /tmp/neoclr-slices-check \
  --runtime "$NEOCLR_RUNTIME"
```

The compiler build used here rejects `\0` in string literals. The boundary sample
uses the supported `\u0000` spelling. This is a Raven source-escape limitation,
separate from the runtime's handling of embedded NUL; no compiler change is included.

No SDK/VSIX was rebuilt for this slice. The installed experimental language server
was tested with fresh declarations; the saved-project runner still uses the source
bridge. See [the coverage plan](raven-runtime-api-coverage.md) for remaining APIs and
[the release procedure](experiments/raven-target/RELEASING.md) for packaging.

## Strict UTF-8 conversion (2026-09-19)

`System.Text.Utf8.Encode(string)` returns `Sequence<byte>`.
`Utf8.Decode(Sequence<byte>)` returns `Result<string, InvalidUtf8Error>`.
Use ordinary `match` patterns or `?` to extract or propagate a result; see the
[executable UTF-8 sample](experiments/raven-target/samples/library-utf8.rvn).
Malformed input is a typed error, with no replacement characters. Empty input is
valid. Encoding adds no BOM; decoding preserves a present BOM as U+FEFF. NUL and
normalization forms are preserved. `InvalidUtf8Error` currently carries no offset.

The encoder returns a detached byte snapshot through the read-only Sequence
contract. The decoder snapshots indexed input and produces an independent String;
changing the source array afterwards cannot change the decoded text. This contract
does not promise zero-copy conversion or deep immutability of collection providers.
A null receiver/input, allocation limit or uninitialized byte is a runtime fault,
not an invalid-UTF-8 result. Native services require StringOperations and ManagedArrays.

The Raven implementation owns collection traversal and typed Result construction.
Two narrow native services copy UTF-8 bytes and validate them with Rust's strict
UTF-8 decoder. They introduce no new opcode or compiler keyword. The reference
catalog projects the exact signatures; System.Void/unit runtime configuration is
unchanged. Regenerate the reference core and selected System library together.
`IsEmpty()` has been replaced by the `IsEmpty` property: rebuild callers using
`text.IsEmpty`. No specialized Utf8String, Encoding hierarchy, lossy decoder or
scalar Char implementation is included in this minimal conversion slice. The
scalar redesign was the direction at that stage; the current grapheme contract supersedes it.

### Comparison and provisional choices

.NET's [UTF8Encoding constructor](https://learn.microsoft.com/en-us/dotnet/api/system.text.utf8encoding.-ctor?view=net-10.0)
allows strict decoding via `throwOnInvalidBytes`; its default replaces invalid
input. This preview chooses a strict typed Result so boundary failures must be
handled without exceptions or silent replacement. Unlike .NET byte-array APIs,
Sequence exposes the collection contract while leaving provider choices open.
The costs are snapshot allocations and a less detailed error. Streaming, offset
reporting, UTF-16 interchange and broader encoding policy remain future work.
The underlying validator follows [Rust's UTF-8 validity rules](https://doc.rust-lang.org/std/str/fn.from_utf8.html).
Sources reviewed 2026-09-19. The author subsequently confirmed native UTF-8 as the selected direction.
CompareOrdinal uses UTF-8/scalar order. Char subsequently moved through a scalar
implementation to the grapheme contract below. See [the ordering change](ordinal-text.md#utf-8-direction-confirmed--2026-09-19).

Validation: the UTF-8 sample and three existing String samples pass alongside 23
saved-project edit/rejection checks. Three UTF-8 runtime tests and thirteen existing
String/disposal tests pass. The signature probe, String/Utf8 editor completion,
source ownership (850 declarations, 69 services) and clean bootstrap regeneration
also pass. The public error remains intentionally minimal pending usage feedback.

## Historical scalar Char integration — 2026-09-19

This intermediate implementation is superseded by the grapheme contract below.
The historical neoCLR project props selected `RavenUnicodeScalarChar=true`. Raven keeps
`System.Char` in CLI signatures but uses 32-bit scalar loads, stores and numeric
conversions on this target. Supplementary literals (`'🌍'` and `'\U0001F600'`)
have type `char`; surrogate literals are rejected. The runtime validates Char
storage, so invalid numeric scalar values fault rather than becoming surrogates.
The ordinary Raven .NET target retains its UTF-16 Char contract.

Char classification covers supplementary Unicode values. `IsSurrogate`,
`IsHighSurrogate` and `IsLowSurrogate` are removed: their inputs cannot be scalar
Char values. Existing binaries and reference packs must be rebuilt together.
This is target-specific policy and remains on Raven's `neoclr` branch.

Compared with .NET Char, the benefit is one value per Unicode scalar without
surrogate pairs; the cost is incompatible 32-bit storage and scalar validation.
.NET Rune is the closer semantic comparison. A scalar is not a grapheme, and
this slice does not add grapheme indexing or general encoding abstractions.

Validation: 76 focused Raven lexer/literal/semantic cases; saved-project ScalarChar
and Primitives samples plus edit/rejection checks; 125 scalar/Boolean outcomes
and four invalid Int32-to-Char conversions; runtime classification/integer/library
checks. The LSP exposes the remaining classification methods. Unicode escapes are
already covered by the Raven TextMate grammar.

Numeric casts retain ordinary numeric narrowing to UInt32 before scalar validation;
this is not a checked conversion from arbitrary wide numeric inputs. Scalar String
length/index/iteration remains a separate minimal API slice. No new source debugger
or native FFI Char ABI is introduced.

## Grapheme characters

`Char.FromString` requires exactly one cluster and faults on invalid input. Raven
literals such as `'é'` and `'👨‍👩‍👧‍👦'` have type char. Use `ToString`, `Equals`
and `CompareTo` for text operations, and `System.Text.UnicodeScalar` for explicit
scalar classification. See the [tested sample](experiments/raven-target/samples/library-grapheme-strings.rvn).
