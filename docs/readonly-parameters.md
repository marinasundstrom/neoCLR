# Readonly managed input parameters

This slice implements a runtime access restriction for managed input references.
It does not implement immutable runtime slots, readonly receiver declarations, deep
immutability, or general readonly type syntax for locals, fields and returns.

## Consumer contract

```swift
record Counter(Age: int)

func ReadAge(readonly counter: Counter&) -> int {
    return counter.Age
}

func Main() -> int {
    let local = Counter(42)
    return ReadAge(&local)
}
```

`readonly` qualifies the parameter's access contract; T& retains managed-reference
type identity. Neo still reads targets automatically. An explicit `&local` argument
can address an immutable owned binding in this readonly context, without granting
writable access to the callee. Existing writable reference arguments also work.
No implicit temporary or heap promotion is introduced. Existing block-local address
restrictions remain in place.

IL declares the same contract as `.function ReadAge(readonly Counter& counter) ->
Int32`. Calls retain the ordinary `ReadAge(Counter&)` signature: readonly does not
create a distinct overload. Function metadata records zero-based `readonly_parameters`
indices, excluding this. Only managed-reference input parameters may be marked;
outputs, native imports and InternalCall declarations do not support the modifier.
The inline parameter syntax is the initial IL projection.

On entry, the runtime narrows the received access view. The caller's original alias
is unchanged. The capability follows reference copies, returns, stored heap references,
erased payloads, interface views and derived addresses into owned fields or array
elements. Stores and initialization through a restricted view fault. Passing it as
a writable parameter, output parameter or writable receiver faults at the call boundary.
Interface implementations must match the declared readonly parameter contract exactly.

Readonly does not mean a stable snapshot. Other writable aliases can change the
same value. Loading a stored reference field or reference-array element preserves
that reference's own permission: readonly access to a Holder does not freeze a
separately referenced Counter. Copying an owned value yields an independent copy;
it does not grant a writable alias to the original. Identity and GC/frame lifetime
rules are independent of permission.

## Diagnostics and limitations

The verifier tracks readonly argument loads on the evaluation stack, including
field/element address and interface projections. It rejects direct writes and known
writable calls. It does not yet retain permission facts through arbitrary locals,
stored payloads or returned signatures, and joins remain conservative. Consequently,
some misuse can compile and must fault at runtime; verification is not a complete
readonly proof. Unverified IL receives the same runtime restriction.

A restricted reference returned under today's T& return type stays restricted at
runtime, even though its signature does not advertise that permission. General
readonly return/local/field contracts are a follow-up; public APIs should not rely
on this incomplete static projection. Existing readonly-parameter code can forward
references to other readonly inputs. No writable instance method is automatically
reclassified as readonly, and Neo does not insert defensive copies to call one.
Explicit value-receiver APIs can still consume an ordinary value copy.

`System.Reflection.ParameterInfo.IsReadOnly` reports this enforced input contract.
It is a neoCLR extension; it is not an alias for .NET ParameterInfo.IsIn. Debugger
reference displays include a readonly marker, and reachability reports include the
parameter indices. No new opcode or separate reference type was introduced.

Reassemble external System artifacts for the expanded ParameterInfo layout. JSON
format 5 remains provisional; artifacts using readonly metadata require this runtime.
Neo reserves `readonly` as a keyword. Existing writable signatures retain their behavior.

## .NET comparison and decision

Sources checked 2026-09-08:

- [Microsoft's readonly. opcode documentation](https://learn.microsoft.com/en-us/dotnet/api/system.reflection.emit.opcodes.readonly?view=net-10.0)
  describes controlled-mutability array addresses, with verification restrictions
  aimed at array type safety. Instance calls are permitted, so this is not a general
  prohibition on mutating the addressed value. Reusing that opcode name for our
  stronger contract would obscure an intentional semantic difference.
- The [C# 12 ref-readonly parameter design](https://github.com/dotnet/csharplang/blob/main/proposals/csharp-12.0/ref-readonly-parameters.md)
  describes parameter attributes/modifiers and compatibility accommodations for
  existing compilers and in/ref signatures. It is a language design document, not
  a specification of neoCLR's desired capability enforcement.

We compared three placements: compiler-only restrictions, adoption of CLR's array
prefix, and explicit parameter metadata plus runtime capability narrowing. The last
fits this slice because neoCLR executes IL without requiring verification and needs
one rule across frame/heap addresses. It adds a permission bit and runtime checks;
there is no demonstrated performance advantage. Keeping T& identity avoids a new
parallel type family, at the cost of incomplete static permission propagation until
readonly storage/return contracts are designed.

Compatibility accommodations are evidence about .NET's design, not constraints on
neoCLR. The goal is consistent APIs and behavior where appropriate, with deliberate
runtime improvements where justified. Later readonly receivers and richer signatures
must preserve the same enforcement model rather than add unrelated special cases.

### Reproducible comparison

The [comparison program](experiments/readonly-dotnet/Program.cs) is original test code.
With .NET SDK 10.0.100 installed, run from `docs/experiments/readonly-dotnet`:

```sh
dotnet run --project Probe.csproj
```

The adjacent global.json pins that SDK. On macOS ARM64 with .NET 10.0.0, observed:

```text
.NET 10.0.0
C# in: observed=0, original=0
CLR readonly. + mutator: original=1
```

The first case demonstrates C#'s defensive-copy behavior for the selected mutating
struct method. The second emits readonly. + ldelema followed by the same mutator;
it changes the original array element. This bounded result does not claim that all
.NET readonly facilities are equivalent or that arbitrary unsafe code can be made
safe. It substantiates the distinction relevant to this slice.

## neoCLR validation

```sh
cargo run --locked -- run examples/source/readonly.neo
cargo test --locked --test readonly_references
```

The Neo example prints 20, 22 and 23 and returns 42. Tests exercise source/artifact
round trips, readonly interface parameter contracts, reflection, unchanged writable
aliases, owned versus referenced fields, invalid metadata and unchecked-IL attempts
to write, initialize or forward restricted references. The .NET experiment is optional;
normal neoCLR builds and tests do not require .NET.

## C++ analogy and future native compilation

The useful analogy is const T& access, rather than a compile-time constant. The
[C++ working draft, dcl.type.cv paragraph 3](https://eel.is/c++draft/dcl.type.cv#3)
separates a const-qualified access path from the actual object's mutability through
other paths (consulted 2026-09-08). neoCLR adopts an enforceable managed capability;
it does not adopt C++ const_cast or undefined-behavior rules as an escape mechanism.

Making contracts explicit in metadata and runtime operations gives every frontend,
verifier and future JIT/AOT backend the same semantic foundation. This can make
proof obligations clearer; it does not automatically make all verification easier
or demonstrate an optimization benefit. The current verifier is deliberately partial.

Readonly permits a backend to rule out writes through that access path. It does not
prove that another alias, a callback or an interop boundary cannot change the value.
Load elimination/hoisting, immutable-memory assumptions and concurrency optimizations
require separate alias, effect, lifetime and memory-model evidence. A JIT may remove
a runtime permission check only when it proves the equivalent contract, preserving
observable Fault behavior and relevant side effects. The alias-mutation regression
test deliberately demonstrates why readonly alone is not a stable-value guarantee.
