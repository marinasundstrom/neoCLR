# Early unit lookup identity — 2026-10-07

Raven 3390f151b; neoCLR 2787a557 plus this harness slice. The tested compiler was
built from 7abe0adf7 plus the exact-owner fix, with binaries recorded by hash.

Source assembly lookup may temporarily return a referenced type before source shells
exist. Early UnitTypeSymbol creation captured bootstrap Void, and later generic
interface signatures retained it even after source Void was declared. The resolver
now rejects a result whose assembly differs from the RuntimeUnitContract. Unit storage
resolves when the selected source declaration exists. This retains CLI-compatible
signature categories without emitter-side name rewriting or reopening importer data.

Two C# regressions initialize unit after compiler setup but before source declaration
completion, then inspect a Result<Void,E>-returning interface in both file orders.
Both failed before the fix (PE Void instead of source Void), and pass afterwards.
52 focused unit/profile/configuration tests pass, including ordinary .NET controls.

The source-unit NativeMemory gate now includes a small UnitBox<T>/UnitConsumer fixture.
A separately compiled consumer instantiates UnitSink, calls through UnitConsumer with
UnitBox<System.Void>, and exits 42. Existing inhabited unit parameter, allocation/free,
double-free, overflow and failed-publication checks also pass. Production runtime APIs
are unchanged. This fixture tests generic unit identity, not a substitute runtime API.

- [Native source-unit commands and hashes](unit-owner-2026-10-07.json).
- [Ordinary-bootstrap control](unit-owner-control-2026-10-07.json).
- [195-input full-System audit](native-bootstrap-unit-owner-2026-10-07.json).

The full audit passes binding and clears the bootstrap Void reference, then rejects
`constructor calls require the direct base of the current constructor`. No full-System
artifact is emitted. The next bounded task is identifying and correcting that
constructor relationship; full bootstrap and ordinary imported-source-root consumers
remain open. No metadata/runtime implementation or public API signature changed here.
