# Function-kind constraint groundwork — 2026-10-16

Feature branch: `codex/function-kind-constraint`. This is native runtime and host
SDK groundwork, not enabled Raven syntax or a function-list/event API.

The `.constraint F func` IL restriction maps to native `FunctionType`. It accepts
structural function signatures, preserves their exact parameter/result signature,
and rejects nominal types, primitives and containers of functions. The initial
profile requires the category constraint alone. No nominal interface, delegate
ancestry, dynamic invocation operation or event metadata is introduced.

The retained consumer stores a typed callback in `Signal<F>`, retrieves it and uses
the existing typed invocation operation. It returns 42 in interpreter/AOT/JIT.
AOT uses UBSan/bounds; the native hosts check cleanup. Source metadata retains the
constraint after specialization. [Results](results.json) record exact source,
binary and bundle hashes on macOS ARM64.

```sh
python3 docs/validation/function-kind-constraint/run.py \
  --bundle target/variance-argument-sequence-20261016 \
  --output target/function-kind-native-20261016
cargo test --test function_kind_constraints
cargo test -p neoclr-codegen --lib function_kind_constraint
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests \
  -p:WarningLevel=0 -- --function-kind-constraint
```

Focused SDK tests cover authored and imported constraints, primitive/nominal
rejection (including a class declaring Invoke), constraint combination rejection,
native round-trip introspection, and CLI-export rejection. The full host SDK suite
passes 183/183 checks. API fingerprint validation passes; guest API inputs do not
change.

Raven syntax remains pending structural-versus-nominal function symbol identity
through the compiler bridge. An experimental global symbol substitution broke
RuntimeAwait/RuntimeGenerator binding and was not integrated. The intended spelling
is `where T: func`, only for structural functions in neoCLR. Invocation APIs are
postponed; the subsequently proposed `Reflection.InvokeFunction(functionObj, args)`
is a candidate only. See [direction](../../proposals/structural-types.md#20-generic-constraints-on-function-types).


## Raven source and separate provider

`raven/Library.rvn` defines `FunctionSlot<T> where T: func`. The consumer stores a
bound `Incrementer.Apply` function, retrieves it as `(int) -> int` and invokes it,
returning 42. The receiver's state supplies the increment. `Rejected.rvn` verifies
that an integer cannot be substituted for the constrained parameter.

Run `python3 raven/run.py --compiler /path/to/rvnc.dll --bundle /path/to/bundle
--output /path/to/output --imported` as one command. The retained bundle is
`target/variance-argument-sequence-20261016`. This is a test holder, not an observer
or event API. Native method constraints and signature bounds remain separate gaps.
