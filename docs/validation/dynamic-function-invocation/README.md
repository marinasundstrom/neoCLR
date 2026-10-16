# Provisional dynamic Function invocation — 2026-10-16

Feature branch: `codex/function-kind-constraint`. The reserved native primitive is
`neoCLR.Runtime.InvokeFunction(System.Object, arrayref<System.Object>) -> System.Object`.
It takes an existing function reference, not a method descriptor or name. The
reference retains its bound receiver/closure. The Raven sample supplies a local
InternalCall declaration and facade; no permanent class-library API is introduced.

The interpreter lowers each invocation to ordinary checked IL. JIT/AOT use the same
adapter builder over selected closed function bindings, then compile ordinary
casts, unboxing, typed invocation and boxing. No runtime code generation or unchecked
native function-pointer call is introduced. Function Object views retain descriptor
identity; casts back validate the descriptor header before reading its target slot.

The initial contract requires a non-null Object array, including an empty array for
zero arguments. Arguments use exact boxed scalar types or assignable reference
views. No numeric coercion, optional arguments, ref/out copyback or arbitrary value
record/tuple/union marshalling is provided. Unsupported signatures fail explicitly.
No-result calls return null; reference results retain their identity; scalar results
are boxed. Invalid argument counts or casts fail before entering the target. Target
faults propagate through the existing runtime mechanism, without wrapping.

.NET comparison: [Delegate.DynamicInvoke](https://learn.microsoft.com/en-us/dotnet/api/system.delegate.dynamicinvoke?view=net-10.0)
provides the analogous boxed argument/result boundary. Our provisional divergence
uses structural function references, exact conversions, a required argument array,
and existing terminal faults instead of reflection exceptions/target wrappers.
These restrictions keep the primitive usable without defining a permanent binder
policy. The runtime owns this operation because checked function bindings, GC roots
and execution frames are runtime invariants; an FFI helper would expose those internals.

`Main.neoil` checks dynamic no-result mutation and integer invocation on the same
receiver, returning 42. The Raven fixture invokes a bound callback through a
sample-local `FunctionReflection.Invoke` facade. Both run in interpreter/AOT/JIT;
AOT uses UBSan/bounds and the native host checks root/heap cleanup. Negative cases
check wrong counts, wrong boxed types and target faults in all three backends.

```sh
python3 docs/validation/dynamic-function-invocation/run.py \
  --bundle target/variance-argument-sequence-20261016 \
  --output target/dynamic-function-20261016
# Repeat with --case wrong-count, wrong-box or target-fault and distinct outputs.
python3 docs/validation/dynamic-function-invocation/raven/run.py \
  --compiler /path/to/rvnc.dll --bundle /path/to/bundle --output /path/to/output
cargo test --test dynamic_function_invocation
```

Results retain binary/source/bundle hashes. The bundle is the existing
`variance-argument-sequence-20261016`; the runtime/compiler backend must include
this branch's primitive. No guest API snapshot inputs change in this slice: the
facade is sample-local and the host lowering helper has a manual API entry.
Function signature bounds and multicast/event policies remain outside this slice.
