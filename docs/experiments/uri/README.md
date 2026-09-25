# Managed URI contract probe

The installed development library supplies System.Uri and UriError. This probe
compiles an ordinary Raven app against matching reference/bridge/library artifacts:

```sh
python3 docs/experiments/uri/verify.py \
  --toolchain-root /path/to/matching/neoclr-sdk \
  --runner target/release/examples/measure_async
```

The verifier generates its app from checked-in RFC cases. Both string and Uri
Resolve overloads must give the expected text. It checks relative/absolute flags,
Object virtual dispatch, exact-text equality and hashes, EquatableTo<Uri>, collections,
invalid escaped input, unsupported IP literals, nonabsolute bases and size limits.
The runtime uses a 256-object limit and must finish with zero live managed objects.
This is a functional/GC check, not a performance benchmark or cross-platform matrix.

The independent .NET 10 probe asserts common relative-resolution behavior, prints
other differences from the RFC examples and records which invalid neoCLR inputs
.NET accepts. Differences are expected because .NET normalizes URLs and supports
forms outside this bounded contract; they are not automatically .NET defects.
The script reports the actual .NET version. See [URI design](../../uri-design.md)
for the provisional grammar, lexical identity and sources.

HttpError and HttpClient.BaseUri are subsequent integration slices. This test does
not use networking or imply the existing HTTP transport accepts every parsed Uri.

## Local result — 2026-09-24

Passed 46 stored reference-resolution cases through both overloads, three additional
base-path cases, invalid grammar, Object/interface/collection checks and boundary
checks on macOS. The runtime reports `allocated=504 live=0 peak=64 collections=9
reclaimed=504`. .NET 10.0.0 passes the common-contract assertions and records four
resolution differences, described in the design notes. The full probe has a
10-million-instruction budget and a five-minute host timeout. It parses the
maximum-size input once and reuses that Uri for the resolved-length rejection case.
This boundary-heavy interpreted test remains slow; it is not a throughput result.

Matching bootstrap regeneration, API snapshot validation, the combined 986-page
website build and all 17 website tests pass. No remote CI matrix, release or website
publication is claimed.
