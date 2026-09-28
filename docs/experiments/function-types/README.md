# Function migration evidence

Development branch work, 2026-09-28. The [design](../../function-types.md) and
[tracker](../../tracking/runtime-language.md#function-types-and-objects--2026-09-28)
own scope and completion. Named function types remain possible future work.

Native `tests/function_types.rs` exercises structural binding and signature
identity without a nominal callable declaration, generic substitution and higher-order
storage, artifact round trips, retained receivers, frame-receiver rejection,
no-result invocation and nominal classification. The no-result case covers both
call and callvirt; the latter exposed and corrected a verifier stack-result bug.

`Main.rvn` checks the public descriptor split: nominal declarations implement
NominalTypeInfo and MemberInfo, arrays implement only TypeInfo, common display and
type queries remain available, equality works across providers and object discovery
and collection preserve descriptors. A constructed Option<int[]> keeps a nominal
outer descriptor and a structural generic-argument descriptor. `verify.py` also rejects declaration-only
members on a plain TypeInfo parameter.

Run against a freshly built bridge, generated library and matching reference:

```sh
python3 docs/experiments/function-types/verify.py \
  --runtime target/debug/neoclr \
  --bridge docs/experiments/raven-target/bin/Debug/net11.0/Probe.dll \
  --system target/function-types/System.neoil \
  --reference target/function-types/NeoCLR.CoreProbe.dll
```

The descriptor consumer compiles, passes native verification, and executes with
`Nominal and structural descriptors passed`. Raven callbacks import to structural
Function signatures. Native legacy declarations, instructions and serialized
representations are now rejected. Bounded Object/introspection limitations are
documented separately from the completed callable migration.

## Deferred general compiler candidate

With Raven `5dde32fbf` on its isolated `neoclr` branch, placing the new provider's
expression-bodied Module getter before its Handle field emitted a null-returning
body despite the source calling RuntimeServices.TypeModule. The importer rejected
that body as `expected System.Introspection.ModuleInfo, found FaultNull`.
Placing declaration getters after the storage and other methods emitted the expected
field read and service call, and the same consumer then passed. This is a source-order
observation, not a diagnosed root cause or a compiler fix. The current library uses
that validated ordering. Reduce and validate this candidate on ordinary CLI metadata
in a main-based Raven feature branch before integration; do not copy target policy
or fixtures into Raven main.

Consumer migration also compiles the assembly/nested-type/introspection/array
examples and the attribute, union-construction and route-mapper fixtures against
the matching reference. The older introspection-object and introspection-flags fixtures still hit the
existing nonpublic multi-argument conversion admission limit. An exploratory
public-helper compile of the flags fixture additionally hit unsupported System.Type[]
attribute metadata. These broader fixtures are not counted as passing evidence;
the focused descriptor consumer uses a public assertion helper and passes.

Validation for the descriptor checkpoint: 76 native Function/callback/reflection
tests pass across function_types, delegates, reflection, reflection_arrays,
reflection_construction, reflection_members, reflection_properties and
reflection_hierarchy. Full library bootstrap regeneration and source/output hash
validation pass, as does the RavenDoc reference snapshot check. The migrated
attribute-introspection consumer executes successfully, including nominal type
attributes and union-case discovery. No website build or publication was performed.

The migrated reflection, flags and type-preview consumers execute, and the old
System.Reflection descriptor namespace is rejected by the compiler. Raven's
matching integration note is committed on its isolated neoclr branch as `0218f75af`.

## Callable transport migration checkpoint

Raven source callbacks now use function syntax. Its isolated neoCLR compiler policy
selects inhabited Func transport for unit results, matching generic `() -> T`
instantiated with unit. The importer converts CLI Func/Action carriers into native
structural `fn<...>` signatures; a CLI no-result target is adapted to return the
inhabited unit, and imported Action.Invoke discards that value for CLI stack balance.
CLI carrier names are transport details, not runtime nominal type declarations.
The source library Func declarations and their generated slice are removed.
The source callback consumer executes, including structural identity, generic unit
results, mutable captures, collection retention and an extension on `(int) -> int`.
Full library regeneration and async consumer validation use the compiler fix below. Ordinary CLR target selection remains unchanged.

## General function-construction compiler fix

The async fixture exposed a null Promise callback list. An independent ordinary
.NET reduction on Raven main-based branch `fix/function-type-construction` found
that `List<() -> ()>()` produced an invalid semantic operation with no diagnostic
and silently omitted construction during emission. The expression-side type binder
was missing FunctionTypeSyntax handling. Commit `e316703ca` fixes that binding and
is integrated into the isolated target as `6b5418e57`; it was independently fast-forwarded into main after a clean-worktree check; the
experimental target branch was not merged wholesale.

Fifteen focused compiler tests pass afterward; three function-signature constructor
cases failed before the fix, while the non-function control passed. Tests check
ObjectCreation operations, initialized runtime state and invalid-type diagnostics.
The source-order getter observation above remains a separate deferred candidate.

With the fixed compiler, the descriptor and callback/extension consumers pass.
The async consumer verifies and executes with `42`, `True`, `41`, `21`, `7`,
covering shared captures, unit callbacks and nested Task results. The validation
script's initially lowercase Boolean expectation was corrected to the observed
runtime display contract. Full source/bootstrap hash, API snapshot and source
coverage audit checks pass. No website build or publication was needed.

## Final structural replacement validation

The removal checkpoint passes 157 tests across 16 selected native suites: Function
shapes/objects/receivers, library loading, closures, collection/query callbacks,
archived frontend consumers and workers. Separate debugger, generic array and
no-result suites pass; six generated task-queue atomicity checks and three Function
GC/shared-work checks pass. Thirteen external-I/O GC probe checks also pass with
Function carriers, along with the TCP completion consumer exercising both VM
collection paths and an empty-queue wait. Optimized binaries were used for the large full-library consumer
runs; debug loading was slow, and an initial linker run required selecting the SDK
from the active Xcode installation instead of a mismatched CommandLineTools SDK.

The final verify.py run passes all twelve checks: descriptor selection, Function
callbacks/extensions, async/shared capture/unit/nested Tasks, renamed comparer
adapters, named-delegate rejection and seven nominal-only member rejections.
Full library regeneration, bootstrap hashes, API reference/snapshot checks and
source coverage inventory/audit pass. Named function types, Object conversion,
Function parameter/result descriptor APIs and general structural member enumeration
remained outside that removal checkpoint. No website build or publication was performed.

## Signature descriptors and OfType validation — 2026-09-28

The follow-up implements FunctionTypeInfo.Parameters, ReturnType and InvokeMethod,
TypeInfo.IsFunctionType, and GetMethods discovery of the same synthesized public
instance Invoke. It preserves shape-owned parameter types/modes, equality, member
filters and absent declaration metadata. Named function inheritance and a generalized
function-info interface are not introduced. OfType<U>() lazily filters and narrows
Iterable<T>, including module.GetTypes().OfType<NominalTypeInfo>().

The updated `verify.py` passes thirteen checks. Its descriptor case covers direct
Parameters/ReturnType projection, zero-parameter unit functions, agreement with
InvokeMethod/GetMethods, absent metadata/attributes, filtering, owner/equality/hash,
generic-argument provider selection, and UnboundMetadata for dynamic reflection
invocation. The OfType case covers mixed/null/boxed values, numeric non-coercion,
value inputs, empty input, descriptor narrowing, module discovery, deferred evaluation,
order, repeated enumeration, and disposal at early termination and exhaustion.

Sixty selected native cases pass across function_types, reflection,
reflection_members, query_terminals and raven_reflection. Four older Raven reflection
cases initially used the removed common FullName/token contract; the migrated cases
pass focused reruns. Unaffected passing cases were not rerun. The six migrated
introspection-object, reflection-members, reflection-execution, attribute-introspection,
union-construction and runtime-route-mapper consumers compile, verify and execute.
The route mapper uses its existing 100,000,000-instruction runner budget; the ordinary
CLI default was insufficient for its complete scenario.

Full library regeneration and bootstrap hashes, API reference/source fingerprints,
source inventory and coverage audit pass. Generic OfType provider storage uses
explicit fields because the current importer rejects the private var/val storage
form; this bounded constraint is recorded in the query design. No website build or
publication is needed for this change.

The three updated documentation samples, library-assembly-info,
library-introspection-tour and library-reflection, also compile, verify and execute.

## Transitional target inspection

Function objects expose a synthesized Function: MethodInfo property. Native tests
cover same-target equality, distinct generic instantiations, module-function targets,
null property access and reachability (inspection does not count as invocation).
The Raven callback consumer exercises the property, target descriptor equality and
shape-based Function equality; the descriptor consumer checks property/getter
membership and filtering. FunctionInfo remains deferred. The compiler reference
projects Function onto CLI callback transport types, and the importer lowers that
access to the structural get_Function contract and callable comparisons to native
value equality. No Runtime Contract switch or general Raven callable hierarchy is
introduced. Use matching regenerated runtime/reference artifacts.
