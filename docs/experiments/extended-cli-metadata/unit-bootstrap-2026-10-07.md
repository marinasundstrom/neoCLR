# Canonical bootstrap Void lookup — 2026-10-07

Raven e141006f3 fixes namespace-dependent selection of the primitive bootstrap Void
instead of the explicit RuntimeUnitContract owner. Web contains generic fields such as
Promise<Result<System.Void, HttpError>>; importing the wrong nominal argument required
a competing retained-seed declaration and produced NEOMETA003 intermittently.

The binder now recognizes the exact bootstrap core identity and canonical System.Void
name under the explicit NeoCLR unit contract. Both qualified and unqualified syntax
normalize to the selected unit symbol. This requires a resolved selected owner and does
not redirect arbitrary lookalikes, change ordinary .NET policy, relax native validation
or reopen importer objects from emission. Unit values versus no-result returns retain
their existing representation. No metadata-format or public API change was required.

## Validation

Two deterministic C# regressions force bootstrap namespace lookup rather than depend on
hash or traversal order. Both failed before the correction; all 29 focused unit-contract
tests pass afterwards, including existing .NET behavior and pointer rules. Six fresh Web
compilations into separate output directories all succeed without retries. Temporary
stack-trace instrumentation used to locate the original failure is not in either repository.

The matched compiler was built from e141006f3's source changes (before its documentation
commit), with the explicit primitive Core, retained seed and separately compiled
Runtime/Data/Networking artifacts. The original six-build commands and the subsequent
consumer commands/hashes are in the [evidence file](unit-bootstrap-2026-10-07.json).
All five source-free consumers compile, verify and execute successfully with exact output,
exit 0 and empty execution stderr: deadline, headers, base-address, JSON-client and routes.
The debug routing run was slow but completed within the driver's existing timeout. This proves the reduced
bug and the selected build/execution gate, not that every bootstrap issue is resolved.

## Release ownership and next work

The separate bare-return Task<()> problem also reproduces on .NET. User direction assigns
that general fix and main/release integration to task 01a11579-77d7-7500-8561-950c20b1f6bc.
A main-based branch, patch and focused runtime tests were handed over; this task does not
claim that fix is merged or fully qualified. Native execution should consume its completed
revision next. The bootstrap still needs real native project-reference support, Platform
integration ownership, editor catalogs and collected shipping artifacts. The authorized
neoCLR main merge remains conditional on bootstrap readiness.
