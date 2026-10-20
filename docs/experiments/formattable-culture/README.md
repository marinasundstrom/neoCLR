# Culture-aware Formattable (development, 2026-10-20)

`System.Formattable.ToString(format: string, culture: Culture)` replaces the
one-argument development interface. Custom implementations must accept the culture;
recompile consumers against the matching library. `String.Format` offers culture-first
one-, two-, three-argument and params overloads. Existing overloads (including Console
formatting) resolve one current-culture snapshot lazily for all Formattable arguments.
Calls without Formattable arguments do not resolve a culture. This is an ordinary
library interface change, with no runtime ABI, compiler configuration or host binding change.

| Type | Supported formats |
| --- | --- |
| Duration | empty/c/t/T: invariant constant; g: optional days, trimmed fraction; G: days and seven fraction digits |
| Date | empty/G/d: existing numeric date convention; O/o: invariant Gregorian date |
| TimeOfDay | empty/G: existing seven-digit culture convention; O/o: invariant seven digits; t: HH:mm; T: HH:mm:ss |
| LocalDateTime | empty/G: existing culture convention; O/o: invariant seven-digit civil representation; s: invariant seconds |

Duration supports the entire signed Int64 tick range. Swedish g/G uses a decimal
comma; invariant and Hebrew use a dot. Date/time formatting retains the existing
explicit culture/calendar policies. Roundtrip forms use Gregorian regardless of
selected calendar. Local civil values never acquire an offset or timezone.
Unsupported formats fault, including custom patterns. Calendar range failures remain
Result errors on DateTimeFormat and faults on ToString. Numeric primitive format
specifiers, custom providers, arbitrary locale data and custom date patterns remain gaps.

## Comparison and tradeoffs

[.NET IFormattable](https://learn.microsoft.com/en-us/dotnet/api/system.iformattable.tostring?view=net-10.0)
passes a format and IFormatProvider. This development API uses the existing concrete
Culture instead: it makes propagation explicit without introducing a provider system
before the library can support one. The cost is no pluggable provider and a future
migration if provider extensibility is needed. Non-null arguments follow existing
neoCLR API conventions; an empty format selects the default.

[TimeSpan standard formats](https://learn.microsoft.com/en-us/dotnet/standard/base-types/standard-timespan-format-strings)
guide Duration c/g/G. [Date/time standard formats](https://learn.microsoft.com/en-us/dotnet/standard/base-types/standard-date-and-time-format-strings)
guide O/o, s and t/T. Civil G/d retain neoCLR's existing bounded numeric conventions
and seven-digit precision rather than claiming .NET's complete culture-pattern
behavior. This preserves existing display but is a deliberate limitation: full locale
pattern data is deferred, not replaced by an assertion of equivalent globalization.

Formatting allocates output and builders; composite value arguments can box. Fixed
arities avoid argument arrays. The lazy snapshot avoids resolving the host culture
for literal-only/nonformattable calls and avoids repeating it per placeholder. These
are implementation properties, not a measured throughput or allocation-free claim.

## Validation

`Program.rvn` exercises explicit/default culture dispatch, fixed arities and params,
Formattable and Object dispatch, date/time formats, duration boundaries and roundtrip.
`Invalid.rvn` checks terminal rejection for unsupported formats on all four types.
`check.py` records commands, outputs and binary hashes for interpreter/JIT/AOT.
The existing composite-formatting consumer is also recompiled with the new interface.
The matching compiler is the existing record-policy compiler; no compiler changes
are required. See the JSON reports beside this file for actual results.

Validation completed: all 18 public/regression/fault executions pass across
interpreter/JIT/AOT (three current-format, three composite-regression and twelve
invalid-format executions). Four native API snapshot checks pass. The website build
checks 4913 pages; no publication or release qualification is implied.
