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

Current enum storage is Int32 and metadata member names are ASCII identifiers.
The generic methods currently validate enum type arguments at runtime: native
method constraints cannot yet express `struct, Enum`. Invalid type arguments fault,
even on null input; they do not return a parsing error. This is a compiler/metadata
constraint gap, not an intended permanent divergence from typed .NET overloads.
Unicode enum member metadata and other backing widths are separate existing gaps.

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

`runtime/raven/tests/enum-parsing` uses the built-in TestAttribute discovery,
assertions and runner. Four tests cover names/aliases/case, imported enums, lists,
Int32 endpoints/overflow, unnamed values, empty enums, malformed/null input, output
reset and ordinary display after collection. Run with `scripts/test-runtime-library.py
--suite enum-parsing` using the matched bundle and all three execution modes.

[Retained report](validation-20261020.json): four tests pass in each engine (12
executions) on macOS ARM64. The backend malformed-signature/no-enum-token regression
passes; four API snapshot checks and the 5,036-page website build pass. These are
development checks, not a replacement package or cross-platform release qualification.

Compiler follow-up: the forwarding TryParse overload explicitly initializes its out
parameter before calling the other overload. Without that initialization this compiler
reported RAV0269 (unassigned out parameter). Initialization preserves the documented
reset contract; general forwarded-out definite-assignment behavior needs an isolated
Raven-main reproduction before deciding on a compiler fix. This is not evidence
that ordinary caller out arguments fail (those are exercised by the built-in tests).
