# Native Result entrypoint qualification — 2026-10-30

Development qualification on macOS ARM64. Windows/Linux execution is not claimed.
The compiler base is Raven `551883cf8` on `codex/neoclr`, plus entry adaptation committed as `1a759015f`.
The native runtime/codegen are the isolated, validated Configuration/defaults build
recorded in [the preceding receipt](../configuration-reload-20261030/README.md).
No runtime instruction or metadata change is required for this slice.

## Checks

- Build the integrated Raven compiler against the repository metadata SDK.
- Run `scripts/validate-result-entrypoints.py` against the matching bundle. Check
  exact exit status, empty stdout and exact stderr independently in native and
  interpreter execution. Fixtures cover synchronous/asynchronous integer/unit
  Results, null errors, user-defined implicit conversion, Error-interface propagation,
  class structural unions, platform error unions and scalar/reference alternatives.
- Run the native integrated extensions sample once and with `--watch`; edit its
  isolated appsettings.json during the 30-second timer loop. The DI consumer sees
  Reloaded, and the Result-returning Main completes with status zero and no stderr.
- Review website source links and `git diff --check`; no full website rebuild is
  needed for the entrypoint prose. No public runtime API signature changes here.

`results.json` consolidates successful cases from focused runs. The initial nonunion
entry checks and integrated sample are reused after the union-only adapter change;
ordinary CompleteInt/CompleteUnit/Report behavior is unchanged. Intermediate probes
caught whole-union boxing and bare-generic composition restrictions; the final adapter
uses concrete alternative tests and preserves those broader restrictions. The expected
platform numeric-error display was corrected to its normal qualified ToString value.

`provenance.json` records source and executable hashes; `sample.json` retains observed
one-shot and reload output. Reproduce with the command in
[the entry contract](../../native-result-entrypoints.md#qualification).
