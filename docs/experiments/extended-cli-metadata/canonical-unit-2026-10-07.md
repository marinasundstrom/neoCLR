# Canonical native Void — 2026-10-07

The author's direction is one NeoCLR unit type: `void`/`System.Void`, inhabited in
value positions including generic arguments. The separate .NET carrier exists because
CLR void cannot occupy those positions. No-result call/return remains an execution
convention, not a second NeoCLR language type. Existing .NET lowering is unchanged.

Raven **e769eb4b8** explicitly designates the RuntimeUnitContract source owner and
output-owned imported references as native Void storage. The metadata implementation
is based on neoCLR **7eb61914** plus this slice. Tested compiler snapshot binaries,
source inputs, runtime, dependency artifacts, commands and results are recorded in
[the evidence](canonical-unit-2026-10-07.json). No runtime code changes are needed.

The source declaration must be empty, nongeneric, top-level and constructor-free.
Definition and builder APIs share that validation; native function signatures preserve
value versus no-result convention. External unit aliases retain exact assembly ownership
and dependency declarations. Reader/introspection resolves the same unit owner through
callback results and ordinary signatures. No name-only empty-struct substitution or
importer reuse is introduced. Nominal CLI transport remains necessary wherever CLI void
is illegal; executable CLI primitive-provider authoring remains unsupported.

## Verification

- 165/165 C# metadata groups: manual/builder ownership, signature/introspection round
  trips, callbacks, generic unit arguments, external unit identity and invalid storage.
- API-authored native callback → generic identity execution returns 42. A separate PE
  consumer with the explicit unit-provider library also verifies and returns 42.
- The existing source-unit NativeMemory harness compiles unchanged production sources,
  then separate Raven consumers without those sources. Unit parameters and generic
  interface consumption return 42; double-free and overflow fault as expected; unsupported
  managed pointer emission publishes no artifact. No retained System implementation is
  used in this focused harness (explicit empty seed only).
- 13/13 Raven RuntimeUnitContractTests pass on .NET, including generic value-carrier
  execution and no-result returns. Compiler build and API snapshot check pass.

## Remaining bootstrap gate

All 197 aggregate inputs emit, and runtime admission passes DNS callback binding. It
now rejects `module System does not reference NeoMetadata_79C5D7429CAE2158FCD0DE05E0A1DE959B01A78720392AE49806BC8ADAD82380`.
That is the retained seed's dependency edge to the source-owned aggregate. The runtime
check uses a separate API-authored app with the full library loaded; it is not proof
that an ordinary Raven application can consume and execute the entire System library.

Next repair the explicit retained-seed/source ownership catalog and resume runtime
admission before the core/Data/Networking/Web separate-compilation gates. No optional
assembly split or full bootstrap completion is claimed. These compiler changes are
native-target-specific; there is no independent shared fix to backport to Raven main.
