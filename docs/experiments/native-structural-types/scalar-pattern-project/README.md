# Raven scalar-union pattern consumer

This feature-branch consumer checks a freshly emitted Raven program containing
`int | long | bool`, typed bindings, guarded arms, fallback arms of the same type,
wide Int64 and false Boolean payloads, and a result used after the match-arm join.
Six calls must produce the expected result; Main returns 42 only if all pass.
The native run must return status 0/result 42 with no fault and zero arena use.
The native object must have no undefined imports, including allocator imports.

The project explicitly uses `target/structural-integrated-libraries-v3`. This is a
retained matched library bundle, not a claim that libraries were rebuilt by the
consumer runner. Its original build labels and every input artifact's hash are
recorded in `../raven-scalar-pattern-results.json`. Raven was rebuilt at the
recorded compiler checkout revision before qualification.

From the neoCLR checkout, with the matching Raven feature checkout and the v3
bundle present:

```sh
export SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX26.2.sdk
python3 docs/experiments/native-structural-types/verify-raven-scalar-patterns.py \
  --compiler /path/to/raven/src/Raven.Compiler/bin/Debug/net11.0/rvnc.dll \
  --output target/raven-scalar-pattern-qualification
```

Use an unused output directory. The runner builds the Raven project, executes its
actual output in the interpreter, disassembles that artifact, compiles it with
closed-world AOT, links a C host and executes the native result. It requires two
`isinst` and two `unbox.any` instructions for each scalar in the fresh output.
The full native selection report and disassembly stay in the output directory;
the checked-in compact result records commands, hashes and outcome.

This establishes one real Raven-produced macOS ARM64 pattern consumer. It does
not qualify mixed reference/value unions, imported union-returning calls, JIT,
collector stress, whole-union boxing or arbitrary escaping `isinst` results. Zero
allocation in this workload is not a general throughput measurement.

## Separate parser observation

While preparing the consumer, direct `number < 0` guards followed by a later
`number > ...` guard produced parser errors in the freshly rebuilt compiler. The same comparison in a normal function body
compiles successfully.
Parenthesizing the first comparison did not resolve that probe. This is a deferred
general parser candidate, not a structural-union semantic limitation; it needs
independent reproduction and validation on Raven main before being carried to the
neoCLR line. This consumer uses named guard functions to exercise
guard execution without mixing that issue into native pattern qualification.

The retained `direct-guard-parser-probe.rvn.txt` is excluded from compilation.
Replacing Main.rvn with it reproduces the failed direct-guard form; keep it
separate from the passing qualification consumer.
