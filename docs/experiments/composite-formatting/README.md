# Composite formatting — 2026-10-19

Development APIs: `String.Format`, `Console.Write` and `Console.WriteLine` have
one-, two-, three-argument and `params object?[]` overloads. `System.Formattable`
provides `ToString(format: string) -> string` for type-defined formatting.

The shared Raven parser accepts `{index[,alignment][:format]}`, decimal indexes,
optional alignment (negative means left aligned), and `{{`/`}}` literal braces.
Null formats as empty text. Formattable receives the exact UTF-8 format substring,
or empty for the default. Other objects fall back to parameterless ToString,
ignoring the specifier, as .NET composite formatting does. Braces inside a format
specifier are rejected. Repeated arguments are formatted per occurrence; unused
arguments are not formatted. Formatting completes before Console writes anything.
Write returns Task<unit> and flushes; WriteLine preserves its synchronous contract.

## Comparison and limits

[.NET composite formatting](https://learn.microsoft.com/en-us/dotnet/standard/base-types/composite-formatting)
and [IFormattable](https://learn.microsoft.com/en-us/dotnet/api/system.iformattable?view=net-10.0)
are the API/grammar reference. neoCLR follows its existing interface naming by
omitting the I prefix. Empty non-null format requests the default representation.
The current interface has no format-provider parameter: ambient culture and custom
provider dispatch remain gaps, not an assertion of full .NET compatibility. Existing
explicit DateTimeFormat culture/calendar services remain independent. Integer,
floating-point and date specifier integration needs its own qualification; adding
the interface does not make all existing primitives Formattable.

Alignment counts neoCLR graphemes, not .NET UTF-16 units or terminal columns. This
fits the platform text contract but changes widths for combining sequences/emoji.
The existing StringBuilder limits formatted output to 65536 UTF-8 bytes; literal-only
formats reuse the input and are not subject to that output-buffer limit. Alignment
also has a 65536 limit. Invalid syntax/indexes/limits produce RuntimeError, rather
than .NET FormatException. These are documented implementation gaps/limits.

The parser scans borrowed UTF-8 without temporary byte or Char arrays. Fixed arities
avoid an argument array, but values may box; a params call constructs an array.
Literal fragments, nonempty format specifiers, type formatting results and final
output can allocate strings, and StringBuilder keeps a fragment array. Direct
UTF-8 writer formatting remains future work; no zero-allocation or throughput claim.
String view/storage improvements remain deferred by explicit author direction.

## Validation

The executable checks fixed and params overloads, repeated/reordered indexes,
nulls, class/struct Formattable dispatch, empty/nonempty Unicode format strings,
non-Formattable fallback, grapheme alignment, escaped braces, NUL, custom ToString,
and all console overloads. The separate fault consumer verifies malformed syntax,
index and alignment overflow, nested braces, and no partial console output.

The native consumer exposed missing boxed-value interface views. The repair retains
nominal box conformance during linking, discovers its methods during specialization
and selection, preserves verified dispatch inventory through boxing projection, and
includes boxed record tags in interface tests. Calls borrow the rooted payload.
This is backend work; Raven compiler source and Runtime Contract settings are
unchanged. Primitive box interface mappings and provider-based formatting are not
added by this change. The regression includes both empty and reference-bearing
structs and verifies exact text after a later allocation.

## Measured allocation cost (2026-10-20)

A 128-iteration native managed-allocation diagnostic uses the same three string
arguments and output for fixed versus explicit-array overloads. Whole-process
counts are baseline 515, fixed 3459 and array 3587, all reclaimed. Fixed arities
therefore remove exactly one argument-array allocation per invocation in this
workload; formatting still adds 23 allocations per iteration over baseline.
Eager collection counts are respectively 1540, 25732 and 26244. This is not a
CPU/throughput comparison, and it does not count host allocations. The direct
writer/allocation work remains significant.

Matched artifacts: composite-format-libraries-v3, composite-format-bundle-v6,
composite-format-main-v7, composite-format-invalid-v7 and
composite-format-allocation-v7. Application build inputs use the same v3 libraries
from bundle-v5 with the final explicitly supplied codegen; JIT executes bundle-v6.
Compiler code is 2d22055dc; compiler docs-only revisions do not alter that binary.
The codegen source repair is included in the same commit as this experiment.
Native documentation audit v1 uses these unchanged library sources. Earlier build
attempts remain unqualified; final evidence files supersede their failures.

```
cargo test --release -p neoclr-codegen --lib
cargo test --release -p neoclr-codegen --test console rooted_boxed_records_copy_trace_and_dispatch_payloads
python3 docs/experiments/composite-formatting/check.py target/composite-format-bundle-v6/bundle target/composite-format-main-v7 --invalid target/composite-format-invalid-v7 --output target/composite-format-validation-final.json
python3 docs/experiments/composite-formatting/measure.py target/composite-format-allocation-v7 --output target/composite-format-allocations-final.json
```

Backend unit validation: 109 passed, one existing ignored. The focused native boxed
record copy/tracing/dispatch regression also passes. API snapshots pass their checks;
the website builds 4572 pages with 124 legacy pages retained. Existing unrelated
missing-description warnings remain. No website publication or release is performed.

Final public qualification: all 39 executions pass across interpreter, JIT and AOT,
including the formatted-output limit fault without partial console output.
