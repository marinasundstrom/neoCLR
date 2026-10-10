# Civil parsing and host-culture parity

The runtime framework's `time-parsing` suite tests Date, TimeOfDay and LocalDateTime
Parse against boundaries, malformed input, fraction precision and culture profiles.
[Eight-case evidence](../../../runtime/raven/tests/time-parsing-validation.json)
records native macOS ARM64 and interpreted passes. See [the design](../../date-time.md)
for grammar, .NET comparison and remaining parser gaps.

`Culture.rvn` prints the host preference, the provider's resolved culture and
Culture.Current. The probe compares exact native/interpreted output on one host;
it does not presume the host uses one particular language. [Local evidence](validation.json)
reports en-US with invariant fallback. The Windows action runs the same probe.

```sh
python3 docs/experiments/time-parsing/verify.py \
  --bundle /path/to/bundle --aot /path/to/neoclr-aot-poc \
  --runtime /path/to/neoclr --output target/culture-parity
```

A fresh output is required. `--existing-build` can reuse a successful build of
Culture.rvnproj after checking its recorded input/artifact hashes. Both target
profiles must use matching runtime libraries and adapters. Native host preference
uses CFLocaleCopyPreferredLanguages on macOS and GetUserPreferredUILanguages on
Windows, matching the interpreter's sys-locale 0.3.2 provider. Unsupported preferences
are resolved by the existing Raven SystemCultureProvider; this is not host date parsing.
