## NeoCLR Globalization API — Current Proposal

The goal is a .NET-inspired globalization model that keeps the familiar ergonomics while separating concepts more cleanly, using immutable values and NeoCLR's `Result`/`Option` conventions.

The globalization domain is separate from the Time API and from the future localization/resource API.

```text
System.Time              System.Globalization          System.Localization
───────────              ────────────────────          ───────────────────
Instant                  Culture                       resources
Date                     CultureId                     messages
Time                     Language                      localization
LocalDateTime            Region
ZonedDateTime            Script
TimeZone
Calendar                 NumberFormat
Duration                 DateTimeFormat
Period                   Collation

Clock                    CultureProvider
                         CultureResolver
```

`System.Localization` remains a later proposal.

### 1. `Culture`

`Culture` replaces the role of .NET's `CultureInfo`.

```raven
let culture = Culture.Parse("sv-SE")?;
```

A `Culture` is an **immutable description of cultural conventions**.

```raven
class Culture
{
    Id: CultureId

    Language: Language
    Region: Option<Region>
    Script: Option<Script>

    NumberFormat: NumberFormat
    DateTimeFormat: DateTimeFormat

    DefaultCalendar: Calendar
    Collation: Collation
}
```

Unlike `CultureInfo`, it isn't intended to be cloned and mutated to alter formatting behavior.

Explicit customized formatting can instead be represented through immutable format values.

---

### 2. `CultureId`

Culture identity should be separated from resolved globalization data.

```raven
let id = CultureId.Parse("sv-SE")?;
```

Conceptually:

```raven
struct CultureId
{
    Language: Language
    Script: Option<Script>
    Region: Option<Region>
}
```

It represents a standardized language tag without implying that all globalization data has been resolved.

Therefore:

```text
CultureId("sv-SE")
        │
        │ resolve
        ↓
      Culture
        │
        ├── NumberFormat
        ├── DateTimeFormat
        ├── DefaultCalendar
        └── Collation
```

This also gives protocols, configuration and metadata a lightweight culture identifier to carry around.

---

### 3. `Language`, `Region`, and `Script`

These are strongly typed standardized identifiers rather than arbitrary strings.

```raven
Language.Parse("sv")?
Region.Parse("SE")?
Script.Parse("Latn")?
```

They compose a `CultureId`:

```text
sv-Latn-SE

Language = sv
Script   = Latn
Region   = SE
```

They should probably be compact immutable value types.

We don't necessarily need thousands of static members such as `Region.Sweden`; parsing standardized identifiers can remain the primary mechanism.

---

### 4. `CultureResolver`

Resolving an identifier into actual globalization data is a separate responsibility:

```raven
interface CultureResolver
{
    fn Resolve(id: CultureId)
        -> Result<Culture, CultureError>
}
```

For example:

```raven
let id = CultureId.Parse("sv-SE")?;
let culture = cultures.Resolve(id)?;
```

This separation is important:

```text
CultureId       identity
Culture         resolved cultural conventions
CultureResolver source/resolution of those conventions
```

The runtime implementation could eventually use bundled Unicode/CLDR data, OS globalization facilities, or another backend without changing the public culture model.

---

### 5. `CultureProvider`

`CultureProvider` answers a different question:

> Which culture applies to the current context?

```raven
interface CultureProvider
{
    Current: Culture
}
```

This parallels the Time API:

```text
Clock                       CultureProvider
  │                              │
  ↓                              ↓
Instant                        Culture
```

An application can therefore inject its culture dependency:

```raven
class InvoiceRenderer
{
    init(culture: CultureProvider)

    fn Render(invoice: Invoice) -> String
    {
        return invoice.Total.ToString(culture.Current);
    }
}
```

Tests can use something like:

```raven
let culture = Culture.Parse("sv-SE")?;

let provider = FixedCultureProvider(culture);
```

without changing ambient process state.

The runtime can provide:

```text
SystemCultureProvider
FixedCultureProvider
```

The exact concrete names aren't locked yet.

---

### 6. Ambient culture remains ergonomic

Dependency injection should not become mandatory for ordinary formatting.

This should work:

```raven
price.ToString()
date.ToString()
```

and mean approximately:

```raven
price.ToString(Culture.Current)
date.ToString(Culture.Current)
```

where `Culture.Current` is an ergonomic facade over the runtime's contextual culture provider.

Explicit culture remains available:

```raven
price.ToString(culture)
date.ToString(culture)
```

So the API has a useful gradient:

```text
value.ToString()
        ↓
ambient culture

value.ToString(culture)
        ↓
explicit culture

Service(CultureProvider)
        ↓
injectable contextual culture
```

The ambient culture should be execution-context-local rather than simply mutable global process state.

---

### 7. `NumberFormat`

`NumberFormat` is an immutable description of number conventions.

Conceptually:

```raven
class NumberFormat
{
    DecimalSeparator: String
    GroupSeparator: String

    DecimalDigits: Int32

    PositiveSign: String
    NegativeSign: String

    PercentSymbol: String

    Currency: Option<CurrencyFormat>
}
```

Normal numeric APIs remain familiar:

```raven
amount.ToString()
amount.ToString(culture)
amount.ToString("N2", culture)
```

We can later decide whether NeoCLR should retain .NET-style format strings unchanged or introduce something better alongside them.

---

### 8. `DateTimeFormat`

`DateTimeFormat` describes cultural presentation of the types from `System.Time`.

```raven
class DateTimeFormat
{
    ShortDatePattern: String
    LongDatePattern: String

    ShortTimePattern: String
    LongTimePattern: String

    DateSeparator: String
    TimeSeparator: String

    DayNames: ...
    MonthNames: ...

    FirstDayOfWeek: DayOfWeek

    Calendar: Calendar
}
```

The important separation is:

```text
System.Time                         System.Globalization

Date          ────────────────────→ DateTimeFormat
LocalDateTime ────────────────────→ DateTimeFormat
ZonedDateTime ────────────────────→ DateTimeFormat

Calendar      ←──────────────────── Culture
```

Temporal values **do not carry cultures**.

---

### 9. `Calendar` belongs to the Time domain

We've now separated this conceptually from globalization.

Something like:

```raven
use System.Time;

Calendar.Gregorian
Calendar.Hebrew
Calendar.Islamic
Calendar.Julian
```

A calendar defines temporal/calendar semantics: years, months, days, leap years and calendar arithmetic.

Globalization merely describes which calendar is conventionally associated with a culture:

```raven
culture.DefaultCalendar
```

Potentially:

```raven
culture.SupportedCalendars
```

Therefore the conceptual dependency is:

```text
System.Time
    Calendar
       ↑
       │
System.Globalization
    Culture
    DateTimeFormat
```

rather than making the Time API dependent upon globalization.

---

### 10. `Collation`

Instead of copying `.NET`'s `CompareInfo` terminology, NeoCLR should model the actual concept explicitly:

```raven
class Collation
{
    fn Compare(
        left: String,
        right: String,
        options: CollationOptions = ...
    ) -> Ordering

    fn Equals(
        left: String,
        right: String,
        options: CollationOptions = ...
    ) -> Bool

    fn GetSortKey(...) -> ...
}
```

A culture supplies its linguistic collation:

```raven
culture.Collation
```

This creates a deliberate distinction between:

```text
ordinal comparison
        │
        └── Unicode/code-unit semantics

linguistic comparison
        │
        └── Collation + Culture
```

That distinction should remain visible in the API.

---

### 11. Parsing follows NeoCLR conventions

There is no need for the historical `.NET` pairing of:

```csharp
Parse(...)
TryParse(..., out value)
```

Parsing can fail normally:

```raven
Culture.Parse(text)
    -> Result<Culture, CultureParseError>

CultureId.Parse(text)
    -> Result<CultureId, CultureParseError>

Date.Parse(text, culture)
    -> Result<Date, DateParseError>

Decimal.Parse(text, culture)
    -> Result<Decimal, NumberParseError>
```

and Raven gets normal propagation:

```raven
let culture = Culture.Parse(input)?;
```

`Option<T>` remains reserved for genuine absence rather than malformed input.

---

### 12. Invariant culture

NeoCLR retains the useful concept:

```raven
Culture.Invariant
```

for stable culture-independent human formatting:

```raven
value.ToString(Culture.Invariant)
```

But protocol and serialization formats shouldn't blindly depend on invariant culture. JSON, ISO representations, wire formats, etc. should define their own canonical representations.

---

## Current architecture

So at this point I'd consider this our working model:

```text
                 CultureId
                     │
                     │
              CultureResolver
                     │
                     ↓
                  Culture
        ┌────────────┼─────────────┐
        ↓            ↓             ↓
 NumberFormat  DateTimeFormat   Collation
                     │
                     ↓
                  Calendar
                     │
                     ↓
                System.Time


 CultureProvider
        │
        ↓
 current Culture
```

with three important design principles:

> **Values don't carry cultures. Culture describes how values are presented and linguistically interpreted.**

> **Culture is immutable; contextual culture comes from a provider.**

> **Time, globalization, and localization are separate domains that compose rather than being bundled together.**

That gives us something recognizably descended from the .NET BCL, but `Culture` is considerably less overloaded than `CultureInfo`, ambient state isn't fundamental to the model, culture-data resolution is separated from culture identity, and the Time API remains independently coherent.
