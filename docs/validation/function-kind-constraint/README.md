# Function-kind constraint groundwork — 2026-10-16

Feature branch: `codex/function-kind-constraint`. This is native runtime and host
SDK groundwork plus the separately qualified Raven class-constraint slice below. It is not a function-list/event API.

The `.constraint F func` IL restriction maps to native `FunctionType`. It accepts
structural function signatures, preserves their exact parameter/result signature,
and rejects nominal types, primitives and containers of functions. The initial
profile requires the category constraint alone. No nominal interface, delegate
ancestry, dynamic invocation operation or event metadata is introduced.

The retained consumer stores a typed callback in `Signal<F>`, retrieves it and uses
the existing typed invocation operation after forwarding through `Keep<F>` and
`Forward<F>`, each with a method-level `func` constraint. It returns 42 in interpreter/AOT/JIT.
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

Raven now distinguishes native function symbols from nominal delegates and supports
`where T: func` on generic classes. Invocation APIs remain postponed; the proposed
`Reflection.InvokeFunction(functionObj, args)` is a candidate only. See
[direction](../../proposals/structural-types.md#20-generic-constraints-on-function-types).

Native AOT/JIT admit function-kind method constraints only. Closed method arguments
are checked before specialization reuse; proven constraints are removed from the
executable clone while the source declaration remains intact. Other method bound
categories stay gated. The focused codegen regression verifies forwarding, source
preservation and rejection of an invalid integer construction, including a call
that bypasses assembler validation.


## Raven source and separate provider

`raven/Library.rvn` defines `FunctionSlot<T> where T: func`. The consumer stores a
bound `Incrementer.Apply` function, retrieves it as `(int) -> int` and invokes it,
returning 42. The receiver's state supplies the increment. `Rejected.rvn` verifies
that an integer cannot be substituted for the constrained class or method parameter,
and that unconstrained method forwarding rejects. The provider also exports
`Keep<F> where F: func`; the consumer forwards its callback through a constrained
method and the external provider before invoking the concrete signature.

Run `python3 raven/run.py --compiler /path/to/rvnc.dll --bundle /path/to/bundle
--output /path/to/output --imported` as one command. The retained bundle is
`target/variance-argument-sequence-20261016`. This is a test holder, not an observer
or event API. Method emission/import and interpreter/AOT/JIT execution are qualified in
[raven/results.json](raven/results.json). Signature bounds remain a separate gap.
