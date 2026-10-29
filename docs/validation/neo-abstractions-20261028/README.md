# Neo abstraction assemblies and integrated sample — 2026-10-28–29

Author-directed second slice; verification is focused rather than a full suite.

- Raven `codex/neoclr` compiler `34f14c8ce` preserves nested native callback
  nullability across imports. The focused emit/import/generic-forwarding check
  passes; legacy opaque annotations require rebuilding to recover nested facts.
- All eleven runtime/extension/provider libraries build, including the three
  new abstraction assemblies. See [build commands](library-build.json).
- [ExtensionsConsumer](../../../samples/extensions/src/ExtensionsConsumer/ExtensionsConsumer.rvnproj)
  compiles using only System.Runtime and the three abstraction projects.
- Four component-layout checks and four native API snapshot checks pass; the
  CLI reference/source fingerprint check passes. The native snapshot includes
  the new declaring assemblies and source locations. No full website build or
  broad runtime suite was repeated.
- Focused native regressions check that unreachable boxing does not retain an
  unboxed value's executable interface view, and that actual boxed interface
  implementations remain selected. Original conformance and reflection metadata
  are preserved. The expanded bundle also exercises removal of the old eight-module
  CLI/linker cap; aggregate type/method limits remain unchanged.

The integrated sample uses default DI, default console logging and a JSON resource
localizer. It resolves a scoped GreetingService with typed logger/localizer
constructor parameters and disposes the scope and root. Its retention manifest
covers the service and default adapter constructors. The compiler retains the
existing DAM2001/DAM2004 conservative diagnostics for indirect DI registration and
dynamic implementation bodies; explicit sample retention supplies those roots.

The first native run exposed source-wide boxing keeping FileReadError interface
views after unreachable implementation methods were trimmed. The assistant's
proposed in-memory fallback was rejected by the author; the final sample retains
file-backed resources and the native selection/projection fix is included.

Commands to build and run the sample are in its [README](../../../samples/extensions/README.md).

Final completion (2026-10-29): the normal native project build passed and the
JSON-backed executable exited 0, printing:

```text
info: Examples.Extensions.GreetingService[0] Hej Ada! Logging and localization are composed through DI.
```

See [sample evidence](sample.json). The compiler bridge fix is committed in Raven
`34f14c8ce`; the implementation is committed in neoCLR `c9289d551` with formatting follow-up
`4c303ef7c`.
