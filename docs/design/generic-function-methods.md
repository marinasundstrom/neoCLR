# Generic function method signatures

**Development implementation, updated 2026-10-30.** This is a general function
capability, requested while discussing [signals](signals-and-reactive-streams.md).
Signals, forwarding wrappers, interceptors and adapters are consumers; none owns
the language or runtime mechanism. The original investigation below is retained;
implementation progress and qualification are recorded here.

The development implementation now supports native format 6,
`MethodInfo.FunctionType`, whole-signature substitution and general `FunctionReturn`
expressions. Raven `codex/neoclr` binds `func name: T with args` and `ReturnType<T>`;
`System.Runtime.Reflection.FunctionReflection.Invoke<T>` supplies typed forwarding.
The separate `Neo.Signals` extension uses this mechanism through an ordinary
subscription-only property. See [its README](../../extensions/Neo.Signals/README.md),
[the generics guide](../../website/content/features/generics/index.md) and the
[qualification record](../validation/generic-function-methods-20261030/README.md).

## Implemented profile

- Whole method signatures use a function-constrained parameter of the declaring
  type; initial Raven bodies are nonvirtual, non-async and have no method type
  parameters. Abstract declarations carry no argument binding/body.
- `args` is an owned `object?[]`, with null references, boxed values and no receiver.
  The native format retains the boxed implementation ABI separately from the public
  signature; materialization generates a typed adapter. This is general function
  support, not a signal-specific runtime hook.
- `ReturnType<T>` is a target-specific type expression. Open bodies must return that
  opaque type, forward a matching projected result, or explicitly use its default;
  an arbitrary int/unit does not satisfy every possible T.
- Reflection's Object overload retains `Result<Object?, ReflectionError>`. The typed
  overload extracts it and returns the projected result; validation failures call
  Panic and target faults propagate. Unit-valued and no-result callbacks produce
  unit for the typed caller. No Task processing is added.
- `SignalSource<T>` checks unit return policy at construction. `Signal<T>` exposes
  subscriptions only. Dispatch is ordered, synchronous and uses a registration
  snapshot; duplicate registrations have independent idempotent disposal handles.
  Concurrent access needs external synchronization. Completion, reactive error
  channels, operators and schedulers remain separate proposed library work.
- Initial argument marshalling covers exact scalar boxes and compatible references;
  ref/out, arbitrary value aggregates and rich signature constraints remain gaps.
  CLI executable export and neoIL text export of the new signature forms remain
  unsupported. Use native metadata and a matching toolchain.

The remainder preserves the original design discussion; proposed broader APIs and
alternatives below do not expand this implemented profile.

The author specifies that `func Invoke: T`, with any method name, belongs to an
open generic definition. Its signature materializes with generic arguments. On a
closed construction, ordinary method reflection exposes ordinary parameters and
a result. The author also permits redesign and breaking compatibility during this
experimental stage; preserving a duplicate legacy signature model is not a goal.

## Proposed authoritative signature

Represent a method's signature by a **function-type reference**. It can refer to
a structural function type or to an in-scope function-constrained generic
parameter. Name, declaration identity, accessibility, owner, static/instance
status and method generic parameters remain ordinary method metadata.

```text
// Development Raven syntax for the neoCLR target.
class Forwarder<T> where T: func {
    private val target: T

    init(target: T) {
        self.target = target
    }

    func Invoke: T with args {
        return FunctionReflection.Invoke<T>(target, args)
    }
}
```

The author-selected syntax is `func name: T with args`. The binding `args` has
static type `object?[]`: an owned array of boxed arguments, preserving null values
and excluding the receiver. This is initially an interop capability. The helper
name above is now the implementation home. It returns T's result; multicast
result aggregation is a separate library policy.

The proposed metadata meaning is:

```text
TypeDefinition Forwarder<T where T: func>
    MethodDefinition
        name: Invoke
        instance: true
        functionType: TypeParameter(0)

Forwarder<(Int32, String) -> Boolean>.Invoke
    functionType: Function([Int32, String], Boolean)
    parameters: [Int32, String]             // derived
    returnType: Boolean                    // derived
```

This is a semantic sketch, not a new encoded file format. Do not encode the
generic reference as a single ordinary parameter or as a zero-parameter method.
It denotes the **whole function signature**, including parameter modes and the
result/no-result distinction. The instance receiver remains separate: the owner's
`self` does not become an extra structural-function parameter.

The clean redesign should use this authoritative representation for ordinary
methods too: their function type is an explicit structural signature. Parameter
names, defaults, nullable annotations and declaration attributes stay on method
parameter/return metadata because they are not all structural type identity.
Reject conflicting parameter-side information when materializing a projected
signature. Initially projected declarations supply positional arguments without
invented caller-visible names, defaults or `params` semantics.

## Reflection and partial substitution

The author's proposed additional property fits the public model:

```text
MethodInfo.FunctionType: TypeInfo
```

The property's invariant is exactly **a structural function type or a generic
type parameter constrained as a function**. TypeInfo is the common API base, not
permission to store an arbitrary type. An open structural function with generic
components is still a function type; a bare `T: func` is the second case. Enforce
this invariant in authoring, loading and substitution, including partial views.

`TypeInfo` is necessary: a function-constrained generic parameter is not itself a
concrete `FunctionTypeInfo`. An ordinary method can expose a structural function
descriptor; a dependent open method exposes the referenced generic parameter.
The author explicitly connects this property to the general direction of exposing
actual function signatures in type form. It belongs on ordinary methods as well
as methods declared through a generic signature, complementing the existing
FunctionTypeInfo invocation descriptor without conflating method identity and shape.
This property describes a signature, not a bound function value or a pointer to
executable code. Instance-method binding still supplies a receiver separately.

| Substitution | FunctionType view | Parameter/result view |
| --- | --- | --- |
| None, `T` open | The scoped parameter T with `func` constraint | Arity and result unknown; do not pretend this is an empty method. |
| `T → U`, U constrained by `func` | Scoped parameter U | Still dependent. |
| `T → (U, int) -> U` | Structural function with open components | Two parameters, one component/result still U. |
| `T → (string, int) -> string` | Closed structural function | Ordinary complete method signature. |

Partial closure is therefore meaningful. Known shape and fully closed component
types are different facts. Closing the declaring type is sufficient only when
no relevant method-level generic arguments remain unresolved.

On complete materialization, `MethodInfo.GetParameters`, result information,
ordinary lookup and invocation use the resolved signature. No special closed
method reflection class or special signal member is needed. Keep the original
definition identity and normal generic construction provenance.

For a bare unresolved T, existing parameter/result APIs need a documented
unresolved-signature outcome. The exact host SDK exception versus guest explicit
error API is open; returning an empty list or fabricating a return type is invalid.
An open shape `(U) -> U` can expose its known parameter list normally.

## Source findings

Inspected Raven `codex/neoclr` at `b4c5790e199563a502ac20460dc260e87e0d0add`
and neoCLR HEAD `9979002939052985cf682d7353e2669dd71de868`. neoCLR also contains
unrelated work in progress; this investigation does not validate or modify it.
Paths below identify implementation owners, not completed feature support.

| Layer | Current source evidence | Required change |
| --- | --- | --- |
| Raven syntax | `Syntax/Model.xml` requires ParameterList on MethodDeclaration and FunctionStatement; `TypeDeclarationParser.ParseMethodOrConstructorDeclaration` always parses it. | Add a signature annotation alternative with correct recovery and generated syntax support. |
| Raven declaration binding | `Binder/TypeMemberBinder.cs` builds parameter symbols from declared syntax. | Bind the signature type in generic scope; retain an unresolved signature reference when T is open. |
| Function symbols | `Symbols/Constructed/StructuralFunctionTypeSymbol.cs` already owns parameter types, RefKinds, result and synthesized Invoke. | Reuse structural identity and substitution rather than defining nominal callback types. |
| Generic method symbols | `ConstructedMethodSymbol.Parameters` maps the existing definition's parameters one-for-one. | Resolve the whole signature before building constructed parameter/result symbols; permit different arities for different constructions. |
| Raven portable/native emission | `CodeGen/Portable/CallableSignature.cs` has a fixed parameter array; `NeoClrCallableDefinitionBuilder` builds a fixed SDK MethodSignature. | Preserve a dependent function-type reference through plans and native emission. |
| Native SDK model | `Model/MethodSignature.cs` stores ReturnType plus a concrete ParameterTypes collection. | Redesign around the authoritative function-type reference and derive expanded views. |
| Native import/reflection | SDK `Introspection/MethodInfo.cs` resolves each fixed parameter; Raven `NativeAssemblySymbol.NativeMethodSymbol` consumes GetParameters. | Preserve and substitute FunctionType, supporting unresolved definitions and partially resolved shapes. |
| Runtime metadata | `src/metadata.rs` Function and FunctionRef carry parameter vectors; FunctionType carries parameter/result contracts. | Method definitions/references need dependent signature identity, with ordinary closed executable signatures derived from it. |
| Native generics | `runtime/codegen/src/specialization.rs` substitutes each existing parameter/local/result. | Expand dependent signatures and pack operations as part of materialization, not just element substitution. |
| Dynamic forwarding | `src/function_invocation.rs` and `runtime/codegen/src/function_invocation.rs` build checked adapters for known bound function shapes. | Reuse for an explicit boxed path; these services neither describe dependent method declarations nor capture current arguments. |

Raven paths above are relative to `src/Raven.CodeAnalysis`, except native emission
and imports in `src/Raven.CodeAnalysis.NeoClr`. SDK paths are relative to
`tools/metadata/NeoCLR.Metadata.Experimental`.

## Argument representation and lowering

Bind `with args` as an invocation-local `object?[]`. Materialization supplies
ordinary typed entry parameters and an adapter that creates the array in argument
order. Reference values retain their identity and may be null; supported value
arguments are boxed. The receiver is excluded. This is an owned managed array,
not a borrowed frame, stack walker or thread-local invocation context. Its type
does not encode T's arity, so invocation must validate its length and elements.

This author clarification supersedes the earlier scoped `Arguments<T>` proposal
for the initial implementation. A future optimization can eliminate packing or
boxing only when observable array identity, mutation and escape behavior permit
it. No JIT/AOT optimization is promised by the design.

### Signature-derived helper results

The author proposes this use (uncompiled proposal notation):

```text
class Forwarder<T> where T: func {
    val handlers: List<T> = ...

    func Invoke: T with args {
        for handler in handlers {
            let result = FunctionReflection.Invoke<T>(handler, args)
        }
    }
}
```

Two distinct dependencies are involved:

- `Forwarder<T>.Invoke` has the entire signature T.
- `FunctionReflection.Invoke<T>` has fixed parameters `(T, object?[])`
  and a result projected from T.

Conceptually the helper's signature is `(T, object?[]) -> ReturnTypeOf(T)`.
`ReturnTypeOf` is design notation, not selected Raven syntax or a library type.
`MethodInfo.FunctionType: TypeInfo` accommodates the outer function type, but
whole-signature substitution alone does not implement this nested projection.
An implementation must either represent result projections as general type
expressions in compiler/native metadata, or admit this helper through an explicit
compiler intrinsic contract. The first is the general solution; the second must
remain visibly a provisional interop facility, not an ordinary generic method
whose declared object result silently changes at call sites.

For T = `(int, string?) -> bool`, the helper has parameters
`(T, object?[])` and returns bool. For open T, `result` has a dependent result
type, not object?. It can be forwarded or discarded, but the generic body cannot
assume it is bool or perform operations requiring an unproven concrete type.
Partial substitution retains the dependency until the relevant result resolves.
The runtime may invoke through a boxed representation and check/cast/unbox the
result; that representation does not determine the source expression's type.

The shown loop discards each handler result. It does not yet define the return
value of the surrounding method when T returns a value: choosing the last result,
combining results, or handling zero registrations requires an explicit library
policy. Neither default(T-result) nor a silent null result is implied. Preserve
the distinction between an inhabited unit result and a native no-result call;
local binding rules for no-result calls need explicit treatment. Task usage and
async aggregation are separate issues, as the author requests; there is no
implicit awaiting or Task-result unwrapping in this capability.

Initially admit supported by-value shapes. Preserve ref/out/readonly contracts
in signature metadata, but diagnose unsupported interop execution. An object
array does not by itself preserve aliases, copy-back or definite-assignment rules.

### Runtime reflection integration

The current library helper is:

```text
FunctionReflection.Invoke(function: Object?, arguments: Object?[])
    -> Result<Object?, ReflectionError>
```

It calls checked `neoCLR.Runtime.InvokeFunctionCheck` and then
`neoCLR.Runtime.InvokeFunction`. The runtime builds adapters that cast/unbox,
perform an ordinary bound Function call and box the result. The codegen path
collects selected closed `BindFunction` shapes; retention and adapter inventory
must include newly materialized callable bindings before adapter generation.

This supports an explicit dynamic prototype. It does not provide the method's
signature, incoming arguments or arbitrary generic return typing. A wrapper
returning ResultTypeOf(T) needs a checked result adaptation and an explicit
reflection-error policy. The proposed generic helper must specify how these reflection failures are
reported. Returning `Result<ReturnTypeOf(T), ReflectionError>` would make the
source result a Result rather than T's result; a direct-result helper instead
needs an explicit failure policy. This decision is still open.

All new compiler-recognized operations must be admitted through a neoCLR target
contract with matched metadata/runtime support. `NeoClrCapabilities`, portable
instruction admission and native method-body lowering are the owners to inspect.
`NeoClrRuntimeServiceDeclaration` already provides explicit runtime-service
admission where an InternalCall is actually needed. No new configuration key is
selected here, and there is no need to fabricate a runtime service for an argument
load that specialization can produce directly.

## Materialization sequence

1. Load and validate the open signature reference, its scope and `func` proof.
   Preserve this metadata when importing a provider without seeing its consumers.
2. Substitute declaring-type and method generic arguments. Retain partial
   substitutions for open introspection/compiler views; require executable
   specialization to resolve every necessary shape and component.
3. Resolve FunctionType and derive the complete method signature, parameter modes
   and result. Validate arity limits and target-supported ABI types.
4. Expand signature-dependent body operations and locals into the concrete
   argument/result layout. Resolve calls and function bindings against the closed
   signature and stable method definition identity.
5. Run ordinary verification, GC-root planning and backend lowering on the
   expanded body. Use the same semantics in interpreter, JIT and AOT.
6. Expose an ordinary MethodInfo for the construction; retain its relationship to
   the original open definition. Cache using definition plus both generic scopes,
   not just name or an unresolved parameter vector.

No unresolved pack, result projection or dependent arity may reach a backend that
assumes a concrete call ABI. Detect invalid constraints, unsupported shapes and
recursive resolution before publishing a partial materialization.

Method references must preserve declaring construction and definition identity.
Overload candidates are compared after substitution; collisions introduced by
closing different signature expressions must diagnose ambiguity. Bare-open
generic callers can forward a matching pack, but cannot call an unknown T with an
arbitrary ordinary argument list. Virtual/interface slots, overrides, method-level
signature parameters and inferred method arguments need dedicated rules; start
with nonvirtual methods whose signature refers to an enclosing generic parameter.

## Metadata and compatibility

The native writer/reader, SDK authoring/import snapshots, runtime metadata loader,
signature equality/hashing, generic reference resolution, reflection and
specialization must agree on the new authoritative representation. Replacing the
old signature fields is permitted. Derived closed executable fields may remain
for backend convenience, but cannot be independent competing metadata truth.

The bounded CBOR codec serializes the native object structure; most design work
belongs in semantic schema readers/writers and validation rather than inventing a
new binary primitive. Choose a new format/capability boundary so older consumers
cannot ignore the dependent reference and execute it as a zero-argument method.
Rebuild matching compiler, SDK, runtime bundles and generated reference snapshots;
no compatibility shim or fallback interpretation is required by this investigation.

.NET/CLI MethodDefSig encodes an explicit parameter count, result and parameter
sequence, with generic substitution inside that signature. It does not directly
encode "use the entire function signature of generic parameter T". See
[ECMA-335 II.23.2.1](https://www.ecma-international.org/wp-content/uploads/ECMA-335_6th_edition_june_2012.pdf).
The proposed native dependency therefore needs an explicit neoCLR representation.
Reject unresolved executable CLI export; emitting a generic delegate parameter or
an Object-array method would change the method contract. A closed wrapper can
have an ordinary CLI signature only if all component types and body semantics
are otherwise representable. Ordinary Raven/.NET behavior stays unchanged.

## Evidence and next bounded work

Source inspection establishes the fixed-list limitations above. Minimal probes
using the existing local net10.0 rvnc driver confirm that a conventional explicit
method analyzes as a class library and colon-signature syntax produces parser
diagnostics. Native probes without a selected native core also reject the `func`
constraint; that is a configuration limitation, not evidence that integrated
native constraints are missing. The driver was not rebuilt and its binary is not
claimed to match inspected HEAD. Sources, commands, hash and complete outputs are
retained in [the investigation receipt](../validation/generic-function-methods-20261029/README.md).

Next, implement a **general metadata/materialization vertical slice**: an SDK-authored
`Forwarder<T>.Invoke` referencing TypeParameter(0), closed with zero-, one- and
two-parameter function signatures and both value/no-result profiles. Verify ordinary
closed reflection and invocation, open/partial reflection, invalid constraints,
and provider/consumer round trips. Then expose Raven syntax and argument-pack
binding against that contract. A source-only expansion with no exported dependency
would not satisfy the author’s open-generics requirement.

Follow with typed forwarding, explicit boxed conversion, semantic-model/editor
tests and all three execution modes. Test stale-cache separation between different
T shapes, result forwarding, captured receiver retention, missing adapters,
unsupported modes and body expansion before root planning. Broad async, byref and
virtual support need separate qualification. No benchmark is needed to prove the
metadata feature; measure allocation/call overhead when choosing a forwarding
implementation on performance grounds.

## Author-directed initial delivery profile (2026-10-29)

The acceptance target is a library `Signal<T>` / `SignalSource<T>` (or corresponding
Event names) that is useful wherever ordinary .NET events are used. A property
exposes subscriptions; the source retains raising authority. Compound assignment
operators may provide `+=`/`-=` syntax through Raven's ordinary operator support.
No event metadata or special signal type is introduced. An end-to-end unit-returning
notification consumer is the first qualification target; reactive operators and
Task handling are separate work.

Use the straightforward interop implementation: owned `object?[]` arguments,
reflection invocation, checked result adaptation and normal generic return-path
checking. Do not expand this slice into return-shape constraints, allocation
optimizations or Task integration. For arbitrary open T, the expected result is
opaque `ReturnType<T>`; returning a matching helper result is valid, but falling
through cannot establish a result for every possible construction. The distinction
between native no-result and inhabited source unit remains relevant to adaptation.

The author selected return projection as a general capability. The implementation
uses a native `FunctionReturn` type expression, retaining its operand until generic
substitution. The initial Raven spelling under development is `ReturnType<T>`;
this is a type operator on the neoCLR target, not a nominal wrapper. The metadata
SDK's `SignatureType.FunctionReturn` and `FunctionReturnTypeInfo` describe the same
operation. Open signatures retain the projection; a concrete function operand
resolves directly to its result. .NET-target admission is not implied.

Implementation is in progress. SDK dependent-signature and result-projection
roundtrips pass, and runtime/native-codegen compilation passes. Raven's existing
11 structural-function tests passed before compiler changes. New projection
semantic tests, projected method syntax, typed invocation helper, guest reflection
API, and executable signal qualification are not yet claimed complete.

The author subsequently selected `FunctionReflection` for the typed overload,
placing it beside the existing dynamic reflection method rather than introducing
a separate helpers class. Explicit generic arity selects the typed contract.

Execution qualification: the signal consumers pass in the interpreter. The native
compiler builds and shares signature specialization, but the signal JIT probe stops
at unsupported boxing of inhabited unit (`Void`). JIT/AOT execution of this library
is not qualified; unit boxing/unboxing support remains follow-up work.
