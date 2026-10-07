# Next AOT slice: value types and members

**2026-10-07 — producer inventory and implementation plan, not native type support.**
After Raven-to-native Hello World, the author selects types and members, especially
value types enabling Result/Some union samples and control flow. These small Raven
sources establish what the current producer actually emits before selecting an ABI.

[Counter](counter.rvn) constructs a value, mutates it through a member and branches
on its returned Int32. [Choice](choice.rvn) uses an ordinary union with Some(Int32)
and None, then extracts the payload with a union pattern. Both compile to neoCLR
metadata and run successfully under the interpreter. The current AOT profile rejects
both because declared types are unsupported; neither is presented as a native demo.
[Inventory](inventory.json) records exact commands, compiler/runtime revisions,
artifact/input hashes and instruction sets. This reuses the pinned native bundle
from the [clean bootstrap qualification](../extended-cli-metadata/clean-bootstrap-reproduction-2026-10-07.md);
it does not establish support in an arbitrary Raven checkout.

## Observed requirements

Counter is a Record value with an Int32 backing field. Its emitted members include
constructor, property getter/setter and Bump. Calls use managed by-reference receivers;
Main takes the local's address. Required lowering includes value locals and copying,
construction, local addresses, field loads/stores, initialization, member resolution,
no-result member returns, Boolean comparison results and branching. The existing
scalar branch backend alone is insufficient.

Choice and its Some/None cases are Record values: the carrier has a tag and nested
payload fields. Pattern matching emits a member call with an out parameter followed
by a branch, then payload access. The producer also emits supporting attribute classes
and generated members, including boxing, virtual calls and string-related behavior.
The inspected Counter has one type/five methods; Choice has six types/twenty methods. Native support must account for those metadata declarations
and all generated bodies; compiling only the pattern's branch instructions would
not establish general union support. The full native Runtime reference/ownership
configuration is required by this probe. An initial attempt through Hello World's
host primitive bootstrap rejected generated union ToString with
`Native emission does not support union body ToString: optional/expanded arguments.`
The explicit native dependency configuration succeeds; no compiler fix is claimed.

## Proposed bounded sequence

1. Compile Counter: define value storage/copy and managed receiver rules; lower its
   actual constructor, fields and members. Compare original/copy mutation, normal
   return and Fault behavior against the interpreter, not only the final number.
2. Extend to nested value payloads, tag tests and out-parameter initialization.
   Exercise Some/None first and both branches, then a small Result success/error
   consumer. Do not encode a union by recognizing its source name. Resolve actual
   emitted definitions and field contracts.
3. Admit reference-bearing values after defining ownership/root descriptors and
   Fault cleanup. Select and test a native heap strategy independently of JIT.

These are next-slice boundaries, not implemented capabilities. General library
compilation and metadata retention remain explicit work. Trimming is still a later
milestone; the generated union members expose a real dependency/coverage question
rather than permission to silently discard unsupported code.
