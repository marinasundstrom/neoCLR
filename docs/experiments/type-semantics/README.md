# Reference/value semantics investigation — 2026-10-18

The author requires .NET-compatible compiler mapping and roughly corresponding
metadata for classes, structs, reference records and record structs, unless a
concrete divergence is justified. This is an active compatibility repair, not a
new semantic contract.

## Mutable positional record structs

Preview 16 accepts positional `val` components but rejects `var` components because
native emission admits only init-only setters. The repaired emitter admits ordinary
property setters as well. `Main.rvn` checks class aliasing, struct and mutable
record-struct copying, independent boxing/unboxing, value-type classification,
class init accessors and immutable positional record initialization. All checks
pass with identical output across interpreter, JIT and AOT; native GC is enabled.

[Commands/hashes/results](mutable-three-mode-results.json.gz) use a source-built
compiler on Raven codex/neoclr, source-built libraries and development runtimes on
macOS ARM64. The focused compiler baseline has 34 passing tests; adding the existing
init-accessor plan checks gives 36 passing tests. No ordinary .NET emission or
Runtime Contract setting changes. The installed Preview 16 bundle is untouched.

Run `verify.py` with `--library`, `--compiler`, `--runtime`, `--backend`, `--host`,
and a new `--output` directory, as in the JSON record experiment. The host supplies
services to an already compiled AOT object; it does not JIT the AOT application.

## Remaining investigation

[Minimal Preview 16 rejections](preview16-rejections.json) retain separate reference
record and mutable value-record repros. Mutable setters are repaired in this slice.
Reference-record storage is repaired in the next slice below. ValueType already exists and derives from Object, but
value-type ancestry metadata and assignability need further work. Record synthesized
equality/hash/display support is also separate from positional storage. These gaps
must not be described as deliberate semantic differences.

## Reference records and Object ancestry

The native target now explicitly admits positional reference-record storage, using
a separate portable-plan capability from value-record storage. `record` remains a
reference class. Root classes and reference records bind the imported runtime Object
as their base, and constructors perform the corresponding base initialization.
Ordinary .NET compilation and Runtime Contract configuration are unchanged. Additional
record members and synthesized equality/hash/display remain separate limitations.

`Main.rvn` additionally checks reference-record aliasing, reference classification
and `BaseType == typeof(Object)` for classes and records. The verifier asserts native
reference flags, explicit Object base metadata and init-only versus ordinary setters.
All checks agree across interpreter/JIT/AOT in
[the retained reference-record run](reference-three-mode-results.json.gz).

This exposed a missing native `TypeInfo.BaseType` service. Native snapshot lowering
now reuses the interpreter's base descriptor recipe for closed type tokens in the
compiled program; unknown/unretained handles fault. One focused recipe test covers
base identity, no base and unknown type. No new guest signature or reflection
invocation permission is added. ValueType ancestry is the next slice.
