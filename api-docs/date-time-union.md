# Optional DateTime union declaration

Prefer the specific LocalDateTime or ZonedDateTime type. Use DateTime only when an
API intentionally represents either a local or a zoned date and time.

The pinned RavenDoc publisher renders the DateTime heading as `union struct DateTime`
and generates constructor/extractor pages, but omits the parenthesized alternative
list from that heading. This supplement gives the exact source declaration; no public
type is excluded from the generated reference.

```raven
public union DateTime(LocalDateTime | ZonedDateTime)
```

[LocalDateTime](xref:System.Time.LocalDateTime) carries civil Date and Time without a zone.
[ZonedDateTime](xref:System.Time.ZonedDateTime) retains an Instant, named TimeZone, civil
projection and actual TimeOffset. Values of either type implicitly convert to
[DateTime](xref:System.Time.DateTime). There are no nested Local/Offset wrapper cases.

| Member | Contract |
| --- | --- |
| `DateTime(LocalDateTime value)` | Activates the civil alternative. |
| `DateTime(ZonedDateTime value)` | Activates the resolved alternative. |
| `TryGetValue(out LocalDateTime value) -> bool` | Assigns only when the active alternative is civil. |
| `TryGetValue(out ZonedDateTime value) -> bool` | Assigns only when the active alternative is zoned. |
| `HasValue: bool` | False for the inactive zero/default union. |
| `Value: object?` | Boxed civil value or referenced zoned value; null when inactive. |
| `ToString() -> string` | Compiler-generated diagnostic text, not cultural formatting. |

Prefer type patterns to calling extraction members directly. The tested
[DateTime feature example](/features/time/) accepts both alternatives and shows named
zone mapping. This is a nominal Raven union, not .NET DateTime with a Kind flag.
