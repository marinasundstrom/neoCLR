# Union conventions and source integration

Use standard Raven `union` declarations for class-library unions, including unions
with members. Prefer case construction, patterns and propagation. Per-case `Is*`
properties and checked `Get*` accessors are not required conventions.

## Current Raven library contract

`Option<T>`, `Result<T,E>` and `TaskOutcome<T>` use the generated contract.
Option and Result follow Raven.Core's source declarations and generated
union contract. Other migrated unions include networking, HTTP, storage, stream,
JSON and reflection errors. Carriers implement `IUnion.Value`; generated
`HasValue` and typed `TryGetValue` distinguish active cases from default carriers.
`Value` boxes the active case. An unsuccessful case extraction clears its output to the payload default.
This follows Raven's shared synthesized-body contract; `out(true)` guarantees
initialization on success, not preservation on failure. Older legacy bridge
fragments preserved outputs; rebuild callers and libraries together.
The [Option and Result API guide](raven-union-api.md) explains propagation and defaults.

A union's discriminator and payload belong to its ordinary carrier representation.
The runtime executes imported types and methods without a union opcode or a Raven
metadata dependency. Ordinary value-copy and reference-aliasing rules apply.

## Compiler metadata boundary

Raven's `RavenUnionCaseAttribute` and `RavenUnionCompanionAttribute` associate
source cases with carriers in CLI metadata. The bridge validates and preserves
these associations for separate compilation. Generic carriers can use a nongeneric
companion containing both generic payload cases and nongeneric empty cases.
Actual CLI arity controls metadata construction; an empty case does not become
generic merely because its carrier has type parameters.

Compared with CLI custom attributes and ordinary generic types, this keeps source
association at the import boundary and execution in fields, calls, boxing and
interfaces. The benefit is standard source syntax without a compiler-specific VM
ABI; the cost is a provisional adapter that must track Raven's emitted convention.
No richer platform case registry is committed. See the
[union integration evidence](experiments/http-error-unions/README.md).

## Historical bootstrap boundary

The separate Neo bootstrap profile retains frozen manual carriers under
[runtime/legacy](../runtime/legacy/README.md). Its predicate/accessor APIs are not
the Raven API or a model for new unions. Do not mix bootstrap and Raven profile artifacts.

The Raven runtime source audit found no remaining manual union carriers after the
TaskOutcome migration. Standalone errors such as OverflowError are ordinary value
types; JsonValue uses a polymorphic class hierarchy. Neither is a disguised union
carrier requiring this migration. A subsequent mapped HTTP check found the native
reflection snapshot adapter still constructing the old Option layout. That boundary
now validates and constructs the selected profile’s layout; accessor regression
checks cover both profiles.

## Composition

[Option and Result operators](raven-outcome-operators.md) are ordinary library
methods with generic callbacks. Expected failures stay explicit return values;
terminal runtime faults and task cancellation retain their own contracts.
