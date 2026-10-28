# Dynamic-member class override contracts — 2026-10-28

DAM2006 reports unequal TypeInfo parameter requirements between a nongeneric nominal
reference-class override and its nearest inherited declaration. Missing, weaker and
stronger requirements all mismatch, following the same [.NET IL2092 rule](https://learn.microsoft.com/en-us/dotnet/core/deploying/trimming/trim-warnings/il2092)
used for the prior interface check. Abstract declarations participate.

The nearest-member lookup is extracted from inheritance verification and reused by
both verifier and diagnostic pass. It preserves exact name/signature matching and
ancestor order. The diagnostic does not root implementation bodies, widen retention
or change virtual dispatch. It checks the supplied load set, including unused types.

## Validation

- 17 `dynamic_access::tests` pass. New checks cover equal requirements, missing
  annotations on either side, weaker/stronger flags, nearest-ancestor selection,
  abstract declarations and absence of additional retention.
- Seven `class_interfaces` integration tests and one `inheritance::` unit test pass,
  checking existing inheritance and method-owner behavior after the lookup extraction.
- [Native matching consumer](../../experiments/dynamic-access/overrides/Main.rvn):
  Reader.Read and DefaultReader.Read both declare PublicConstructors. Build succeeds,
  no DAM2006, base-typed invocation prints `7`, exit 0.
- Mismatch control: only DefaultReader.Read changes to PublicProperties (512).
  Build succeeds with exactly one DAM2006 naming expected 3 and actual 512; dispatch
  still prints `7`, exit 0. Both builds infer only Model's constructor requirement,
  with no property requirement silently added. The method performs no reflection,
  isolating contract diagnostics from runtime missing-retention faults.
- Existing DAM2004 remains: dynamic implementation-body traversal is a separate gap.
  No other SDK override mismatch is reported by these loaded-library consumers.
- API reference fingerprint check passes. Public API signatures, compiler attribute
  transport and Runtime Contract settings are unchanged; no snapshot rebuild required.

[Recorded results](results.json) contain diagnostics, retained type rows, executable
hashes and output. Inputs are compiler bundle `ed1d08def`, the already qualified
annotated libraries and a stable debug-backend copy on macOS ARM64. The HTTP native
service profile is used; the app performs no networking. No website build/publication,
Windows/JIT execution or performance qualification is claimed.

Reproduce using the [consumer instructions](../../experiments/dynamic-access/README.md)
with `--project docs/experiments/dynamic-access/overrides/App.rvnproj`. Change only
the override's annotation for the mismatch control. Focused commands:

```sh
cargo test -q --lib dynamic_access::tests
cargo test -q --test class_interfaces
cargo test -q --lib inheritance::
python3 scripts/build-api-docs.py --check
```

Generic ancestry/methods, value-type overrides and implicit runtime-owned Object
adaptations remain outside this check. The latter currently carry no TypeInfo
parameter contracts. Additional metadata traversal has a build-time cost; no speedup
is claimed. See [current contract and gaps](../../dynamically-accessed-members.md).
