# Raven String helpers on neoCLR

Implemented 2026-09-13. The Raven target projects these existing runtime APIs:

| Call | Result |
| --- | --- |
| `String.Concat(left, right)` | String |
| `String.CompareOrdinal(left, right)` | Int32 comparison sign |
| `text.Equals(other)` | Boolean |
| `text.ContainsOrdinal(value)` | Boolean |
| `text.StartsWithOrdinal(value)` | Boolean |
| `text.EndsWithOrdinal(value)` | Boolean |
| `text.GetUtf8ByteCount()` | Int32 byte count |
| `text.IsEmpty()` | Boolean |
| `text.SliceUtf8(byteStart, byteLength)` | Result<String, Utf8SliceError> |

The [String sample](experiments/raven-target/samples/library-strings.rvn) demonstrates
the first eight calls. `Hello, värld!` occupies 14 UTF-8 bytes. The
[boundary sample](experiments/raven-target/samples/library-string-boundaries.rvn)
checks empty patterns, embedded NUL, distinct normalized spellings, supplementary
characters and UTF-16 ordinal ordering. Use the sign of CompareOrdinal's result,
not its magnitude; the samples use Math.Sign.

The [slicing sample](experiments/raven-target/samples/library-string-slices.rvn)
uses `?` and typed Result matches to handle successful byte-range copies, OutOfRange
and InvalidBoundary. Empty ranges are accepted only at code-point boundaries,
including the end of the string. Range validation precedes boundary validation.

All nine currently declared String methods are now projected. The error carrier
exposes IsOutOfRange and IsInvalidBoundary; [case constructors, checked accessors
and ToString](raven-error-api.md) are also projected. Broader inherited/interface API coverage
is still tracked separately. Inherited metadata such as Object.ToString can appear in
completion but is not admitted by this catalog. Unqualified Contains, Substring,
Join, Format and IsNullOrEmpty are not imported from the host .NET library.

## Semantics and implementation layers

This slice reuses [the existing ordinal contract and .NET comparison](ordinal-text.md)
and [the String model](text-model.md). Text is immutable, valid Unicode and currently
non-null. Searches are case-sensitive without normalization or culture processing;
empty patterns match. CompareOrdinal retains UTF-16 code-unit ordering, while
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
