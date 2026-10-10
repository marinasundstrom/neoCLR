# Native init accessors (development)

The author requested native `init` accessor support on 2026-10-10, following the
positional KeyValuePair record slice. The goal is initialization syntax for
properties that ordinary callers cannot subsequently assign, including across
native library references. This does not select full native record equality,
hashing, formatting, or a runtime object-freezing mechanism.

## Contract and comparison

The shipped C# `init` feature permits assignments in its language-defined construction
phase. Its CLI representation is a setter with a required return modifier naming
`System.Runtime.CompilerServices.IsExternalInit`. The modifier distinguishes the
setter contract for compilers; it does not by itself make reflection invocation
impossible. Sources consulted 2026-10-10: [C# reference](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/keywords/init)
and [C# 9 design specification](https://github.com/dotnet/csharplang/blob/main/proposals/csharp-9.0/init.md).
The latter is historical design rationale; executable comparison is in
`NativeInitAccessorChecks`, including reflection invocation after allocation.

neoCLR uses a native property `init_only` Boolean instead of requiring a .NET marker
type in its semantic model. Missing means false. A true value requires an instance
setter. Both native metadata containers preserve it; unknown-field rejection in
older readers prevents interpreting the new contract as an ordinary setter.
CLI transport/reference projections retain the standard required return modifier.
This is a development format extension: use matching compiler, metadata tools and
runtime. It is not a published release capability.

Raven imports the flag as `MethodKind.InitOnly`, so its existing object/with
initializer restrictions apply across library boundaries. The native adapter admits
instance automatic and implemented init accessors, including positional record
components, behind an explicit portable emission capability. Syntax, editor grammar
and ordinary .NET compilation remain unchanged. This slice reuses Raven's existing
language rules; it does not promise every C# construction-phase context or full
native `with`/record support.

The verifier permits an associated init setter to write readonly fields declared
by its own type, just as the declaring constructor can. Ordinary methods retain
the readonly restriction. Runtime invocation remains an ordinary method call:
reflection, a serializer using reflection, or authored native instructions can call
an init setter after construction. This is deliberately a compiler-enforced usage
contract, not a security boundary or deep immutability guarantee. Existing runtime
PropertyInfo getter/setter invocation continues to follow that policy; no new
runtime-reflection query API is introduced in this slice.

A runtime-enforced construction token/freeze phase would prevent more bypasses but
would need rules for aliases, constructor calls, object/with initializers, serializers,
and exceptional initialization. It would change behavior and add runtime state.
Keeping properties getter-only avoids that cost but blocks the requested initializer
syntax. The chosen contract preserves familiar ergonomics without adding per-object
construction state; it makes no performance claim. General library-only workarounds
cannot preserve an accessor's source restriction across independent compilers.

## Ownership and validation

- Metadata tooling: `PropertyDefinition.IsInitOnly`, `PropertyInfo.IsInitOnly`,
  and optional `isInitOnly` authoring arguments. Accessor associations are validated.
  CLI flags are not repurposed; the semantic fact has its own native field.
- Raven: declaration admission, lowering, metadata emission, native symbol import
  and source diagnostics. Ordinary assignment must still fail after initialization.
- Runtime: validates metadata shape and readonly-field privileges, using resolved
  declaring/accessor identities. No special init call instruction is needed by the
  interpreter or native backend.
- Focused tests: metadata authoring, both containers, CLI modifier and reflection
  comparison, invalid static/missing setters, readonly writes, and Rust property
  round trips. The separate-library Raven consumer lives in
  [the experiment](experiments/init-accessors/).

The native flag replaces the previous getter-only record bridge limitation. The
CLI modreq is a transport projection, not the future native ABI. Record identity,
generated equality/hash/display methods, concurrent publication and runtime
suspension are separate work.
