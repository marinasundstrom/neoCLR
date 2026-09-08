# Delegate contract and CLR comparison

Status: first runtime/IL and Neo implementation, 2026-09-08. See the
[delegate guide](delegates.md) for the implemented surface and commands. The earlier
[interface-adapter example](../examples/source/callback-groundwork.neo) remains
groundwork evidence, not a delegate implementation.

## Reuse familiar behavior

Delegates are the platform's shared callable abstraction. Keep CLR/.NET behavior
unless a documented simplification fits neoCLR's existing model. Functions remain
definitions; Neo lambdas build on delegates and captured environments.

The pinned [SDK 10.0.100/net10.0 probe](experiments/delegates-dotnet/Program.cs) checks
static and closed generic targets, shared class receivers, copied struct receivers,
virtual selection, private binding, reference contracts, equality, multicast and GC.
It also checks rejection of Void generic arguments, an incompatible output signature
and a lambda capturing a ref parameter. These observations do not describe every
possible .NET delegate construction API.

Primary sources consulted 2026-09-08:

- [System.Delegate](https://learn.microsoft.com/en-us/dotnet/api/system.delegate?view=net-10.0):
  nominal callable types, method/target information and invocation lists.
- [C# expressions specification](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/language-specification/expressions):
  delegate creation and value receiver binding. The probe confirms a copied boxed
  struct receiver, unlike a shared class receiver.
- [Func<T,TResult>](https://learn.microsoft.com/en-us/dotnet/api/system.func-2?view=net-10.0):
  familiar input/result order; neoCLR currently omits variance.
- [Type.MakeGenericType](https://learn.microsoft.com/en-us/dotnet/api/system.type.makegenerictype?view=net-10.0):
  .NET rejects Void and byref generic arguments. neoCLR's existing composed type
  model allows them, so Func<T,Void> replaces Action<T>. The benefit is one callback
  family; the cost is an explicit API migration and no direct CLR signature match.
- [C# lambda expressions](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/operators/lambda-expressions):
  captured outer variables provide the behavioral baseline for closures.
  Compiler-generated shared environments are the chosen direction, not implemented
  lambda support.
- [ldvirtftn](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.ldvirtftn?view=net-10.0):
  the CLI resolves a virtual implementation and produces a function pointer.

## Binding and invocation

A fieldless nominal delegate definition declares exactly one bodyless public instance
Invoke member. Generic owner parameters are supported. The representation has no
mandatory Object base, fields, inheritance, layout controls or user-supplied body.
It is a callable value; D& separately denotes managed delegate storage.

The internal binding is a closed FunctionRef plus either no receiver or a heap-backed
managed receiver. No-receiver is an internal alternative, not an invalid or nullable
address. default(D), including defaults of records containing D, is rejected.

Signature checks include returns, parameter types, readonly, out and conditional-out
contracts. There are no implicit signature adapters. Access is checked at binding;
invocation does not reapply the eventual consumer's access to a private target.
Serialized IL contains symbolic binding instructions, never forged live capabilities.
Module-reference and visibility checks apply to binding dependencies too.

Virtual/class and interface bindings resolve the implementation using existing
dispatch rules, including explicit/default interface bodies, then retain the original
owner through its appropriate view. Constructors, native imports, runtime InternalCall
targets and by-value instance implementations are excluded. Invocation reuses normal
frames and their argument, output-completion and reference-validity checks.

| Instruction choice | Outcome |
| --- | --- |
| CLI ldftn/ldvirtftn and pointer-shaped construction | Not adopted: a raw code-pointer convention is unnecessary for a checked managed binding. |
| Compiler-only interface adapters | Useful existing baseline, but lacks a common runtime callable contract. |
| delegate.bind D = Target(...) | Implemented fused operation: validate type, method and optional managed receiver together. New encoding/tool support is the cost. |
| call / callvirt instance D::Invoke(...) | Reused ordinary call pattern; runtime transfers into the bound method. No separate invocation opcode. |

No allocation or performance advantage is claimed without measurements.

## Lifetime, equality and tooling

A delegate copy shares its receiver. Binding never implicitly boxes, copies or
promotes an instance. Heap targets, including interior references, keep the complete
allocation alive through GC. Tracing follows delegates inside ordinary composites.

Frame-bound receivers remain rejected, even for a delegate stored locally. Supporting
them requires aggregate provenance and escape rules, not only GC tracing. Ordinary
reference arguments to a static callback can point into a caller frame and retain
the existing return/escape rules. An unpublished construction receiver cannot be bound.

Single-target equality compares the nominal closed delegate type, exact closed method
and receiver location, independently of readonly view permissions. Equality of D&
storage remains reference identity. There is no new guest Equals/GetHashCode API.

Reachability reports separate retained binding targets from typed indirect Invoke
sites. Binding targets include possible virtual/interface implementations. Invoke
alone cannot serve as a concrete graph root. Debugger snapshots show the closed target
and its receiver; calls use normal source frames and step/next behavior.

Host input schemas reject live delegates, including erased delegate payloads. Host
results can contain opaque bindings for inspection, but cannot be imported as callable
capabilities into another execution. Guest reflection can inspect the nominal Invoke
signature through Type's existing APIs; Delegate.Method/Target needs a later design,
particularly because free functions have no declaring Type.

## Validation and remaining boundaries

Regression coverage includes artifact round trips, closed generic targets, nominal
mismatch without verification, readonly/output contracts, private access transfer,
constructor/frame rejection, virtual/explicit/default interface dispatch, shared
receiver copies, GC pressure, debugger stepping, Func<Void> and ForEach over frame
and heap arrays. The pinned .NET probe supplies the external behavioral comparison.

Multicast is not implemented. Its .NET baseline is invocation order, last return
value and stopping on exception; combining/removing lists and empty-list representation
need a separate slice. Scoped captures, variance, native trampolines and guest delegate
introspection also remain separate. Nullable metadata is not introduced here.

New Delegate representation and delegate.bind artifacts require this runtime; existing
artifacts remain compatible. Rust consumers exhaustively matching Representation,
Instruction or Value need new cases. Reachability adds binding/indirect-site fields.
Neo reserves delegate; its standalone grammar is synchronized with the current guide.

## Closure lowering

Implemented in Neo on 2026-09-08. The C# expressions specification (§12.22.6)
provides the language baseline: lambdas capture outer variables, multiple delegates
can share a variable, and its lifetime extends beyond the declaring call. C# rejects
capturing ref/out/in parameters. The pinned .NET probe now also checks shared mutation,
returned captures and fresh foreach bindings. These are language behaviors, not a
requirement for a new CLI instruction or a particular environment layout.

Neo adopts shared-binding semantics with generated IL methods and managed objects,
using the existing delegate.bind, field access, heap.new and GC contracts. Each
captured binding gets a typed cell at initialization; environments reference those
cells. Early lifting preserves address identity even before a lambda is evaluated.
Only cells needed by a lambda are retained in its environment. Neo's range `for`
introduces a fresh iteration binding, matching the foreach case in the probe.

A value snapshot would be simpler but would hide subsequent outer assignments.
A scope-wide environment can reduce allocations but complicates incremental compiler
lowering and may retain unrelated captures. Per-binding cells are the preliminary
choice: explicit sharing is easy to validate, at the cost of extra allocations and
indirection. No performance improvement is claimed. A future optimization can change
layout without changing the shared-binding contract. Stack-only environments and
escape analysis are deferred; this implementation allocates captured storage even
when a lambda branch is never taken.

Unlike C# ref parameters, Neo's ordinary T& parameters are reference-valued inputs.
Capturing one therefore stores that reference in a heap cell: runtime composite-store
validation accepts heap-backed references and rejects frame-backed ones. It does not
promote the referent. Out captures and constructor-this captures are rejected by Neo;
the runtime independently prevents storing unpublished/frame-backed references.
Uninitialized captured locals are also rejected in this bounded compiler rather than
adding definite-assignment analysis or invalid default capture fields. Parameter and
local values captured by a lambda have managed storage, a deliberate language choice
that can extend their lifetime. Ordinary instance method-group binding retains its
existing no-copy/no-promotion rule.

No opcode, metadata capability or GC algorithm changes are needed. Generated methods
retain source positions; the debugger currently exposes their implementation names
and cells. Optimized capture views, debugger reconstruction, delegate equality APIs,
multicast and nullable captures need separate validation. Runtime limits account for
these extra objects. Regression tests cover shared addresses, generic/nested helpers,
iteration freshness, heap/frame references, collection of closure cycles, artifact
roundtrips and stepping through original source.
