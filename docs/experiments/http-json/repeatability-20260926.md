# HTTP/JSON repeatability follow-up — 2026-09-26

This checks the generic-helper application implemented in neoCLR `afcc8c3d`, after
the roadmap commit `73e27938`. Raven's generic method-group fix is on main at
`13b9105d8` and its neoCLR branch at `56083626e`. The test bundle's compiler matches
the local Raven Debug/net11.0 compiler byte-for-byte. This is a development bundle,
not a refreshed installed SDK or release qualification.

## Reproduction

Use matching compiler, bridge, core references, library and runtime artifacts:

```sh
python3 docs/experiments/http-json/verify.py \
  --mapped --case pair --repeat 3 \
  --toolchain-root /absolute/path/to/matching/development/bundle \
  --runner /absolute/path/to/matching/measure_async
```

Use `--case client --repeat 2` for the neoCLR client against Python and
`--case server` for the twelve independent requests to the neoCLR server. Builds
finish before the selected checks begin; each repetition starts fresh peers.
Existing heap/instruction/process guards and socket/HTTP deadlines are unchanged.
These checks stop on failure rather than retrying until a pass masks the failure.

## Observations

- The three-run managed-pair attempt failed on **iteration one** with TimedOut.
  Client assembly/loading/verification took 4,066/2,845/678 ms; guest execution
  reported 8,090 ms before the fault. These are whole-program phases, not DNS,
  connect or transfer timings. They do not identify which deadline fired. The
  verifier terminated the server during cleanup, so no final server heap result
  is available. Iterations two and three did not run.
- The neoCLR client against Python passed twice, with execution times 11,614 and
  13,363 ms. Each run allocated 789 objects, performed 13 collections and finished
  with zero live objects. These execution totals include both requests and JSON
  processing; they are not individual request latency measurements.
- The neoCLR server passes all twelve independent request cases, including malformed
  JSON, invalid UTF-8, missing/null fields and unknown routes. It allocates 3,786
  objects, performs 53 collections and finishes with zero live objects. Total guest
  execution is 86,293 ms for the complete batch, not one request.
- A process snapshot after the pair failure showed another Raven compiler process
  using substantial CPU alongside other background activity. No competing processes
  were terminated. This was not a controlled quiet-machine run; load sensitivity
  is a hypothesis, not an established cause of this failure.
- A separate minimal `func Main() { missing() }` build through the same bundle's
  `NeoCLRImport` target exited zero and emitted a Main body ending in `ret`, with
  no call. This reproduces the unresolved-call diagnostic defect independently
  of the repaired generic method-group issue. The existing full MSBuild acceptance
  test already covers that negative case; it was not rerun in full here.

Artifact hashes and measured results are retained in the
[evidence snapshot](repeatability-20260926.json). Raw process logs were captured
locally; the snapshot preserves the relevant outcome without relying on temporary
paths as reproducible build inputs.

## Next bounded investigation

Retain the managed pair as a failing acceptance case. Capture per-operation timing
for DNS, connection, request write, response read and server handling/completion
before selecting a deadline, scheduling or interpreter fix. Compare the same pinned
artifacts in a quiet run and a deliberately specified load case; do not label a
retry as a repair. Independent peer success narrows the experiment but does not
exonerate either managed peer or establish performance guarantees.

Track the unresolved-call compiler defect separately. Neither a passing generic
helper fixture nor these HTTP checks establish full compiler/SDK acceptance.
