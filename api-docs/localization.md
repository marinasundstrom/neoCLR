---
title: Localization
---
# Localization (development)

`Neo.Localization.Localizer` provides `strings[key]` and `Find(key)`.
The indexer returns the key when absent; Find returns `Optional<string>`.
Empty translations are present. Keys are ordinal and case-sensitive; stored UTF-8
text is preserved. `Localizer<T>` associates a catalogue with a resource type.

## JSON provider

`JsonLocalizerProvider`, lookup interfaces, formatting, diagnostics and DI registration
live in the separate `Neo.Localization.dll` assembly under `Neo.Localization`.
It references System.Runtime, System.Data and Neo.DependencyInjection. The native
reference includes their members. Replace old System.Localization imports and
retention names and rebuild consumers; there is no old-name compatibility shim.
See the [assembly boundary and migration](https://github.com/marinasundstrom/neoCLR/blob/main/docs/application-services.md).

Construct `JsonLocalizerProvider(directory)` to enable invariant fallback, or
`JsonLocalizerProvider(directory, fallbackToInvariant: false)` to disable it.
Construction performs no I/O. `Load(culture)` loads unscoped resources;
`Load(culture, scope)` selects an explicit scope. `Load<T>(culture)` returns
`Result<Localizer<T>, LocalizationError>` and uses T's nominal simple name as scope.
Structural types are rejected. Types with identical simple names share a scope;
use an explicit scope for collision avoidance or stability across renames.

| Selection | Files in priority order |
| --- | --- |
| Swedish, scope FeatureX | FeatureX.sv-SE.json, FeatureX.sv.json, FeatureX.json |
| Swedish, no scope | sv-SE.json, sv.json, invariant.json |
| Invariant, scope FeatureX | FeatureX.json |
| Invariant, no scope | invariant.json |

Fallback is **per key**. Empty values override fallbacks. Disabling automatic
invariant fallback omits the last file for a non-invariant culture; explicitly
selecting Culture.Invariant always reads it. Culture selection uses the existing
five formatting profiles; arbitrary locale tags/script fallback are future work.
Missing files are skipped, and all-missing returns an empty catalogue. Other I/O,
UTF-8, syntax, duplicate-key and limit failures return errors, including failures
in fallback files. Only the configured directory is used. Scope cannot contain
NUL, slash, backslash or colon, or be `.`/`..`.

Each file is a flat JSON object of strings:

```json
{"hello":"Hej världen","empty":"","menu.save":"Spara"}
```

Dots are literal key characters. This is the simple JSON format supported by
localization tools, not full i18next or ICU MessageFormat. Nested objects, arrays,
numbers, booleans and null values are rejected. Plural selection, named interpolation,
ambient culture, reload watching and resource discovery are not implemented.
Each file is limited to 65536 UTF-8 bytes and 4096 entries; JSON escapes and
surrogate pairs are validated. Duplicate names are checked after escape decoding.

`Create(string)` and `Create(ReadOnlySlice<byte>)` parse an in-memory catalogue.
The byte input is borrowed only for the call; the returned catalogue owns its
strings. Static `Load(path)` reads one explicit file synchronously through the
current blocking storage provider. Instance Load methods also perform blocking
file reads. `Read(InputStream)` returns a Task, reads through EOF and leaves the
borrowed stream open. It uses bounded text buffering; it is not an incremental
JSON parser. File resources are closed by the storage provider before return.

## Formatted lookup

`Localizer.Culture` is the selected formatting culture. Single-file and in-memory
catalogues use invariant culture. Culture-based loading retains the requested
culture regardless of which fallback supplies a template.

`LocalizerFormatting` adds indexers and `Find` overloads for one, two or three
`object?` arguments and a `params object?[]` argument list. Lookup happens first:
`strings["Greeting", name]` formats a resolved template, or returns the key unchanged.
`strings.Find("Greeting", name)` returns `Some(formatted)` or `None`. No-argument
lookup preserves the raw template. Null arguments produce empty text. Templates
use `String.Format` composite syntax, alignment and format specifiers; malformed
formats retain its terminal failure contract, rather than becoming missing keys.
Fixed overloads avoid an argument array; value arguments may box and formatting
can allocate output text. Import `Neo.Localization.*` to use the extensions.

## Missing-key diagnostics

`strings.WithMissingKeyHandler(handler)` returns a wrapper. It calls the handler
once per missing indexer lookup, then returns the key. Find remains quiet and
empty translations do not trigger the callback. Attach diagnostics after loading
the fallback chain so only genuinely absent keys are reported. The application
can capture culture/scope and call Console.WriteLine, a logger or a counter.
Handlers run synchronously; throttling and avoiding recursive lookup are caller
responsibilities. Handler failures propagate. The original localizer is unchanged.

Loading allocates the owned catalogue; lookups reuse stored strings and do not
allocate a LocalizedString wrapper. HashMap is used with ordinal equality.
Concurrent access is not currently promised by the underlying map implementation.
Keep loaded instances for reuse; the provider does not cache them automatically.
Native/JIT file loading currently uses the macOS host file-input service; Windows
native file input remains a host capability gap. Interpreter storage uses its
existing host implementation.

[Localization feature guide](/features/localization/)

## Dependency injection (development)

`LocalizerFactory.Create(resourceType: TypeInfo) -> Localizer` is the nongeneric
factory contract. `DefaultLocalizer<T>(factory: LocalizerFactory)` implements
`Localizer<T>` by delegating to `factory.Create(typeof(T))`.

`JsonLocalizerFactory(provider: JsonLocalizerProvider, culture: Culture)` fixes
resource configuration and culture without I/O. Create caches a lazy localizer
per exact resource type. First lookup synchronously loads the fallback chain;
invalid resources fault. Use `provider.Load(culture, resourceType)` for a
recoverable `Result<Localizer, LocalizationError>` instead. Find reports a missing
key, not loading or formatting errors. Instances are for serialized guest access.

`ServiceCollection.AddLocalization(factory: LocalizerFactory)` returns
`Result<unit, ServiceError>`. It conditionally registers the caller-owned singleton
factory and open transient `Localizer<> -> DefaultLocalizer<>` mapping. Existing
explicit registrations win. Different resource types need no separate registration.
Each resolution creates a small typed adapter; the JSON factory reuses catalogues.
Retain DefaultLocalizer's constructors for native activation, as well as consumer
constructors. No ambient culture, reload or plural rules are introduced.

For a native project, add the adapter retention alongside your consumer roots:

```sh
--reflection-constructor-type 'Neo.Localization.DefaultLocalizer`1'
```

The native project builder resolves this exact metadata name from the supplied
class-library modules. It retains the closed adapter shapes observed through
`Localizer<T>` requests; it does not enable arbitrary runtime code generation.
