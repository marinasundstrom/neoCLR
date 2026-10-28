# Interface dynamic-member contracts — 2026-10-28

The native backend now emits DAM2005 when corresponding parameter requirements
differ between a concrete nongeneric nominal class implementation and a nongeneric
interface method. It resolves implicit, explicit and inherited mappings using the
existing interface implementation resolver and original source identities. It does
not add implementation bodies to retention, strengthen requirements or suppress
other flow diagnostics.

The exact-match rule follows [.NET IL2092](https://learn.microsoft.com/en-us/dotnet/core/deploying/trimming/trim-warnings/il2092).
This bounded implementation checks the supplied load set, including unused classes;
it does not claim full .NET override/generic analysis. See [contract and limits](../../dynamically-accessed-members.md).

## Qualification

- 15 focused `dynamic_access::tests` pass. New checks cover equal requirements,
  missing annotations on either side, weaker/stronger flags, explicit implementation
  identities and inherited interfaces. Contract checks do not retain unused bodies.
- Native matching consumer: [Reader/DefaultReader](../../experiments/dynamic-access/contracts/Main.rvn)
  both require PublicConstructors. Build succeeds, no DAM2005, prints `7`, exit 0.
- Native mismatch control: change only DefaultReader's parameter requirement to
  PublicProperties (512), while Reader retains PublicConstructors (3). Build succeeds
  with exactly one DAM2005 describing the two requirements; prints `7`, exit 0.
- Both builds infer exactly the same Model constructor requirement. The sample method
  performs no reflection, isolating contract diagnostics from runtime failure.
  Existing DAM2004 remains because implementation-body traversal is a separate gap.
- Both use compiler bundle `ed1d08def`, the previously qualified annotated libraries
  and a stable copy of the debug backend on macOS ARM64. The same loaded SDK does not
  produce spurious DAM2005 warnings. The native HTTP service profile is used; the app
  performs no networking. [Results](results.json) preserve hashes and diagnostics.
- `python3 scripts/build-api-docs.py --check` passes. No public runtime API signatures
  or attribute transport changed, so no snapshot rebuild is needed. No website
  build, publication, Windows execution, JIT execution or performance claim.

Reproduce using the [consumer instructions](../../experiments/dynamic-access/README.md)
with `--project docs/experiments/dynamic-access/contracts/App.rvnproj`. Create the
mismatch control by changing the annotation on DefaultReader.Read only. Run focused
analysis tests with `cargo test -q --lib dynamic_access::tests`.

## Remaining boundaries

Generic classes/interfaces/methods, value-type implementations, abstract class
mappings and standalone class overrides are not covered. The pass is diagnostic:
DAM2005 does not prove that the implementation has sufficient retention. Reusing
mapping resolution adds metadata traversal and build-time cost; no speedup is claimed.
Runtime Contract configuration and compiler emission are unchanged.
