# Globalization: cultures, languages and calendars

Culture describes language and
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
Date, TimeOfDay and LocalDateTime Parse overloads accept the three
numeric Gregorian culture formats, with current or explicit culture and typed errors.
They preserve up to seven fractional-second digits. General format strings,
non-Gregorian parsing, timezone rules and bidirectional layout remain unsupported. The UI controls text direction when embedding Hebrew output.

## Localization is independent

`System.Localization` provides string lookup independently of cultural formatting.
See [localization](/features/localization/) for JSON resources, typed scopes,
per-key fallback and optional missing-key diagnostics.

[API reference →](../../docs/namespaces.html) · [Date/time APIs →](../time/)
· [Design comparisons and evidence →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/calendar-globalization.md)

Development Formattable now receives the existing Culture explicitly. Culture-first
String.Format overloads and bounded date/time/duration formats connect composite
formatting to these policies. See [supported time formats](/features/time/).
This does not add numeric format providers or general locale-pattern data.


## Monetary values (development)

Use `Money`, a record struct combining Decimal and Currency, to keep
money's denomination with its value. Both named `let Money(amount, currency) = money`
and unnamed `let (amount, currency) = money` deconstruct the record. Parsing returns that record; equality and
hashing include both components. Construction preserves precision, and changing
the formatting culture never changes the currency. Display rounding leaves the
stored amount untouched. Decimal C formatting remains a lower-level convenience.

The bounded catalog covers SEK, ILS, GBP, USD, EUR and JPY, with XXX for unspecified
currency. Exact expected-currency parsing and culture formatting cover the five
built-in profiles. Exchange rates, cash rounding and full locale data are outside
this foundation. See the [monetary API reference](/docs/text-numbers.html).

`Language.IsRightToLeft` exposes Hebrew's default writing direction; the other
supported languages default to left-to-right. It does not perform bidi layout.
