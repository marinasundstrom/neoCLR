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

The inspector emits JSON or exits 1 with a diagnostic. Initially it supports only
opaque optional sections and explicitly reports `executable: false`.
The hand-authored hexadecimal golden vector in `test_codec.py` fixes the byte layout
independently of the writer. Six focused tests pass on 2026-09-30, including every
truncation of the golden image, required schema negotiation, duplicate kinds, invalid
ranges/flags/versions, size limits and byte-preserving re-encoding.

Structural signature payloads are the next slice. PE embedding, ordinary CLI reader
compatibility, native runtime execution and Raven integration remain unimplemented.
No public runtime/library API or reference snapshot changes are involved.
