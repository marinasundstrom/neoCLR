# Local time mapping payloads

RavenDoc renders named union cases inline but does not emit their payload-property
pages. Use case patterns to extract these values. This reference supplies those
members until renderer support is available.

## Unique.Value

<a id="unique-value"></a>

**Property value:** [ZonedDateTime](xref:System.ZonedDateTime). The sole instant
with the requested local civil fields.

## Ambiguous.Earlier

<a id="ambiguous-earlier"></a>

**Property value:** [ZonedDateTime](xref:System.ZonedDateTime). The chronologically
earlier instant, with its actual offset and named zone.

## Ambiguous.Later

<a id="ambiguous-later"></a>

**Property value:** [ZonedDateTime](xref:System.ZonedDateTime). The chronologically
later instant, with the same local fields and its own actual offset.

Skipped has no payload. Default LocalTimeMapping is inactive. Successful MapLocal
returns an active case. See [DateTime and zones](/features/time/) for the executed example.
