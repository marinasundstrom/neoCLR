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
This local-only schema has no nominal references or generic instantiation. The reference
profile below adds those in separate sections. CLI heap validation, standalone byref
result types and the full primitive set remain pending. Synthesized-member references
are covered by the later section-4 profile.
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

Step 1 remains partial: CLI embedding, actual CLI token/heap resolution and conventional
CLI compatibility fixtures still need their own slice. The next section records the
subsequent standalone nominal-reference and owner-context experiment.

## Reference profile (2026-09-30)

Two mandatory sections extend the standalone experiment without changing schema 1
local-signature images. Section 2/schema 1 carries reference bindings; section 3/schema 1
carries a signature with nominal nodes enabled. Neither may occur alone. Do not combine
them with section 1. Directory order is immaterial. Unknown optional schema versions
remain opaque unless a supported counterpart requires a complete profile.

Reference payload header: u16 reference count, u16 type-owner index, u16 method-owner
index, u16 Self-contract index. Owner index zero means absent; otherwise all indices are
one-based into this table. At most 256 references are allowed, without duplicates.
Each 36-byte reference is an assembly UUID (16 bytes in UUID/network byte order), a
module UUID (16 bytes in the same order), then a little-endian u32 defining token.
UUIDs must be nonzero. Defining tokens are TypeDef (high byte 0x02) or MethodDef (0x06),
with a nonzero 24-bit row. Type/Self owners require TypeDef; method owners require
MethodDef. Length must match exactly; no trailing data is accepted.

These UUIDs are **host-catalog scope identifiers for this experiment**, not a new CLR
assembly identity scheme. Defining tokens refer to entries in that host catalog, not
to the consuming image's CLI tables. This is not validation of real PE table bounds.
The catalog must assign unambiguous assembly/module scopes and provide authoritative
definitions; untrusted declarations in an image cannot grant themselves conformance.
This slice does not serialize, authenticate or discover the host catalog.

Section 3 uses the schema 1 signature framing plus node tag 15: u16 reference index,
u16 generic-argument count, followed by that many child nodes. Count is at most 256;
reference index is nonzero and at most 256. Section 1 continues rejecting tag 15.
After decoding both sections, local validation checks actual reference-table bounds,
type-token kinds, binder-owner presence and agreement of Self flag/owner presence.

`references.resolve` takes the decoded tree, local context, bindings and an externally
supplied catalog of Reference → Definition entries. It resolves every reference,
checks definition kinds, generic argument counts, exact binder arities and method/type
ownership. Self currently requires a nongeneric interface contract. This identifies
symbolic Self in a contract, not its substitution at a concrete implementation; generic
Self contexts and conformance evidence remain unsupported. Generic parameters retain
their declaring type or method identity rather than just an ordinal.

The result is an immutable equality key within that catalog. Nominal identity uses
the resolved assembly/module/defining-token triple and constructed argument keys.
Consumer-local reference numbers are absent from the key. No identity stability across
rebuilds, type forwarding, duplicate assembly loading or catalog changes is promised.
Names are not keys. This follows the structural branch's distinction between loaded
program identities and persistent names; it does not duplicate its runtime loader.

For the experimental semantic key, union/intersection operands flatten like operators,
ignore order and remove duplicate resolved operands using sets. Codec bytes still
preserve source order. Singleton operator nodes are retained; there is no distributive
expansion, subtype simplification, null equivalence or recursive-type support. This
bounded normalization is a provisional identity experiment, not runtime assignability.
Tuples/Function parameters stay ordered; modes, no-result and both array forms remain
distinct. Existing node/depth limits apply before key construction.

Compared with ordinary CLI use-site tokens, the explicit local-reference/resolved-key
split prevents token numbering from leaking into structural identity. It costs a host
catalog, owner checks and an extra resolution pass. Reuse the design's ECMA baseline;
this experiment supplies no evidence about CLR or third-party reader compatibility.
No public runtime API or Raven compiler behavior changes.

## Reference-profile evidence

All 22 focused tests pass on Python 3.9.6, 2026-09-30. Independent hex vectors cover
the reference table and nominal node. Two separately encoded fixture images simulate
consumers using different local reference numbering and directory order; their byte
sequences differ while their resolved Function keys compare equal against the same
catalog. They are not independently compiled Raven modules or real CLI assemblies.

Tests distinguish assembly/module scope, method binders, Self contracts, parameter
modes, no-result and owned/reference arrays. Negative cases include every reference-table
truncation, missing definitions, wrong kinds, zero/duplicate/out-of-range references,
arity/owner mismatch, unsupported generic Self contracts, and incomplete profiles.
The inspector is exercised as a process for success and rejection. It reports
`resolved: false` for these images because it has no host catalog; the resolver is
exercised directly by the tests. Inspection success is not semantic acceptance.

| Fixture | SHA-256 |
| --- | --- |
| `fixtures/references-a.neox` | `cda3bd97fe29c2a7f02df80702576f85bc80197ea95ab44e297b2056a86e69b4` |
| `fixtures/references-b.neox` | `b35fa38129dfda62269eee21a7902794d4c9e0d05942432e5d59d271f6df0abd` |

The next slice adds synthesized structural-member references, recorded below.
Interface conformance records, PE embedding, actual CLI heap/token resolution,
runtime execution and Raven integration remain pending.

## Synthesized structural-member references (2026-09-30)

Section 4/schema 1 holds descriptions of intrinsic operations on the section-3 root.
It requires mandatory sections 2 and 3 and must itself be mandatory. This bounded
profile has exactly one signature owner, designated by local handle 1. That handle is
neither a CLI TypeSpec token nor semantic member identity. Multiple owners, paths into
nested signatures and an owner-expression table remain later format work.

The payload begins with a u16 row count (0–256). Each six-byte row is u16 owner handle,
u8 operation, u8 flags (zero), u16 operand. Rows preserve input order. Duplicate rows,
unknown operations/flags, non-1 owners, unexpected operands, malformed lengths and
trailing bytes fail. There are no user-defined names, claimed signatures or executable
addresses in these records. The codec validates rows; profile inspection additionally
checks each operation against the structurally validated owner.

| Operation | Code | Valid owner | Derived contract |
| --- | --- | --- | --- |
| array_length | 1 | array or array_ref | No explicit parameters; native unsigned integer result |
| tuple_element | 2 | tuple | No explicit parameters; selected element result |
| tuple_deconstruct | 3 | tuple | One out parameter per ordered element; no result |
| function_invoke | 4 | function | Owner's parameter types, modes, result and no-result flag |

Only tuple_element takes an operand: the zero-based element index, less than the
owner's arity. Other operands must be zero. These contracts describe operations on
an implicit owner value; they do not yet prescribe receiver passing, borrowing,
boxing, field access, property mutability or an executable call ABI. TupleElement is
a read projection, not a field declaration or a promise of a setter. TupleDeconstruct
is a provisional structural operation; existing nominal System.Tuple is unchanged.

`resolve_members` first resolves the complete owner/context against the host catalog,
then derives descriptors. The immutable member key contains the operation-schema
version, resolved owner key, operation identity and operand. It does not contain the
member's row number, local owner handle or display name. Return/parameter contracts
are derived, so a producer cannot forge an Invoke signature inconsistent with its
Function owner. No MethodDef, dispatch address or interface conformance is synthesized.

Function modes/no-result follow the structural branch's contracts already recorded
above. ArrayLength's intrinsic result descriptor is `('intrinsic', 'native_uint')`,
matching `Type::UIntPtr` in `src/verifier.rs` at structural branch revision
`a081c6e3c9e7674e1050ddf4441d13f4a1bbbb9b`. The prototype's limited signature grammar
cannot yet serialize UIntPtr. This descriptor is not a new guest type or an Int32
property result; full primitive encoding/identity normalization remains future work.
The owned/reference array identities remain distinct even when the operation matches.

Compared with ordinary CLI member rows, these descriptions let a frontend discover
operations without fabricating nominal declarations. The cost is a shared versioned
operation vocabulary and a later backend mapping. Reuse the original structural draft
and design's CLI baseline; no new .NET behavior or compatibility claim is made here.
Neither a valid member record nor successful catalog resolution enables execution.

## Structural-member evidence

All 30 focused tests pass on Python 3.9.6, 2026-09-30. An independently written hex
vector fixes the member-row layout. Tests cover all four operations, derived result
and output contracts, preservation of every Function parameter mode/no-result,
renumbered nominal references producing equal member descriptors, and resolution
failure when dependencies are missing. Negative cases cover unknown operations,
wrong shapes, tuple bounds, owner handles, flags, duplicates, every truncation of the
member vector, resource limits and missing/optional profile dependencies.

The inspector process checks a tuple-member fixture and rejects an incompatible
FunctionInvoke owner without emitting partial JSON. It reports
`owner_shape_validated: true`, `resolved: false`; it has no catalog and performs no
runtime invocation. Unknown optional member schema versions remain opaque.

Fixture: `fixtures/tuple-members.neox`.
SHA-256: `c6a4a260f31c36177b1c906ccc8e792cda8e8f83f833ff8ca53c7c9276e2be00`.
The test verifies byte-preserving re-encoding, reordered section discovery and derived
descriptors. The fixture contains a two-element tuple, its second-element projection
and deconstruction operation.

Next bounded task: a PE/CLI container and reader-compatibility probe for an experimental
#Neo stream. The standalone codec has answered enough framing/identity questions to
start checking the main unresolved container assumption before expanding conformance
records. Preserve conventional CLI streams and test ordinary metadata inspection
separately from semantic decoding. Do not treat success as execution compatibility.
