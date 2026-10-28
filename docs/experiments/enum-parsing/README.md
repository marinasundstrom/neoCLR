# Enum parsing foundation

Development increment, 2026-10-20. `Enum.Parse<TEnum>(text, ignoreCase)` returns
`Result<TEnum, NumberParseError>`; `Enum.TryParse<TEnum>(text, ignoreCase, out result)`
returns Boolean and resets the output to zero on failure. Overloads without
`ignoreCase` use case-sensitive ordinal matching. Nullable input is accepted and
null/empty input fails parsing.

## Contract and comparison

The [.NET Enum.TryParse contract](https://learn.microsoft.com/en-us/dotnet/api/system.enum.tryparse?view=net-10.0)
and [runtime implementation](https://source.dot.net/System.Private.CoreLib/src/runtime/src/libraries/System.Private.CoreLib/src/System/Enum.cs.html)
provide the baseline: exact names, aliases, whole signed decimal values (including
unnamed values), surrounding whitespace and comma-separated names. Name lists are
bitwise combined even without Flags. Numeric lists, empty tokens, hexadecimal and
fractional input are rejected. Numeric overflow is distinguished from invalid format
by Parse; TryParse returns false for either. Parse uses neoCLR's existing Result
error contract rather than .NET exceptions, allowing ordinary error propagation.

Current enum storage is Int32. Unicode metadata names are preserved without normalization.
The generic overloads now require `where TEnum: struct`, matching .NET. Reference
types and unconstrained generic forwarding reject at compile time. Enum identity
is validated at runtime: a non-enum value type faults even on null input and does
not return a parsing error. A stronger struct/Enum bound is deliberately not added
to this .NET-shaped API. Other backing widths remain a separate gap.

## Implementation and allocation

Raven owns scanning, numeric parsing and explicit ordinal comparison. Four private
runtime queries expose count/name/bits and ordinary enum boxing. No metadata-name
or value snapshot array is allocated. Native lowering selects retained closed type
handles and uses existing enum construction/boxing; no additional native ABI is added.
Single-name parsing reuses unchanged input. Trimming or comma-list tokens can allocate
strings. Successful parsing uses the ordinary enum box and Result carrier; TryParse
currently shares that path. This is not allocation-free parsing or a throughput claim.

The compiler bundle is `4c051e332-dirty-record-policy-metadata-function-bound`, with
matched runtime/backend binaries and source-built libraries. No Runtime Contract
configuration or Raven compiler source change is required. The private queries are
exact-signature InternalCalls; native metadata is authoritative. The legacy CLI
reference declarations exist only for documentation fixtures, not as runtime bodies.

## Validation

`runtime/System.Runtime/test/EnumParsingTests` uses the built-in TestAttribute discovery,
assertions and runner. Four tests cover names/aliases/case, imported enums, lists,
Int32 endpoints/overflow, unnamed values, empty enums, malformed/null input, output
reset and ordinary display after collection. Run with `scripts/test-runtime-library.py
--suite enum-parsing` using the matched bundle and all three execution modes.

[Retained report](validation-20261020.json): four tests pass in each engine (12
executions) on macOS ARM64. The backend malformed-signature/no-enum-token regression
passes; four API snapshot checks and the 5,036-page website build pass. These are
development checks, not a replacement package or cross-platform release qualification.

## Returned out forwarding qualified (2026-10-21)

Raven main `addfcb11e` fixes definite assignment for an out call in a return
expression; native branch `b83a618a0` carries that fix deliberately. Both branches
pass 44 focused byref semantic/codegen tests, including rejection of conditional
paths that do not call the assigning method. The compiler now analyzes the returned
expression before checking normal-exit assignment.

The rebuilt compiler and all four source class libraries compile the forwarding
TryParse overload without a redundant default assignment. The underlying overload
still resets output on failure. [Built-in qualification](out-forward-validation-20261021.json)
passes all four enum tests in interpreter, JIT and native execution (12 passes).
The compiler bundle now records the committed revision; the earlier report remains
historical evidence. No Runtime Contract configuration, ABI, metadata encoding or
public signature changes. Generic method constraints remain open.

## Unicode metadata names (2026-10-21)

Enum literals now use the ordinary metadata name validation instead of an extra
ASCII identifier restriction. The producer, native reader and runtime preserve
names exactly, reject empty/control-containing names and duplicates, and retain the
reserved storage-name and 1024 UTF-16-unit metadata limit. Raven continues to own
source identifier syntax; metadata acceptance is not a new source lexer rule.

[ECMA-335, Partition I §8.5.1](https://ecma-international.org/publications-and-standards/standards/ecma-335/)
permits Unicode metadata names and distinguishes metadata identity from language/CLS
identifier restrictions. Following that model avoids enum-only naming machinery.
No native metadata encoding, layout or ABI change is needed; existing UTF-8 storage
already represents the names. Ordinal enum parsing does not inherit String’s
canonical-equivalence default. Explicit ignoreCase uses the existing ordinal folding
policy; it does not normalize canonically equivalent spellings.


The broader Raven fixture exposed the same ASCII restriction on constructor
parameter metadata (`värde`). Runtime validation now also accepts ordinary Unicode
parameter/local, type/method generic-parameter and property names. It validates
metadata as descriptive text, not as neoIL tokens; ordinary numeric or space-containing
metadata names are accepted too, while empty/control-containing names, duplicates
and reserved slots remain rejected. Instructions continue to reference slots by
index. The legacy neoIL/frontend textual grammar is unchanged and its helper is
explicitly named `valid_source_slot_name` to prevent reuse for metadata admission.


[Unicode execution report](unicode-validation-20261021.json): six built-in tests
pass in each of interpreter, JIT and native execution (18 passes) on macOS ARM64.
The ordinary identifier fixture includes a generic class, property, method, constructor
parameter, local and generic function using Unicode names. Two enum runtime tests,
25 naming/generic-metadata/property regressions and the metadata CLI/native round-trip
and rejection suite pass. Strict runtime library Clippy, API fingerprint checks and
the 5,036-page website build pass. These are development gates; the replacement
package, platform and performance gates remain open.


## Generic contract qualified (2026-10-21)

Raven `77dbe0302` and neoCLR `db1b06067` now support ordinary method categories and
nongeneric class bounds. All four Parse/TryParse overloads use `where TEnum: struct`,
matching .NET's contract without adding a stronger Enum bound. Existing enum identity
validation remains authoritative for value types. This closes the earlier missing
method-category gap. Source callers forwarding an unconstrained type parameter must
add the struct constraint; reference-type calls now reject before execution.

All four source libraries rebuild from the committed compiler. The [built-in
report](constraint-validation-20261021.json) passes six tests in all three engines
(18 executions), including a generic forwarding helper. [InvalidTypeArguments.rvn](InvalidTypeArguments.rvn)
is an intentionally noncompiling diagnostic fixture: both the String argument and
unconstrained forwarding produce RAV0320 against the rebuilt library. Runtime API
checks use the built-in test framework; a compile-rejection fixture cannot live in
the executable suite. CLI/native API references are refreshed from the matching
source and compiler. The five-library documentation bundle includes the optional
PostgreSQL provider already present in the website inventory. These are development
checks, not a selected release package or cross-platform acceptance.
