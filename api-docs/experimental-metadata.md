# Experimental .NET metadata library

**Feature-branch development only; not part of Preview 11 or the neoCLR guest API.**
Assembly and namespace: `NeoCLR.Metadata.Experimental`. Target: .NET 10.
Source project: `tools/metadata/NeoCLR.Metadata.Experimental`.
This is the first reusable reader/writer slice intended for Raven's future symbol
loader and code-generation adapters. It reads/writes **NEOX 0.1 framing, structural signatures, reference tables and synthesized-member tables**,
and derives structural identities/member contracts against an explicitly supplied host catalog.
Bounded PE32 recognition and a read-only manifest-module/TypeDef model are implemented;
explicit AssemblyRef, nominal TypeRef and bounded method MemberRef resolution are
implemented. A controlled static-primitive builder writes ordinary CLI PE and native
format-5 assemblies, including native top-level functions. Direct PE/#Neo runtime loading now uses a transitional native execution section
with a reference-only CLI projection. A bounded binary native payload now avoids JSON parsing at runtime. General rewriting
and guest Introspection assembly loading remain pending.

## Namespace and types

- [Model namespace](#model-namespace): Cecil-inspired assembly/module/type definitions and scoped references.
- [Primitive signatures](#primitive-signatures-development-2026-10-01): Int32/Boolean parameters and results.
- [MethodDefinition](#methoddefinition): callable declarations and bounded signature recognition.
- [MemberReference](#memberreference): physical references and explicit method resolution.
- [Branch labels and control flow](#branch-labels-and-control-flow-development-2026-10-01): Boolean conditions, joins and loops.
- [Int32 local slots](#int32-local-slots-development-2026-10-01): method-owned locals, raw indices and initialization checks.
- [OpCode and MethodBuilder.Emit](#opcode-and-methodbuilderemit): bounded opcode/typed-operand construction.
- [NativeLibraryDefinition and NativeFunctionDefinition](#nativelibrarydefinition-and-nativefunctiondefinition): native inventory and explicit partial callable views.
- [NativeModuleContainer](#nativemodulecontainer): existing native JSON translation without a CLI projection.
- [RuntimeAssemblyContainer](#runtimeassemblycontainer): direct PE/#Neo native execution transport.
- [MetadataArtifactReader](#metadataartifactreader): bounded PE extraction and recognition.
- [MetadataArtifact](#metadataartifact): ordinary classification or owned extended profile.
- [MetadataProfile](#metadataprofile): reference-profile reader and writer entry points.
- [MetadataProfileDocument](#metadataprofiledocument): owned typed view and explicit catalog resolution.
- [MetadataSection](#metadatasection): immutable, owned opaque payload and section metadata.
- [MetadataEnvelope](#metadataenvelope): bounded envelope read/write operations.
- [TypeExpression](#typeexpression): immutable raw signature syntax tree.
- [SignatureContext](#signaturecontext): local generic arities and Self permission.
- [StructuralSignature](#structuralsignature): validated signature read/write operations.
- [MetadataReference and MetadataDefinition](#metadatareference-and-metadatadefinition): catalog keys and declarations.
- [ReferenceBindings](#referencebindings): owned local references and binder owners.
- [ReferenceTable](#referencetable): reference payload read/write operations.
- [StructuralIdentity](#structuralidentity): catalog validation and normalized equality.
- [ResolvedTypeIdentity](#resolvedtypeidentity): immutable resolved equality key.
- [StructuralMemberReference](#structuralmemberreference): operation and local owner/ordinal.
- [ResolvedMemberIdentity](#resolvedmemberidentity): resolved owner/operation identity.
- [StructuralMemberDescriptor](#structuralmemberdescriptor): immutable derived contract.
- [StructuralMembers](#structuralmembers): member table codec and contract resolver.

These .NET-host-only types have this complete manual reference because the site's
RavenDoc source is the neoCLR guest compiler-reference assembly. They are not omitted
guest types or claimed executable neoCLR APIs. The maintenance record names this
boundary explicitly; source XML documentation also ships beside the built .NET DLL.

## MetadataSection

```csharp
public sealed class MetadataSection
{
    public MetadataSection(ushort kind, ushort version, bool required,
                           ReadOnlySpan<byte> payload);
    public ushort Kind { get; }
    public ushort Version { get; }
    public bool Required { get; }
    public int PayloadLength { get; }
    public byte[] GetPayload();
}
```

The constructor requires nonzero `kind` and `version` and at most 1 MiB of payload.
It copies the span into private storage. Invalid arguments raise
`ArgumentOutOfRangeException`. The payload-size limit does not imply the section fits
inside a complete envelope; headers also consume its 1 MiB budget.

`Kind` and `Version` identify the section schema. `Required` determines whether an
exact schema match is required by the consuming layer. `PayloadLength` is the byte
count. `GetPayload()` returns a new caller-owned byte array on each call; changing it
or the constructor's original buffer cannot change this section. Payload semantics
are not interpreted. A supported schema dictionary alone does not establish that a
payload is valid or executable.

## MetadataEnvelope

```csharp
public static class MetadataEnvelope
{
    public const int MaxImageSize = 1048576;
    public const int MaxSections = 64;
    public static IReadOnlyList<MetadataSection> Read(
        ReadOnlySpan<byte> image,
        IReadOnlyDictionary<ushort, ushort> supportedSchemas);
    public static byte[] Write(
        IReadOnlyList<MetadataSection> sections,
        IReadOnlyDictionary<ushort, ushort> supportedSchemas);
}
```

`Read` takes one complete image. It validates magic/version, size/count limits,
nonzero unique section kinds/versions, known flags, canonical contiguous ranges and
required schemas. It returns a read-only section list with owned payload copies, in
input order; the input buffer is no longer needed after the call. Unknown optional
schemas are retained byte-for-byte. Unknown required schemas fail. An empty dictionary
admits only optional sections. Callers must not mutate inputs during a call.

`Write` takes sections in desired output order, validates duplicate kinds, total size
and required schemas, and returns a new caller-owned image. Empty envelopes and
zero-length payloads are supported. Both operations reject malformed/unsupported
framing with `InvalidDataException`; null list/dictionary arguments raise
`ArgumentNullException`. A null section entry raises `InvalidDataException`.
The helpers perform no file I/O or assembly loading.

The schema dictionary maps each kind to one admitted schema version. It is an explicit
capability declaration supplied by the higher-level consumer, which must still decode
and validate known payloads. These framing routines cannot be used by themselves as
semantic acceptance for Raven or neoCLR. Preserving opaque bytes does not validate a
rewrite of the declarations/tokens to which they might refer.

The current NEOX layout preserves section order and exact payload bytes and rejects
gaps/overlaps/trailing data. Reading then writing a valid image therefore reproduces
its bytes. This is deterministic framing, not structural type canonicalization.
Copies simplify ownership at a memory cost; no zero-copy/performance claim is made.

## Example and validation

The separate `tools/metadata/MetadataConformance` executable builds this example
against the project reference and tests it through Python's independent reader:

```csharp
var schemas = new Dictionary<ushort, ushort> { [7] = 2 };
var bytes = MetadataEnvelope.Write(
    [new MetadataSection(7, 2, true, "abc"u8),
     new MetadataSection(60000, 9, false, [0, 255, 0])], schemas);
var sections = MetadataEnvelope.Read(bytes, schemas);
```

Kinds 7 and 60000 here are opaque test payloads, not production declarations.

```sh
python3 docs/experiments/extended-cli-metadata/verify_dotnet.py
```

Four shared fixtures round-trip identically through .NET. Independently emitted .NET
bytes match Python emission; both reject 49 malformed vectors. The consumer also
checks required-schema mismatch, writer limits, unknown optional preservation and
buffer ownership. The reference assembly/source snapshot check remains unchanged and
passes; there are no new guest reference types in this slice.

## TypeExpression

```csharp
public sealed class TypeExpression
{
    public TypeExpression(string kind,
        IReadOnlyList<TypeExpression>? children = null, int index = 0,
        IReadOnlyList<string>? modes = null, bool noResult = false);
    public string Kind { get; }
    public IReadOnlyList<TypeExpression> Children { get; }
    public int Index { get; }
    public IReadOnlyList<string> Modes { get; }
    public bool NoResult { get; }
}
```

The constructor copies both lists into read-only owned collections. Children themselves
are immutable nodes. Null lists mean empty. Null `kind` raises `ArgumentNullException`;
null list entries, more than 257 children, or more than 256 modes raise
`ArgumentException`. Do not mutate supplied lists while constructing the node.
Construction alone does not validate a tree's shape: unknown spellings, inappropriate
indices/modes and context-dependent errors are diagnosed by the codec.

`Kind` uses the exact lowercase spellings `int32`, `string`, `bool`, `unit`,
`type_parameter`, `method_parameter`, `self`, `array`, `array_ref`, `tuple`, `function`,
`union`, `intersection`, `nullable`, `nominal`. These are experimental wire syntax,
not resolved runtime type names. `Index` is a zero-based generic ordinal, or a
one-based nominal reference index, and must otherwise be zero. `Children` are ordered;
Function parameters precede its last child, the result. `Modes` correspond to the
parameters only: `value`, `ref`, `readonly_ref`, `out`, `out_when_true`.

`NoResult` applies only to Function and requires a plain unit result node. A unit
result without this flag is inhabited. Conditional output requires a bool result.
Other nodes cannot carry Function modes/flags. `array` and `array_ref` remain distinct.
Unions/intersections retain order and duplicate children. This class has ordinary
object identity; it does not implement structural equality, normalization or subtyping.

## SignatureContext

```csharp
public sealed class SignatureContext
{
    public SignatureContext(int typeParameters = 0, int methodParameters = 0,
                            bool selfAllowed = false);
    public int TypeParameters { get; }
    public int MethodParameters { get; }
    public bool SelfAllowed { get; }
}
```

Both counts must be 0–256 inclusive; other values raise `ArgumentOutOfRangeException`.
Properties retain these immutable counts and Self permission. Parameter indices must
be less than the corresponding count, and Self requires `SelfAllowed`. The context
does not contain declaring type/method/contract identity and cannot establish runtime
Self conformance or cross-module generic identity.

## StructuralSignature

```csharp
public static class StructuralSignature
{
    public static (TypeExpression Root, SignatureContext Context) Read(
        ReadOnlySpan<byte> payload, bool allowReferences = false);
    public static byte[] Write(TypeExpression root, SignatureContext context,
                               bool allowReferences = false);
}
```

`Read` consumes a complete signature payload, not an envelope or PE image. `Write`
produces a new caller-owned payload array. Decoded objects are independent of the
input bytes; callers must not mutate a span during reading. Both operations validate
context, kind-specific shape, flags/modes, lengths and resource limits. Invalid or
unsupported input raises `InvalidDataException`; null writer root/context raises
`ArgumentNullException`.

Limits are 1 MiB per payload, 4,096 nodes, depth 32 with root at zero, generic arity
256 and 256 Function parameters/compound operands. Tuple requires at least one child;
union/intersection at least two. Arrays and nullable require one; primitives, generic
parameters and Self have none. The only supported calling convention is managed.
Unknown nodes, flags, modes, trailing bytes and out-of-bounds counts fail.

With `allowReferences: false`, nominal nodes are rejected (section-1 profile).
With `true`, the section-3 grammar admits a nominal index in 1–256 and up to 256
argument children. This is grammar permission, **not** a check that the reference exists,
is a type or has matching generic arity; `StructuralIdentity.Resolve` performs those
checks against an explicit catalog. Standalone byref-result syntax, full primitive support, declaration resolution,
position legality, member synthesis and execution remain outside this codec.

The library preserves canonical payload framing, so valid read/write round-trips are
byte-identical. Payload encoding does not normalize structural semantic identity.
These type constructors and the following usage are exercised by the conformance
consumer's signature commands:

```csharp
var signature = new TypeExpression("function",
    [new("int32"), new("bool")], modes: ["out_when_true"]);
var bytes = StructuralSignature.Write(signature, new SignatureContext());
var decoded = StructuralSignature.Read(bytes);
```

```sh
python3 docs/experiments/extended-cli-metadata/verify_dotnet_signatures.py
```

Fourteen cross-reader vectors pass, including existing reference-profile fixtures,
Function modes and nominal syntax. Independent .NET nested emission matches Python.
Both readers reject 103 malformed vectors, including every nested-fixture truncation,
invalid binders/Self contexts, unknown nodes/modes, bad conventions/flags, malformed
lengths and depth/node limits. Writer/ownership and exact depth/arity boundaries are
also exercised. None of these tests claims that the decoded metadata can execute.


## MetadataReference and MetadataDefinition

```csharp
public readonly record struct MetadataReference(Guid Assembly, Guid Module, uint Token);
public sealed record MetadataDefinition(string Kind, int Arity = 0,
                                        MetadataReference? Owner = null);
```

Positional constructors and matching public init properties store the given values.
Records provide value equality, hashes, deconstruction, printable representations and
`with` copies. They do not validate at construction; table/resolver operations validate
before use. Default MetadataReference is invalid. Assembly and Module must be nonempty
host-assigned scope UUIDs; Token must be a TypeDef (0x02) or MethodDef (0x06) token with
a nonzero row. These scopes are **not** CLI assembly names, MVID-derived assembly
identity or a promise that a token exists in a physical PE image.

Definition Kind is `type`, `interface` or `method`, with Arity 0–256. Only methods
require Owner, a declaring-type reference. All other declarations require null Owner.
The host supplies authoritative declarations, including referenced dependencies, and
must use compatible scope assignments across catalogs when comparing resolved keys.

## ReferenceBindings

```csharp
public sealed class ReferenceBindings
{
    public ReferenceBindings(IReadOnlyList<MetadataReference> references,
        int typeOwner = 0, int methodOwner = 0, int selfOwner = 0);
    public IReadOnlyList<MetadataReference> References { get; }
    public int TypeOwner { get; }
    public int MethodOwner { get; }
    public int SelfOwner { get; }
}
```

Copies the list into owned read-only storage. Null references raises
ArgumentNullException; more than 256 entries raises ArgumentException. Owners are
one-based local indices, with zero meaning absent. Further validation occurs in
ReferenceTable and StructuralIdentity. TypeOwner and SelfOwner must name TypeDefs;
MethodOwner must name a MethodDef. References must be unique.

## ReferenceTable

```csharp
public static class ReferenceTable
{
    public static ReferenceBindings Read(ReadOnlySpan<byte> payload);
    public static byte[] Write(ReferenceBindings bindings);
}
```

Reads/writes section 2, schema 1: four little-endian u16 values for count/type owner/
method owner/Self owner, followed by 36-byte reference rows. UUID bytes use network
order, tokens little-endian order. At most 256 rows are allowed; input must have the
exact expected length. Invalid GUIDs, token kinds/rows, duplicates, owner kinds/indices,
truncation or trailing bytes raise InvalidDataException. Null writer bindings raises
ArgumentNullException. Read owns its data; Write returns a fresh caller-owned array.
Neither resolves declarations or checks that nominal uses exist in the table.

## StructuralIdentity

```csharp
public static class StructuralIdentity
{
    public static ResolvedTypeIdentity Resolve(TypeExpression root,
        SignatureContext context, ReferenceBindings bindings,
        IReadOnlyDictionary<MetadataReference, MetadataDefinition> catalog);
}
```

Validates bounded signature syntax, reference-table rules, catalog kinds and arities,
nominal generic arguments, and declaring owners before returning an identity. Null
arguments raise ArgumentNullException; malformed or unresolved data raises
InvalidDataException. Do not mutate the catalog during resolution. Every reference
must resolve, including unused entries. A nominal use must point to a type/interface
with the exact number of arguments. Generic binders require matching declaring-owner
arities; a bound method's declaring type must match TypeOwner. Self permission must
match the presence of SelfOwner, which must identify a nongeneric interface contract.

Generic parameters are keyed by declaring reference and ordinal; Self by contract.
Local indices and table/directory order do not enter resolved identity. Unions and
intersections flatten the same operator, discard duplicate operands and ignore order.
A deduplicated singleton retains its operator wrapper. Tuples/arguments retain order;
Function modes and no-result, and Array versus ArrayRef, remain distinct. This does
not implement assignability, subtype reduction, distribution or null equivalences.

Callers select and validate the envelope profile: section 2/schema 1 plus section
3/schema 1 are required, and section 1 must not be mixed in. These payload-level APIs
do not implement a production profile loader or PE recognition. They cannot establish
that an assembly is safe to execute or populate Raven/neoCLR objects themselves.

## ResolvedTypeIdentity

```csharp
public sealed class ResolvedTypeIdentity : IEquatable<ResolvedTypeIdentity>
{
    public bool Equals(ResolvedTypeIdentity? other);
    public override bool Equals(object? obj);
    public override int GetHashCode();
}
```

Created only by Resolve; no public constructor or mutable state. Equals compares full
resolved keys, returns false for null/other object types, and is consistent with the
process-local hash. Use Equals or a normal equality-based collection; `==` is not
overloaded. Neither hash codes nor private canonical bytes are persistence formats.

The separate conformance consumer exercises these APIs with Python-generated tables,
signatures and explicit catalogs:

```sh
python3 docs/experiments/extended-cli-metadata/verify_dotnet_references.py
```

All 95 shared vectors pass: 22 equality/distinction cases, four table round-trips and
69 rejections. Independent C# golden UUID emission/reading, writer validation and list
ownership checks also pass. The test-only JSON transport is not a library API.


## StructuralMemberReference

```csharp
public sealed record StructuralMemberReference(string Operation, int Owner = 1,
                                               int Element = 0);
```

The constructor and matching init properties retain values without validation.
The record supplies value equality, hashes, deconstruction, printable representation
and `with` copies. StructuralMembers validates before reading/writing/resolving:
Operation must be `array_length`, `tuple_element`, `tuple_deconstruct` or
`function_invoke`; Owner must be 1 (the section-3 root); Element is 0–255 and must be
zero except for a zero-based tuple element. A reference is not a MethodDef token.

## ResolvedMemberIdentity

```csharp
public sealed record ResolvedMemberIdentity(ResolvedTypeIdentity Owner,
                                            string Operation, int Element);
```

A schema-1 identity comprising a resolved owner, operation and ordinal. Positional
constructor/init properties and generated record equality/hash/deconstruction/`with`
behave as value operations. The resolver returns validated identities; constructing
one directly performs no validation and grants no capability. Equality compares full
resolved owner identity, so local reference numbering is irrelevant but declaration
scopes, array storage, Function modes/no-result and tuple ordinals remain significant.
Hashes are process-local and neither identity nor its string form is a persistence API.

## StructuralMemberDescriptor

```csharp
public sealed class StructuralMemberDescriptor
{
    public ResolvedMemberIdentity Identity { get; }
    public IReadOnlyList<ResolvedTypeIdentity> Parameters { get; }
    public IReadOnlyList<string> Modes { get; }
    public ResolvedTypeIdentity Result { get; }
    public bool NoResult { get; }
}
```

Created only by StructuralMembers.Resolve, with copied read-only parameter/mode lists.
Parameters exclude the receiver; modes correspond one-to-one in order. Result is unit
when NoResult is true; a unit result with NoResult false is distinct. Descriptor object
equality is ordinary reference equality; compare Identity for member identity and
properties for contract content. No method body, dispatch target or invocation API exists.

## StructuralMembers

```csharp
public static class StructuralMembers
{
    public static ResolvedTypeIdentity NativeUnsignedResult { get; }
    public static IReadOnlyList<StructuralMemberReference> Read(ReadOnlySpan<byte> payload);
    public static byte[] Write(IReadOnlyList<StructuralMemberReference> members);
    public static IReadOnlyList<StructuralMemberDescriptor> Resolve(
        IReadOnlyList<StructuralMemberReference> members, TypeExpression root,
        SignatureContext context, ReferenceBindings bindings,
        IReadOnlyDictionary<MetadataReference, MetadataDefinition> catalog);
}
```

Read/Write encode section 4, schema 1: little-endian u16 count followed by six-byte
rows (u16 owner, u8 operation, u8 zero flags, u16 operand). Operation codes 1–4 follow
the order listed above. At most 256 unique rows are permitted, including an empty table.
Read returns owned read-only rows; Write returns a fresh byte array. Table operations
validate counts, exact lengths, operations, flags, owners, operands and duplicates,
but do not validate owner shapes. Invalid data or null rows raises InvalidDataException;
a null writer list raises ArgumentNullException.

Resolve validates the entire owner using StructuralIdentity.Resolve, even for an empty
member list, then checks each operation's owner shape and derives these contracts:

| Operation | Owner | Parameters/modes | Result |
| --- | --- | --- | --- |
| array_length | Array or ArrayRef | None | NativeUnsignedResult; NoResult false |
| tuple_element | Tuple; ordinal within arity | None | Selected element; NoResult false |
| tuple_deconstruct | Tuple | All elements, each `out` | Unit; NoResult true |
| function_invoke | Function | Declared parameters and modes | Declared result and NoResult |

NativeUnsignedResult is an intrinsic descriptor identity for native unsigned/UIntPtr,
matching the experimental runtime ArrayLength contract. It adds no serializable
signature opcode and does not claim ordinary .NET Array.Length's Int32 contract.
Compare result identities with Equals. Resolve returns owned read-only descriptors in
input order. Null arguments raise ArgumentNullException; invalid tables, unresolved
catalog declarations, incompatible shapes or out-of-range tuple ordinals raise
InvalidDataException. Callers must not mutate input lists/catalogs during operations.

Envelope composition remains the caller's responsibility: this member profile needs
mandatory schema-1 sections 2, 3 and 4, with no local section-1 signature. These APIs do
not perform artifact recognition, PE loading, assembly emission or runtime invocation.

The compiled consumer validates all public operations with 49 Python/.NET shared cases:
14 contract vectors, five identity comparisons and 30 rejections. Coverage includes
256 tuple elements/deconstruction outputs, empty tables, all Function modes, reference
renumbering and malformed framing. Independent C# golden emission, writer rejection
and reader ownership checks also pass.

```sh
python3 docs/experiments/extended-cli-metadata/verify_dotnet_members.py
```


## MetadataProfile

```csharp
public static class MetadataProfile
{
    public static MetadataProfileDocument Read(ReadOnlySpan<byte> image);
    public static MetadataProfileDocument Create(TypeExpression root,
        SignatureContext context, ReferenceBindings bindings,
        IReadOnlyList<StructuralMemberReference>? members = null,
        IReadOnlyList<MetadataSection>? optionalSections = null);
    public static byte[] Write(MetadataProfileDocument document);
}
```

These are the typed entry points for the **reference profile**, not a general CLI
assembly API. Read requires mandatory schema-1 sections 2 and 3. Section 1 is rejected
in any version, including when optional. A schema-1 section 4 must be mandatory when
present. Unknown required kinds/versions fail; unknown optional payloads are retained
opaquely, including future optional section-4 versions. Unsupported optional versions
of sections 2 or 3 cannot satisfy the mandatory reference-profile requirements.
A local-only, empty or incomplete envelope fails instead of producing a partial view.

Read performs envelope validation, signature/table decoding, local nominal index/kind
checks, generic/Self owner presence checks, and member shape/ordinal validation. It
owns the resulting data independently of the input buffer. It does not consult a
catalog: nominal arity, declaring method ownership and Self interface requirements
are checked later by the document's Resolve methods. Do not mutate input during a call.
Malformed or unsupported input raises InvalidDataException.

Create accepts typed syntax, explicit local bindings and optional members. It copies
through the codecs and performs the same local validation as Read. Null root/context/
bindings raises ArgumentNullException. Null members omits section 4; an empty list
emits a supported mandatory empty table. OptionalSections accepts only non-null,
optional sections with kinds greater than 4. Duplicate kinds, required extensions,
reserved kinds 1–4, invalid shapes/references or exceeded bounds raise
InvalidDataException. Output order is 2, 3, optional 4, then supplied extensions.
All existing envelope/signature/table resource bounds apply.

Write accepts only a document returned by Read/Create and returns a fresh array.
Null document raises ArgumentNullException. Read/Write preserves original order and
all bytes, including opaque optional sections. Semantic editing is explicit: construct
a new document and supply consistently remapped indices. Create does not copy unknown
extensions automatically or claim their meaning survives changes. In particular, an
unknown future optional section-4 payload can be round-tripped but cannot be attached
to a newly constructed schema-1 document through optionalSections.

## MetadataProfileDocument

```csharp
public sealed class MetadataProfileDocument
{
    public IReadOnlyList<MetadataSection> Sections { get; }
    public TypeExpression Root { get; }
    public SignatureContext Context { get; }
    public ReferenceBindings Bindings { get; }
    public IReadOnlyList<StructuralMemberReference> Members { get; }
    public bool HasMemberTable { get; }
    public IReadOnlyList<MetadataSection> UnknownOptionalSections { get; }
    public ResolvedTypeIdentity ResolveType(
        IReadOnlyDictionary<MetadataReference, MetadataDefinition> catalog);
    public IReadOnlyList<StructuralMemberDescriptor> ResolveMembers(
        IReadOnlyDictionary<MetadataReference, MetadataDefinition> catalog);
}
```

No public constructor or setters. Sections contains all owned immutable sections;
Root/Context/Bindings expose decoded syntax and owners. Members contains supported
schema-1 rows. HasMemberTable distinguishes an absent/unsupported table from a present
empty schema-1 table. UnknownOptionalSections exposes unrecognized sections without
claiming to understand them; callers must inspect it when their own workflow needs
additional semantics.

ResolveType returns full catalog-scoped identity. ResolveMembers validates the owner
even when Members is empty, then derives supported contracts in table order. Both
require a non-null authoritative catalog (ArgumentNullException otherwise); unresolved
or incompatible declarations raise InvalidDataException. These methods do not cache
or retain the catalog; do not mutate it during a call. No assembly loading or runtime
object construction occurs.

This independently emitted example is compiled by the conformance consumer and matches
the shared Python tuple-member fixture:

```csharp
var document = MetadataProfile.Create(
    new TypeExpression("tuple", [new("int32"), new("string")]),
    new SignatureContext(), new ReferenceBindings([]),
    [new("tuple_element", Element: 1), new("tuple_deconstruct")]);
var bytes = MetadataProfile.Write(document);
var imported = MetadataProfile.Read(bytes);
var contracts = imported.ResolveMembers(
    new Dictionary<MetadataReference, MetadataDefinition>());
```

The focused consumer covers 36 shared profile vectors (29 rejections), independent
creation/emission, owned data, optional preservation and explicit resolution failure.

```sh
python3 docs/experiments/extended-cli-metadata/verify_dotnet_profiles.py
```

The future Raven adapter and potential Raven implementation for Metadata Introspection
are still planned. This one-root reference profile is not yet sufficient to represent
complete assemblies, their declarations, IL bodies or all compiler signatures.


## MetadataArtifactReader

```csharp
public static class MetadataArtifactReader
{
    public const int MaxImageSize = 4 * 1024 * 1024;
    public static MetadataArtifact Read(ReadOnlySpan<byte> image,
                                        bool expectedExtended = true);
}
```

Read consumes complete PE bytes without loading or executing the assembly. The default
requires a recognized extension: it rejects ordinary input and input with both markers
removed. With expectedExtended false, input having neither a `neoCLR.` version marker
nor a #Neo stream returns an ordinary classification. An unmarked #Neo stream, a marked
image missing #Neo, unknown marker versions or inconsistent bindings always fail,
regardless of that option. Automatic classification cannot recover erased provenance.

Supported input is deliberately restricted to the Python fixture transport contract:
at most 4 MiB, 1–16 nonoverlapping sections, PE32 with a 224-byte optional header and
16 directories, supported power-of-two file/section alignment (512–65536 for file
alignment), no overlay, no certificate directory or checksum, and a 72-byte CLI header
with flags exactly IL-only and no strong-name/native-header directory. PE32+, signed
and other unsupported layouts fail. This is not general-purpose PE validation.

The metadata root requires BSJB, an aligned version field of at most 256 bytes and at
most 16 streams. Stream names must be nonempty, unique, ASCII, null-terminated within
32 bytes and zero-padded to four bytes. Stream ranges must be aligned, file-backed,
outside the directory and nonoverlapping. All length/range arithmetic is checked
before accessing input; oversized and truncated values raise InvalidDataException.

The recognized marker is `neoCLR.NEOX.0.1;sha256=` plus 64 lowercase hexadecimal
characters and zero termination/padding. SHA-256 binds the domain separator and all
exact stream names/data in ordinal name order, including #Neo padding. #Neo's declared
envelope length must leave at most three zero padding bytes. The extracted envelope
must then pass MetadataProfile.Read, so a valid digest cannot bypass required schemas
or reference-profile validation. Local-only section-1 artifacts are not supported by
this higher-level reader.

All malformed, inconsistent or unsupported inputs raise InvalidDataException. Read
copies data needed by its result; callers must not mutate the span during the call.
The result does not depend on the input buffer or a retained stream lifetime. Only
span input is currently exposed; path/stream options and typed diagnostics remain
future API work.

## MetadataArtifact

```csharp
public sealed class MetadataArtifact
{
    public bool IsExtended { get; }
    public MetadataProfileDocument? Profile { get; }
}
```

No public constructor or mutable state. IsExtended is true exactly when Profile is
non-null: both recognition and local reference-profile validation passed. Ordinary
classification has IsExtended false and Profile null. It does not expose ordinary
CLI declarations or claim that conventional tables/signatures/bodies are valid.

The digest is a consistency check, **not authentication, IL verification or an
unaware-runtime execution guard**. A writer can recompute it around changed conventional
metadata; the reader deliberately does not claim to resolve or verify those tables.
Future compiler use still requires physical CLI binding and declaration validation.
The reader is a lower layer beneath the proposed Cecil-inspired assembly object model.

```sh
python3 docs/experiments/extended-cli-metadata/verify_dotnet_artifacts.py
```

The compiled consumer passes 50 shared Python/.NET cases (44 rejections), including
Cecil's extension-stripping rewrite, marker erasure, rebinding, reordered streams,
malformed ranges/names, wrong profiles, image-size limits and result ownership.


## Model namespace

Assembly: `NeoCLR.Metadata.Experimental`; namespace:
`NeoCLR.Metadata.Experimental.Model`. This first Cecil-inspired object-model slice is
**read-only and experimental**. It uses the existing artifact recognition layer and
System.Reflection.Metadata internally to read actual declarations, with no Cecil
package dependency. It does not load runtime assemblies. These names are distinct
from System.Reflection.Metadata and Mono.Cecil types; qualify or alias when needed.

### AssemblyDefinition

```csharp
public sealed class AssemblyDefinition
{
    public string Name { get; }
    public Version Version { get; }
    public AssemblyIdentity Identity { get; }
    public ModuleDefinition MainModule { get; }
    public MetadataProfileDocument? Profile { get; }
    public uint EntryPointToken { get; }
    public MethodDefinition? EntryPoint { get; }
    public byte[] Write();
    public static AssemblyDefinition ReadAssembly(ReadOnlySpan<byte> image,
                                                  bool expectedExtended = true);
}
```

No public constructor or setters. Name is the simple assembly name and Version is the
manifest version. Identity adds culture, normalized public-key token and retained
flags for exact explicit dependency matching (see below). MVID is not an assembly
identity. CLR binding redirects, unification and trust policy are not implemented.
MainModule is the owned manifest module; Profile is attached structural metadata, or
null for explicitly admitted ordinary CLI input. EntryPointToken is a managed MethodDef
token or zero for a library; nonzero tokens are checked against the method table.
EntryPoint returns that same owned MethodDefinition instance, or null for a library.
Write returns a fresh byte-for-byte copy of the original immutable snapshot, including
opaque data. It does not rebuild declarations or apply edits. Use AssemblyBuilder for
new controlled output.

ReadAssembly first applies MetadataArtifactReader's existing 4 MiB, unsigned IL-only
PE32, stream and required-profile rules. The default requires extended metadata;
expectedExtended false permits ordinary CLI assemblies. The method then reads the
assembly and module rows, TypeDef names/namespaces, generic parameter counts and
NestedClass ownership, AssemblyRef identities, and MethodDef names, ownership, flags,
generic arities and owned signature blobs. It accepts at most 256 AssemblyRefs,
a cumulative 4 MiB of key/token blobs, and at most 4096 TypeDefs and a cumulative 4 Mi UTF-16
code units of decoded declaration names/namespaces (counting repeated uses). It rejects missing/invalid
or cyclic declaring-type relationships. MethodDefs are limited to 4096 and copied
signature bytes to a cumulative 4 MiB shared with MemberRef blobs (including repeated
references to one blob). MemberRefs are limited to 4096, require nonempty signature
blobs, and have their parent table kind and row bounds checked.
Missing/empty method signatures and instance global functions are rejected. Nonempty
signatures outside the supported decoder remain opaque and are not generally validated.
Missing assembly manifests (netmodules),
invalid/unsupported artifacts and malformed inspected metadata raise
InvalidDataException; underlying BadImageFormatException is retained as InnerException
when converted. Callers must not mutate the span during reading.

The returned graph owns its strings/values and profile. Readers/streams are disposed
before return; no assembly loading, dependency discovery or open resource remains.
Only the inspected declaration subset is validated: this is not full signature, IL,
attribute, constraint or execution verification. MetadataProfile's catalog scope UUIDs
are not automatically derived from assembly names or MVIDs.

### ModuleDefinition

```csharp
public sealed class ModuleDefinition
{
    public AssemblyDefinition Assembly { get; }
    public string Name { get; }
    public Guid Mvid { get; }
    public IReadOnlyList<TypeDefinition> Types { get; }
    public IReadOnlyList<MethodDefinition> Methods { get; }
    public IReadOnlyList<MethodDefinition> Functions { get; }
    public MethodDefinition? GetMethodDefinition(uint metadataToken);
    public IReadOnlyList<MemberReference> MemberReferences { get; }
    public MemberReference? GetMemberReference(uint metadataToken);
    public IReadOnlyList<AssemblyReference> AssemblyReferences { get; }
    public IReadOnlyList<TypeReference> TypeReferences { get; }
    public TypeDefinition? GetTypeDefinition(uint metadataToken);
}
```

No public constructor. Assembly points back to the owning snapshot. Name/Mvid are the
stored module values; Mvid is not an experimental catalog module scope. Types is an
owned read-only collection of every TypeDef in row order, **including nested types
and `<Module>`**. This intentionally exposes a flat table view in the initial slice;
it does not promise Cecil's exact collection organization. AssemblyReferences is an
owned read-only list of physical AssemblyRef rows in metadata order; reading it performs
no resolver calls.

Methods contains all physical MethodDef declarations in row order. Functions contains
those physically owned by the first, top-level, empty-namespace `<Module>` row. In the
model these have null DeclaringType and the pseudo-type's Methods collection is empty.
Other definitions retain their actual type owner. GetMethodDefinition returns the same
snapshot object as these collections, or null for absent and non-MethodDef tokens.
No overload selection or dependency lookup is implied. MemberReferences contains all
physical MemberRefs in row order, including field/general signatures that this model
cannot yet resolve. GetMemberReference returns the owned row or null for missing/wrong
kinds; neither operation resolves a dependency.

GetTypeDefinition performs local physical TypeDef-token lookup and returns the same
owned definition instance. Zero, other token kinds and absent rows return null. It
does not follow exports or dependencies; TypeReferences exposes physical TypeRef rows
for explicit resolution. At most 4096 TypeRefs and 32 enclosing TypeRef scopes are
admitted; missing/cyclic parent and out-of-range AssemblyRef scopes are rejected.

### TypeDefinition

```csharp
public sealed class TypeDefinition
{
    public ModuleDefinition Module { get; }
    public uint MetadataToken { get; }
    public string Namespace { get; }
    public string Name { get; }
    public int GenericArity { get; }
    public TypeDefinition? DeclaringType { get; }
    public IReadOnlyList<MethodDefinition> Methods { get; }
    public TypeReference ToReference();
}
```

No public constructor. Module is the owning snapshot and MetadataToken its physical
TypeDef address. Namespace/Name preserve metadata strings, including any backtick arity
suffix; display names are not identity. GenericArity counts owned GenericParam rows,
including captured outer parameters if encoded; it does not expose or validate the
full generic constraint contract. DeclaringType links to the same graph's enclosing
definition, or null for top-level types. ToReference creates a new definition-backed
reference that resolves to this exact object. Methods exposes directly declared
callables. Fields, custom attributes, base types and interfaces remain outside this slice.

### MethodDefinition

```csharp
public sealed class MethodDefinition
{
    public ModuleDefinition Module { get; }
    public TypeDefinition? DeclaringType { get; }
    public uint MetadataToken { get; }
    public string Name { get; }
    public ushort Attributes { get; }
    public ushort ImplementationAttributes { get; }
    public int GenericArity { get; }
    public bool IsStatic { get; }
    public byte[] GetSignature();
    public bool TryGetStaticInt32Signature(out int parameterCount, out bool returnsValue);
    public bool TryGetStaticPrimitiveSignature(out PrimitiveMethodSignature? decoded);
}
```

No public constructor or mutation. Module and DeclaringType refer to the same owned
snapshot; null DeclaringType means a global function. TypeDefinition.Methods contains
only its directly declared methods, not inherited or nested-type methods. MetadataToken
is a physical MethodDef token, not cross-module identity. Name is the stored name;
Attributes/ImplementationAttributes retain CLI flag bits and IsStatic tests bit 0x10.
GenericArity counts method GenericParam rows without interpreting their constraints.

GetSignature returns a fresh copy of the CLI signature blob. It preserves unsupported
encodings and never simplifies them into a supported signature. Returned bytes and
original input bytes can be modified without changing the snapshot. No body, parameter
names, custom attributes, constraints or general signature type resolution is provided
by MethodDefinition. Use MemberReference for the supported reference-resolution subset.

TryGetStaticInt32Signature recognizes the original Int32-only writer contract: static,
nongeneric, default calling convention, 0–256 Int32 parameters, and Int32 or absent
CLI void result. Success sets parameterCount and returnsValue; false resets them to
zero/false. Instance/generic/vararg headers, other types, noncanonical count encodings,
truncation and trailing data return false. False means the narrow decoder cannot accept
the signature; it does not distinguish a valid unsupported signature from malformed
opaque data. ReadAssembly is not a general signature or execution verifier.

The C# consumer reads an emitted entry point, globals and overloaded type-owned methods,
checks shared object identity and copy isolation, and covers counts 0/1/127/128/256,
no-result returns, unsupported signatures, 4096/4097 rows and repeated-blob amplification.

### MemberReference

```csharp
public sealed class MemberReference
{
    public ModuleDefinition Module { get; }
    public uint MetadataToken { get; }
    public uint ParentToken { get; }
    public string Name { get; }
    public byte[] GetSignature();
    public MethodDefinition ResolveMethod(IAssemblyResolver? resolver = null);
}
```

The snapshot owns these rows; there is no public constructor or mutation. Module is
the consuming module. MetadataToken is its physical MemberRef token; ParentToken is
the physical MemberRefParent token, and Name is the referenced name. GetSignature
returns a fresh copy, including opaque field or unsupported method signatures.

ResolveMethod supports the writer's static, nongeneric default-convention Int32/Boolean
parameters (0–256) and Int32/Boolean/no-result contract. It resolves a local TypeDef or nominal
TypeRef parent, requiring the explicit resolver for external scopes. It then selects
exactly one directly declared method by ordinal name and decoded parameter/result
contract, returning that target snapshot's owned MethodDefinition. There is no implicit
filesystem probing, binding cache or access-policy decision. Resolver errors propagate;
missing/wrong-identity dependencies, unsupported contracts/parents, and absent/ambiguous
matches raise InvalidDataException. A return-contract mismatch cannot select a method.

Field, instance/generic/vararg and nominal signature types remain opaque. ModuleRef,
TypeSpec and MethodDef parents are range-checked on read but unsupported for resolution.
Global MemberRefs and inherited method lookup are also unsupported. Local global calls
emitted by the writer use MethodDef tokens and remain available through method lookup.
Resolution never compares module-relative nominal signature tokens as cross-module
identity; general type-aware signature comparison remains future work.

C# tests resolve writer-emitted cross-assembly overloads and no-result methods, and
exercise local TypeDef/TypeRef parents, copy isolation, missing/mismatched dependencies,
return mismatches, ambiguity, host failures, unsupported signatures, invalid parents,
4096/4097 rows and the shared decoded-signature budget.

### TypeReference

```csharp
public sealed class TypeReference
{
    public ModuleDefinition Module { get; }
    public uint MetadataToken { get; }
    public string Namespace { get; }
    public string Name { get; }
    public uint ResolutionScopeToken { get; }
    public TypeDefinition Resolve(IAssemblyResolver? resolver = null);
}
```

Created from TypeDefinition.ToReference or a physical TypeRef row. Module is the target
module for a definition-backed reference and the consuming module for a physical row.
MetadataToken identifies that TypeDef/TypeRef in Module; Name/Namespace preserve stored
metadata strings. ResolutionScopeToken is the physical scope, or zero for a
definition-backed reference/nil scope.

Resolve returns the same owned TypeDefinition for a definition-backed reference. For
physical references it supports local Module scope, exact AssemblyRef dependencies
through the supplied resolver, and nested TypeRef scopes. Names/namespaces and nesting
must identify exactly one target definition; missing or ambiguous targets, missing
resolvers, nil/multi-module scopes or mismatched dependencies raise InvalidDataException.
Host resolver exceptions propagate. Exported-type forwarders and TypeSpec/generic
instantiation resolution are not supported. No implicit IO or assembly loading occurs.

All read-model classes use ordinary reference equality. Independent reads yield
distinct graphs even when tokens/MVIDs match. Structural identity remains separate.

### AssemblyIdentity

```csharp
public sealed class AssemblyIdentity : IEquatable<AssemblyIdentity>
{
    public AssemblyIdentity(string name, Version version, string culture = "",
                            string publicKeyToken = "", uint flags = 0);
    public string Name { get; }
    public Version Version { get; }
    public string Culture { get; }
    public string PublicKeyToken { get; }
    public uint Flags { get; }
    public bool Equals(AssemblyIdentity? other);
    public override bool Equals(object? obj);
    public override int GetHashCode();
}
```

Immutable exact metadata-matching identity. Name is nonempty; Version has four
components in 0–65535; Culture is the stored string (empty for neutral). PublicKeyToken
is empty or 16 hexadecimal characters, normalized to lowercase. Flags retains metadata
flags except PublicKey (bit 1), which expresses full-key versus token representation
rather than identity. Null constructor inputs raise ArgumentNullException; invalid
values or the PublicKey representation bit raise ArgumentException.

Equals uses ordinal name/culture comparison, exact version/flags and normalized token.
Null and other object types are unequal. Hashes are process-local and consistent with
Equals; `==` is not overloaded. This intentionally conservative policy does not implement
CLR case folding, redirects, version roll-forward or retargeting. Nonmatching flags,
including retargetable flags, fail exact resolution until a deliberate policy exists.

ReadAssembly converts a full public-key blob to the reversed final eight bytes of its
SHA-1 hash, yielding the conventional key token. Assembly definitions require a full
key with PublicKey set or an empty key with it clear; references permit a full nonempty
key with the flag, or an empty/eight-byte token without it. Invalid representations
raise InvalidDataException. Cryptographic key structure and signatures are not verified;
token matching is identity comparison, not authenticity or execution permission.
Original full keys, hash blobs and a complete writer representation are not exposed yet.

### AssemblyReference

```csharp
public sealed class AssemblyReference
{
    public ModuleDefinition Module { get; }
    public uint MetadataToken { get; }
    public AssemblyIdentity Identity { get; }
    public AssemblyDefinition Resolve(IAssemblyResolver resolver);
}
```

No public constructor. Module is the **consuming** module, MetadataToken its physical
AssemblyRef token (0x23), and Identity the requested dependency identity. Resolve calls
the supplied resolver, rejects null/missing or mismatched candidates, and returns the
matching snapshot unchanged. Null resolver raises ArgumentNullException; missing or
wrong identity raises InvalidDataException. Host resolver exceptions propagate without
being disguised as malformed metadata. Calls are not cached; dependency search, IO,
cache ownership and lifetime are explicit host policy. No runtime assembly is loaded by
the reference itself. Nominal TypeRef lookup uses this resolver; constructed TypeSpec/forwarder binding remains pending.

### IAssemblyResolver

```csharp
public interface IAssemblyResolver
{
    AssemblyDefinition? Resolve(AssemblyIdentity identity);
}
```

The host receives the requested identity and returns a candidate metadata snapshot or
null. It may choose how to obtain that candidate; AssemblyReference always rechecks
exact identity. Implementations own their concurrency, cache and resource policy.
Returning an arbitrary same-name assembly cannot bypass version/culture/key/flag checks.
The C# tests include both a candidate resolver and a throwing resolver.

Run the standalone C# contract suite directly (no Python or test packages required):

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests/NeoCLR.Metadata.Experimental.Tests.csproj --no-launch-profile
```

The original 11 identity/resolution groups (now part of a 14-group suite) pass: physical AssemblyRef ownership, explicit resolution and repeat calls,
missing/mismatched candidates, exact identity/hash rules, ECMA full-key/token golden
normalization, constructor and wire validation, 256/257-reference boundary, owned data
and exception propagation. Fixtures are real PE metadata images built in C# with
System.Reflection.Metadata. This executable returns nonzero on any failure; it is not
a `dotnet test` discovery project.


## Controlled PE and native assembly construction

Namespace: `NeoCLR.Metadata.Experimental.Model`. These builders are the first writer
part of the primary compiler abstraction. They construct new assemblies and allow body
editing before another write; they do **not** rewrite arbitrary read snapshots or claim
full Cecil compatibility. The current executable subset is top-level functions, public
static classes and methods with Int32/Boolean parameters and Int32/Boolean or CLI no-result return. This is a
compiler integration proof, not a complete language backend.

### AssemblyBuilder

```csharp
public sealed class AssemblyBuilder
{
    public AssemblyBuilder(AssemblyIdentity identity, AssemblyIdentity coreLibrary);
    public AssemblyIdentity Identity { get; }
    public AssemblyIdentity CoreLibrary { get; }
    public IReadOnlyList<TypeBuilder> Types { get; }
    public IReadOnlyList<MethodBuilder> Functions { get; }
    public MethodBuilder AddFunction(string name, int parameterCount = 0,
                                     bool returnsValue = true);
    public MethodBuilder? EntryPoint { get; set; }
    public TypeBuilder AddType(string @namespace, string name);
    public byte[] Write();
    public byte[] WriteNativeAssembly();
}
```

The constructor requires explicit output and core-library identities. Null raises
ArgumentNullException; output identities with a key token or flags raise
ArgumentException because signing/flagged output is unsupported. No host core library
is inferred. The core supplies the System.Object base reference. Types is a read-only
view of this mutable graph. Functions is its separate read-only view of top-level
functions. Do not mutate participating graphs during either write operation.

AddType adds a public abstract sealed class with a unique namespace/name pair. Namespace
may be empty; name must be nonempty and not `<Module>`. Combined length is at most 1024
characters and the assembly admits at most 256 types. Invalid/duplicate inputs raise
ArgumentException. EntryPoint may be null for a library or a local parameterless
Int32-returning or no-result method; it is checked at Write/WriteNativeAssembly.
A no-result entry uses CLI void for ordinary PE output and native `Void` with
`no_result: true` for format 5. Reference-only projections still have no CLI entry.
The native declaration reader accepts both supported results. Parameterized/foreign
entries remain invalid, and no-result bodies must return with an empty stack.
`EntryPointChecks.cs` verifies both global and type-owned entries, CLI invocation,
native container roundtrips and failures; Raven's consumer also verifies native zero exit.

AddFunction creates an assembly-owned function with no declaring type. Names must be
nonempty and at most 1024 characters, parameter counts 0–256, and an assembly admits
at most 256 top-level functions. Duplicate name/parameter-count pairs raise
ArgumentException. Function and type-method name scopes are independent.

Write validates every local body, assigns physical tokens/RVAs, imports foreign call
references, and emits an owned unsigned IL-only PE32 image. Bounds: 4096 total methods,
131072 total instructions, 256 imported assembly identities and 4 MiB output. Foreign
calls require the same core identity; an external dependency with the same identity as
the output is rejected. Invalid bodies, entry points, incompatible imports or exceeded
limits raise InvalidDataException. Call targets are typed builder methods; output
references include exact assembly identity, nominal type name and method signature.
Top-level functions are emitted as CLI global methods on the physical `<Module>` row;
that row is a transport detail, not a declaring TypeBuilder. Local calls and global
entry points are supported. Cross-assembly top-level calls currently raise
InvalidDataException in PE emission; use native emission for that case.

Unchanged repeated writes of the same graph are byte-identical. Each builder has a
fresh MVID, stable over its edits; equivalent independently constructed graphs need not
have identical bytes. PE content IDs/timestamps are derived deterministically.
No strong-name signing, resource/debug data, #Neo attachment or conventional-image
rewriting is implemented by Write. The writer cannot silently discard such data because
it accepts only its explicitly constructed subset, not an arbitrary loaded image.

WriteNativeAssembly validates the same local graph and emits owned UTF-8 bytes in
neoCLR's existing JSON assembly format 5. Save as `.neo.json` and pass dependencies
explicitly with `--module`; no PE importer, subprocess or runtime loading occurs inside
the writer. Native top-level functions have no type owner. Type-owned methods retain
an explicit owner. Both local and cross-assembly calls are supported; no-result returns
remain distinct from inhabited Void.

Native output contains assembly/module/type/method origin metadata, scoped tokens,
parameter-row absence (zero tokens), and exact dependency revisions. The transport
module name uses SHA-256 of a canonical JSON identity tuple; member names encode UTF-8
bytes with ownership separators. This is deterministic binding, not authentication.
The descriptive assembly identity retains name/version/culture/token/flags in that
canonical tuple. Different dependency builder objects sharing one identity are rejected.
Names used as native descriptions must be nonblank, free of control characters and
well-formed Unicode; failures raise InvalidDataException. Output is bounded to 4 MiB.
Native emission carries no MVID (format 5 has no such field), and does not attach NEOX
structural sections or promise arbitrary CLR/PE compatibility. The explicit core identity
is checked between builder call targets, but no core assembly is loaded for this
primitive-only subset. This format-specific backend is provisional and will evolve
with the native metadata format.

### TypeBuilder

```csharp
public sealed class TypeBuilder
{
    public AssemblyBuilder Assembly { get; }
    public string Namespace { get; }
    public string Name { get; }
    public IReadOnlyList<MethodBuilder> Methods { get; }
    public MethodBuilder AddMethod(string name, int parameterCount = 0,
                                   bool returnsValue = true);
}
```

Created only by AddType. Methods is a read-only view of owned methods in declaration
order. AddMethod adds a public static hide-by-signature method. The legacy overload uses Int32 parameters; returnsValue selects Int32 or CLI void/no-result.
The signature overload preserves Int32/Boolean parameter and result types. Names must be nonempty and at
most 1024 characters; parameter counts are 0–256; a type admits at most 256 methods.
Duplicate name/parameter-count pairs and invalid inputs raise ArgumentException.
Generic methods, fields, instance receivers and signature variants are future work.

### MethodBuilder

```csharp
public sealed class MethodBuilder
{
    public AssemblyBuilder Assembly { get; }
    public TypeBuilder? DeclaringType { get; }
    public string Name { get; }
    public int ParameterCount { get; }
    public bool ReturnsValue { get; }
    public void LoadConstant(int value);
    public void WriteConsoleLine(string text);
    public void LoadArgument(int index);
    public void Add();
    public void Subtract();
    public void Multiply();
    public void Call(MethodBuilder target);
    public void Return();
    public void ClearBody();
}
```

Created by AddMethod or AddFunction. Assembly always identifies its owning builder;
DeclaringType is null for a top-level function. Body operations append conventional IL: Int32 constants,
zero-based argument loads, arithmetic, static calls and return. Call accepts a method
from this graph or another builder; null raises ArgumentNullException. Each append
rejects more than 4096 instructions with InvalidDataException. ClearBody permits
replacement without changing signature/ownership.

Write checks argument indices, typed primitive stack effects, call parameter/result contracts,
a final return with the exact declared stack shape, and no earlier return. It derives
max stack and rejects underflow, extra results or missing return. Loops, branches,
general local types, exceptions and arbitrary raw IL are intentionally absent. Int32 locals are documented below. Runtime overflow
behavior remains that of the selected target/bridge; this slice does not reconcile
all CLR versus neoCLR arithmetic policies.

The C# writer consumer builds a dependency's `Twice(Int32) -> Int32` and an application
that calls it with 20, adds 2 and returns 42. It tests repeat writes, body edits, imported
TypeRef/MemberRef resolution, entry-point ownership and invalid-stack rejection. Twenty-one
standalone C# contract groups pass, including earlier identity/reference tests.

### neoCLR acceptance test

The C# integration runner creates both PEs through the public builder API, invokes the
existing Raven CLI bridge, assembles its output into a native `.neo.json` artifact,
then asks neoCLR to load/verify and run that artifact. Expected result and process exit
code are 42. No application IL or JSON is hand-authored for this test.

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- \
  --runtime-integration /path/to/neoclr /path/to/Probe.dll \
  /path/to/NeoCLR.CoreProbe.dll /path/to/System.neoil /fresh/output/directory
```

Inputs must be a matching runtime, bridge, core reference and composed Raven System
library. The runner refuses an existing output directory and writes a report with
artifact/tool hashes. It is a test CLI, not a library API. The bridge converts ordinary
CLI signatures and bodies into the current native format; it does not enable native
#Neo semantics. Direct PE/#Neo runtime loading and general compiler coverage remain
separate work.

### Direct native acceptance test

```sh
dotnet run --project tools/metadata/NeoCLR.Metadata.Experimental.Tests -- \
  --native-integration /path/to/neoclr /fresh/output/directory
```

The C# test emits a top-level `Twice` function in one assembly and a top-level entry
function in another. It also exercises a no-result function, a type-owned method,
argument loads and all three arithmetic operations. neoCLR directly verifies/loads
these API-produced files and returns 42. Missing dependencies and incorrect revisions
are rejected with the corresponding diagnostics. The runner preserves the artifacts
and a hash report. This requires only the metadata test executable and native runtime,
with its bundled System library; no Raven bridge or hand-authored application JSON is
involved. See [recorded evidence](https://github.com/marinasundstrom/neoCLR/blob/codex/extended-cli-metadata/docs/experiments/extended-cli-metadata/native-validation.json).

### Importing a read-only callable (development)

```csharp
ImportedMethodReference AssemblyBuilder.ImportReference(
    MethodDefinition definition, AssemblyIdentity dependencyCoreLibrary);
void MethodBuilder.Call(ImportedMethodReference target);

public sealed class ImportedMethodReference
{
    public AssemblyBuilder Owner { get; }
    public AssemblyIdentity AssemblyIdentity { get; }
    public string? Namespace { get; }
    public string? DeclaringTypeName { get; }
    public string Name { get; }
    public int ParameterCount { get; }
    public bool ReturnsValue { get; }
}
```

`ImportReference` copies a static, nongeneric primitive signature from an external
read-only definition. No producer builder, body, runtime load or resolver is needed.
`Owner` is the consuming builder; `AssemblyIdentity` is the exact dependency identity.
The namespace and type name are null for a global function. `ReturnsValue` is false
for no result. References expose no body editing or signature mutation.

The caller must supply the dependency's core-library contract explicitly; it must
equal the consuming builder's `CoreLibrary`. This is a host assertion, not a deduction
from CLI primitive bytes or a verification of the dependency's implementation. Native
dependencies must be separately supplied and use this writer's format-5 naming
contract. PE output uses ordinary AssemblyRef/TypeRef/MemberRef rows for type-owned
methods. Cross-assembly globals remain native-only. Access checks are not performed.

Nested/generic owners, instance/generic/other signatures, signed or flagged dependency
identities and imports of the output identity throw `InvalidDataException`. A single
builder admits at most 256 imported assembly identities and 4096 imported methods.
Different module MVIDs under one identity, or differing callable contracts under one
MVID/token, are rejected. Repeated compatible imports return the same reference;
MVID consistency is not content authentication. Null arguments throw
`ArgumentNullException`. `Call` rejects references from another consumer with
`ArgumentException`; stack validation still occurs at emission. A null literal passed
to the overloaded `Call` now requires a cast to the intended target type.

```csharp
var dependency = AssemblyDefinition.ReadAssembly(dependencyPe, expectedExtended: false);
var method = dependency.MainModule.Types.Single(t => t.Name == "Math").Methods.Single();
var reference = output.ImportReference(method, dependencyCoreIdentity);
var main = output.AddFunction("Main");
main.LoadConstant(21);
main.Call(reference);
main.Return();
output.EntryPoint = main;
var nativeImage = output.WriteNativeAssembly();
```

Twenty-two C# contract groups now pass. They cover owned imports after producer/input
mutation, exact PE method resolution, native global no-result calls, owner/core/identity
rejections, opaque unsupported signatures and stack errors. The native C# gate imports
its cross-assembly global from a PE snapshot and returns 42. The Raven consumer also
imports from the read-only snapshot and returns 42 in neoCLR.

### Native declaration reader and compiler reference projection (development)

```csharp
namespace NeoCLR.Metadata.Experimental.Model;
public sealed class NativeAssemblyDefinition
{
    public AssemblyIdentity Identity { get; }
    public IReadOnlyList<AssemblyIdentity> References { get; }
    public static NativeAssemblyDefinition ReadAssembly(ReadOnlySpan<byte> image);
    public byte[] CreateReferenceAssembly(AssemblyIdentity coreLibrary);
}
```

`ReadAssembly` copies an owned declaration snapshot from the bounded writer's native
format-5 UTF-8 JSON. `Identity` retains the exact unsigned assembly identity.
`References` retains exact direct native dependency identities in manifest order,
including implementation-only references omitted from the PE projection. The list is
owned and read-only; identities include version/culture/key/flags. It is bounded to
256 entries, performs no resolution or file/runtime loading, and is not a transitive
closure. Hosts must supply the required native dependency graph to the runtime.

This is a metadata reader, not a native verifier, arbitrary format-5 reader, or body
translator. Bodies remain opaque; the original native artifact must pass neoCLR's
verifier before execution. Disposing JSON parsing state or changing the input buffer
has no effect on the snapshot. No native file is loaded into a runtime by either API.

Supported declarations are public static Int32/Boolean/no-result functions (including globals)
and public static classes with no fields. The reader checks canonical identity tuples,
encoded module/type/function names, references, origins, tokens, owner order, entry
point signature, duplicate declarations and unsupported declaration fields. Unknown
root/declaration fields and duplicate JSON properties are rejected. It accepts at most
4 MiB of input, depth 64, one manifest/module, 256 references/types, 4096 methods,
256 methods per owner (including global scope), and 256 parameters per method.
Unsupported, inconsistent or malformed metadata throws `InvalidDataException`.
These consistency checks are not authentication and do not validate instruction bodies.

`CreateReferenceAssembly` creates owned PE bytes for the existing .NET semantic-loader
bootstrap. `coreLibrary` is explicit and must supply System.Object and
System.Runtime.CompilerServices.ReferenceAssemblyAttribute; null throws
`ArgumentNullException`. The PE contains callable/type declarations, the reference-only
attribute, and throwing placeholder bodies. Native bodies are never translated or
replaced with executable behavior. Entry points and native body dependency references
are omitted; this supported signature subset needs only primitive/core types. Exceeded
writer limits or incompatible assembly identities throw `InvalidDataException`.
Each projection can receive a fresh MVID; callers should reuse one projection/snapshot
for a compilation instead of treating repeated projections as the same module scope.

```csharp
var native = NativeAssemblyDefinition.ReadAssembly(nativeBytes);
var referencePe = native.CreateReferenceAssembly(explicitCoreIdentity);
var definitions = AssemblyDefinition.ReadAssembly(referencePe, expectedExtended: false);
// Register referencePe with the existing compiler loader and import definitions
// through AssemblyBuilder.ImportReference. Execute the original nativeBytes only.
```

This follows the .NET distinction between implementation and reference assemblies,
including ReferenceAssemblyAttribute and throwing placeholder bodies. The temporary
bridge adds a projection and cannot preserve arbitrary native semantics; a native
semantic-data provider should replace it. See Microsoft's
[reference assembly contract](https://learn.microsoft.com/en-us/dotnet/standard/assembly/reference-assemblies).

Twenty-four C# contract groups pass, including native Unicode/global/no-result
roundtrips, input ownership, malformed/unsupported metadata and count limits, reference
marker inspection, and .NET execution-load rejection with BadImageFormatException.
The Raven consumer starts from the native dependency, binds the projected declarations,
and emits applications that neoCLR executes with the original dependency to 42.

Native reference tests additionally cover exact same-name/different-version identities,
manifest order, repeated-call deduplication, collection immutability, source-buffer
ownership, omission of implementation-only PE references, and inconsistent/duplicate
native references. The compiler consumer now executes a three-assembly chain in neoCLR.

## RuntimeAssemblyContainer

Development-only host type in `NeoCLR.Metadata.Experimental`:

```csharp
public static class RuntimeAssemblyContainer
{
    public static byte[] Write(ReadOnlySpan<byte> nativeImage, AssemblyIdentity coreLibrary);
    public static byte[] WriteBinary(ReadOnlySpan<byte> nativeImage, AssemblyIdentity coreLibrary);
    public static byte[] Read(ReadOnlySpan<byte> image);
    public static AssemblyDefinition ReadCliProjection(ReadOnlySpan<byte> image);
}
```

`Write` accepts the bounded writer's native format-5 JSON and explicit core-library
identity supplying Object and ReferenceAssemblyAttribute. It validates declarations,
creates a reference-only CLI projection, then embeds a required NEOX execution section
(kind 256, schema 1) in a new PE32 `.neometa` section. Both metadata views come from
the same native snapshot. The native image plus its 32-byte envelope header/directory
must fit 1 MiB; the PE must fit 4 MiB. Null core throws ArgumentNullException;
unsupported declarations, exceeded bounds or malformed containers throw InvalidDataException.
Each call creates a fresh projection MVID; whole-PE byte determinism is not promised.

`Read` validates unsigned PE32 framing, recognition marker/digest, envelope and native
declarations. It returns an owned copy of native UTF-8 JSON; mutation cannot affect
the input or future reads. Ordinary PEs, unknown required schemas, optional/unsupported
execution sections, missing/changed bindings and malformed data throw InvalidDataException.
Optional unknown sections are ignored. Bodies are opaque to this host API: successful
inspection is not runtime admission, dependency resolution or typed verification.

`ReadCliProjection` performs the same checks and reads the physical CLI declarations
as a Cecil-style `AssemblyDefinition`. Its `Profile` is null (no structural reference
profile), `EntryPointToken` is zero and `Write()` preserves the complete original PE.
It additionally rejects unsupported/malformed CLI metadata. The native payload is the
runtime authority; this method does not prove semantic equality of arbitrary native
and CLI views. Consumers should use the aware writer and keep their compiler-reference
file and read snapshot identical. General rewriting and automatic resolution are absent.

```csharp
byte[] native = builder.WriteNativeAssembly();
byte[] pe = RuntimeAssemblyContainer.Write(native, builder.CoreLibrary);
File.WriteAllBytes("Program.dll", pe);
var declarations = RuntimeAssemblyContainer.ReadCliProjection(pe);
var nativeDeclarations = NativeAssemblyDefinition.ReadAssembly(RuntimeAssemblyContainer.Read(pe));
```

neoCLR `verify Program.dll` / `run Program.dll` accepts this container, including
`--module Library.dll` dependencies. The Rust host API is
`metadata_container::native_json(&[u8]) -> Result<&str, Fault>` for borrowed transport
extraction and `metadata_container::load(&[u8]) -> Result<Module, Fault>` for legacy validation, including bundled System linking. `assembler::ModuleInput::MetadataPe(&[u8])` supports mixed
explicit module sets; `LoadedProgram` remains responsible for linking and execution.
There is no guest loader API yet. CLI bodies are throwing reference stubs, never
executed by neoCLR. The payload still uses JSON; direct loading does not yet eliminate
text parsing or establish a speedup. See the [profile/design](../docs/design/extended-cli-metadata.md#direct-runtime-container-checkpoint--2026-09-30).

### Native console literal output

`MethodBuilder.WriteConsoleLine(string text)` appends native console output without
changing the surrounding Int32 stack. Text is limited to 64 KiB of valid UTF-8; null
throws ArgumentNullException, unpaired UTF-16 surrogates and exceeded literal bounds
throw ArgumentException. The assembly's aggregate console-literal bytes are limited
to 4 MiB before serialization (InvalidDataException); encoded output/container limits
still apply. `WriteNativeAssembly` lowers the operation to `ldstr`,
`System.Console.WriteLine(String)` and `pop` for the bundled System's Void-valued result.
System is the runtime's implicit platform dependency. CLI `Write()` rejects this
native-only operation with InvalidDataException; reference-only projections preserve
declarations and continue emitting throwing stubs. General string signatures and
Console overload import are not introduced.

The compiled Raven acceptance case prints Hello World directly from Main, then from
a separate Greet function called by Main. Both emitted containers load/verify/run
in neoCLR and exit zero. The API remains independent of Raven; compiler-side mapping
requires an explicit registered Console reference and only accepts string literals.

### Binary execution payloads (schema 2)

```csharp
public static byte[] RuntimeAssemblyContainer.WriteBinary(
    ReadOnlySpan<byte> nativeImage, AssemblyIdentity coreLibrary);
```

`WriteBinary` accepts the bounded writer's format-5 JSON (at most 4 MiB) as a host-side
intermediate and writes section 256/schema 2 using a definite-length CBOR profile.
The binary payload plus its 32-byte directory must fit 1 MiB; the PE still fits 4 MiB.
It has the same core/reference projection, ownership and null-core contract as Write.
InvalidDataException covers unsupported declarations, non-integer numbers, invalid
encoding, duplicate fields and exceeded bounds. Only signed Int64 numbers, valid UTF-8
text, arrays, text-keyed maps, booleans and null are encoded; tags, bytes and floats
are not part of this profile. Decode is limited to depth 64 and 262,144 items (including
keys), and rejects duplicate keys, nonminimal encodings and trailing bytes.

`Read` and `ReadCliProjection` now accept schemas 1 and 2. For schema 1, Read returns
the original JSON bytes; for schema 2 it reconstructs owned format-5 JSON with equivalent
values for the existing host reader. Original whitespace/numeric spelling are not
preserved. Reconstructed JSON remains bounded to 4 MiB. Snapshot Write still preserves
the full original PE byte-for-byte. The writer continues to use fresh projection MVIDs.
The original `RuntimeAssemblyContainer.Write` keeps emitting schema 1 for compatibility.

Rust `metadata_container::decode(&[u8]) -> Result<Module, Fault>` validates the container
and decodes either payload without linking; `load` applies the legacy validation,
including bundled System linking. Use ModuleInput::MetadataPe for explicit dependency sets.
Schema 2 is deserialized directly to the runtime model, with no JSON roundtrip.
`native_json` remains a borrowed schema-1-only extractor and faults for schema 2.
The CLI and ModuleInput::MetadataPe use decode/load and accept both encodings.
Binary containers have no embedded text for the debugger's source pane.
See the [binary profile and tradeoffs](../docs/design/extended-cli-metadata.md#binary-native-execution-profile--2026-09-30).


## NativeModuleContainer

Public static host class in `NeoCLR.Metadata.Experimental`:

```csharp
byte[] NativeModuleContainer.WriteBinary(ReadOnlySpan<byte> nativeImage);
byte[] NativeModuleContainer.Read(ReadOnlySpan<byte> image);
byte[] NativeModuleContainer.WriteLibraryBinary(ReadOnlySpan<byte> nativeImage);
```

`WriteBinary` translates existing format-5 JSON values to a standalone NEOX envelope
with required section 256/schema 2. Unlike RuntimeAssemblyContainer it does not require
canonical API-writer declaration names and does not construct a PE or CLI projection.
`Read` reconstructs equivalent owned JSON values. Neither call checks full declaration
schemas, resolves references or verifies bodies; the runtime performs those checks.
Header validation requires numeric format 5, nonempty module name and a functions array.
Members, origins, identities, native names, dependencies and instruction operands are
preserved as values. No tokens or references are remapped. Unknown optional envelope
sections are accepted but omitted from the returned JSON; retain original bytes if
opaque section preservation is required.

All calls throw InvalidDataException for malformed headers, framing, unsupported
binary values or exceeded limits. WriteBinary retains schema 2: input JSON is limited
to 4 MiB, the envelope to 1 MiB; the same integer-only, Unicode, depth and item limits as the PE binary profile
apply. Lexical JSON spellings are not preserved. There is no CLI stream binding digest
in this standalone form and no authenticity guarantee. It cannot be passed to Raven's
.NET MetadataReference loader.

```csharp
var image = NativeModuleContainer.WriteBinary(File.ReadAllBytes("System.neo.json"));
File.WriteAllBytes("System.neox", image);
```

Rust `metadata_container::decode_envelope(&[u8]) -> Result<Module, Fault>` accepts a
standalone execution envelope (schema 1, 2 or 3), deserializes the native model and rejects
invalid framing/required schemas. `load_envelope` additionally applies legacy validation,
including bundled-System linking for non-System modules. `ModuleInput::NativeEnvelope`
uses decode for explicit dependency sets. The CLI detects NEOX magic for the root,
`--module` and `--system`; existing PE and JSON handling is unchanged. No embedded
source text is available to the debugger.


`WriteLibraryBinary` emits required execution schema 3 in a standalone envelope.
It accepts negative Int64 values and nonnegative UInt64 values; Double operands retain
their exact unsigned IEEE-754 bits, including negative zero and NaN payloads. CBOR
floating-point values themselves remain unsupported. Input/reconstructed JSON is limited
to 32 MiB, the envelope to 8 MiB, and item count (including map keys) to 2,097,152.
Depth remains 64. `Read` accepts schema 2 with its original limits and schema 3 with
these larger budgets. The general MetadataEnvelope/MetadataSection public APIs retain
their 1 MiB limits; the larger envelope is scoped to NativeModuleContainer. The PE
RuntimeAssemblyContainer APIs continue to support schemas 1/2 only.

Older runtimes reject required schema 3; choose WriteBinary when the old profile's
bounds and signed-only numbers suffice. The translator tool now emits schema 3 by
default. This is an explicit compatibility change for that experimental tool, not a
silent relaxation of schema 2. Semantic admission, linking and verification remain
runtime responsibilities. Larger budgets increase possible memory/CPU costs; they
are finite limits, not streaming or lazy-loading guarantees.


### Rust host native assembly writer

`metadata_container::write_module(module: &Module) -> Result<Vec<u8>, Fault>` encodes
an immutable borrowed native module into owned standalone schema-3 NEOX bytes. It
serializes the model directly, without JSON, and preserves its definitions/references.
Unsupported semantic format (anything except 5), empty/whitespace module name, encoding
failures, payload/item/depth
budget violations return Fault. The byte sink rejects output beyond 8 MiB minus envelope
framing before extending its payload buffer; the completed encoding is checked by the
same schema guard as the reader. It does not link dependencies, validate declarations
or type-verify bodies. Callers perform those steps separately.

The CLI `assemble --format neox` uses the existing assembler/resolver and verifies the
complete load set before invoking this API and creating the output file. Only the root
module is serialized. This Rust-host function is not a new guest introspection API or
an addition to the .NET Cecil-style object model. The independent .NET NativeModuleContainer
reader validates the emitted wire format in the cross-reader consumer tests.


## NativeLibraryDefinition and NativeFunctionDefinition

Development APIs in `NeoCLR.Metadata.Experimental.Model`. These .NET-host types
are not guest Introspection APIs. They adapt native declarations to the existing
Raven .NET semantic importer; they do not constitute a complete native symbol provider.

```csharp
public sealed class NativeLibraryDefinition {
    public string ModuleName { get; }
    public IReadOnlyList<string> TypeNames { get; }
    public IReadOnlyList<NativeFunctionDefinition> Functions { get; }
    public static NativeLibraryDefinition ReadAssembly(ReadOnlySpan<byte> image);
    public byte[] CreateStaticInt32ReferenceAssembly(
        AssemblyIdentity projectionIdentity, AssemblyIdentity coreLibrary,
        IEnumerable<NativeFunctionDefinition> functions);
}
public sealed class NativeFunctionDefinition {
    public NativeLibraryDefinition Library { get; }
    public int TableIndex { get; }
    public string Name { get; }
    public string? DeclaringTypeName { get; }
    public bool TryGetStaticInt32Signature(out int parameterCount);
}
// Additional MethodBuilder overload:
public void Call(NativeFunctionDefinition target);
```

`ReadAssembly` accepts standalone binary schemas 2/3 through NativeModuleContainer,
including the translated System library. It owns the decoded data and inventories all
functions, even unsupported signatures. `ModuleName` and `Name` retain native spellings;
`TypeNames` retains table order and can repeat for differing generic arities. Duplicate
name/arity types fail. Inventories are limited to 65,536 types/functions, descriptive
names to 4,096 characters, and transport retains its existing byte/depth/item limits.
Malformed framing/inventory raises InvalidDataException. This is not runtime body or
complete declaration validation. `TableIndex` is snapshot-local position, not an inferred
CLI token or persistent native identity. `DeclaringTypeName` is a nominal Named owner;
null also covers generic/primitive owner forms that this callable view does not support.

`TryGetStaticInt32Signature` accepts static nongeneric nominal-owner functions with
0–256 Int32 parameters and an Int32 result. Private, instance, virtual, abstract,
byref-receiver, no-result, Result, generic and other signatures return false and count
zero. Native visibility defaults to public when omitted; explicit nonpublic visibility or
nonpublic origin access is rejected. It does not promise the owning type is
projectable; the projection separately checks owner existence, public visibility and
nongeneric shape.

The projection requires an explicit **partial** selection of 1–4096 owned function
objects and an unsigned synthetic identity distinct from the supplied core identity.
At most 256 distinct owners are projected. Empty, duplicate, foreign or unsupported
selections, duplicate signatures and invalid owners fail with InvalidDataException.
Null arguments fail with ArgumentNullException. The returned ordinary reference-only
PE preserves selected method names and Int32 signatures under original type names,
using static containers and throwing bodies. It does **not** preserve native assembly
identity, type instance/field/property shape, generic contracts, parameter names or
attributes. No methods outside the explicit selection are claimed to be available.
Treating the projection as a complete core library or as an executable implementation
is unsupported. Native/native-host type-name collisions still require a proper core
provider; the current command removes host facade references in selected-native mode and the
consumer verifies that Math.Min binds to the native view, not the host implementation.

`MethodBuilder.Call(NativeFunctionDefinition)` accepts only a recognized Int32
callable from module `System`; null raises ArgumentNullException, other modules or
signatures raise InvalidDataException. The native writer retains the original owner,
function name and parameter signature; it does not synthesize writer-specific hashed
identities for System methods. Stack underflow is rejected during writing. Ordinary
CLI writing rejects this native-only operation. Runtime verification and execution must
use the matching explicit System assembly. No revision or image digest is encoded for
that implicit System dependency; this is a bootstrap limit to replace with a general
native assembly binding contract.

Compiled examples and failures: `NativeLibrarySymbolChecks.cs` in the metadata C# tests
and Raven's `SystemSymbolChecks.cs` consumer. The builder now provides the bounded opcode/operand Emit API below. An editable
instruction collection and full Cecil body-editing support remain future work.


## OpCode and MethodBuilder.Emit

Development APIs in `NeoCLR.Metadata.Experimental.Model`. The enum describes logical
instructions supported by both writer backends; its numeric values are **not** physical
CLI or native opcode bytes.

```csharp
public enum OpCode { Ldc_I4, Ldarg, Add, Sub, Mul, Call, Ret, Ldloc, Stloc, Ceq, Clt, Cgt, Br, Brtrue, Brfalse, Ldc_Bool }
public sealed partial class MethodBuilder {
    public void Emit(OpCode opCode);
    public void Emit(OpCode opCode, int operand);
    public void Emit(OpCode opCode, MethodBuilder operand);
    public void Emit(OpCode opCode, ImportedMethodReference operand);
    public void Emit(OpCode opCode, NativeFunctionDefinition operand);
}
```

| Opcode | Operand | Contract |
| --- | --- | --- |
| Ldc_I4 | int | Push a signed Int32 constant. |
| Ldarg | int | Load the zero-based declared primitive argument; bounds checked when writing. |
| Add, Sub, Mul | none | Consume two Int32 values and push the arithmetic result. |
| Call | MethodBuilder | Use the target signature; external core/identity constraints checked when writing. |
| Call | ImportedMethodReference | Must belong to the consuming assembly builder. |
| Call | NativeFunctionDefinition | Native-only static Int32 System callable; matching System must be supplied at runtime. |
| Ret | none | Must finish the body with its exact declared return stack. |

Unknown enum values, missing/unexpected operands and wrong opcode/operand overloads
raise ArgumentException before appending. Null call operands raise ArgumentNullException;
foreign imported references raise ArgumentException. Unsupported native module/signature
raises InvalidDataException, as does exceeding the existing 4096-instruction method limit.
Failed Emit calls leave the body unchanged. Stack/argument/return validation remains
at Write/WriteNativeAssembly; raw emission does not bypass it. Ordinary CLI writing still
rejects native System calls. No caller-supplied object, byte array or metadata token
operand is accepted.

LoadConstant, LoadArgument, Add, Subtract, Multiply, all Call overloads and Return now
call Emit. Their supported semantics are unchanged. WriteConsoleLine remains a native
convenience expansion; general string operands, non-primitive local variables, exception
regions and editable instruction collections are unsupported. Future ILProcessor-like
editing must define instruction ownership and branch/exception target repair separately.

Example from the C# consumer, used with an Int32 method and a two-parameter Int32 helper:

```csharp
main.Emit(OpCode.Ldc_I4, 10);
main.Emit(OpCode.Ldc_I4, 4);
main.Emit(OpCode.Call, helper);
main.Emit(OpCode.Ret);
```

`EmitChecks.cs` proves helper/raw byte equivalence on the same graph, executes a CLI
arithmetic/call entry point to 42, and checks imported/native operands and failures.
Raven's native emitter now uses this surface; native runtime and translated-System
integration remain its executable consumer evidence.

## Int32 local slots (development, 2026-10-01)

The host-only `NeoCLR.Metadata.Experimental.Model` API adds:

```csharp
public sealed class LocalDefinition {
    public MethodBuilder Method { get; }
    public int Index { get; }
}
public sealed partial class MethodBuilder {
    public IReadOnlyList<LocalDefinition> Locals { get; }
    public LocalDefinition DeclareInt32Local();
    public void LoadLocal(LocalDefinition local);
    public void StoreLocal(LocalDefinition local);
    public void Emit(OpCode opCode, LocalDefinition local);
}
```

`DeclareInt32Local` allocates a stable zero-based slot, at most 256 per method;
exceeding that bound throws `InvalidDataException`. `Locals` is a read-only ordered
view. `ClearBody` clears instructions and retains locals/handles; rebuilding the method
is required to discard its declarations. The typed Emit overload accepts only `Ldloc`
and `Stloc`. Null throws `ArgumentNullException`, a different method owner or incorrect
opcode throws `ArgumentException`, and the existing instruction bound throws
`InvalidDataException`. Rejected instructions do not alter the body.

The existing `Emit(OpCode, int)` overload also accepts raw `Ldloc`/`Stloc` slot indices.
Writes reject out-of-range slots, stack underflow and loads before a store in the linear
body with `InvalidDataException`. Each write recomputes initialization; a previous body
or successful write does not initialize a rebuilt body. CLI emission writes Int32 local
signatures and init-locals method headers; native emission writes format-5 `locals` and
native local instructions. The reader accepts absent locals in older producer artifacts
and validates declared Int32/Boolean local lists; reference projections still omit executable
body details. Older experimental readers may reject the added `locals` field.

Unlike unrestricted Cecil bodies, this bounded API enforces initialization and stack
contracts when writing. Typed owner handles prevent accidental cross-method use; raw
indices support assembler consumers. General local types, address-taking and
scope/debug metadata remain outside this slice. The compiled C# `LocalChecks` consumer
executes CLI locals and checks native roundtrips and rejected contracts; Raven's local
assignment probe additionally verifies and executes the binary artifact in neoCLR.

## Branch labels and control flow (development, 2026-10-01)

```csharp
public sealed class BranchLabel { public MethodBuilder Method { get; } }
public sealed partial class MethodBuilder {
    public BranchLabel DefineLabel();
    public void MarkLabel(BranchLabel label);
    public void Emit(OpCode opCode, BranchLabel label);
    public void Emit(OpCode opCode, bool operand);
}
```

`DefineLabel` allocates a method-owned symbolic destination (maximum 4096;
`InvalidDataException` beyond that). `MarkLabel` marks its current position once.
The label Emit overload accepts `Br`, `Brtrue` and `Brfalse`; the Boolean overload
accepts only `Ldc_Bool`. Null labels throw `ArgumentNullException`; wrong owners,
repeated marks and incompatible opcodes throw `ArgumentException` without changing
instructions. Instruction-limit violations throw `InvalidDataException`. `ClearBody`
retains handles but removes marks, so reused destinations must be marked again.

Operand-free `Ceq` pops two matching Int32 or Boolean values and pushes Boolean;
mixed operand types reject. `Clt` and `Cgt` pop two Int32 values and push Boolean,
using signed comparisons. Boolean equality permits logical negation (`false; ceq`)
without treating native Boolean as Int32. Conditional branches pop Boolean, not Int32; conditions may
also use `Ldc_Bool`. General Boolean locals/signatures are not introduced here.

Writing computes typed stack states and definitely stored locals over the control-flow
graph, including backward edges. Unmarked targets, incompatible stack joins, wrong
operand types, uninitialized loads on any incoming path, reachable fallthrough and
unreachable executable instructions fail with `InvalidDataException`. Return instructions
may occur on multiple reachable paths, each with the declared result and no extra stack
values. Labels emit no runtime instruction. Unused/unreachable label marks are harmless.
The earlier linear-only validation contract is superseded for branch-capable bodies.

CLI destinations become byte displacements; native destinations become instruction
indices after convenience-operation expansion (including Console output). Labels do not
encode byte offsets themselves, so backend layouts remain independent. The native
runtime already supports these format-5 instructions; no binary schema changes are
needed. Compared with Cecil's general ILProcessor, this supports forward/backward branches
with owned labels and checked typed control flow, but still has no insertion/removal API,
exception regions or arbitrary opcode surface. C# FlowChecks covers executable loops and
invalid joins/initialization; Raven's executable consumer includes Console inside a loop.


## Primitive signatures (development, 2026-10-01)

All types below are in `NeoCLR.Metadata.Experimental.Model`.

```csharp
public enum PrimitiveType { Void, Int32, Boolean }
public sealed class PrimitiveMethodSignature
{
    public PrimitiveMethodSignature(PrimitiveType returnType,
                                    IEnumerable<PrimitiveType> parameterTypes);
    public PrimitiveType ReturnType { get; }
    public IReadOnlyList<PrimitiveType> ParameterTypes { get; }
}
MethodBuilder AssemblyBuilder.AddFunction(string name, PrimitiveMethodSignature signature);
MethodBuilder TypeBuilder.AddMethod(string name, PrimitiveMethodSignature signature);
PrimitiveMethodSignature MethodBuilder.Signature { get; }
PrimitiveMethodSignature ImportedMethodReference.Signature { get; }
bool MethodDefinition.TryGetStaticPrimitiveSignature(out PrimitiveMethodSignature? decoded);
```

The signature constructor copies up to 256 ordered Int32/Boolean parameters. Results
may also be Void, meaning no result, not an inhabited native Void value. Null parameters
throw ArgumentNullException; invalid enum values, Void parameters or excessive counts
throw ArgumentException. ParameterTypes is an immutable view of the copied array.

The typed declaration overloads retain existing name/owner/count bounds. A null
signature throws ArgumentNullException; duplicate name plus ordered parameter types
throws ArgumentException, regardless of result type. Thus `Identity(int)` and
`Identity(bool)` coexist, while return-only overloads reject. Legacy count/result
overloads still mean Int32 parameters and Int32/no-result return.

MethodBuilder.Signature and ImportedMethodReference.Signature expose the immutable
contract. ParameterCount includes both types; ReturnsValue includes Boolean. Calls
consume declared parameter types in reverse stack order, and Return checks the declared
result. Int32 is not implicitly interchangeable with Boolean. Entrypoints remain
parameterless Int32/no-result; Boolean entrypoints reject at write time.

The primitive recognizer returns a fresh signature on success or null/false for
unsupported or malformed encodings. It accepts only static nongeneric default CLI
calling convention, canonical parameter counts and exact primitive encodings.
TryGetStaticInt32Signature keeps its earlier stricter behavior and rejects Boolean.
MemberReference.ResolveMethod and AssemblyBuilder.ImportReference match full primitive
signatures, including ordered parameter types and result. Existing explicit resolver,
identity, core-contract and snapshot checks remain in force.

Native writers, readers and reference-only projections preserve these types without
a format/schema change. Older experimental readers may reject Boolean declarations.
Local declarations support Int32 and Boolean; selected System inventory imports remain the
separate Int32-only contract. Native bodies are still verified by neoCLR.

```csharp
var predicate = output.AddFunction("IsPositive",
    new PrimitiveMethodSignature(PrimitiveType.Boolean, [PrimitiveType.Int32]));
predicate.LoadArgument(0);
predicate.LoadConstant(0);
predicate.Emit(OpCode.Cgt);
predicate.Return();
```

C# contract tests cover mixed arguments, overloads, CLI execution, native projection,
MemberRef resolution, immutable signatures and invalid calls. Raven's integration
probe also executes the emitted Boolean calls in neoCLR, including a separately
compiled library referenced through its CLI declaration projection.


### Typed local declarations (development, 2026-10-01)

```csharp
LocalDefinition MethodBuilder.DeclareLocal(PrimitiveType type);
PrimitiveType LocalDefinition.Type { get; }
```

DeclareLocal accepts Int32 or Boolean, returning a stable method-owned slot with an
immutable Type. Void and unknown enum values throw ArgumentException; the shared
256-local limit throws InvalidDataException. DeclareInt32Local remains shorthand for
DeclareLocal(PrimitiveType.Int32). ClearBody preserves the slot and type, but resets
body-derived initialization: a load must follow stores on all reachable paths.

StoreLocal and raw Stloc require the value's type to match the slot, including Boolean;
LoadLocal/Ldloc push that type. Wrong types reject before either writer emits an image.
CLI local signatures and native local lists retain the type. Native declaration reading
accepts both primitive local types, including old artifacts with no local-list field.
Reference-only projections omit native executable bodies and locals as before.

The C# local contracts execute Boolean locals in CLI, inspect their reflected type,
roundtrip native projections and reject cross-type stores and uninitialized loads.
Raven's shared planner now carries local types to each backend and validates a program
that stores a predicate result, reassigns it and compares Boolean locals on both runtimes.
