# Configuration reload and native defaults — 2026-10-30

Qualified on macOS ARM64 with Xcode 26.2. Windows uses the existing shared watcher
adapter but was not executed in this slice; other platform watcher support remains
subject to the existing platform gates.

- Real file lifecycle test passes in native AOT and interpreter execution. It verifies
  opt-in watching, default-disabled behavior, named optional arguments across a library
  boundary, live sections through DI, malformed-input preservation/error reporting,
  editor-style atomic replacement/recovery, required deletion, optional creation and
  deletion, and disposal. The async test returns Task<Result<(), TestFailure>>.
- Metadata SDK tests pass native/CLI/projection round trips, scalar values, explicit
  null versus absence, negative-zero bits, snapshot immutability, clearing defaults,
  empty/default alignment and malformed metadata rejection.
- Three runtime metadata tests pass, including instance receiver exclusion and
  unchanged fixed-arity execution. A separately compiled constructor/method consumer
  exits zero using imported integer, bool and string defaults and named arguments.
- ParameterDefaultCapabilityTests pass for int/bool/string with explicit target gating.
  These three test cases were run in a small executable harness referencing the built
  compiler. The broad Raven test project could not run in the isolated checkout because
  unrelated generated Macro/Core/analyzer fixtures were not built; its prerequisite
  build was stopped during Macros. This is not a full Raven-suite pass.
- The integrated compiler build against the current metadata SDK passes. Native API
  rendering and the API snapshot check pass; no full website build was needed.
- The full extensions sample is built natively and tested in one-shot and --watch modes.
  Its live mode reads a changed name through the same DI-resolved GreetingService.

The sample exposed a generated ReflectionAssignable body exceeding the 8,192-op
bound. Codegen now shares exactly equal target-check sets across source descriptors;
identity fast paths, distinct relationships and invalid-token handling remain intact.
The method bound is unchanged. Focused relationship tests and the sample qualify this
necessary supporting fix; no general performance improvement is claimed.

## Reproduction

With a matching bundle containing the rebuilt Neo.Configuration.Json and compiler:

```sh
python3 scripts/validate-configuration-reload.py --bundle /path/to/bundle \
  --runtime /path/to/neoclr --aot /path/to/neoclr-codegen --output target/reload-check
cargo test --test parameter_defaults
cargo test -p neoclr-codegen relationship
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- --parameter-defaults
python3 scripts/build-api-docs.py --check
```

Library.rvn and Consumer.rvn are the small imported-default fixture. Compile the
library using `rvnc neoclr --library --native-core-reference System.Runtime.dll
--object-library System.Runtime -o Library.dll Library.rvn`, then the consumer with
that native core and `--reference Library.dll`. Execute with the matching runtime
seed and both modules. Imported values are inserted by the compiler, not the runtime.

The compiler change is committed on Raven codex/neoclr as 551883cf8. The frozen native
qualification compiler uses base 555c0d7fb plus that default-argument patch, to isolate
concurrent function-dependent work; the integrated compiler was also built against
4b33ecd0e plus the default patch. Unaffected core libraries come from the previously
qualified filewatch-union bundle; only the JSON configuration assembly was rebuilt.
Runtime API overload cleanup and arbitrary struct defaults remain the next slice.
Native async Result entry-point adaptation was discovered during sample refinement
and is intentionally a separate following slice.
