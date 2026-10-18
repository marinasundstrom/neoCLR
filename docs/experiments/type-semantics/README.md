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
Reference records are still rejected by native emission, with an inaccurate
record-struct diagnostic. ValueType already exists and derives from Object, but
value-type ancestry metadata and assignability need further work. Record synthesized
equality/hash/display support is also separate from positional storage. These gaps
must not be described as deliberate semantic differences.
