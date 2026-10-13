# Rooted reference-array qualification

Generated GC calls select rooted allocation for string/function arrays, nominal
reference arrays and reserved record arrays. Native/non-GC entry points remain
noncollecting. Construction initializes all headers, nulls and flags before
publication, with no intervening collection. The private host must match generated
symbols; no managed API or Raven contract changes.

The local sanitized gate passes 25 GC/root executions. The pressure fixture checks
live exhaustion, unchanged outputs, invalid sizes, missing/foreign contexts,
referent retention after removing caller roots, empty arrays and final reclamation.
Compiled array tests cover callback ownership, nominal/interface dispatch, record
snapshots and nested references, primitive controls, reflection and fault parity.
See the retained log for the exact tests. Relocated package checks pass.

[Hosted run 38446608133](https://github.com/marinasundstrom/neoCLR/actions/runs/38446608133) passes at `10b8ac6225b7d4b4dd06d7f16978f07d585ce84a`: 25 GC/root executions plus sockets and relocated-package checks on macOS ARM64 and Windows x64, with macOS profiler attachment. This is not a fresh managed Windows/JIT/HTTP or Windows ARM64 matrix. These checks do not claim a new
HTTP performance result or enable general collection pacing.
