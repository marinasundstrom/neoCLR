# Extended metadata codec experiment

This standalone Python standard-library experiment implements **NEOX 0.1 framing**.
It is not a PE image, a CLI signature extension, a #Neo stream implementation or an
executable format. Codes below belong only to this experiment. The
[design](../../design/extended-cli-metadata.md) owns the intended integration path.

## Envelope schema

All integers are unsigned little-endian, with no alignment padding. Header:

| Offset | Field |
| --- | --- |
| 0 | Four bytes `NEOX` |
| 4 | u16 major = 0 |
| 6 | u16 minor = 1 |
| 8 | u32 section count |
| 12 | u32 total byte length |

Each 16-byte directory entry contains u16 kind, u16 schema version, u32 flags,
u32 absolute payload offset and u32 byte length. Kind and schema version are nonzero.
Kinds are unique within an image; ordering is retained. Flag bit 0 means required;
all other flags are rejected. Payloads follow the complete directory contiguously in
entry order. Zero-length sections are valid. Gaps, overlaps and trailing bytes fail.
Images are limited to 1 MiB and 64 sections before payload copies are made.

A caller supplies supported kind/schema pairs. Unknown required pairs fail;
unknown optional pairs remain opaque and are preserved byte-for-byte. Knowing a
schema in this framing API does not validate its payload or confer execution support.
No version other than 0.1 is accepted. There are no CLI tokens or heap references yet;
validation of those references awaits an embedding profile, not fabricated row counts.

## Run and evidence

```sh
python3 -m unittest discover -s docs/experiments/extended-cli-metadata -v
python3 docs/experiments/extended-cli-metadata/codec.py path/to/image.neox
```

The inspector emits JSON or exits 1 with a diagnostic. It decodes structural signature section kind 1/schema 1, preserves unknown optional
sections and explicitly reports `executable: false`.
The hand-authored hexadecimal golden vector in `test_codec.py` fixes the byte layout
independently of the writer. Six focused tests pass on 2026-09-30, including every
truncation of the golden image, required schema negotiation, duplicate kinds, invalid
ranges/flags/versions, size limits and byte-preserving re-encoding.

Structural signature payloads are implemented below. PE embedding, ordinary CLI reader
compatibility, native runtime execution and Raven integration remain unimplemented.
No public runtime/library API or reference snapshot changes are involved.

## Structural signature schema 1

Section kind 1, schema 1 holds exactly one signature root. The payload starts with
u16 type-parameter count, u16 method-parameter count, and u8 Self-context flag (0/1).
These counts bind parameter indices locally; the flag permits a contextual Self node.
This fixture context is not a resolved declaration identity or evidence of conformance.
Real metadata embedding must supply checked owner identities before semantic use.

Each node is u8 tag, u32 payload byte length, then exactly that payload. Nested nodes
use identical framing. Integers remain little-endian. Unknown node tags fail even in
a known optional signature section; callers cannot use an incompletely decoded type.

| Tag | Kind | Payload |
| --- | --- | --- |
| 1–4 | int32, string, bool, unit | Empty |
| 5–6 | type_parameter, method_parameter | u16 index within the corresponding binder |
| 7 | self | Empty; requires Self context |
| 8 | array | One child; owned array form |
| 9 | tuple | u16 count, then children; 1–256 elements |
| 10 | function | u8 convention (0 = managed), u8 flags, u16 parameter count, parameters, result node |
| 11–12 | union, intersection | u16 count, then children; 2–256 alternatives/requirements |
| 13 | nullable | One child |
| 14 | array_ref | One child; managed array reference form |

Each Function parameter is a mode byte followed by its type node. Modes are 0 value,
1 ref, 2 readonly_ref, 3 out, 4 out_when_true. The latter requires a bool result.
Only Function flag bit 0 is defined: no-result; it requires a unit result node.
Unit without that flag is an inhabited result. At most 256 parameters are allowed.
Max node depth is 32 (root depth zero); max total nodes is 4,096. Both encode and decode
check these bounds. Generic arities are at most 256. Unexpected payloads, flags,
conventions, indices, counts and trailing bytes fail with FormatError.

Union/intersection ordering and duplicates are deliberately preserved, not normalized.
Node equality is syntactic and context-local, not cross-module semantic type identity.
This grammar does not yet enforce runtime storage-position legality or assignability.
No nominal references, CLI heap/token validation, generic instantiation, standalone
byref result types, synthesized-member references or full primitive set are implemented.
No CLI signature prefix has been allocated. This private grammar is a codec experiment,
not a second authoritative type model or a production alternative to CLI signatures.

## Reuse of the structural runtime work

The author explicitly permits building on `codex/structural-types`. This slice reviews
`src/metadata.rs` and `src/type_identity.rs` at
`a081c6e3c9e7674e1050ddf4441d13f4a1bbbb9b`. It reuses these contract distinctions:

- Function parameter/result shape is separate from executable target and captures.
- `no_result` distinguishes absent results from inhabited Void (called unit here).
- `ByRef`, `ReadOnlyByRef`, `out_parameters` and `out_when_true` map to parameter modes;
  each parameter has one mode, so output sets are disjoint by construction.
- `Array` and `ArrayRef` remain separate. Structural status alone does not merge them.

No runtime commits were cherry-picked; this is an isolated codec test of those concepts.
Tuples/unions/intersections/nullability here do not imply branch runtime support. Self
conformance resolution and full runtime Function validation remain outside this harness.

## Signature evidence (2026-09-30)

All 14 envelope/signature tests pass on Python 3.9.6. They include the independent
hexadecimal Function vector, nested forms, every truncation of the nested signature,
generic/Self context failures, malformed payloads, depth/node/arity limits, branch
contract distinctions and inspector process success/failure. The checked-in
`fixtures/nested.neox` also passes the executable inspector command:

```sh
python3 docs/experiments/extended-cli-metadata/codec.py docs/experiments/extended-cli-metadata/fixtures/nested.neox
```

Fixture SHA-256: `ab5798a379c1769e425970233f55b3101bb8be7926b245c34483fd990ed10ce5`.
The inspector prints a Function containing Array/Tuple, Intersection with contextual
Self, and Union/Nullable with a method parameter. It does not execute that signature.
The test checks the fixture against re-encoding; changes require an intentional update.

Step 1 remains partial: CLI embedding, token/heap references and conventional CLI
compatibility fixtures still need their own slice. Next, establish nominal reference
and owner contexts so cross-module structural identity can be tested without assuming
that local row indices, names or this fixture's binder counts are global identities.
