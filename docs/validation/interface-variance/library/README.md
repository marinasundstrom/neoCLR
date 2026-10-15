# Annotated class-library variance — 2026-10-16

The library declares seven producer/read-only interfaces `out T` and four
comparison interfaces `in T`. Concrete classes and mutable contracts remain
invariant. `Main.rvn` exercises imported Collection/Sequence/Iterable/Iterator,
Comparer/EqualityComparer/ComparableTo/EquatableTo, ConvertibleInto and async
source/iterator reference views. Async acquisition tests conversion and identity;
it does not perform asynchronous advancement or claim scheduler qualification.

`MetadataCheck.cs` checks all eleven annotations and invariant mutable/value-wrapper
contracts in the built runtime image. `Rejected.rvn` must produce five RAV1503
errors: reverse producer/consumer conversions, mutable alias widening, implicit
boxing inside variance and Sliceable result widening.

Reproduce with a freshly built native bundle and matching compiler/runtime:

```sh
python3 docs/validation/interface-variance/library/run.py \
  --compiler /absolute/path/to/rvnc.dll \
  --bundle /absolute/path/to/bundle \
  --output /absolute/path/to/fresh-output
```

The runner validates metadata and rejection diagnostics, then requires interpreter,
macOS arm64 AOT and managed JIT to return 42. The native host checks faults, root
cleanup and complete collection; AOT also uses undefined/bounds sanitizers.
`results.json` retains exact commands and artifact/log hashes. Qualification uses
Raven `1e6771d5f` and `target/variance-libraries-20261016-final`.

This uncovered and fixed private nominal rewrites dropping native variance proofs
(neoCLR `229cc95ff`) and general Raven variance incorrectly accepting boxing
(main `aefd5d4ed`, deliberately carried as `1e6771d5f`). Thirteen focused semantic
tests pass on both Raven branches. The native rewrite regression passes.

Reference views do not allocate adapters or convert Result payloads. Native intrinsic
String widening remains a materialization gap, and open-definition native reflection
remains a separate coverage gap. Neither changes the requirement to retain generic
metadata or resumes structural work.
