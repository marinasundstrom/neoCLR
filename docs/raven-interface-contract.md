# Raven-facing library interface contract

Recorded 2026-09-12. This is the contract and admission-probe slice toward the
[shared runtime library demo](raven-target-experiment.md#desired-demo-the-existing-runtime-library-through-raven-2026-09-12).
The candidate collection declarations compile through Raven. An adapted runtime library
profile now executes in neoCLR tests; Raven collection import/execution is not yet implemented. Nominal interface dispatch, including closed generic classes, now has runtime support below.
The existing Neo interface/collection path remains available and unchanged.

## The library surface to preserve

Use the existing names and relationships, without an `I` naming prefix:

| Contract | Candidate Raven-facing members |
| --- | --- |
| Disposable | Dispose() with no result |
| Iterable<T> | GetIterator() returning Iterator<T> |
| Iterator<T> : Disposable | MoveNext() returning Boolean; Current returning T |
| List<T> : Iterable<T> | Count; Item indexer; Add(T) with no result |
| ArrayList<T> : List<T> | Allocate(capacity); the List and Iterable implementations |

ArrayList should be an ordinary managed class in this target projection. Interface
variables carry references to implementing objects, with the same object retained
across assignment, calls and returns. Converting a class reference to an implemented
interface must not copy the collection or allocate another wrapper. A managed byref to
a slot is a different capability from this ordinary object reference. Internal runtime
representation need not match CoreCLR, but must preserve identity, GC reachability and
dispatch after the caller returns. Invalid interface conversions must be rejected or
fault according to the instruction's contract; metadata declarations alone cannot
supply missing implementation behavior. The first slice should use invariant interface
generics and inherited contracts, without implicitly promising variance.

The existing iteration behavior in [common interfaces](common-interfaces.md) remains
the compatibility baseline: independent cursors, Current valid only during a successful
iteration, sticky exhaustion, and explicit idempotent Dispose. Keep buffer/extent
capture semantics unless a separate API decision changes them. Classifying ArrayList
as a nominal class is a migration from the existing value wrapper with shared state;
its Copy behavior needs explicit review in the Raven-facing contract. Neo callers
and compiler migration are outside this experiment. Do not silently switch to
.NET's fail-fast mutation policy during this migration.

The old readonly byref receiver qualifiers do not automatically become CLI class
receiver qualifiers. Preserve observable getter/iterator contracts, then specify how
runtime readonly capabilities apply to nominal references separately. Generic Void
remains supported by the platform; ordinary Add/Dispose methods should have no-result
signatures in this target rather than requiring a dummy payload.

## .NET/CLI comparison and placement

Primary sources consulted 2026-09-12:

- [ECMA-335, sixth edition](https://www.ecma-international.org/wp-content/uploads/ECMA-335_6th_edition_june_2012.pdf),
  Partition II §12 and §22.23, defines interface implementation and InterfaceImpl metadata;
  Partition III §4.2 and §2.1 describe callvirt and constrained calls.
- [OpCodes.Callvirt](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.callvirt?view=net-10.0)
  documents receiver-based method invocation and the separate value-receiver concern.
- The [existing collection comparison](common-interfaces.md#net-baseline-alternatives-and-tradeoffs)
  covers IEnumerable/IEnumerator naming, protocol and mutation-policy differences.

Use standard class/interface signatures, generic type arguments, InterfaceImpl relations
and callvirt tokens at the compiler boundary. Runtime dispatch must validate conformance
and substitute generic arguments when matching implementations. Dispatch should select
by the receiver's concrete type and requested interface member, not merely by a method
name. Metadata MethodImpl support and default interface bodies are later requirements;
the initial contract uses public implicit implementations and abstract interface members.
The ECMA sixth edition is not a complete account of modern default/static interface
features, and this slice makes no claims about supporting them.

The successful reference-class probe has no boxing or constrained calls. This is also
a normal CLI class-reference scenario, not evidence that neoCLR eliminates boxing for
values. Value receivers and constrained generic calls require separate decisions and
validation. Reusing legacy stack-address interface views for ordinary class references
would conceal that distinction and create lifetime/ABI ambiguity. Adding a new Raven
interface opcode would instead duplicate a standard facility without demonstrated need.
The preferred path is runtime adaptation under the existing compiler metadata surface.

## What currently blocks execution

| Layer | Evidence and missing work |
| --- | --- |
| Raven compilation | Candidate class/interface declarations bind and emit with standard calls; tested without compiler changes at Raven 5b773ae3536f52ef077c8897867950249d6dde90. |
| Nominal classes | Nominal classes now admit generic owner parameters, interface conformance and dispatch. Class inheritance, methods with their own type parameters on nominal classes and virtual/byref class receivers remain unsupported. |
| Existing interface runtime | Legacy views remain unchanged. Ordinary interface object references now support explicit casts, typed slots/fields, GC and dispatch; interface arrays and indirect slot access are now admitted; ordinary array references and implicit class/interface upcasts are implemented; importer admission remains later work. |
| Library | An isolated profile adapts the existing implementations to nominal classes and ordinary managed arrays. Runtime tests pass; the default legacy library remains unchanged. String defaults and predicate/delegate contracts are outside the profile. |
| Import bridge | UnionImport deliberately rejects these collection types and emits no executable. Candidate declarations are isolated from the working core surface. |

The smallest implementation sequence is:

1. Support a nominal class through an ordinary interface reference: conformance,
   assignment/return, GC retention, dispatch and incompatible-type rejection. Runtime
   tests may isolate the mechanism, but this is groundwork for the library scenario.
2. Support the closed generic class/interface combinations required by ArrayList<Int32>
   and Iterator<Int32>, including inherited Disposable dispatch and signature matching.
3. Adapt the real ArrayList/iterator implementation, expose matching declarations and
   bindings, and execute the checked-in Raven sample. Its proposed output is 1 then 42;
   that output has not yet been observed on neoCLR for this sample.
4. Extend the collection case only as the demo needs. Union propagation follows the
   established interface contract; broader interface features remain explicitly scoped.

This order adds runtime complexity and migration work, but keeps the existing library
as the implementation and avoids per-API fake dispatch adapters. No performance or
allocation benefit is claimed before runtime validation.

## Reproduce the contract probe

From the neoCLR repository root, using a new output directory:

```sh
dotnet run --project docs/experiments/raven-target/Probe.csproj -p:RavenRoot=/absolute/path/to/Raven -p:BuildProjectReferences=false -p:WarningLevel=0 -- --interfaces /tmp/neoclr-interface-contract
```

Build Raven first as described in the [probe instructions](experiments/raven-target/README.md).
The probe emits a separate reference assembly and compiles
[library-interfaces.rvn](experiments/raven-target/samples/library-interfaces.rvn), checks
dependency closure and interface-call shapes, rejects wrong element/receiver types, and
requires the current runtime importer to reject admission without creating executable IL.
[interface-results.json](experiments/raven-target/interface-results.json) records the
observations. These declaration bodies never execute and are not installed in the
working demo's compiler reference set. Existing Result/Option/Void tasks remain bounded
to their prior runtime profile.

## Implemented nominal interface groundwork (2026-09-12)

Non-generic `.type class` declarations may implement interfaces whose abstract methods
use ordinary `instance` receivers. Interface contracts with no-result returns are
supported, and implementation matching now distinguishes no-result from inhabited Void.
The existing `.method instance byref` contracts retain their legacy receiver model.
No ArrayList declarations or implementations have been migrated by this slice.

`castclass Interface` establishes an ordinary interface view of an implementing class
object. It changes the static view carried by the existing heap handle, without copying
the object or creating a guest wrapper. Casts back to the exact concrete class preserve
identity; incompatible casts fault. A null object casts to a typed null interface, and
calling through it faults. Plain interface locals, parameters, returns and fields retain
the owning heap reference. `initobj` supplies the same typed null default as nominal
class storage. This adopts the current class-default policy, not a new non-null contract.

`callvirt` accepts an implementing nominal class, the named interface, or a derived
interface exposing the requested parent contract. The runtime selects the concrete
implementation by full interface signature and passes the original class reference as
its receiver. The typed verifier checks conformance; runtime checks also enforce it
when execution is requested without typed verification. Missing implementations and
incompatible signatures fail metadata validation. Class virtual slots, explicit/default
nominal implementations and generic class receivers are not enabled here.

This reuses standard castclass/callvirt operations and the comparison above. One
transitional limitation remains: typed neoIL storage/call arguments require an explicit
castclass when changing a reference's static type. Ordinary CLI implicit assignability
must be handled when expanding the importer or the storage rules; the Raven sample has
not been adapted to execute by silently inserting a legacy borrowed view. Interface
arrays and indirect ldobj/stobj access also remain outside this slice. Existing scoped
interface views keep their previous restrictions and do not gain heap ownership.

`tests/nominal_interfaces.rs` covers mutation/aliasing, inherited and concrete-receiver
dispatch, return escape, field retention through forced GC, null and incompatible casts,
wrong receivers, missing implementations and no-result signature mismatches. The host
ObjectReference API exposes `target()` for the static view and `concrete_type()` for the
allocation type; identity comparison ignores the view. Ten new tests and the existing
class/interface/default/no-result/initialization suites pass (118 focused tests total).
The next library prerequisites are closed generic nominal classes and the associated
import/storage conversions, before adapting ArrayList and its iterator.

## Closed generic nominal classes (2026-09-12)

The runtime now admits `.type class Cell<T>` using the existing generic metadata and
substitution machinery. Constructed owners such as `Cell<Int32>` retain ordinary class
reference semantics. Constructor parameters, fields, instance methods and inherited
interface contracts substitute the same owner arguments. Different instantiations remain
distinct: a `Cell<Int32>` cannot call a `Cell<Boolean>` method or cast to an unimplemented
`Read<Boolean>` interface. Existing generic constraints remain enforced.

This fills an implementation gap against the .NET generic-definition/constructed-type
model already researched in [generic source records](generic-source-records.md#net-comparison-and-boundaries).
It extends the class/interface contract above without a new opcode, erased Object payload,
per-instantiation source declaration or Raven compiler change. Reusing substitution keeps
owner and method parameters separate. This makes no claim about CLR JIT code sharing or
performance; methods with their own type parameters on nominal classes, variance and class inheritance remain outside the
slice. `Void` remains neoCLR's deliberate supported generic argument, distinct from a
method that returns no stack value.

Tests in `tests/generic_classes.rs` cover constructor dispatch, interface mutation,
closed-type rejection, constraints, typed null defaults, unavailable native layout, Void
payloads and nested class references surviving return and GC after an artifact round trip.
Constructor allocation still initializes fields before entering the body. Existing
unsupported defaults, including String and array fields, still fault; supplying a
constructor argument does not bypass this rule. The array/default storage contract and
implicit reference assignability/import support are therefore the next collection
prerequisites. The candidate Raven collection probe remains compile-only and rejected
by the importer. The existing Neo collection implementation is unchanged.

## Interface elements and slot access (2026-09-12)

[Object-reference array elements](managed-arrays.md#nominal-object-reference-elements-2026-09-12)
now support typed null defaults, exact-type stores, interface dispatch and indirect
load/rebinding through element addresses. These paths reuse existing GC retention and
frame provenance checks. The remaining array blocker is the array reference itself:
legacy `T[]&` is not the CLI ordinary `T[]` object reference. Constructor defaults and
import admission must account for that distinction before the Raven collection demo
can run; the existing Neo collection implementation is unchanged.

## Ordinary array-reference runtime support (2026-09-12)

The [ordinary array slice](managed-arrays.md#ordinary-array-references-and-migration-2026-09-12)
now implements newarr object-reference results, array-field null defaults and generic
constructor allocation. Internal `ArrayRef` signatures distinguish CLI arrays from
Neo's existing owned arrays. The Neo compiler preserves its source behavior by emitting
`array.new`; old neoIL artifacts need migration. Reference conversions/import admission,
String defaults and actual collection adaptation remain ahead. No Raven changes or
executable collection-import claim are made by this slice.

## Bounded Raven array import (2026-09-12)

Raven Int32 vectors now compile/import/execute in the [array probe](experiments/raven-target/README.md#executable-raven-arrays-2026-09-12).
The bridge maps standard signatures to ArrayRef and checks array stack operands before
writing neoIL. Its static-function profile does not yet admit nominal class fields or
interface conversions. The candidate collection probe remains rejected; the array demo
establishes one prerequisite rather than substituting for the real collection library.

## Implicit ordinary-reference assignability (2026-09-12)

The runtime and typed verifier now accept an implementing nominal class reference where
an ordinary interface reference is declared. A derived-interface reference also converts
to its inherited interface, with generic arguments substituted and matched invariantly.
This applies to local/argument assignments, calls and constructor arguments, returns,
record/class fields, array elements and indirect slot stores. A conversion preserves the
allocation identity and changes the stored handle's static view to the declared target.
Typed null references convert only along the same declared conformance relationship.

This implements the class-to-interface and interface-to-parent subset of
[C# implicit reference conversions, §10.2.8](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/conversions#1028-implicit-reference-conversions)
(primary source consulted 2026-09-12), following the CLI contract cited above. The
language defines conversion applicability; neoCLR's metadata supplies conformance and
the verifier/runtime enforce the storage contract. The alternative of inserting
castclass at every importer store/call would add target-specific rewriting for ordinary
CLI assignments. Runtime normalization avoids that requirement without a new opcode or
wrapper allocation. This is compatibility work, not a measured performance improvement;
conformance is currently checked at conversion time without a specialized cache.

```text
.local System.Collections.List<Int32> values
ldc.i4 2
call System.Collections.ArrayList<Int32>::Allocate(Int32)
stloc values
```

The fragment illustrates the intended reference assignment once the adapted library is
admitted; it is not a runnable collection sample yet. Existing explicit casts remain
valid. Unrelated interfaces, mismatched generic arguments and implicit downcasts are
rejected, including during execution without typed verification. `castclass` remains
available for checked downcasts. Byref slots stay invariant: `Cell&` cannot substitute
for `Read&`, since the callee could otherwise replace the caller's Cell with a different
implementation. This does not change legacy borrowed interface views or make native
pointer storage accept object references. Arrays remain invariant as whole types even
though an interface element slot accepts an implementing class reference.

The rule is implemented at module-aware interpreter boundaries. Module-free host value/
slot operations retain exact typing. Generic-constraint-based conversions from open
parameters, general stack-merge common-type inference, class base conversions, array
covariance and Raven class/interface importer admission remain separate work. The
collection metadata probe still rejects execution; this removes its runtime assignment
prerequisite without claiming the importer or library migration is complete.

`tests/reference_assignability.rs` covers assignments/calls/returns, constructor and field
stores, element/indirect stores, identity, nulls, inherited generic contracts, GC retention
and rejected downcasts, unrelated types and byref widening.

Validation: eight new regressions and 104 related tests pass (112 focused tests total).

## Adapted runtime collection profile (2026-09-12)

The isolated [profile generator](experiments/raven-target/collection_library.py) now
adapts the existing ArrayList/iterator source algorithms into a selectable System
library. ArrayList, its shared state and ArrayIterator become nominal classes;
List/Iterable/Iterator/Disposable use ordinary reference receivers and no-result
mutation/disposal methods. Buffers are ordinary array references created by newarr.
The runtime's existing interface upcasts and callvirt perform real dispatch. No new
instruction or per-method host implementation was added; declaration-stub bodies are
not executed. The default library and Neo compiler are unchanged.

Allocate, Count, Capacity, indexing, Add, Copy, GetIterator, MoveNext, Current and Dispose
are included. Growth, shallow Copy and buffer/extent capture retain the existing
algorithms. The extra shared-state class is retained for this first adaptation; removing
it is a possible later simplification, not a requirement for reference semantics.
Unlike the old reserved/uninitialized buffer, newarr requires a supported default for
T. Int32, nominal class references and Void are tested. String defaults remain unsupported;
this is not an unrestricted replacement for the old collection library. Predicate/delegate
helpers are excluded until their target contracts are admitted.

This follows the CLI class/array/reference baseline cited above and the existing
[collection contract comparison](common-interfaces.md#net-baseline-alternatives-and-tradeoffs).
The profile avoids copying and maintaining a second set of collection algorithms, but
its source transformations are temporary experiment tooling and must be reviewed when
the source shapes change. It is not a general retargeter or the future binary library
format. The selected target library must accompany verification and execution; using
the default System library would select different signatures.

`tests/raven_collections.rs` verifies growth through interface aliases, independent Copy,
returned iterators retained across GC, captured buffer/extent, disposal, invalid access,
wrong element types, reference-element identity and generic Void. The runtime fixture
below returns 42. Raven's collection importer still rejects the candidate program;
import admission and target declarations are the next slice, not completed by this test.

From the repository root, choose a fresh output path:

```sh
python3 docs/experiments/raven-target/collection_library.py /tmp/RavenCollections.neoil
cargo run -- verify docs/experiments/raven-target/samples/collection-runtime.neoil --system /tmp/RavenCollections.neoil
cargo run -- run docs/experiments/raven-target/samples/collection-runtime.neoil --system /tmp/RavenCollections.neoil
cargo test --test raven_collections
```

Future language-level iteration must use [target-specific Raven contracts](raven-target-contracts.md),
so .NET's names remain supported alongside neoCLR's Iterable/Iterator names.
