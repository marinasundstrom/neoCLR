# Signed range behavior tests

Fourteen discovered `NeoClr.Testing` cases replace the active numeric-exit probe
in `docs/validation/range-iteration/Main.rvn`. That original source and its reports
remain historical evidence. The current semantic contract is unchanged: signed
progression stops before leaving its domain, bounds/step evaluate once in order,
and continue/break preserve advancement. The original compiler qualification also
compares the .NET target; this migration changes tests only.

Run all three modes with a matching development bundle:

```sh
python3 scripts/validate-native-range-iteration.py --bundle /absolute/bundle --output /fresh/output
```

Or use `scripts/test-runtime-library.py --suite range-iteration` and explicit
`--mode interpreter --mode jit --mode native`. Named test output and the common
runner report replace the single PASS line and numeric failure codes. The legacy
command still writes `results.json`, now identical to the common `report.json`.
Python owns artifact integrity and process execution; Raven owns behavior assertions.
