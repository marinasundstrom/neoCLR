# Provisional dynamic Function invocation — 2026-10-16

Feature branch: `codex/function-kind-constraint`. The reserved native primitive is
`neoCLR.Runtime.InvokeFunction(System.Object, arrayref<System.Object>) -> System.Object`.
It takes an existing function reference, not a method descriptor or name. The
reference retains its bound receiver/closure. The Raven sample now imports the provisional class-library
`System.Runtime.Reflection.FunctionReflection` API; it has no local facade or
InternalCall declaration.

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
and Result-based validation errors instead of reflection exceptions. Target faults
still propagate without wrapping.
These restrictions keep the primitive usable without defining a permanent binder
policy. The runtime owns this operation because checked function bindings, GC roots
and execution frames are runtime invariants; an FFI helper would expose those internals.

`Main.neoil` checks dynamic no-result mutation and integer invocation on the same
receiver, returning 42. The Raven fixture invokes a bound callback through a
public `FunctionReflection.Invoke` API, checks its runtime type through
Object.GetType against a structural typeof token, and dynamically passes a function
as an argument/result through a second callback before invoking it. Both run in interpreter/AOT/JIT;
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

Results retain binary/source/bundle hashes. The raw primitive fixtures retain their
original `variance-argument-sequence-20261016` evidence. The public Raven consumer
uses the fresh `function-reflection-library-validated-20261016` five-library bundle,
including the API and native service declarations. Its matching compiler includes
reserved bootstrap RuntimeServices Func/Action parameter projection; ordinary CLI
delegates remain nominal. Guest reference/type selection/XML and native API snapshot
are refreshed for the new public class.
Function signature bounds and multicast/event policies remain outside this slice.


The follow-up native reflection slice retains one token per selected function shape,
not a fabricated nominal declaration. ObjectTypeHandle checks the descriptor header
and bound target identity, then returns that exact signature token. Focused codegen
checks cover deduplication and malformed service contracts; the source sample
qualifies runtime identity, nested function values and execution across all backends.
Raven's host SDK permits native Function ldtoken and rejects its CLI projection.
Detailed method-snapshot/native generic reflection coverage remains separate.


The Raven sample additionally qualifies source-level function patterns:
`obj is () -> T` in a generic method, `(int) -> int captured` with receiver-preserving
invocation, and `() -> T callback => callback()` in a generic match arm. Negative
checks include null, a nominal object, and different parameter or result types.
The corresponding shared parser fix is on Raven main (`0c8cda8dc`), where 40 focused
parser/runtime tests pass using ordinary .NET delegates. Native validation uses the
feature-branch compiler and the same bundle/profile above, returning 42 in all three
backends; this does not introduce function variance or resume other structural work.


Reflection/type-pattern agreement is also checked through
`typeof((int) -> int).IsInstanceOfType(value)` and the generic
`typeof(() -> T).IsInstanceOfType(obj)`. Positive bound functions and negative null,
nominal-object, parameter-mismatch and return-mismatch cases are retained. The
consumer explicitly compares the generic reflection result with the language `is`
result. Both use existing runtime relationship rules; no new API or compiler
lowering is introduced by this validation slice.


The public consumer verifies `Error(InvalidArguments)` for wrong counts, wrong boxes
and null in a scalar slot; `Error(InvalidReceiver)` for null/nominal receivers; and
`Ok(null)` for a valid null reference argument/result. The focused runtime test also
proves `InvokeFunctionCheck` never enters the target, even for valid inputs. Existing
raw-primitive target-fault/no-result tests remain applicable. Unsupported marshalling
returns `UnsupportedType`; generic overloads are not introduced.


Run the public target-fault consumer with `raven/run.py --case target-fault` and a
fresh output directory. It verifies that a fault raised inside the function propagates
through the Result-returning facade in interpreter/JIT/AOT, rather than becoming Ok
or Error. The positive consumer remains the default case.
