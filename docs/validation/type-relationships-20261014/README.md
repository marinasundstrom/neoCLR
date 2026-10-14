# Type relationship qualification — 2026-10-14

Worktree from main: `codex/typeinfo-assignability`. Development API only; no release,
SDK installation or website publication in this task.

- Six managed cases pass in interpreter, JIT and AOT on macOS ARM64, using a
  rebuilt Preview 15 library bundle and the source-built runtime/backend.
- Three shared-runtime unit tests cover ancestry, interface closure, value identity,
  boxing, pointer/byref boundaries, arrays, generic identity and Function signatures.
- Five native descriptor-lowering tests pass, including malformed relationship
  service contract rejection.
- The retained null-arguments fixture fails with RAV1503/RAV1509 at all three calls;
  nullable instance testing passes with null returning false.
- Aggregate API fingerprint/inventory and four native snapshot checks pass. The four
  rendered member pages retain declaration source links and correct nullability.
- Windows was not executed for this platform-neutral API slice.

Run the managed suite with `scripts/test-runtime-library.py --suite type-relations`
using matching `--bundle`, `--runtime`, `--aot` and a fresh `--output`; select
`--mode interpreter --mode jit --mode native`. Runtime tests use
`cargo test -p neoclr --lib type_relationships`; lowering checks use
`cargo test -p neoclr-codegen --lib reflection_metadata::tests`.

The additional native-source Function consumer could not compile with the qualified
SDK. [Design and exact limit](../../design/type-relationships.md#validation-limits).
No claim is made that those compiler paths were repaired by this API change.

## Consumer adoption

[Results](consumers.json): JSON passes AOT, interpreter and JIT. The typed-construction
and acquisition fixtures pass in interpreter mode. The optimized runtime was used for
the full JSON corpus; its debug run was cancelled and is not counted as a failure.

JSON's node-type admission now uses AssignableTo(JsonValue), and its concrete-node
check uses IsInstanceOfType. The existing native JSON consumer adds direct hierarchy
assertions and keeps its nested-node, collection, round-trip and wrong-kind controls.
Numeric/scalar mapping still uses exact type equality.

The generic reflection constructor now calls the public AssignableTo API. The new
`docs/experiments/reflection-members/TypeRelationships.rvnproj` tests exact creation,
Object widening and incompatible-result rejection in interpreter mode. Generic
constructor invocation is not newly qualified for JIT/AOT by this change.

The acquisition sample uses instance/assignment predicates and a class interface view;
its string equality example deliberately remains an exact-type comparison. It is
qualified in interpreter mode. Its existing DisplayName query is not bound by the
current native backend. The older reflection-members CLI harness also rejects stream
metadata before running its consumer; its full native-source variant has an existing
local-before-store compiler failure. The new focused constructor fixture avoids those
unrelated paths. These gaps remain separate work, not silently claimed as passing.
