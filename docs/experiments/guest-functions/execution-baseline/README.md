# Module-function execution baseline — 2026-10-12

Follow-up to [issue #27](https://github.com/marinasundstrom/neoCLR/issues/27), after
website qualification. The consumer compiles with native Raven `c1a938ccd` and
uses the coherent shared-GC development bundle. Reports pin inputs and binaries.

- [Interpreter](interpreter.json): passes, including absent declaring type,
  namespace ownership, parameter descriptors, TestAttribute identity and fixed/named
  descriptions, without executing discovered bodies; post-run GC reports zero live objects.
- [JIT](jit.json): fails before execution at
  `neoCLR.Runtime.TypeDeclaringType: unsupported value member contract`.
- [AOT compilation](aot-compile.json): the same consumer fails at the same service.
  This is object compilation evidence only; no native link or execution is claimed.

The previous interpreter harness relied on the old execution default. It now selects
`--execution interpreter` explicitly; `--execution jit` runs the same contract and
retains its failed report. An optional `--jit-compiler` pins the backend under test.
Native failures are not accepted as passing validation or silently interpreted.

Next bounded implementation: retain and bind the declaring-type query reached by
this consumer, then rerun to expose the next actual dependency. Module-function
catalog retention/enumeration and ownerless attribute support still need native
implementation. Generic signatures and invocation remain separately tracked in #27.
Do not infer native discovery support from existing host-side recipe helpers.

```sh
python3 docs/experiments/guest-functions/verify.py \
  --bundle /path/to/jit-bundle --compiler /path/to/rvnc.dll \
  --runner /path/to/jit-bundle/bin/neoclr --execution interpreter \
  --output target/guest-functions-interpreter

# Currently fails with the retained diagnostic above:
python3 docs/experiments/guest-functions/verify.py \
  --bundle /path/to/jit-bundle --compiler /path/to/rvnc.dll \
  --runner /path/to/jit-bundle/bin/neoclr --execution jit \
  --jit-compiler /path/to/neoclr-aot-poc --output target/guest-functions-jit
```
