# Localization: strings for your application

`System.Localization` separates resource lookup from cultural formatting.
Load a JSON catalogue from a directory you choose, then use `Localizer` for
key-based lookup. `Localizer<T>` associates resources with a feature type.

## Small JSON catalogues

Resources are flat UTF-8 JSON objects whose keys and values are strings:

```json
{"Greeting":"Hej {0}!","menu.save":"Spara","empty":""}
```

Dots are literal key characters. Keys are case-sensitive and ordinal; translated
text retains its Unicode content. Empty translations count as present.

## Choose a directory and optional scope

`JsonLocalizerProvider(directory)` configures loading without performing I/O.
`Load(culture)` selects unscoped resources. `Load(culture, scope)` selects a feature,
and `Load<T>(culture)` uses the type's simple name as its scope.
Loading returns a `Result`, making file or catalogue errors explicit.

For Swedish resources, each key is resolved in this order:

| Scope | Regional | Language | Invariant |
| --- | --- | --- | --- |
| None | sv-SE.json | sv.json | invariant.json |
| FeatureX | FeatureX.sv-SE.json | FeatureX.sv.json | FeatureX.json |

Fallback is per key, so a regional file need contain only its overrides. Missing
files are skipped. Set `fallbackToInvariant: false` on the provider to omit the
invariant fallback; explicitly selecting `Culture.Invariant` still loads it.

## Lookup first, then format

`strings["Greeting", name]` resolves the translation before inserting arguments
using composite placeholders such as `{0}`. A missing resource returns the key
unchanged. `strings.Find("Greeting", name)` instead returns `None` when absent.
Without arguments, lookup returns the stored template unchanged. These assertions
come from the built-in localization contract tests:

```raven
{{LOCALIZATION_FORMAT_SAMPLE}}
```

Formatting uses the selected culture even when the translation comes from a
fallback file. Invalid composite templates follow `String.Format` error behavior;
`Find` distinguishes missing resources, not malformed formatting.

Attach `WithMissingKeyHandler` to report absent display keys to a console or logger.
`Find` remains quiet, and successful fallback does not trigger a missing-key report.

## A bounded foundation

Reuse loaded localizers. Loading creates owned catalogues; ordinary lookup reuses
stored strings without allocating a result wrapper. Formatting creates result text;
fixed overloads for one to three arguments avoid an argument array.

This foundation supports the five built-in cultures and flat string catalogues,
with a 64 KiB and 4,096-entry limit per file. Nested resources, plural rules, reload
watching and automatic caching are not implemented. File loading is synchronous;
borrowed input streams can be read asynchronously. Current native/JIT file loading
requires the macOS host file-input service.

[Localization API reference →](/docs/localization.html) ·
[Culture and formatting →](/features/globalization/)
