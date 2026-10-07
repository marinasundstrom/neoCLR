# Source/native unit ownership — 2026-10-07

Raven `7abe0adf7`, neoCLR `a705beda` plus this harness slice; the tested compiler
snapshot was built from e492490dc plus the unit changes (binary hashes recorded).
The compiler changes are target-specific; ordinary .NET unit policy is unchanged.
76 focused compiler tests pass. The baseline had two stale diagnostic-text assertions;
the correction independently passes 17 tests on local Raven main (`8c53fa55b`).

The existing RuntimeUnitContract selects a public empty nongeneric System.Void in a
source library or a native artifact. That declaration backs inhabited values; unit
callable results remain CLI no-result and pointers to the selected unit use PTR VOID.
Missing owners, private/nonempty declarations and unselected lookalikes are tested.
Compared with CLR void, the native unit has usable nominal value storage, without
changing the established instruction encoding for no-result calls or untyped pointers.

Run `verify_source_native_memory.py --source-unit` with the paths/revision recorded in
[the executable evidence](source-unit-2026-10-07.json). It compiles unchanged production
Void, NativeMemory and allocation adapters, then compiles consumers from their own
sources against the artifact. Both allocation forms, pointer pass-through, Free and
an inhabited `Ignore(())` parameter execute (exit 42). Double-free and checked overflow
fault as expected; a managed string pointer rejects without publishing output.

The seed is explicitly empty. Runtime internal calls are defined by the production
adapters; native-width input constants are the existing metadata-API-authored fixture.
No released Numbers/NativeIntegers library is used in this mode: those artifacts still
reference the old seed-owned unit. The harness's ordinary mode remains a separately
labelled [bootstrap control](source-unit-control-2026-10-07.json), including success,
faults and failed publication. No consumer stubs or production-source rewrites are used.

The full-owned-handle audit assigns Void to the source library and removes the seed
copy. [Across 195 inputs](native-bootstrap-source-unit-2026-10-07.json), binding and
pointer signature validation now pass, but encoding fails with `native type missing
or ambiguous: System.Void`. A remaining bootstrap reference still needs migration
through the semantic unit contract. Full bootstrap, importing a source Object root
through ordinary Raven consumer commands, and broad application execution remain open.
No runtime or metadata format changes were needed in this slice.
