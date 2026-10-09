# Metadata format

Metadata connects neoCLR's compiler, tools and runtime. It records assemblies,
declarations, signatures, references and instruction bodies so that a compiled library
can be imported, inspected and executed without its source files.

The current native format has several layers. Its CLI-derived design direction and
its implemented transport should be distinguished: today the native execution model
is carried in NEOX, either on its own or inside a PE file's `#Neo` stream.

![PE transport, NEOX framing, execution encoding and semantic module](../metadata-format.svg)

## What an artifact contains

| Part | What it describes |
| --- | --- |
| Assembly manifest | Identity, referenced assemblies and ownership information |
| Declaration modules | Named containers for top-level types, functions and constants |
| Type/member definitions | Signatures, generic parameters, constraints and member relationships |
| Definition and member references | The identities used when dependencies are bound |
| Instruction bodies | The operations checked by the verifier and executed by a backend |

An assembly can contain several [declaration modules](../features/modules/). An assembly
identity and a module name identify a logical declaration container. Physical metadata
images are separate: a module does not automatically have its own file or load lifetime.
Source tokens and executable definition identities also serve different purposes.

For a separately compiled library, tools first read its declarations and signatures.
The runtime later resolves the application's references against the supplied dependency
set and verifies executable bodies. Reading a signature does not execute the library.
[Introspection](../features/introspection/) and [reflection](../features/reflection/)
explain the supported runtime views.

## NEOX framing and payload

The envelope starts with `NEOX`, version 0.1, a total length and a section directory.
Each directory entry gives a section kind, schema version, required/optional flag,
offset and length. The native execution section is kind 256 and must be required.
Unknown required sections, invalid ranges and unsupported schemas are rejected.

| Layer | Current representation |
| --- | --- |
| Envelope header | 16 bytes; magic, versions, count and length |
| Directory entry | 16 bytes; kind, schema, flags, offset and length |
| Execution schema 1 | Legacy UTF-8 JSON |
| Execution schemas 2/3/4 | Bounded CBOR object encoding |
| Semantic model | Format 5, independent of the execution schema number |

The current standalone writer selects schema 3 up to an 8 MiB envelope and schema 4
above that, up to 16 MiB. Legacy schemas retain smaller budgets. Binary decoding
checks definite lengths, nesting/node limits, UTF-8 text and duplicate map keys before
constructing the metadata model. This is a restricted CBOR profile, not arbitrary CBOR
and not a memory dump of Rust objects.

## PE is a container, not a compatibility promise

A native PE image contains a recognition marker, metadata streams and a `#Neo` stream
holding NEOX. A SHA-256 binding detects inconsistent metadata streams; it is not a
publisher signature. The reader also checks layout, stream overlap and padding.

The runtime executes the native model carried by `#Neo`, not the CLI method bodies
in the wrapper. Ordinary CLI tools may parse the outer file without understanding
neoCLR's executable semantics. The typed instruction records are not automatically
ECMA CIL byte streams: branch indices, for example, are not byte displacements.

This transport makes native metadata experiments possible while retaining a familiar
outer container. Its cost is a separate payload and a need for neoCLR-aware readers.
Standard CLI tables and opcode meanings remain the design baseline where they fit;
a fully CLI-authoritative representation remains a future migration direction.

## Validation and evolution

Framing checks, payload decoding, dependency resolution and typed verification are
separate gates. Passing an early gate does not imply the later ones passed.
Versioned contracts permit readers to reject features they cannot preserve. For example,
the version-1 declaration-module table preserves empty modules; older inputs expose
marked namespace projections, while older readers reject the new manifest field.
Use matching compiler, library and runtime artifacts rather than editing version numbers.

See [architecture](../architecture/) for how the reader fits into execution, or the
[detailed format document](https://github.com/marinasundstrom/neoCLR/blob/main/docs/metadata-format.md)
for byte offsets, admission budgets, source references and existing test evidence.
The underlying references are [ECMA-335](https://ecma-international.org/publications-and-standards/standards/ecma-335/)
and [CBOR RFC 8949](https://www.rfc-editor.org/rfc/rfc8949.html); neither alone defines
neoCLR's semantics or guarantees .NET compatibility.
