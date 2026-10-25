# Private concurrent native collector

This directory owns the nonmoving live-marking candidate, its mutation gate and
admitted native-provider projection. Ordinary hosting still uses synchronous GC.
These sources are integrated for development and qualification; their location does
not imply a supported public selector, a release package, or arbitrary native/shared
mutator admission.

`generate.py --output PATH` produces the owned candidate with scheduled epochs and
bounded synchronous native scopes. `native-providers.py --output PATH` produces
the matching admitted text-arena provider. Both read only runtime sources; no
benchmark output, retained executable or downloaded toolchain is needed to generate
C. Compile with the GC/memory headers, this directory's headers and `gate.c`.
`policy.json` names the selected owner-only configuration; `policy.py` rejects
missing/conflicting flags in the standard HTTP build. Independent research variants
remain explicitly separate. The compiler must use its qualified stack/byref policy
and admitted live-store/native-provider contracts.

The generation stages retain the existing scanner/remark projection over synchronous
`../native-gc.c`: `shadow.py`, `remark.py`, `live.py`, `automatic.py`, then
`generate.py`. This reuses the synchronous scanner and metadata semantics rather
than maintaining a copied collector. Exact text transformations remain a maintenance
constraint; source changes must pass generator and differential validation. Converting
these stages into explicit shared C hooks is a future structural change, not part
of this source move. Headers are implementation fragments, not public C APIs.

## Ownership and current limits

- One mutator owner per heap. Marker workers read admitted payloads through the
  gate and detached allocation identities; reclamation stays on the owner.
- Nonmoving storage, retained current roots and complete compiler/provider admission
  remain required. New objects and overwritten/inserted references retain the
  existing conservative epoch contract.
- The selected path requires `NEOCLR_GC_PROFILE`, pacing and POSIX pthread support.
  Windows candidate qualification, allocation-failure recovery for collector metadata,
  public host selection and release packaging remain open.
- Restricted shared-reader/registered-mutator code remains available for existing
  experiments, but is not enabled by `owned-nonmoving-v1`. The policy validator
  rejects shared-worker overrides.
- Historical benchmark paths are compatibility forwards, including headers and gate
  source. Fault-injection tools read the canonical implementation explicitly.

See [integration evidence](../../../../docs/validation/concurrent-native-integration-20261025/README.md),
[reachability and mutation contract](../../../../benchmarks/execution/concurrent-live/README.md),
[standard benchmark procedure](../../../../benchmarks/gc-suite/README.md), and
[compiler policy qualification](../../../../docs/validation/concurrent-stack-policy-20261025/README.md).
