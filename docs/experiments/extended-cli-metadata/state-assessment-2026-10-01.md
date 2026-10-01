# Metadata and Raven target checkpoint — 2026-10-01

## Conclusion

The initial end-to-end integration works: Raven lowers supported source through shared
plans, the independent C# metadata API writes a binary PE/#Neo assembly, and neoCLR
loads, verifies and executes it. Ordinary .NET emission remains available. This is a
substantial bounded target implementation, **not complete emission of the Raven runtime
class library**, and not yet the final extended-CLI executable format.

The author requested this assessment after the generic-constraint round. It records
current evidence without treating “continue” as acceptance of every remaining design.

## Proven coverage

| Area | Evidence and boundary |
| --- | --- |
| Independent producer | Cecil-like C# definitions, typed helpers/raw emits, CLI output, native output and declaration projection. 67 C# contract groups pass at this checkpoint. |
| Shared compiler path | Source/signature/body plans and explicit capability checks feed backend-owned builders. This covers assembly functions, primitive control flow, owned root classes, construction, fields, properties/indexers, arrays and generic static/instance calls. It is not a replacement of every Reflection.Emit path. |
| Generic identity | Owner and method parameter scopes, constructed signatures/calls/fields, nested class values, defaults and alias mutation execute on both targets. |
| Constraints | Owned nongeneric nominal class bounds plus class/struct/new requirements on type parameters survive CLI/native metadata and CLI reference projection. Concrete instantiation checks are enforced. Method constraints and operations through open constrained parameters are not covered by this producer. |
| Runtime | Binary assembly loading, verification and execution are exercised directly. New distinct ReferenceType/ValueType/DefaultConstructor kinds are implemented; 10 focused native constraint tests pass, including missing/private/abstract constructor rejection. |
| Source acceptance | Selected original integer Math functions previously passed 11 boundary cases. The expanded generic consumer uses the unchanged Order declaration and added stress code, returning 42 on both runtimes in both file orders. It is not the unchanged full application. |

[Machine-readable consumer evidence](generic-runtime-validation.json) records source,
consumer and runtime hashes and explicitly reports `fullConsumer: false`. The current
round passes 17 focused Raven C# tests and the API snapshot check. No full-suite,
full-library or performance result is claimed.

## Compatibility and constraints

Ordinary CLI metadata remains the compatibility baseline: GenericParam flags and
GenericParamConstraint, constructed TypeSpec/MemberRef/MethodSpec, and ordinary field
and property tables. Native execution currently uses an explicit #Neo payload with a
CLI reference projection. This proves useful binary integration, not that arbitrary
.NET executable assemblies can run unchanged on neoCLR.

The [.NET special-constraint categories](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.genericparameterattributes?view=net-10.0)
are represented distinctly. Reusing native notvoid/notreference would lose meaning;
the new kinds preserve reference/value/constructor checks but require a matching
feature-branch runtime. String and managed vectors are references. The producer's
value arguments are presently Int32/Int64/Boolean, not arbitrary structs. `new()` is a
requirement, not implementation of `new T()`. Notnull requires separate nullable
semantics, not an invented special flag. Owned root-class bounds currently require
exact identity because inheritance is outside the producer's bounded type model;
that limitation is not a permanent neoCLR rule.

Runtime Contract configuration is unchanged. Work is on neoCLR
`codex/extended-cli-metadata` and Raven `codex/metadata-consumer`, not a claim of main
or released availability. The runtime hash in the evidence identifies the tested
build. Native symbol import through the compiler's future shared loader remains work;
producing a CLI reference projection alone does not complete that integration.

## Remaining gates and proposed sequence

1. **Choose a real source acceptance unit.** Inventory the dependencies and unsupported
   constructs of one unmodified runtime-library type/function set. Keep the selected
   Math baseline, and use the full [order-collections sample](../raven-target/samples/application-order-collections.rvn)
   as the broader coverage map. Do not count synthetic generic probes as a full build.
2. **Finish constraint use where that source needs it.** Shared method-constraint
   descriptors, interface/dependent bounds, and symbolic constrained operations need
   separate producer/adapter/runtime acceptance. `new T()` and base/interface calls
   through T are likely body-emission gates. They should be driven by the selected
   source rather than implementing every constraint category speculatively.
3. **Connect real collection contracts.** The broad sample calls Map/MutableMap,
   ArrayList, Iterable and LINQ-style operations; it uses delegates/lambdas,
   Option/Result patterns and propagation. Imports, interface dispatch, union
   representation and callback emission need target-specific checks and shared
   lowering where semantics agree. This is larger than adding generic flags.
4. **Complete native metadata symbol loading at the agreed checkpoint.** Reuse the
   semantic importer behind a metadata-source contract where practical. Translated
   System assemblies can supply reference evidence, but cannot substitute for
   compiling and comparing the original library source.

This sequence is a recommendation, not a new permanent roadmap priority. Do not
reopen structural-type experiments before ordinary compiler-required coverage. Keep
performance as a later measured question: binary loading works, but this round did
not benchmark codegen or prove a speedup. Preserve typed backend handles and avoid
repeated reflection or metadata reconstruction in hot paths as coverage expands.


## Follow-up: first whole-source unit

The complete unchanged System.Globalization.Language class now emits and executes
with shared static computed property support; no new opcode or metadata API was
needed. [Whole-source evidence](whole-library-runtime-validation.json) records
und/sv/he output and result 42 in both file orders/targets, plus setter and generic
static getter checks. Static storage remains outside the native adapter.

The [refreshed inventory](class-library-validation.json) shows comparer interfaces
bind under the bootstrap but fail interface declaration emission. ArrayList needs its
native dependencies before emission can be assessed. Interface metadata is the next
small contract; full class-library acceptance remains open.

The author reaffirmed that .NET behavior and CLI instruction semantics are the default
unless an alternative is explicitly chosen. The native payload is a temporary storage
and loader bridge, not a separate instruction-set design. Unsupported features such
as exception handling limit coverage rather than changing supported semantics.


The next declaration gate is now closed: unchanged Comparer<T> and EqualityComparer<T>
emit, project and load as abstract interface contracts on both targets. The test entry
is independent (42); [evidence](interface-library-runtime-validation.json) explicitly
sets interfaceDispatch and entryUsesInterfaces to false. Properties, inherited contracts,
implementation and dispatch remain separate gates; no runtime/ISA change was needed.


Iterator extension: Disposable and Iterator<T> now also emit/load unchanged with
abstract property associations and inherited nongeneric interfaces. Interface-valued
signatures, generic base instantiations and dispatch remain open. The independent-entry
qualification in the machine-readable evidence still applies.


Interface reference extension (2026-10-01): unchanged Iterable<T> now emits beside
Comparer, EqualityComparer, Disposable and Iterator. Owned interface identities and
constructions share CLI CLASS/GenericInst and native Named/Constructed signatures.
The shared compiler descriptor is nominal rather than class-specific, with an explicit
interface-signature capability; nullable reference annotations retain binder semantics
and map to the same reference storage. Parameters, results, locals, defaults and arrays
execute on .NET and binary neoCLR in both source orders (42). The host-core bootstrap
and Runtime Contract configuration are unchanged. No runtime or ISA changes are needed.
Interface invocation/implementation remains the next author-directed acceptance gate;
generic interface inheritance remains deferred. Seven focused C# interface tests and
68 metadata API test groups pass. The evidence exercises null/default reference flow,
not dynamic dispatch.


## Follow-up: assessment after dispatch

Owned nongeneric interface method/property dispatch now passes on both runtimes.
The author then requested larger source/sample-driven steps. The
[new assessment](readiness-assessment-2026-10-01.md) records 31 fresh inventory attempts,
twelve successful ordinary CLI sample emissions, three successful legacy-bridge runtime
controls, an invalid Option constructor in the broad sample, and the direct backend's
remaining target-profile gate. Use that assessment for the next bounded acceptance unit.
