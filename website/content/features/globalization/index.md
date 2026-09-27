# Globalization: cultures, languages and calendars

**Provisional development API, beyond Preview 10.** Culture describes language and
formatting preferences. Calendar defines date arithmetic and the interpretation of
year, month and day. You can choose each independently.

## A day in two calendars

Gregorian September 16, 2023 is Hebrew 1 Tishri 5784. Date stores the same absolute day
in both cases: Calendar.CreateDate creates it from calendar fields; GetYear, GetMonth
and GetDay project it into another calendar. Conversion works both ways without
formatting and parsing strings. Date's own Year, Month and Day are Gregorian.

Israel uses both calendars. Culture.HebrewIsrael defaults to Gregorian; selecting
Calendar.Hebrew retains Hebrew language and changes the calendar representation.
Swedish formatting uses the Gregorian calendar, rather than a separate Swedish calendar.

```raven
{{DATE_FORMAT_SAMPLE}}
```

This tested example outputs:

```text
2023-09-16
א׳ תשרי ה׳תשפ״ד 12:34:56.0000000
5784-01-01T12:34:56.0000000
```

[Download the complete Raven example →](../../samples/library-date-formatting.rvn)

## Explicit preferences or system discovery

Culture.Invariant, Culture.SwedishSweden and Culture.HebrewIsrael are immutable
profiles. Each exposes Language, DefaultCalendar and DateTimeFormat. Date.ToString()
uses Culture.Current; Date.ToString(culture) makes the choice explicit.
DateTimeFormat.WithCalendar returns a formatter with the requested calendar.

CultureProvider allows injection. FixedCultureProvider supplies a chosen culture;
SystemCultureProvider snapshots the host's preferred locale. PreferredCultureName
preserves the reported name; Current resolves Swedish or Hebrew, falling back to
invariant for unsupported languages. IsFallback reports an approximation or fallback.
Culture.Current reads the system preference afresh, without mutable thread-local state.

## Contracts and limits

Like .NET, calendar rules are separate from cultural presentation. This smaller API
uses Date and Result for invalid calendar operations, with immutable formatting
selection instead of writable CultureInfo data. That makes selection explicit but
provides only three fixed profiles, rather than .NET's extensive locale database.

Hebrew supports complete years 5344–5999, civil month numbering with Tishri first,
and fixed calculated calendar rules. It does not determine religious sunset boundaries.
FormatDate and FormatLocalDateTime return errors outside the selected calendar range.
Hebrew presentation includes month names and traditional Hebrew numerals; time remains
24-hour numeric with seven fractional digits. Invariant Hebrew output uses Latin
digits for Hebrew fields and is **not a Gregorian ISO serialization format**.

System discovery was tested on macOS; Windows/Linux adapters have not been exercised.
Preferred language discovery does not promise OS regional pattern overrides.
Arbitrary format strings, parsing, timezone rules and bidirectional layout are outside
this slice. The UI controls text direction when embedding Hebrew output.

## Localization is independent

Resource localization will use shared interfaces across feature areas. Providers and
sources can differ, including JSON and resource files, while callers use the same
localization contracts. Culture supplies language and formatting preferences; it does
not own resource loading. Those unified localization interfaces remain future work.

[API reference →](../../docs/namespaces.html) · [Date/time APIs →](../time/)
· [Design comparisons and evidence →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/calendar-globalization.md)
