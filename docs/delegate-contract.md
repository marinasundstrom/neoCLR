# Typed delegates: contract and groundwork

Status: design checkpoint, 2026-09-08. Guest delegate metadata, binding and invocation
are **not implemented**. The executable examples below use existing interfaces.
This contract narrows the first implementation; nullable metadata remains separate.

## Comparison with .NET

The pinned [probe](experiments/delegates-dotnet/Program.cs) checks static and closed
generic targets, shared class receivers, copied struct receivers, virtual selection,
private method binding, reference parameter contracts, equality, multicast and GC.
It also checks compiler rejection of an incompatible output signature and a lambda
capturing a ref parameter. These are observations of SDK 10.0.100/net10.0, not claims
about every possible delegate construction API or implementation strategy.

.NET provides nominal typed delegates and supports static and instance targets.
Its delegate API also provides target/method inspection and invocation lists.
See [System.Delegate](https://learn.microsoft.com/en-us/dotnet/api/system.delegate?view=net-10.0).
C# method-group binding to a struct instance boxes a copy; our probe verifies that
later changes to the original struct do not change the bound receiver. This is a
language binding rule to compare with neoCLR's explicit reference model, rather
than a reason to introduce an inherent value/reference classification of classes.
See [C# expressions specification](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/expressions).
Sources consulted 2026-09-08.

neoCLR should reuse typed invocation and shared managed receiver behavior. It should
avoid an implicit receiver copy: users explicitly choose an adapter containing a
value if a snapshot is wanted. This preserves identity and makes lifetime decisions
visible, at the cost of restricting which receivers can initially be retained.

## First implementation contract

A delegate declaration defines a nominal callable type with an Invoke signature.
Two declarations with identical signatures are still distinct types. The delegate
is an immutable callable value under the existing addressing model; a managed
reference to that value is a separate choice. No mandatory Object base or intrinsic
reference-type bit is required. This differs from CLR delegate classes and will
require dedicated runtime representation and tooling.

Its internal binding has two alternatives:

- A concrete, closed free/static IL function, with no receiver.
- A concrete IL instance implementation plus a valid heap-backed managed receiver.

The absence of a receiver in the static alternative is structural, not a nullable
reference slot. There is no default callable: default(D) must fault/reject until an
explicit nullable or optional wrapper is used. A zeroed callable is never valid.
Closed generic free/static targets use existing method type arguments; open generic
methods cannot be invoked. Native callbacks, open-instance binding and legacy
by-value receivers are outside the first slice.

Binding must check the full signature, including return type, readonly, output and
conditional output contracts. Start with exact compatibility, without variance or
implicit adapters. A readonly receiver cannot be bound to a mutating method.
Access is checked at binding: a legitimate creator can hand out a callback to a
private method, without making the method publicly accessible. A guest cannot forge
that capability through arbitrary serialized payloads. Runtime checks must cover
unverified IL and artifacts as well as compiler-generated code.

For virtual/interface targets, select the implementation through the existing
runtime dispatch rules and retain the original managed receiver and complete owner.
Do not turn a base view into a copied base value. Validate explicit/default interface
implementations and overrides before accepting this binding path. Incomplete
construction must not become publishable through a delegate.

## Lifetime and storage

Copying a delegate copies its binding and shares its receiver; it does not copy the
receiver's contents. A bound heap receiver keeps the complete allocation alive,
including when the reference points to an interior location. Invocation must retain
existing address validity and readonly checks. Tracing must see receivers held in
locals, fields, arrays, erased values and other delegate-containing composites.

The initial stored delegate must not capture a frame-backed receiver. Current
neoCLR composite storage rejects frame references even when the containing adapter
is itself local. Supporting a scoped delegate therefore needs more than another GC
root: aggregate provenance, return/escape checks and storage contracts must evolve.
Silently copying or promoting the receiver would change the user's chosen semantics.

This restriction does not prevent passing a frame value as an ordinary reference
argument to a static callback. Existing direct interface reference views also work
within the owner's lifetime. Scoped frame-bound delegates are a later feature, with
positive nested-call tests and negative return, field and heap-storage tests required.

## Representation and instruction choices

| Approach | Benefit | Cost / decision |
| --- | --- | --- |
| CLI-style ldftn/ldvirtftn followed by delegate construction | Familiar instruction pattern | Conventional construction exposes pointer/target conventions that need a new safe interpretation here. |
| Compiler-generated interface adapters | Works today with existing dispatch and GC | Boilerplate and no shared runtime delegate contract; useful comparison baseline. |
| Checked binding operation with a typed nominal Invoke contract | Validates receiver, permissions and closed signature together; avoids raw code pointers | New runtime/IL encoding and tool support. Preferred for the first implementation; spelling is provisional. |

The CLR [ldvirtftn instruction](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.ldvirtftn?view=net-10.0)
resolves a virtual implementation and pushes its function pointer. Reusing its name
for a different opaque binding operation would obscure that distinction. Prefer
ordinary call-like Invoke behavior where it fits, but settle the encoding with
verifier and artifact tests before adding it to the documented grammar. No performance
or allocation advantage is claimed without measurements.

## Runtime audit and acceptance work

| Existing area | Required delegate work |
| --- | --- |
| TypeDef / composed Type / FunctionRef | Nominal callable kind, canonical Invoke contracts and closed member identity; reject invalid metadata. |
| Value and ensure_heap_references | Sealed binding representation, target validation, copying and default rejection. |
| GC tracing | Traverse retained receivers from every supported storage position, with collection-pressure and release tests. |
| VM call and virtual dispatch | Reuse argument, readonly/output, receiver and construction checks; preserve source traces and stepping. |
| VM reference-return checks | Keep frame capture rejected initially; recursive provenance is required before relaxing this. |
| Reachability | Binding retains executable code even before invocation; record closed generic targets and indirect invocation explicitly. |
| Input/artifact loading | Resolve trusted metadata, never deserialize a guest-selected live address or unchecked invocation capability. |
| Debugger and reflection | Show signature, closed member and target identity without exposing raw native addresses. |

Guest reflection needs care: a free function does not have a declaring Type, and
current MethodInfo does not describe open method generic parameters. Do not invent
an Object owner or invalid reference to fit Delegate.Method/Target. Host/debugger
inspection can precede a separate FunctionInfo/optional target API decision.

The next slice should implement static and heap-bound delegates end to end: metadata,
checked binding/invocation, GC, closed reachability, debugger/source mapping, and a
small Neo declaration/binding projection with a library callback consumer. Acceptance
must include malformed signatures, readonly/output mismatches, inaccessible binding,
incomplete receivers, frame-target rejection, closed generic targets, virtual/interface
selection and collection under pressure. Existing adapter tests are evidence for the
foundation, not substitutes for these future delegate tests.

## Later decisions

Single-target equality could compare nominal closed delegate type, exact closed
method and receiver reference identity (including interior path). ReferenceEquals on
D& would separately compare delegate storage locations. This is a proposed API,
not implemented equality behavior.

The .NET probe establishes a multicast baseline: invocation order, last return value
and stopping on exception. Combining/removing lists, empty-list representation,
variance, Delegate/MulticastDelegate hierarchy and guest introspection remain later
contracts. Lambda capture lowering belongs in Neo after runtime lifetime rules are
ready. Pinning, native function pointers and callback trampolines remain interop work.

## Run the evidence

From the repository root, with the pinned .NET SDK installed:

```sh
python3 docs/experiments/delegates-dotnet/verify.py
cargo test --locked --test callback_groundwork
cargo run --locked -- run examples/source/callback-groundwork.neo
```

The Neo sample prints 42 three times and returns 42. It uses interface adapters to
show a static forwarding callback, a direct frame reference and an adapter retaining
a heap receiver. The tests round-trip the artifact and force repeated collections
with a four-object heap limit. Negative tests reject storing a frame target and
returning a dead frame interface view; another test makes receiver copying explicit.
These checks passed on 2026-09-08. They establish no new opcode or artifact contract.
