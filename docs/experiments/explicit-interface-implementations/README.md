# Explicit application interface implementations

Development slice, 2026-09-27; not part of Preview 10. This closes the bounded Raven
import gap for ordinary instance methods on non-generic application classes and
application interfaces. Existing native support is broader than this source slice.

`Main.rvn` uses two same-named interface methods and an independent public method.
The private implementations read the same object's state; a void explicit method
mutates that state. An explicit-only class remains callable through its interface.
Reflection preserves qualified source names and private visibility, while runtime
IsVirtual is false for explicit bodies: class virtual slots and interface mappings
are separate in neoCLR. This is a documented difference from CLI method flags.

Run with the development bridge/runtime and matching System/reference:

```sh
python3 docs/experiments/explicit-interface-implementations/verify.py \
  --runtime /path/to/neoclr --bridge /path/to/Probe.dll \
  --system /path/to/System.neoil --reference /path/to/NeoCLR.CoreProbe.dll \
  --evidence docs/experiments/explicit-interface-implementations/validation.json
```

The source checks reject ordinary access to an explicit-only member and a mapping
without declared interface conformance. `cargo test --test explicit_interfaces
--test interface_helpers` covers the new nominal receiver alongside existing native
mapping, access, lifetime, inheritance and invalid-metadata checks. A native case
also rejects direct private calls, public mapping bodies, duplicate mapping targets
and undeclared interfaces, and checks selected bodies in the reachable call graph.

The bridge validates CLI Private/Virtual/Final/NewSlot body flags, ordinary IL bodies,
resolved declaration identity, accessibility and exact signatures. It translates
MethodImpl metadata to `.override` identities while keeping bodies private and
outside class virtual slots. No Raven compiler behavior or Runtime Contract setting
changes. Explicit accessors, value-type bodies, generic application interfaces,
external core-library contracts and derived interface replacements remain outside
this slice; the broader native tests are not Raven admission evidence.

See [contract and .NET comparison](../../explicit-interfaces.md) and the
[authoritative limitations](../../tracking/runtime-language.md#interface-limitations--development-checkpoint-2026-09-27).


Validation recorded 2026-09-27: 15 focused native tests pass (13 explicit-interface
cases and two helper regressions). The Raven consumer executes successfully and
both negative source cases are rejected; [hashes](validation.json) pin its sources
and frozen artifacts. Earlier typed-stack verification of the consumer passed;
the final display-name fix is verified by execution without repeating whole-System
verification. The API snapshot check passes on an exported staged-source snapshot,
excluding unrelated entry-point edits in the shared checkout. Its regenerated
reference DLL is byte-identical. No full suite or website build was run. Raven
integration documentation is committed separately as `f220806a8`; compiler code
is unchanged by this slice.
