# Readonly reference signatures in storage and returns

Status: implemented. Binding immutability remains a Neo/compiler feature; readonly
reference signatures restrict target access without making the containing local
write-once. This extends the [readonly input and receiver contracts](readonly-parameters.md).

## Neo and IL

```swift
record Counter(Age: int)
record Holder(Value: readonly Counter&)

func Observe(readonly counter: Counter&) -> readonly Counter& {
    return counter
}

func Main() -> int {
    let counter = new Counter(40)
    let view: readonly Counter& = Observe(counter)
    counter.Age = 42
    return view.Age
}
```

Reference access remains automatic. readonly T& is accepted in Neo type positions,
including locals, results, record fields and generic arguments. Existing readonly
parameter modifiers and readonly func remain supported. A result's readonly type
flows into an inferred local. No defensive receiver copy or heap promotion is added.

IL uses .local readonly Counter& view and -> readonly Counter&. The recursive
ReadOnlyByRef signature also supports array element and closed generic signatures.
For example, readonly Int32&[] is an array of readonly references; (readonly Int32&[])&
is a writable reference to that array. IL parentheses disambiguate nested signatures.
Neo's explicit array-type grammar remains its existing limited subset; it can carry
these array element signatures through inferred arrays and generic APIs.

Calls to declarations using the existing readonly parameter modifier keep their
ordinary parameter spelling, e.g. Observe(Counter&). Parameter indices and receiver
flags remain accepted declaration metadata; Function.argument_types derives the
qualified argument signatures used for execution and verification. Nested access
information is retained in the Type tree, not new local/return index side tables.

## Checked boundaries

Storing or returning a writable reference into a readonly contract narrows that copy.
The original alias remains writable. A restricted reference cannot enter a writable
local, call argument or return. Runtime checks apply even if verification is skipped;
a failed store leaves the destination unchanged. Value reads create ordinary copies,
not new writable aliases.

Readonly permission survives fields, arrays, erasure and interface views. Mutable
containers retain exact element signatures: ArrayList<Counter&> and
ArrayList<readonly Counter&> are distinct constructed signatures with no implicit
container conversion. The same rule applies to arrays. Loading a stored reference
preserves its own permission rather than inheriting a containing object's restriction.

Readonly locals can be replaced with compatible references by IL; let rebinding
restrictions are language-level. Output operations still require writable access.
Nested managed references and managed-reference native layouts remain unsupported.
Reference identity and referent runtime type are independent of access permission.

The verifier reads declared permissions from locals and call results and rejects
writable boundary mismatches. Compatible evaluation-stack writable/readonly reference
joins become readonly. Joins involving incompatible shapes or provenance-bearing
address states remain conservative; this is not complete alias or escape analysis.
Runtime current-frame return checks remain authoritative after references pass through
locals. A readonly declaration never grants additional lifetime.

## Reflection and migration

System.Type.IsReadOnly identifies a readonly managed-reference signature; IsByRef is
true for both permissions. GetElementType returns the referent type. Closed signature
identities preserve nested permissions. GetType on an addressed object still describes
the referent, not the capability used to access it.

ParameterInfo.IsReadOnly handles both existing input metadata and qualified generic
parameters; ParameterType exposes the qualified input signature. MethodInfo.ReturnType
and FieldInfo.FieldType preserve access. MethodInfo.IsReadOnly continues to describe
the receiver contract. Debugger reference displays retain their readonly marker.

This is a preview compatibility change. Restricted references formerly could pass
through unqualified T& locals/returns and fault at a later write. They now fail at the
offending boundary. Declare readonly destinations/results where required.
Value.ty and qualified type identities now retain readonly reference access.
External System artifacts must be reassembled for Type.IsReadOnly; format 5 remains
provisional and artifacts using ReadOnlyByRef require this runtime.

The [old gap probe](experiments/reference-contracts/return-gap.neoil) now intentionally
fails verification and faults at Observe's return if executed without verification.
Its corrected end-to-end counterpart is [readonly-storage.neo](../examples/source/readonly-storage.neo).

```sh
cargo run --locked -- run examples/source/readonly-storage.neo
cargo test --locked --test readonly_storage --test readonly_references
```

The example returns 42. Regressions cover source/artifact round trips, fields and
generic collections, array element permissions, readonly boundary failures, mutable
IL local replacement, conservative joins, reflection and frame escapes.

## Design evidence

The [reference/storage investigation](reference-storage-contract.md) compares a
recursive signature model with compiler-only annotations and more parallel flags.
The existing .NET comparison remains applicable: preserving familiar reference access
behavior does not require adopting every CLR metadata accommodation. Costs include
signature migration, substitution/identity work and runtime checks. No JIT speedup or
global immutability guarantee is claimed.
