# Date, time and clock APIs

Date and Time represent civil values. Instant identifies a point on the timeline, Duration an elapsed amount, and Clock supplies Now. SystemClock is the current system provider.

**Preview 9 implementation · September 19, 2026.** This is a bounded implementation, not a complete or frozen API. Use the matching Preview 9 packages.

<a id="example"></a>

## Dates, instants and clocks in Raven
```raven
{{CLOCK_SAMPLE}}
```

This helper reads the host clock and prints the local year and hour; those values naturally vary. The complete sample also uses a fixed Clock to check deterministic Instant and Duration comparisons.

[Complete executable sample →](../../samples/library-instants.rvn) · [VS Code setup →](../../try/#development)

<a id="limits"></a>

## Behavior and limits
.NET already separates DateOnly and TimeOnly and supports injectable time through TimeProvider. This API starts with those distinctions and typed validation outcomes. The current local conversion uses host settings; it is not a general timezone or calendar API.

[Detailed contract and comparisons →](https://github.com/marinasundstrom/neoCLR/blob/main/docs/date-time-design.md)

<a id="direction"></a>

## Planned work and open questions

The proposals add timezone rules, calendar-aware values and presentation through globalization. More types can make distinctions clear but bring data and conversion obligations. Formatting, parsing and culture behavior remain separate work.

[Related proposals and open questions →](../../proposals/#time)

<a id="feedback"></a>

## Questions and contributions

Questions, examples and documentation corrections are welcome. See [how to contribute](../../#feedback).

Report issues with a small program, the toolchain version, expected behavior and observed output. API proposals should identify the missing operation or contract.

[Discuss on GitHub ↗](https://github.com/marinasundstrom/neoCLR/issues)
