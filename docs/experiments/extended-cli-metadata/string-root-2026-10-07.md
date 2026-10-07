# Intrinsic String over source Object — 2026-10-07

The native reader now retains an intrinsic System.String declaration over the local
Object root. The runtime admits this relationship only for intrinsic, fieldless
String and the explicitly host-selected fieldless Object definition. General
primitive inheritance and arbitrary storage-category mixing remain unsupported.

Like CLR String, this is a reference with runtime-owned immutable text storage.
NeoCLR retains UTF-8 text and its existing intrinsic receiver ABI. Protected access
uses the declared ancestry. A direct base-constructor call validates the original
String owner identity and single chaining, creates the existing Object handle view,
and executes the actual root constructor. It does not skip the constructor body.
The handle allocation observes the heap limit. This is an explicit runtime adapter,
not a change to String record layout or to the CLI metadata format.

## Validation

- 163 C# metadata contract groups pass, including native round-trip preservation of
  the String primitive marker and canonical Object base.
- [Executable gate](string-root-2026-10-07.json): API-authored native PE library and
  digest-bound separate consumer verify and run, returning 42 with no output.
  Omitting explicit Object selection rejects. A faulting root-constructor fixture
  proves that the actual base body executes. Reproduce with the recorded command.
- All 19 existing String value/ownership/interning Rust tests pass.
- The source-root managed-array regression passes, including alias mutation and
  malformed/unselected backing checks. API snapshot and formatting checks pass.
- [Full audit](native-bootstrap-string-root-2026-10-07.json): all 197 source inputs
  pass binding and advance past the String category check. Encoding then fails with
  `binary payload exceeds envelope limit`; no assembly is published. Next inspect
  the bounded library container budget and actual payload before changing limits.

Raven compiler code remains ca4aeccfb-equivalent with this metadata DLL. No shared
compiler or .NET emission change. This gate does not prove full source String or
System execution. Public guest APIs and website examples do not change; host API
notes are updated. No website build is required for this contract fix.
