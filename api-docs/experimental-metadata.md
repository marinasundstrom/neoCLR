# Experimental .NET metadata library

**Feature-branch development only; not part of Preview 11 or the neoCLR guest API.**
Assembly and namespace: `NeoCLR.Metadata.Experimental`. Target: .NET 10.
Source project: `tools/metadata/NeoCLR.Metadata.Experimental`.
This is the first reusable reader/writer slice intended for Raven's future symbol
loader and code-generation adapters. It reads/writes **NEOX 0.1 framing, structural signatures, reference tables and synthesized-member tables**,
and derives structural identities/member contracts against an explicitly supplied host catalog.
Bounded PE32 recognition and a read-only manifest-module/TypeDef model are implemented;
explicit AssemblyRef, nominal TypeRef and bounded method MemberRef resolution are
implemented. A controlled primitive/root-object builder writes ordinary CLI PE and native
format-5 assemblies, including native top-level functions. Direct PE/#Neo runtime loading now uses a transitional native execution section
with a reference-only CLI projection. A bounded binary native payload now avoids JSON parsing at runtime. General rewriting
and guest Introspection assembly loading remain pending.

## Namespace and types

- [Imported type signatures](#imported-type-signatures-development-2026-10-01): external nominal and constructed reference types.
- [Model namespace](#model-namespace): Cecil-inspired assembly/module/type definitions and scoped references.
- [Integer shifts](#integer-shifts-development-2026-10-01): Shl/Shr with Int32 counts.
- [Integer bitwise operations](#integer-bitwise-operations-development-2026-10-01): And/Or/Xor and helpers.
- [Signed remainder](#signed-remainder-development-2026-10-01): dividend-signed Int32/Int64 remainder.
- [Signed division](#signed-division-development-2026-10-01): typed Int32/Int64 quotient and execution faults.
- [MethodVisibility](#methodvisibility-development-2026-10-01): public/internal/private static method declarations.
- [TypeVisibility](#typevisibility-development-2026-10-01): public/internal static types and projection.
- [Argument stores](#argument-stores-development-2026-10-01): typed by-value slot reassignment.
- [String values](#string-values-development-2026-10-01): literals, signatures, locals and computed console output.
- [Imported generic methods](#imported-generic-methods-development-2026-10-01): bounded static MethodSpec calls across assemblies.
- [Primitive signatures](#primitive-signatures-development-2026-10-01): Int32/Int64/Boolean/String parameters and results.
- [MethodDefinition](#methoddefinition): callable declarations and bounded signature recognition.
- [MemberReference](#memberreference): physical references and explicit method resolution.
- [Branch labels and control flow](#branch-labels-and-control-flow-development-2026-10-01): Boolean conditions, joins and loops.
- [Int32 local slots](#int32-local-slots-development-2026-10-01): method-owned locals, raw indices and initialization checks.
- [Nominal signatures](#nominal-signatures): MethodSignature, SignatureType and owned class parameters/results.
- [Root-class locals](#root-class-locals): owned nominal slots and aliasing.
- [Primitive property associations](#primitive-property-associations): static/instance getter and setter metadata.
- [Root construction and instance bodies](#root-construction-and-instance-bodies): constructors, receiver calls and field operations.
- [Root classes and primitive instance fields](#root-classes-and-primitive-instance-fields): mutable layouts and field snapshots.
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
    public bool TryGetStaticValueSignature(out MethodSignature? decoded);
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

ResolveMethod supports the writer's static, nongeneric default-convention Int32/Int64/Boolean/String
scalar/vector parameters (0–256) and scalar/vector/no-result contract. It resolves a local TypeDef or nominal
TypeRef parent, requiring the explicit resolver for external scopes. It then selects
exactly one directly declared method by ordinal name and decoded parameter/result
contract, returning that target snapshot's owned MethodDefinition. There is no implicit
filesystem probing, binding cache or access-policy decision. Resolver errors propagate;
missing/wrong-identity dependencies, unsupported contracts/parents, and absent/ambiguous
matches raise InvalidDataException. A return-contract mismatch cannot select a method.

Field, instance/constrained-generic/vararg and nominal signature types remain opaque. ModuleRef,
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
static classes and methods with Int32/Int64/Boolean/String parameters and Int32/Int64/Boolean/String or CLI no-result return. This is a
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
    public TypeBuilder AddType(string @namespace, string name, TypeVisibility visibility);
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

### TypeVisibility (development 2026-10-01)

`public enum TypeVisibility { Public, Internal }` describes top-level static type
visibility. `AssemblyBuilder.AddType(namespace, name)` remains public by default;
its three-argument overload accepts only these two values and throws
`ArgumentOutOfRangeException` for other values, before adding a type. Names,
uniqueness and the 256-type bound retain the existing contract. `TypeBuilder.Visibility`
is a read-only `TypeVisibility` property set at creation. Methods remain public static.

CLI output uses standard Public/NotPublic TypeDef flags. Native output retains the
existing runtime `visibility: "internal"` and `origin.publicly_visible: false` for
internal types; public output retains its omitted visibility default. The bounded
native reader accepts old public artifacts and preserves visibility in its reference
projection; unknown visibility or inconsistent origin flags raise `InvalidDataException`.
Internal helpers can be called within the assembly but are inaccessible to external
Raven consumers through the projection. No nested visibility, friend assemblies or
nonpublic methods are added. This host API is separate from the guest API reference.

### TypeBuilder

```csharp
public sealed class TypeBuilder
{
    public AssemblyBuilder Assembly { get; }
    public string Namespace { get; }
    public string Name { get; }
    public TypeVisibility Visibility { get; }
    public IReadOnlyList<MethodBuilder> Methods { get; }
    public MethodBuilder AddMethod(string name, int parameterCount = 0,
                                   bool returnsValue = true);
}
```

Created only by AddType. Methods is a read-only view of owned methods in declaration
order. AddMethod adds a public static hide-by-signature method. The legacy overload uses Int32 parameters; returnsValue selects Int32 or CLI void/no-result.
The signature overload preserves Int32/Int64/Boolean/String parameter and result types. Names must be nonempty and at
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
    public MethodVisibility Visibility { get; }
    public int ParameterCount { get; }
    public bool ReturnsValue { get; }
    public void LoadConstant(int value);
    public void WriteConsoleLine(string text);
    public void LoadArgument(int index);
    public void Add();
    public void Subtract();
    public void Multiply();
    public void Divide();
    public void Remainder();
    public void BitwiseAnd();
    public void BitwiseOr();
    public void BitwiseXor();
    public void ShiftLeft();
    public void ShiftRight();
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

`ImportReference` copies a static signature containing primitives, vectors, dependency-local
reference types/constructions and optional unconstrained method parameters from an
external read-only definition. No producer builder, body, runtime load or resolver is needed.
`Owner` is the consuming builder; `AssemblyIdentity` is the exact dependency identity.
The namespace and type name are null for a global function. `ReturnsValue` is false
for no result. References expose no body editing or signature mutation.

The caller must supply the dependency's core-library contract explicitly; it must
equal the consuming builder's `CoreLibrary`. This is a host assertion, not a deduction
from CLI primitive bytes or a verification of the dependency's implementation. Native
dependencies must be separately supplied and use this writer's format-5 naming
contract. PE output uses ordinary AssemblyRef/TypeRef/MemberRef rows for type-owned
methods. Cross-assembly globals remain native-only. Access checks are not performed.

Nominal signature tokens must identify public top-level unconstrained invariant reference
types in that dependency snapshot (TypeDef). TypeRef signatures require a future explicit
resolver contract. Value types, nested/generic owners, instance/constrained-generic/other
signatures, signed or flagged dependency
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

Supported declarations are public static Int32/Int64/Boolean/String/no-result functions (including globals)
and public/internal static classes with no fields. The reader checks canonical identity tuples,
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
public enum OpCode { Ldc_I4, Ldarg, Add, Sub, Mul, Call, Ret, Ldloc, Stloc, Ceq, Clt, Cgt, Br, Brtrue, Brfalse, Ldc_Bool, Pop, Ldc_I8, Conv_I8, Conv_I4, Neg, Not, Ldstr, Starg, Div, Rem, And, Or, Xor, Shl, Shr }
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
| Pop | none | Discard one value of either supported primitive type; empty stack rejects when writing. |
| Ldarg | int | Load the zero-based declared primitive argument; bounds checked when writing. |
| Add, Sub, Mul, Div, Rem | none | Consume matching Int32/Int64 values and push the same-width arithmetic result. |
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
and validates declared Int32/Int64/Boolean/String local lists; reference projections still omit executable
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

Operand-free `Ceq` pops two matching Int32, Int64 or Boolean values and pushes Boolean;
mixed operand types reject. `Clt` and `Cgt` pop matching Int32/Int64 values and push Boolean,
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
public enum PrimitiveType { Void, Int32, Boolean, Int64, String }
public sealed class PrimitiveMethodSignature
{
    public PrimitiveMethodSignature(PrimitiveType returnType,
                                    IEnumerable<PrimitiveType> parameterTypes);
    public PrimitiveType ReturnType { get; }
    public IReadOnlyList<PrimitiveType> ParameterTypes { get; }
}
MethodBuilder AssemblyBuilder.AddFunction(string name, MethodSignature signature);
MethodBuilder TypeBuilder.AddMethod(string name, MethodSignature signature);
MethodSignature MethodBuilder.Signature { get; }
MethodSignature ImportedMethodReference.Signature { get; }
bool MethodDefinition.TryGetStaticPrimitiveSignature(out PrimitiveMethodSignature? decoded);
```

The signature constructor copies up to 256 ordered Int32/Int64/Boolean/String parameters. Results
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
MemberReference.ResolveMethod and AssemblyBuilder.ImportReference match full primitive/vector
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

DeclareLocal accepts Int32, Int64, Boolean or String, returning a stable method-owned slot with an
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


### Discarding call results (development, 2026-10-01)

`MethodBuilder.Emit(OpCode.Pop)` removes one evaluation-stack value, preserving all
values below it. It accepts no operand and works for Int32, Int64, Boolean or String. Emission with
an operand throws ArgumentException without appending an instruction. Writer flow
validation throws InvalidDataException on underflow, including popping after a
no-result call. A no-result call has no value to discard. CLI emission uses `pop`;
native emission uses neoIL `pop`, with no schema change. Existing limits and branch
join checks continue to apply. Tests execute discarded Boolean and Int32 calls in
CLI and verify native writer rejection of underflow; Raven's binary-assembly consumer
also executes local and imported statement calls in neoCLR.


### Int64 and integer conversions (development, 2026-10-01)

The primitive signature/local API also accepts `PrimitiveType.Int64` in parameters,
results and local slots. `MethodDefinition.TryGetStaticPrimitiveSignature`, typed
imports, MemberRef resolution, native reading and reference-only projections preserve
Int64. The legacy Int32 recognizer still rejects every Int64 signature. Entrypoints
remain parameterless Int32/no-result; Int64 entrypoints reject before writing.

```csharp
void MethodBuilder.Emit(OpCode opCode, long operand); // Ldc_I8 only
// Operand-free Emit also accepts Conv_I8 and Conv_I4.
```

Ldc_I8 carries the exact signed 64-bit operand, including both extrema. Other opcodes
on this overload throw ArgumentException; instruction limits throw InvalidDataException.
Pass a long operand (`42L`), since the int overload continues to recognize only its
existing operands. No implicit opcode/operand widening is performed.

Conv_I8 accepts Int32/Int64, sign-extending Int32; Conv_I4 accepts the same types and
retains the low 32 bits when narrowing. Both operations are unchecked. Boolean and
empty-stack conversions reject at write time with InvalidDataException. Add/Sub/Mul
and signed Clt/Cgt accept matching Int32 or Int64 operands; Ceq additionally accepts
matching Boolean. Arithmetic preserves width, comparisons produce Boolean. There is
no implicit mixed-width arithmetic or Boolean/integer interchange. Pop also discards
Int64 values. Typed calls, returns, local stores and branch joins preserve exact types.

CLI emission uses Int64 signatures, ldc.i8 and conv.i4/conv.i8. Native emission uses
the corresponding existing neoIL operations and Int64 type names, without a schema
change. Older experimental host readers may reject these newly admitted declarations.
Unsigned, floating-point, checked and user-defined conversions remain unsupported by
this bounded writer. Native verification is still required before execution.

C# tests cover extrema, sign extension, truncation, local types, native projection,
imports, strict legacy recognition and invalid conversions/mixed arithmetic. Raven's
consumer executes long locals and signed conversion boundaries in neoCLR and imports
an Int64 callable from a separately compiled binary library.


### Signed unary integer operations (development, 2026-10-01)

`MethodBuilder.Emit(OpCode.Neg)` negates the top Int32/Int64 value without changing
its width. Negating the minimum signed value wraps to that same value, matching the
CLI neg operation. `Emit(OpCode.Not)` complements every bit at the operand's width;
it is not Boolean logical negation. Neither opcode accepts an operand: other Emit
overloads reject with ArgumentException without appending an instruction. Existing
instruction bounds throw InvalidDataException.

Both writers reject empty stacks and Boolean operands with InvalidDataException
during flow validation. CLI emission uses neg/not; native emission uses the existing
neoIL neg/not operations without a format change. Checked, unsigned and floating-point
unary support is not implied. Native verification remains required. C# contracts check
both widths, extrema, invalid stack operands and native projections. Raven's shared
body path handles built-in signed unary + (identity), - and ~; the native consumer
executes minimum-value wrapping and complements in binary assemblies.

### String values (development, 2026-10-01)

```csharp
// NeoCLR.Metadata.Experimental.Model
// PrimitiveType.String and OpCode.Ldstr are appended enum members.
void MethodBuilder.Emit(OpCode opCode, string operand);
void MethodBuilder.WriteConsoleLine();
```

`String` is supported in PrimitiveMethodSignature parameters/results, DeclareLocal,
MethodDefinition.TryGetStaticPrimitiveSignature, imported method references and native
reference projections. It is a built-in text contract, not general nominal type support.
The legacy Int32 recognizer still rejects String; String entrypoints are invalid.

`Emit(Ldstr, text)` pushes a string. Text may be empty and contain embedded NUL or
supplementary Unicode characters; it must be non-null, valid Unicode and at most
65,536 UTF-8 bytes. Null throws ArgumentNullException; an incorrect opcode, unpaired
UTF-16 surrogate or oversized literal throws ArgumentException without appending an
instruction. The assembly-wide string/instruction/image limits still apply.
The CLI writer uses a user-string token and native output uses UTF-8 text with ldstr.

`WriteConsoleLine()` consumes one String value through the existing native System
console bootstrap and discards its inhabited Void result. It is native-only; CLI
Write rejects it. Stack mismatch/underflow fail at write time with InvalidDataException.
The existing string-argument overload remains a constant-output convenience.

String calls, returns, local stores, loads, Pop and control-flow joins retain exact
type checks. Arithmetic, numeric conversions, Ceq/Clt/Cgt and conditional branches do
not accept String. Null literals, string equality/concatenation and String instance
members are outside this bounded writer API; no interning/identity guarantee is made.

The C# StringChecks consumer covers CLI execution, native projection and imports,
Unicode/empty/NUL literals, exact UTF-8 bounds and rejected operations. Raven's native
probe additionally executes computed Unicode text and a separately compiled library.

### Argument stores (development, 2026-10-01)

```csharp
// NeoCLR.Metadata.Experimental.Model
void MethodBuilder.StoreArgument(int index);
void MethodBuilder.Emit(OpCode opCode, int operand); // now also accepts Starg
```

`Starg` consumes a value of the parameter's exact declared Int32/Int64/Boolean/String
type and replaces that by-value argument slot in the current invocation. StoreArgument
is a convenience for the same instruction. Parameters begin initialized; subsequent
Ldarg observes the replacement. Caller storage is not changed. Indices are zero-based
with no implicit receiver, bounded by the method signature (at most 256 parameters).

Invalid indices, including negative values, are rejected at writing even in unreachable
code. Stack underflow and mismatched stored types throw InvalidDataException before
bytes are returned. The general instruction limit applies at emission. The int Emit
overload retains its existing opcode/operand validation; Starg with other operand
kinds rejects with ArgumentException.

CLI output uses standard starg (FE 0B plus UInt16 slot); native output uses existing
format-5 starg. No signature or container schema changes are required. Mutability is
a source binding rule, not additional parameter metadata. Ref/out/in and receiver
assignment are outside this bounded API. C# contracts cover all four admitted types,
caller isolation, the last slot, bounds, empty stack and type mismatch.


### Signed division (development 2026-10-01)

`OpCode.Div` and `void MethodBuilder.Divide()` append operand-free signed integer
division. `Emit(OpCode.Div)` is equivalent to the helper. Writing requires two
matching Int32 or Int64 values and leaves one value of that width; stack underflow,
mixed widths, Boolean and String operands raise InvalidDataException before an image
is returned. Operand-bearing Emit overloads reject Div with ArgumentException.
The new enum member is appended; numeric enum values are not serialized opcodes.

Both CLI div and native div truncate toward zero. Dividing by zero or the minimum
signed value by -1 is legal to emit but faults when executed (CLR arithmetic
exceptions; neoCLR DivideByZero/ArithmeticOverflow faults). Neither helper performs
constant evaluation. No unsigned, floating or checked-context overload is provided.
Existing signature metadata and native transport are unchanged.


### Signed remainder (development 2026-10-01)

`OpCode.Rem` and `void MethodBuilder.Remainder()` append operand-free signed remainder;
`Emit(OpCode.Rem)` is equivalent. The typed stack rules and failures match Div: two
matching Int32/Int64 operands become one value of the same width. Invalid stacks
raise InvalidDataException when writing; operand-bearing overloads reject Rem with
ArgumentException. Rem is appended to the enum without renumbering previous members.

Ordinary nonzero results have the dividend's sign and smaller magnitude than the
divisor. Both writers use existing rem instructions. Zero divisors fault at execution;
neoCLR also faults on minimum/-1. CLI output follows the host CLR's rem behavior
(the tested host faults for minimum/-1 too; .NET documents this edge as platform
sensitive). No unsigned/floating remainder or new exception handling is added.


### Integer and Boolean bitwise operations (development 2026-10-01)

`OpCode.And`, `Or`, `Xor` and the operand-free `MethodBuilder.BitwiseAnd()`,
`BitwiseOr()`, `BitwiseXor()` helpers consume two matching Int32/Int64 or Boolean
values and produce the same type. Boolean operands use eager AND/OR/XOR truth tables;
these instructions do not short-circuit. `Emit(opCode)` is equivalent to the corresponding helper.
Sign bits participate normally; these operations do not overflow. CLI and native
output use existing and/or/xor instructions, with no metadata extension.

Writing rejects underflow, mixed types/widths or String operands with
InvalidDataException. Operand-bearing Emit overloads reject these opcodes with
ArgumentException; the instruction limit still applies. New enum values are appended.
Enum and nullable Boolean semantics are not added. Native Boolean operands require
the runtime Boolean bit-operation support introduced on `codex/extended-cli-metadata`
in `fa25609d`; older runtimes reject them. CLI keeps ordinary and/or/xor encoding.


### Integer shifts (development 2026-10-01)

`OpCode.Shl`/`Shr` and `void MethodBuilder.ShiftLeft()`/`ShiftRight()` consume an
Int32 count above an Int32 or Int64 value and leave a result with the value's width.
`Emit(opCode)` is equivalent. Left shifts discard high bits; signed right shifts
replicate the sign bit. No unsigned shift is included. Counts are stack values,
not Emit operands. Operand-bearing overloads reject Shl/Shr with ArgumentException.
Writing rejects missing operands, an Int64 count or a Boolean/String value with
InvalidDataException; count values are not range-checked by the writer.

These map directly to standard CLI/native shl/shr. For counts 0–31 or 0–63 respectively,
results agree. CLI results for negative/oversized counts are unspecified; native
execution keeps its existing count masking (low 5 or 6 bits). Callers needing portable
masked semantics can emit an explicit AND on the count before shifting. The API does
not insert that policy or change ordinary .NET codegen. The enum values are appended.


### MethodVisibility (development 2026-10-01)

```csharp
public enum MethodVisibility { Public, Internal, Private }
public MethodBuilder TypeBuilder.AddMethod(string name,
    MethodSignature signature, MethodVisibility visibility);
public MethodVisibility MethodBuilder.Visibility { get; }
```

The overload creates a static method with explicit access; existing AddMethod overloads
remain public. Public is subject to owner visibility, Internal restricts access to the
assembly, and Private to the declaring type. Invalid enum values raise
ArgumentOutOfRangeException; null signatures raise ArgumentNullException; name,
duplicate-signature and method-count rules are unchanged. Visibility is immutable.
Assembly functions retain public metadata representation; protected/friend access is
not added. MethodDefinition.Attributes already exposes the corresponding CLI bits.

CLI output uses standard Public/Assembly/Private MethodAttributes. Native output reuses
public/internal/private visibility and matching origin member_access (Internal maps to
Assembly). Public rows keep the omitted visibility default. The bounded native reader
accepts old public rows, validates access/origin consistency and preserves all three
values in the reference projection. It rejects unknown access, inconsistent flags and
nonpublic global functions with InvalidDataException. Older readers reject new nonpublic
rows. Bodies/references can describe forbidden calls: the writer is not an access checker;
Raven binding and runtime verification enforce access. ImportReference retains that rule.


### Assembly function access (development 2026-10-01)

`AssemblyBuilder.AddFunction(string name, MethodSignature signature,
MethodVisibility visibility)` creates an ownerless function with Public or Internal
access. Existing overloads remain Public. Name/signature bounds and duplicate checks
are unchanged. A null signature raises ArgumentNullException; invalid names, duplicate
signatures or exceeded limits raise ArgumentException. Private/unknown visibility
raises ArgumentOutOfRangeException before graph mutation: private access requires a
declaring type in the current native access model.

`MethodBuilder.Visibility` retains the selected access. CLI globals use ordinary
Public/Assembly MethodAttributes; native functions retain no owner and use existing
public/internal visibility. The bounded native reader preserves internal function
access in its CLI reference projection and rejects private ownerless definitions.
Older bounded readers reject internal global rows; no schema version changes. The
writer does not authorize calls; native verification checks resolved module identity.
Explicit entry selection may run an internal function in its defining module.

## Assembly function namespaces

Development host APIs (2026-10-01):

```csharp
MethodBuilder AssemblyBuilder.AddFunction(string @namespace, string name,
    MethodSignature signature, MethodVisibility visibility = MethodVisibility.Public);
string MethodBuilder.Namespace { get; }
```

Adds an assembly-owned function with no declaring type. The namespace is empty for
the global namespace; otherwise it consists of nonblank dot-separated segments,
valid Unicode, and no control characters. Combined namespace/name length is at most
1024 UTF-16 code units. Duplicate namespace/name/parameter-type signatures and more
than 256 assembly functions are rejected. Null namespace/signature throws
ArgumentNullException; invalid namespace/name/duplicate/limit throws ArgumentException;
unsupported visibility throws ArgumentOutOfRangeException. Rejection leaves the graph
unchanged. Existing AddFunction overloads use the empty namespace.

MethodBuilder.Namespace returns the declared function namespace or type-owner
namespace. Name remains the simple source name. ImportedMethodReference.Namespace
now also exposes a namespaced function's namespace (null for global-namespace
functions); DeclaringTypeName remains null for all ownerless functions.

Native metadata stores the namespace separately and includes it in the executable
name encoding. The temporary CLI projection uses global MethodDefs named
`<NeoFunction>{uppercase UTF8 namespace hex}_{uppercase UTF8 name hex}`. The prefix
is reserved by AddFunction even in the global namespace to avoid collisions. Existing
unnamespaced function encoding is unchanged. Native snapshot/reference creation and
ImportReference preserve the namespace through that encoding. Raw CLI snapshots
expose the encoded physical Name; ordinary .NET source loaders do not gain native
namespace-function lookup. Native bodies remain authoritative in PE/#Neo artifacts.

```csharp
var min = assembly.AddFunction("System.Math", "Min",
    new PrimitiveMethodSignature(PrimitiveType.Int32,
        new[] { PrimitiveType.Int32, PrimitiveType.Int32 }));
// min.Namespace == "System.Math"; min.DeclaringType == null
```

The native reader rejects inconsistent namespace/executable names and namespaces
on type-owned methods. The runtime retains the namespace and rejects malformed or
type-owned namespace annotations. Calls use exact executable names; source namespace
lookup belongs to the compiler. New output requires the matching reader/runtime;
older artifacts omit the field and retain their global-namespace behavior.

## Root classes and primitive instance fields

Development host API (2026-10-01), additional to existing static AddType:

```csharp
TypeBuilder AssemblyBuilder.AddClass(string @namespace, string name,
    TypeVisibility visibility = TypeVisibility.Public);
bool TypeBuilder.IsStatic { get; }
IReadOnlyList<FieldBuilder> TypeBuilder.Fields { get; }
FieldBuilder TypeBuilder.AddField(string name, SignatureType type,
    FieldVisibility visibility = FieldVisibility.Private, bool isReadOnly = false);
public enum FieldVisibility { Public, Internal, Private }
```

AddClass creates a nonabstract, nonsealed root reference class. CLI base is
System.Object; native metadata has no explicit base. No constructor is synthesized.
AddType continues to create static classes. Both share the 256-type limit and
namespace/name uniqueness. Invalid names/visibility/duplicates throw ArgumentException
(including ArgumentOutOfRangeException for visibility). Namespaces and type names
retain the existing AddType contract.

AddField declares instance storage, never a property; storage is mutable by default. It accepts Int32,
Int64, Boolean, String or an owned nonstatic root class, a unique nonblank name without controls or invalid Unicode,
up to 1024 characters, and defined FieldVisibility values. Invalid declarations or
more than 256 fields per type throw ArgumentException before mutation. Static owners
throw InvalidOperationException. Writing enforces at most 4096 assembly fields.
Literal and static fields remain unsupported here. Later generic and imported reference-type
field support uses the signature contracts documented below.
Null types throw ArgumentNullException; Void and foreign classes throw ArgumentException
before mutation. Forward and self references use exact output-builder identity.
FieldBuilder exposes read-only DeclaringType (TypeBuilder), Name (string), FieldType
(SignatureType), Visibility (FieldVisibility) and IsReadOnly (bool). Field handles are owned by their type.
Development API migration: rebuild consumers and inspect FieldType.Primitive or
FieldType.ClassType; primitive AddField calls continue through implicit conversion.
CLI fields use ordinary CLASS TypeDef signatures; native fields use existing Named
records, preserved by reference projection. This introduces no new binary schema.
Setting isReadOnly emits ordinary CLI InitOnly and the existing native field_readonly
flag. Executable writes fail with InvalidDataException when an ordinary method or a
constructor of another type stores the field. Declaration/reference projection preserves
these flags. The updated runtime enforces direct stores and returns readonly managed
field addresses outside the declaring constructor; reads and mutation of an object
referenced by a readonly field remain legal. This is shallow storage protection, not
deep immutability or an unsafe-memory sandbox. Raw unmanaged pointers remain outside
managed readonly guarantees. Constructors can acquire writable initialization references.
Development migration: rebuild AddField consumers for the added optional argument and
use the matching runtime; older runtimes do not enforce these flags during execution.
No new binary schema is introduced by this bridge extension.

Nominal stores require the exact declared class; null literals, nullable source
contracts and external class imports are not added by this slice.

Read-only snapshot additions:

```csharp
uint TypeDefinition.Attributes { get; } // physical TypeAttributes
IReadOnlyList<FieldDefinition> TypeDefinition.Fields { get; }
IReadOnlyList<FieldDefinition> ModuleDefinition.Fields { get; }
FieldDefinition? ModuleDefinition.GetFieldDefinition(uint metadataToken);
```

Fields are ordered physical Field rows, with at most 4096 rows and aggregate
signature bytes bounded by the reader's existing 4 MiB limit. GetFieldDefinition
returns null for absent/wrong-kind tokens. FieldDefinition exposes Module,
MetadataToken, DeclaringType, Name and ushort Attributes (FieldAttributes).
GetSignature() returns a new byte array; unsupported encodings remain opaque.
TryGetPrimitiveType(out PrimitiveType type) recognizes exact Int32/Int64/Boolean/
String field signatures, returning false and Void otherwise. No resolution or code
loading occurs. CLI global fields retain the physical module pseudo-type as owner.

```csharp
var order = assembly.AddClass("Example", "Order");
var number = order.AddField("Number", PrimitiveType.Int32);
var pending = order.AddField("Pending", PrimitiveType.Boolean, FieldVisibility.Internal);
```

Native/reference projection preserves class flags, field order/types/access and
module-scoped Field tokens in origin metadata. The bounded native reader rejects
unsupported shapes and inconsistent origin rows. Old readers cannot read new class/
field output. Construction and field instructions are described below.

## Root construction and instance bodies

Development host API (2026-10-01):

```csharp
MethodBuilder TypeBuilder.AddInstanceMethod(string name, MethodSignature signature,
    MethodVisibility visibility = MethodVisibility.Public);
MethodBuilder TypeBuilder.AddConstructor(IEnumerable<PrimitiveType> parameterTypes,
    MethodVisibility visibility = MethodVisibility.Public);
bool MethodBuilder.IsStatic { get; }
bool MethodBuilder.IsConstructor { get; }
void MethodBuilder.Duplicate();
void MethodBuilder.NewObject(MethodBuilder constructor);
void MethodBuilder.LoadField(FieldBuilder field);
void MethodBuilder.StoreField(FieldBuilder field);
void MethodBuilder.Emit(OpCode opcode, FieldBuilder field);
// Added enum values: Dup, Newobj, Ldfld, Stfld.
```

AddInstanceMethod creates a nonvirtual method on a root class. AddConstructor creates
an instance .ctor with Void result. Signatures contain only declared primitive
parameters (maximum 256), excluding the receiver. Argument slot zero holds the exact
declaring-class receiver; declared parameters start at one. Static methods keep their
existing indexing. LoadArgument supports both kinds; StoreArgument rejects receiver
stores when writing. Entry points must be static. Names .ctor/.cctor are reserved in
ordinary AddMethod/AddInstanceMethod; use AddConstructor. Null signatures/parameter
sequences throw ArgumentNullException. Invalid names, access values, duplicate
name/parameter signatures or limits throw ArgumentException; static owners reject
instance declarations with InvalidOperationException before mutation.

Emit(Dup) and Duplicate copy the top stack value, preserving reference identity.
Emit(Newobj, constructor) and NewObject consume declared constructor arguments and
push an owned class reference. Emit(Call, method) and Call consume declared parameters
and, for instance methods, the receiver below them. Direct Call to a constructor and
Newobj to an ordinary method throw ArgumentException. Builder method references retain
the existing external assembly identity/core contract; imported snapshot references
remain static-only.

Emit(Ldfld, field)/LoadField consume the exact declaring-class receiver and push the
primitive or owned nominal field value. Emit(Stfld, field)/StoreField consume receiver then value.
Field handles must belong to the output assembly; foreign fields/wrong opcodes throw
ArgumentException and null operands throw ArgumentNullException before mutation.
Writing rejects stores to readonly fields outside a declaring constructor, stack underflow, mismatched receivers/values/joins and receiver stores
with InvalidDataException; existing instruction/body bounds apply. Access control
remains enforced by the executing target, not a new writer-level access checker.

CLI output uses ordinary HasThis signatures, constructor flags, newobj/dup/ldfld/stfld
and Field tokens. A root constructor's CLI body receives an automatic six-byte
ldarg.0/call System.Object::.ctor prologue; branches target the declared body, never
re-run that prologue. Native roots need no base call, and use the existing instance,
newobj.ctor and field-slot contracts. This is an explicit backend initialization
contract, not arbitrary identical IL bodies or general constructor chaining.
Reference projections preserve instance signatures and constructor flags with throwing
bodies; executable native bodies still reside in the required #Neo payload.

No constructor is synthesized. Inheritance, virtual dispatch, constructor chaining,
indexed properties and external instance snapshot imports remain outside this bounded
producer. Owned nominal parameters/results/locals, fields and properties are supported. The C# fixture constructs an Order,
mutates it through one alias and reads through another; both targets return 42.
The separate Raven Order integration now compiles the unchanged declaration; see the
[bridge evidence](../docs/raven-cli-bridge.md).

## Primitive property associations

Development host API (2026-10-01):

```csharp
IReadOnlyList<PropertyBuilder> TypeBuilder.Properties { get; }
PropertyBuilder TypeBuilder.AddProperty(string name, SignatureType type,
    MethodBuilder? getter = null, MethodBuilder? setter = null);
```

PropertyBuilder exposes DeclaringType (TypeBuilder), Name (string), PropertyType
(SignatureType), GetMethod/SetMethod (nullable MethodBuilder), and IsStatic (bool).
These immutable associations add no storage or bodies. Accessors must already belong
to the same type, be ordinary methods and agree on static/instance shape. The getter
has no declared parameters and returns the property type; the setter takes one value
of that type and returns Void. At least one accessor is required. Accessor visibility
is preserved independently, including private setters. Ordinary accessor names are
allowed; CLI emission adds SpecialName to associated methods.

Non-indexed Int32/Int64/Boolean/String and owned root-class properties are supported.
Nominal types must belong to the output assembly; null throws ArgumentNullException,
and Void/foreign types throw ArgumentException before mutation. Accessor signatures
must use the exact same owned class identity. A unique nonblank
name has at most 1024 characters, no control characters or invalid Unicode. Invalid
contracts, duplicate names, accessors already associated with another property and
more than 256 properties per type throw ArgumentException before mutation. Writing
rejects more than 4096 assembly properties with InvalidDataException. The accessor
reuse/name limits are bounded producer restrictions, not permanent native rules.

CLI output contains Property, PropertyMap and MethodSemantics rows with ordinary
static/HasThis signatures. Native metadata uses existing property/accessor references
and exact origin Property tokens. The reader checks signature, owner, instance shape,
accessor existence, uniqueness and origins before constructing a throwing reference
projection. Outputs without properties retain their previous encoding. New property
outputs require the matching bounded reader. Properties do not imply backing fields;
Indexed and generic-owner properties are supported by the later slices below; external
nominal properties, attributes and default values remain unsupported.

Development API migration (2026-10-01): rebuild consumers of AddProperty and
PropertyBuilder.PropertyType and inspect SignatureType.Primitive/ClassType. Existing
primitive construction calls use implicit conversion. Nominal properties retain standard
CLI CLASS/TypeDef signatures and existing native Named records, including setter
parameter references. Static, read-only, write-only and private accessor associations
are supported without inventing storage or changing the native binary schema.

### Owned property snapshots

```csharp
IReadOnlyList<PropertyDefinition> ModuleDefinition.Properties { get; }
PropertyDefinition? ModuleDefinition.GetPropertyDefinition(uint metadataToken);
IReadOnlyList<PropertyDefinition> TypeDefinition.Properties { get; }
```

The module lists physical Property rows in metadata order; types list their declared
properties. Lookup returns null for absent/wrong-kind tokens. PropertyDefinition
exposes Module, MetadataToken, DeclaringType, Name, ushort Attributes (PropertyAttributes),
nullable GetMethod/SetMethod and IReadOnlyList<MethodDefinition> OtherMethods. Accessors
are the same owned objects returned by Module.GetMethodDefinition, with no assembly
loading or resolution. OtherMethods preserves ordinary Other associations.

GetSignature() returns new owned bytes, including unsupported signature encodings.
TryGetPrimitiveSignature(out PrimitiveType type, out bool isStatic) recognizes exact
non-indexed Int32/Int64/Boolean/String signatures; failure returns Void/false. This
helper decodes the property signature, not accessor compatibility. The general CLI
snapshot retains broader signatures opaquely; full CLI verification is not claimed.
Default values/custom attributes are not exposed by this snapshot.

Reading enforces at most 4096 Property rows, 16384 MethodSemantics rows and the shared
4 MiB aggregate signature budget. Missing/ambiguous owners, absent signatures and
accessors outside the declaring type fail with InvalidDataException. Input buffers
and returned signature arrays can be changed without affecting the snapshot.

## Root-class locals

Development host API (2026-10-01):

```csharp
LocalDefinition MethodBuilder.DeclareLocal(TypeBuilder type);
PrimitiveType? LocalDefinition.Type { get; }
TypeBuilder? LocalDefinition.ClassType { get; }
```

A local is either primitive (Type has a value, ClassType is null) or nominal (Type is
null, ClassType is the exact declared root class). **Development API migration:**
LocalDefinition.Type is now nullable; callers that assumed every local was primitive
must branch on Type/ClassType. Existing primitive overloads and slot ordering remain.
No Void sentinel or System.Type handle represents a nominal slot.

The class overload accepts only nonstatic root classes in the same output graph.
Null throws ArgumentNullException; static/foreign classes throw ArgumentException
before mutation. All locals share the existing 256-slot limit (InvalidDataException).
ClearBody preserves local declarations. Ldloc/Stloc and existing helpers preserve
reference identity and require exact declared class identity. Writing rejects wrong
primitive/class stores, different nominal classes and loads before a store on every
reachable path. Inheritance conversions, null constants and external nominal locals
are not admitted.

CLI local signatures use CLASS plus the TypeDef coded index; native locals use the
existing Named type contract. The bounded native reader validates local class ownership
and shape, then omits implementation locals from reference projections as before.
A matching reader is required for these new bodies. API C# tests and direct binary
neoCLR execution validate local aliasing/mutation to 42. The subsequent Raven
consumer validates source-level aliasing too; see the [integration record](../docs/raven-cli-bridge.md#raven-object-locals-and-aliasing--2026-10-01).


## Nominal signatures

**Development 2026-10-01.** `NeoCLR.Metadata.Experimental.Model` now exposes:

```csharp
public sealed record SignatureType {
    public PrimitiveType? Primitive { get; }
    public TypeBuilder? ClassType { get; }
    public static implicit operator SignatureType(PrimitiveType type);
    public static implicit operator SignatureType(TypeBuilder type);
    public override string ToString();
}
public class MethodSignature {
    public MethodSignature(SignatureType returnType, IEnumerable<SignatureType> parameterTypes);
    public MethodSignature(PrimitiveType returnType, IEnumerable<PrimitiveType> parameterTypes);
    public SignatureType ReturnType { get; }
    public IReadOnlyList<SignatureType> ParameterTypes { get; }
}
MethodSignature MethodBuilder.Signature { get; }
MethodSignature ImportedMethodReference.Signature { get; }
MethodBuilder TypeBuilder.AddConstructor(MethodSignature signature,
    MethodVisibility visibility = MethodVisibility.Public);
```

SignatureType has exactly one representation: a defined primitive (including Void only
for results), an exact nonstatic TypeBuilder identity, or a bounded vector (see
[vector declarations](#vector-declarations-development-2026-10-01)). Invalid primitive values and
static classes throw ArgumentException; a null class throws ArgumentNullException.
Its diagnostic ToString is not a persistent identity. Record equality retains exact
builder identity. MethodSignature copies at most 256 parameters; null input throws
ArgumentNullException and null/Void parameters or too many parameters throw
ArgumentException. Returned lists cannot be modified. Constructors require Void results.

AddFunction (both global and namespaced overloads), AddMethod and AddInstanceMethod
now take MethodSignature instead of PrimitiveMethodSignature. Before mutating declarations
they reject class types from another builder, including another builder with the same
assembly identity. Existing visibility, duplicate-parameter-signature and count limits
remain. Method return type does not distinguish overloads. PrimitiveMethodSignature now
derives from MethodSignature and retains its primitive-typed ReturnType/ParameterTypes
properties; primitive consumers can continue constructing it. **Development API migration:**
rebuild consumers against the changed method signatures. Code inspecting MethodBuilder
or ImportedMethodReference signatures must use `.Primitive` or `.ClassType`, rather than
assuming a primitive enum. Imported read-only method contracts accept primitives and primitive vectors; nominal imports remain unsupported.

CLI output uses CLASS TypeDef signatures; native output uses existing Named type records.
Calls, argument stores and returns enforce exact class identity, just like nominal locals.
No implicit base conversion, null literal or structural
signature support is added. Cross-assembly builder calls with nominal signatures reject
at write rather than fabricating a TypeRef. Entries remain parameterless Int32/Void.
NativeAssemblyDefinition accepts owned nonstatic Named signature references and remaps
them into each reference projection. MethodDefinition preserves CLI blobs, while its
primitive recognizers correctly decline nominal signatures. Reference-only output still
contains throwing placeholder bodies and must not be executed as the native implementation.

Compiled consumer pattern (covered by NominalSignatureChecks):

```csharp
var identity = graph.AddFunction("Identity", new MethodSignature(order, [order]));
identity.LoadArgument(0);
identity.Return();
var self = order.AddInstanceMethod("Self", new MethodSignature(order, []));
self.LoadArgument(0);
self.Return();
```

Here `order` is graph.AddClass's owned result. The C# contract test passes an allocated
instance through both calls, mutates it through a nominal parameter and reads 42 through
its alias on .NET and binary neoCLR. Wrong-class arguments/results and foreign signatures
reject. Raven additionally validates owned nominal constructor parameters and overloads.

## Vector declarations (development, 2026-10-01)

```csharp
SignatureType SignatureType.ArrayOf(SignatureType elementType);
SignatureType? SignatureType.ArrayElement { get; }
LocalDefinition MethodBuilder.DeclareLocal(SignatureType type);
SignatureType LocalDefinition.SignatureType { get; }
```

`ArrayOf` creates a one-dimensional zero-based vector of Int32, Int64, Boolean,
String or an owned non-static root class. Null throws ArgumentNullException; Void
and nested arrays throw ArgumentException. Declaration APIs reject foreign element
owners before mutation. Method, field and property signatures share this contract;
`DeclareLocal` also rejects Void and enforces the existing 256-slot limit.
`SignatureType.ArrayElement` is null for scalar types. `LocalDefinition.Type` and
`ClassType` are both null for vectors; use `SignatureType` to inspect all slot kinds.
For example, `method.DeclareLocal(SignatureType.ArrayOf(PrimitiveType.Int32))`
creates an integer-array slot. Stack joins, calls and stores require exact element
identity; covariance is not admitted by this bounded writer.

CLI output uses standard SZARRAY signatures; native output uses ArrayRef, preserving
reference identity. The native reader validates elements and reconstructs the same
CLI signatures for reference assemblies. These host-only C# APIs remain covered by
this manual reference, not the RavenDoc runtime snapshot. Allocation/indexing APIs are described below. C# contract tests execute array
parameter/result/local and property/field aliasing on .NET and compare native projections.

### Vector body operations

```csharp
void MethodBuilder.Emit(OpCode opCode, SignatureType elementType);
void MethodBuilder.NewArray(SignatureType elementType);
void MethodBuilder.LoadArrayElement(SignatureType elementType);
void MethodBuilder.StoreArrayElement(SignatureType elementType);
void MethodBuilder.LoadArrayLength();
```

The typed Emit overload accepts Newarr, Ldelem and Stelem with scalar element types
accepted by ArrayOf. Wrong opcode/element/ownership throws ArgumentException; null
throws ArgumentNullException, with no body mutation. Helpers have the same validation.
Allocation consumes Int32 length and pushes a reference array with default-initialized
elements. Loads consume array/index and push an element; stores consume array/index/value.
Exact element identity is checked at write time (InvalidDataException); bounds and
negative lengths fault at execution. New arrays of reference elements contain null
references until initialized. The current API does not expose null literals.

Operand-free Emit(OpCode.Ldlen) consumes an array and pushes native unsigned length,
which the bounded flow checker admits into Conv_I4, duplication or discard. Returning
it as Int32 without conversion rejects. LoadArrayLength appends ldlen/conv.i4, checks
the instruction limit before appending either, and produces Int32. CLI uses standard
newarr/ldelem/stelem type tokens and ldlen; native uses the corresponding typed operations.
One scalar primitive TypeRef is cached per output kind. No nested arrays, spans,
covariance, element addresses or imported nominal elements are admitted. C# tests
verify CLI execution and API-produced binary verification/execution on neoCLR.

## Indexed property associations (development, 2026-10-01)

`TypeBuilder.AddProperty` now also accepts indexed getter/setter methods. Its existing
signature is unchanged. `PropertyBuilder.ParameterTypes : IReadOnlyList<SignatureType>`
exposes a copied immutable list of index types, excluding receiver and setter value.
For a getter, all parameters are indices; for a setter-only property, all but its last
parameter are indices. The final setter parameter must equal PropertyType and its
result must be Void. Both accessors must agree on index types and instance/static
shape. Primitive, owned-class and vector index types follow MethodSignature rules.
A getter may have 256 indices; a setter leaves at most 255 within the method limit.

Properties overload by name plus exact index parameter sequence; return type alone
does not distinguish overloads. Duplicate signatures, reused accessors and inconsistent
getter/setter signatures throw ArgumentException without adding a property. This
relaxes the former name-only uniqueness check. Rebuild development host consumers to
use ParameterTypes. Ordinary non-indexed properties have an empty list.

CLI output encodes standard Property signatures/MethodSemantics; native output uses
the existing property parameters list and accessor function references. The native
reader validates these lists and reproduces CLI reference signatures. PropertyDefinition
retains the full signature through GetSignature; its non-indexed TryGetPrimitiveSignature
helper deliberately returns false for indexers. No DefaultMemberAttribute is synthesized:
Raven source binding already identifies its indexers, while other CLI compilers may need
that attribute to recognize source-level indexing. Native introspection's no-index
GetValue/SetValue convenience calls are not expanded by this producer change.

Example: define `GetItem(Int32) -> Int32` and `SetItem(Int32, Int32) -> Void`, then
`owner.AddProperty("Item", PrimitiveType.Int32, get, set)`. The resulting ParameterTypes
contains one Int32. The same name may have a separate Int64-index getter. C# tests
execute these accessor associations on .NET and inspect native reference projections.

The C# `--indexer-integration <runtime> <fresh-output>` check also writes a binary
assembly with overloaded getters and a two-index setter-only association. neoCLR
loads and verifies its property contracts and executes the accessor calls to 42.
This uses existing runtime metadata validation and call instructions, not an indexer
opcode or a new runtime introspection invocation API.

## Generic method declarations (development, 2026-10-01)

```csharp
SignatureType SignatureType.MethodParameter(int index);
int? SignatureType.MethodParameterIndex { get; }
MethodSignature(SignatureType returnType, IEnumerable<SignatureType> parameterTypes,
    IEnumerable<string>? genericParameterNames = null);
IReadOnlyList<string> MethodSignature.GenericParameterNames { get; }
```

MethodParameter creates a positional method type reference (MVAR); indices outside
0–31 throw ArgumentOutOfRangeException. Declaration/local/instruction use checks the
index against the current method's declared arity and throws ArgumentException for
invalid scope. Fields and property accessors cannot use method generic parameters.
The signature constructor copies up to 32 unique nonblank valid-Unicode names of at
most 256 characters without controls; invalid names/counts throw ArgumentException.
Names are immutable and preserved in CLI GenericParam rows and native generic_parameters.
Unconstrained ownerless functions and static/ordinary instance methods on owned root
classes are admitted. Generic constructors, generic types and constraints remain outside
this API slice.

For example, `new MethodSignature(SignatureType.MethodParameter(0),
[SignatureType.MethodParameter(0)], ["T"])` defines `Identity<T>(T) -> T`.
Method generic parameters also work in locals and vector elements. CLI uses MVAR and
TypeSpec operands for typed array instructions; native uses MethodTypeParameter.
Direct Call/Emit(Call, MethodBuilder) on an open generic definition throws
ArgumentException. Generic entries are invalid. Overload uniqueness includes generic
arity; return type still does not distinguish methods. MethodDefinition.GenericArity
and raw signatures retain generic declarations in native CLI reference projections;
primitive-only recognizers continue to decline them. This extends the development
MethodSignature constructor; rebuild host consumers.


### Instantiated generic calls (development)

`MethodBuilder.MakeGenericInstance(params SignatureType[] typeArguments)` returns an
immutable `GenericMethodInstance` exposing `Definition`, copied `TypeArguments` and
substituted `Signature`. `Call(GenericMethodInstance)` and
`Emit(OpCode.Call, GenericMethodInstance)` append an owned call. Null arguments throw
ArgumentNullException; wrong arity, Void, foreign class/definition, wrong opcode,
out-of-scope caller parameters and nested-array substitution throw ArgumentException.
Body stack compatibility is validated on write. Only unconstrained static or ordinary instance definitions
in the current output are supported. Bounded static imported methods now use the separate
[imported generic contract](#imported-generic-methods-development-2026-10-01).
For example, `body.Call(identity.MakeGenericInstance(PrimitiveType.Int32))` consumes
one Int32 for `Identity<T>(T)->T` and produces Int32. Forwarding may instead pass
`SignatureType.MethodParameter(0)` from a caller that declares that parameter.
CLI output uses cached MethodSpec records; native calls carry explicit generic arguments
and substituted parameter types. The binary integration test verifies forwarding,
Int32/Int64/Boolean instantiations, owned-object identity and generic vector creation/access.

Generic-call regression checks also confirm that caller mutation of the argument array
cannot alter an existing instance, and both writers reject a mismatched substituted
stack argument before producing an image. Raw `Emit(Call, instance)` supports forwarded
parameters with the same scope and stack contract as `Call(instance)`.

Instance generic calls consume the exact owned declaring-class receiver before explicit
arguments and return the substituted result. Receiver slot zero remains separate from
MVAR parameter indices. `AddInstanceMethod(name, signature)` accepts named method
parameters; `MakeGenericInstance`, typed `Call` and raw `Emit(Call, instance)` work as
for static calls. No virtual-dispatch contract is introduced. Both writers validate
receiver stack shape; native reference projection preserves instance/generic flags.
Constructors and property accessors cannot declare their own method generic parameters.

### Typed local initialization (development)

`OpCode.Ldloca` accepts an owned `LocalDefinition` or local index; `OpCode.Initobj`
accepts `SignatureType`. `MethodBuilder.LoadLocalAddress(LocalDefinition)` and
`InitializeObject(SignatureType)` expose the same operations. The address may designate
an uninitialized local; initialization establishes definite assignment only for that
exact local. Both writers reject a mismatched type, non-address operand, an address
escaping through value storage/calls/returns, and later loads not initialized on every
reachable path. Writable managed-reference parameters are supported as described below. Address values at joins must identify the same local.

`OpCode.Ldobj` and `OpCode.Stobj` accept an exact non-Void `SignatureType` through
`Emit(OpCode, SignatureType)`. The corresponding helpers are
`MethodBuilder.LoadObject(SignatureType type)` and `StoreObject(SignatureType type)`.
Load consumes an owned local address and pushes its value; store consumes an address
followed by the value and leaves no result. Store establishes definite assignment for
that local; load requires assignment on every incoming path. Primitive, nominal,
vector and scoped generic local types follow the existing signature rules. A null
operand throws ArgumentNullException; Void, invalid scope or foreign ownership throws
ArgumentException before appending. Instruction limits and invalid stack/address/type
or assignment contracts throw InvalidDataException (the latter on writing).
CLI uses standard ldobj/stobj tokens, including TypeSpec for generic/vector operands;
native emits its existing typed ldobj/stobj. This does not admit pointers, field/array
addresses or escaping references. Ldobj/Stobj also accept managed-reference parameters. See `LocalObjectChecks.cs` for an
executable generic copy and branch-merged local update.

`MethodBuilder.LoadDefault(SignatureType)` declares one scratch local, initializes it
and loads its value (three instructions). Types may be primitives, owned root classes,
vectors or scoped method parameters; Void, foreign owners and invalid method scope
throw ArgumentException. Null arguments throw ArgumentNullException. Local/instruction
limits throw InvalidDataException before this helper changes the body; ClearBody retains
scratch declarations. Invalid stack/definite-assignment contracts fail during writing.
CLI uses ldloca/initobj/ldloc with TypeSpec for generic/vector operands; native uses the
same logical operations. Numeric defaults are zero, Boolean false, and reference
(string/class/vector) defaults are typed null references. This does not add nullable
source syntax or general pointer/byref APIs.

### Static generic owners (development)

`AssemblyBuilder.AddGenericType(string namespace, string name,
IEnumerable<string> genericParameterNames, TypeVisibility visibility = Public)` creates
an owned static class with one through 32 copied, unique parameter names. It appends
the CLI arity suffix to Name (for example Helpers`1); the input simple name must not
contain a backtick. Invalid/duplicate names or limits throw ArgumentException; null
parameter sequences throw ArgumentNullException. `TypeBuilder.GenericParameterNames`
is an immutable ordinal list. AddGenericType creates static owners; AddGenericClass
(below) creates instance owners. Both can associate properties using scoped VAR.

`SignatureType.TypeParameter(int index)` and nullable `TypeParameterIndex` represent
VAR independently from method MVAR. Index bounds are 0–31 (ArgumentOutOfRangeException);
method signatures, locals and typed instruction operands validate the declaring-type
scope. Assembly functions cannot use VAR. Entries cannot belong to a generic owner.

`MethodBuilder.MakeConstructedReference(IEnumerable<SignatureType> declaringTypeArguments,
IEnumerable<SignatureType>? methodArguments = null)` binds a generic owner and
all method parameters together. It returns immutable `ConstructedMethodReference`
with `Definition`, copied `DeclaringTypeArguments`, copied `MethodArguments` and the
simultaneously substituted `Signature`. Null owner argument sequences throw
ArgumentNullException. Wrong arity, Void/null/foreign arguments, nongeneric
owners or nested array substitution throw ArgumentException. Supplied caller parameters
are validated in the caller scope when emitted; substitution does not capture them.
`Call(ConstructedMethodReference)` and `Emit(OpCode.Call, ConstructedMethodReference)`
require an owned target and supported caller scope; null throws ArgumentNullException,
invalid opcode/ownership/scope throws ArgumentException, and instruction/stack failures
use InvalidDataException. Open-owner Call and MakeGenericInstance are rejected: bind
owner and method arguments together even when the method itself is nongeneric.

CLI uses GenericParam/VAR and a MemberRef on a constructed TypeSpec, optionally wrapped
in MethodSpec. Native definitions use open Constructed owners and explicit TypeParameter
ordinals; calls substitute owner and method arguments independently. The native reader
retains arities/names and validates open ownership before creating a reference projection.
These are owned-output references; external method imports use the separate bounded
[imported generic contract](#imported-generic-methods-development-2026-10-01).


### Generic reference classes (development)

`AssemblyBuilder.AddGenericClass(string namespace, string name,
IEnumerable<string> genericParameterNames, TypeVisibility visibility = Public)` uses
AddGenericType's naming, ownership, arity and error rules, but creates a nonabstract,
nonsealed reference class rooted at System.Object in CLI. No constructor is synthesized.

`TypeBuilder.MakeGenericInstance(params SignatureType[] typeArguments)` returns an
immutable `GenericTypeInstance` with `Definition` and copied `TypeArguments`.
`Equals(GenericTypeInstance?)`, `Equals(object?)` and `GetHashCode()` compare the exact
owned definition and arguments structurally; `ToString()` is diagnostic only.
Null arrays throw ArgumentNullException; static/nongeneric owners, arity mismatch,
null/Void/foreign arguments or nesting beyond 16 levels throw ArgumentException.
Implicit conversion to SignatureType and `SignatureType.GenericInstance` preserve
this identity in parameters, results, fields, locals, arrays and typed initialization.
Bare generic TypeBuilder-to-SignatureType conversion is rejected: construct the owner,
including with VAR arguments for an open self-reference. Scope is checked on use.

AddField now accepts declaring-type VAR and constructed owned classes (including
nested constructions). MVAR in fields is rejected. Generic definition field handles
can be emitted only inside their declaring type; constructed external field references
are deferred. This keeps CLI/native field access equivalent without losing the owner
instantiation. Readonly fields retain declaring-constructor store checks.

MakeConstructedReference now accepts instance methods and constructors as well as
static methods. `NewObject(ConstructedMethodReference)` and
`Emit(OpCode.Newobj, ConstructedMethodReference)` consume substituted constructor
parameters and produce the exact constructed class. Call is required for other
methods, Newobj for constructors; wrong opcodes, ownership or caller scope throw
ArgumentException, null throws ArgumentNullException, and instruction/stack failures
throw InvalidDataException. Instance calls require an exactly matching constructed
receiver. Constructor chaining remains unsupported. Generic method parameters remain
independent of the declaring type, including nested signature substitution.

CLI emits standard GENERICINST, VAR and MemberRef/MethodSpec; native output uses the
existing Constructed type and field contracts. The native declaration reader validates
field scope and preserves generic fields/constructed signatures in its non-executable
CLI reference projection. Constraints, inheritance, external
owners and external constructed field handles remain outside this bounded API.


### Properties on generic owners (development)

`TypeBuilder.AddProperty` now accepts declaring-type parameters in the value and
index signatures, including vectors and constructed class signatures containing VAR.
Both static and instance owners retain their ordinary accessor shape. Accessors cannot
declare method generic parameters; MVAR, out-of-range VAR, mismatched owner/result/index
signatures and reused accessors throw ArgumentException before adding the association.
No storage or bodies are synthesized by AddProperty; index signatures still come from
the associated methods and visibility stays on each accessor.

CLI keeps Property/PropertyMap/MethodSemantics and VAR signatures. Native metadata uses
canonical open Constructed owners on accessor references. The reader validates exact
owner identity, arity and ordered VAR arguments, resolves each accessor under the
property's declaring-type scope, and preserves both property signatures and associations
in its reference projection. Malformed owner references or scope violations throw
InvalidDataException. Use the matching reader for these new producer outputs; the
runtime already supports the native representation, so no runtime schema migration is
required. This adds no external property import or generic constraints.


### Constructed fields (development)

`FieldBuilder.MakeConstructedReference(params SignatureType[] typeArguments)` returns
an immutable `ConstructedFieldReference` with `Definition`, `DeclaringType` (a
GenericTypeInstance) and substituted `FieldType`. Arguments are copied; original
field signatures stay open. Constructor arguments follow MakeGenericInstance's bounds,
ownership and 16-level nesting rules. Null throws ArgumentNullException; wrong arity,
invalid/foreign types or unsupported substitution throws ArgumentException.

`MethodBuilder.LoadField(ConstructedFieldReference)`, `StoreField(...)` and
`Emit(OpCode, ConstructedFieldReference)` accept Ldfld/Stfld. Caller VAR/MVAR scope
and ownership are checked before appending; invalid opcode/scope throws ArgumentException,
null throws ArgumentNullException, and instruction limits throw InvalidDataException.
Writing checks the exact constructed receiver, substituted value type and existing
readonly constructor rule. CLI emits a Field MemberRef on a constructed TypeSpec with
the open field signature. Native field indices retain the runtime's constructed receiver
identity and layout. External *assembly* imports remain unsupported; this API accesses
owned fields from outside their declaring type and supersedes that earlier restriction.


### Nominal type constraints (development)

`TypeBuilder.AddBaseTypeConstraint(int parameterIndex, TypeBuilder baseType)` adds one
owned nongeneric nonstatic root-class bound to a declared type parameter. Null bounds
throw ArgumentNullException; invalid indices, repeated parameters, foreign/static/generic
bounds throw ArgumentException. `GenericConstraints` exposes read-only
`GenericTypeConstraint` snapshots with `ParameterIndex` and `BaseType`; the public record
constructor creates a value only and does not add a constraint to a definition.

CLI uses GenericParamConstraint; native uses TypeBound. Within this root-class producer
subset, concrete arguments must be the bound class itself (inheritance/interfaces are
not representable yet). Symbolic VAR/MVAR arguments are deferred to runtime substitution.
Constructing an invalid concrete reference throws ArgumentException; writing revalidates
previous uses after added bounds and throws InvalidDataException. The native reader
validates bound ordinals/identity and preserves bounds in the CLI reference projection.
Malformed bound metadata throws InvalidDataException. Existing unconstrained outputs
retain their encoding; constrained outputs need the matching reader.

This is a nominal bound, not a class/struct/new()/notnull flag. Those categories,
method constraints, interface/dependent bounds and dispatch through an open constrained
parameter remain separate contracts. The runtime already enforces nominal TypeBound;
no schema change is needed for this slice.


### Special type-parameter requirements (development)

`NeoCLR.Metadata.Experimental.Model.TypeParameterConstraints` is a flags enum:
`None = 0`, `ReferenceType = 4`, `ValueType = 8`, `DefaultConstructor = 16`.
`ReferenceType` requires a managed reference (including String and vectors);
`ValueType` requires a supported nonnullable value. `DefaultConstructor` accepts a
value type or a concrete class with a public parameterless instance constructor.
The producer currently admits Int32, Int64 and Boolean value arguments.

```csharp
public IReadOnlyDictionary<int, TypeParameterConstraints> TypeBuilder.SpecialConstraints { get; }
public void TypeBuilder.SetSpecialConstraints(int parameterIndex, TypeParameterConstraints constraints);
```

The read-only view maps zero-based declared parameter ordinals to requirements.
`SetSpecialConstraints` replaces that ordinal's special flags; `None` clears them
without removing a nominal bound. Invalid ordinals, unknown bits, reference/value
combinations and value/nominal-class-bound combinations throw `ArgumentException`.
A reference or constructor flag may coexist with a nominal bound. Set both ValueType
and DefaultConstructor for Raven's `struct` declaration. No method flags are exposed.

```csharp
var owner = assembly.AddGenericClass("Example", "Box", ["T"]);
owner.SetSpecialConstraints(0, TypeParameterConstraints.ReferenceType |
    TypeParameterConstraints.DefaultConstructor);
```

Concrete arguments are checked when constructed, except class constructor existence
is deferred until writing so definition order does not matter. Final graph validation
rejects an unsatisfied requirement with `InvalidDataException`, including requirements
added after an earlier use. Symbolic arguments defer concrete checks to runtime
substitution. Flags do not enable construction or constrained dispatch on symbolic T.

CLI uses ordinary GenericParam attributes; native metadata has distinct ReferenceType,
ValueType and DefaultConstructor constraint kinds. The declaration reader rejects
unknown, duplicate or conflicting flags and preserves accepted flags in reference
projections. Native execution needs the matching feature-branch runtime; old runtimes
cannot decode these new kinds. Native notvoid/notreference keep their prior meaning.
See the [integration assessment](../docs/experiments/extended-cli-metadata/state-assessment-2026-10-01.md).


### Interface declarations (development)

`NeoCLR.Metadata.Experimental.Model` exposes:

```csharp
public TypeBuilder AssemblyBuilder.AddInterface(string @namespace, string name,
    TypeVisibility visibility = TypeVisibility.Public);
public TypeBuilder AssemblyBuilder.AddGenericInterface(string @namespace, string name,
    IEnumerable<string> genericParameterNames, TypeVisibility visibility = TypeVisibility.Public);
public bool TypeBuilder.IsInterface { get; }
public IReadOnlyList<TypeBuilder> TypeBuilder.BaseInterfaces { get; }
public void TypeBuilder.AddBaseInterface(TypeBuilder baseInterface);
public MethodBuilder TypeBuilder.AddInterfaceMethod(string name, MethodSignature signature);
public bool MethodBuilder.IsAbstract { get; }
```

The factories return owned interfaces with public/internal visibility. Generic names
are copied (1–32 distinct names); CLI arity is appended to the supplied simple name.
Ordinary type identity/256-type limits apply. Invalid identities, visibility, names or
duplicates throw ArgumentException; null generic names throw ArgumentNullException.
Interfaces are invariant in this API. AddBaseInterface admits directly inherited,
owned nongeneric interfaces; generic base instantiations are not exposed yet.

AddInterfaceMethod creates a public abstract instance contract, with supported
primitive/owned-class/array/declaring-type-parameter signatures. A null signature throws
ArgumentNullException; duplicate/invalid signatures, reserved constructor names,
method-level generics or per-owner 256-method limits throw ArgumentException.
Calling it on a class, or ordinary AddMethod/AddInstanceMethod/AddConstructor on an
interface, throws InvalidOperationException. Interfaces cannot own storage fields; AddField throws InvalidOperationException.
AddProperty now associates abstract getter/setter declarations, including declaring-type
parameter values and index signatures, using the same validation as class properties.
Owned interface references and constructed interface values are admitted as storage,
parameter and result signatures, including vector elements and typed defaults. The
existing SignatureType.ClassType property denotes a CLI CLASS identity and can refer
to a class or interface. MakeGenericInstance now accepts invariant generic interfaces
with the existing arity, scope, ownership and argument validation. Interfaces remain
invalid nominal class bounds and allocation targets. A signature alone does not declare an implementation; the bounded dispatch API is
described below.

```csharp
var comparer = assembly.AddGenericInterface("Example", "Comparer", ["T"]);
var t = SignatureType.TypeParameter(0);
var compare = comparer.AddInterfaceMethod("Compare",
    new MethodSignature(PrimitiveType.Int32, [t, t]));
// compare.IsAbstract is true; no instructions or locals belong to this declaration.
```

Writing rejects abstract methods with instructions/locals and direct calls to them
with InvalidDataException. Direct interface dispatch and implicit nongeneric implementations are now supported
as described below; default/static interface methods remain subsequent work. An abstract method has no CLI
body (RVA zero), including in a reference projection: no throwing placeholder is used.
CLI uses Interface/Abstract type flags with no base class, and public abstract virtual
new-slot method flags. These are ordinary CLI contracts, not extensions; see
[ECMA-335](https://ecma-international.org/wp-content/uploads/ECMA-335_6th_edition_june_2012.pdf),
Partition II, interface and method-definition rules.

The existing native Interface representation carries contract identity. Its separate
type is_abstract field applies to records and remains false; methods carry explicit
abstract/virtual flags and empty bodies. No runtime format or instruction change is
needed. The native reader validates this shape and preserves interfaces through CLI
reference projection. Unknown/mismatched flags, bodies/locals, storage and invalid
owner contracts reject with InvalidDataException. Matching reader/producers are
required; older experimental readers reject the additional declaration category.


Interface inheritance and properties use standard InterfaceImpl, Property and
MethodSemantics metadata. Abstract accessor methods also carry SpecialName and no
body. AddBaseInterface returns no value; it adds an edge visible through the read-only
BaseInterfaces view. Null throws ArgumentNullException; a noninterface owner throws
InvalidOperationException. Foreign, noninterface or generic bases, duplicate edges,
cycles and more than 256 direct bases throw ArgumentException. Interface declaration
order does not matter. No class-implements-interface contract is implied by this API.

```csharp
var disposable = assembly.AddInterface("Example", "Disposable");
disposable.AddInterfaceMethod("Dispose", new MethodSignature(PrimitiveType.Void, []));
var iterator = assembly.AddGenericInterface("Example", "Iterator", ["T"]);
iterator.AddBaseInterface(disposable);
var current = iterator.AddInterfaceMethod("get_Current",
    new MethodSignature(SignatureType.TypeParameter(0), []));
iterator.AddProperty("Current", SignatureType.TypeParameter(0), current);
```

Native reading validates inherited identities, cycles and accessor associations and
preserves them in reference projection. Property-only declarations with zero fields
may include the writer's empty field-origin arrays; all three arrays must be present
and empty when that form is used. Partial/inconsistent field-origin metadata rejects.


### Interface implementation and dispatch (development)

```csharp
IReadOnlyList<TypeBuilder> TypeBuilder.ImplementedInterfaces { get; }
void TypeBuilder.AddInterfaceImplementation(TypeBuilder contract);
void MethodBuilder.CallVirtual(MethodBuilder target);
void MethodBuilder.Emit(OpCode.Callvirt, MethodBuilder target);
```

AddInterfaceImplementation declares an owned nongeneric interface on a nongeneric
root class. Its inherited nongeneric contracts are included. The list is read-only;
null throws ArgumentNullException. Static/interface/generic owners throw
InvalidOperationException; foreign/generic/noninterface contracts, duplicates or more
than 256 direct contracts throw ArgumentException. Writing requires every inherited
contract to have a public instance method with the same name, parameter and result
signature; missing or incompatible members throw InvalidDataException. Member order
does not matter. Explicit MethodImpl mappings are not yet exposed. CLI implementations
are virtual/final/new-slot; native implicit matching uses the existing runtime rules.

CallVirtual and raw Emit(Callvirt, target) consume a receiver followed by declared
arguments, dispatch to its implementation and push any result. Targets must be owned
nongeneric abstract interface methods. Null targets throw ArgumentNullException;
foreign/generic/noninterface targets throw ArgumentException without appending an
instruction. The existing instruction limit throws InvalidDataException. Stack checking
accepts exact references and declared interface upcasts, including inherited contracts;
unrelated receivers reject when writing. Arrays remain invariant. Null references can
be stored but fault when dispatched; no exception-handling instructions are introduced.
Direct Call to an abstract contract still rejects. CLI uses callvirt 0x6f; native output
uses the existing callvirt opcode. Reference projections preserve implementations but
retain placeholder bodies and must not be executed as native code.

```csharp
var contract = assembly.AddInterface("Example", "Value");
var get = contract.AddInterfaceMethod("Get", new MethodSignature(PrimitiveType.Int32, []));
var concrete = assembly.AddClass("Example", "Answer");
concrete.AddInterfaceImplementation(contract);
var implementation = concrete.AddInstanceMethod("Get", new MethodSignature(PrimitiveType.Int32, []));
implementation.LoadConstant(42);
implementation.Return();
// In a body with a Value-typed receiver on the stack:
body.CallVirtual(get);
```

C# contract/integration tests exercise two implementations, inherited contracts,
reference arguments and null faults on .NET and the binary neoCLR loader. Generic
interface instances are legal signatures but not yet legal targets of this dispatch
API. Default/static interface members and class virtual overrides remain separate work.


## Imported primitive vectors (development, 2026-10-01)

```csharp
bool MethodDefinition.TryGetStaticValueSignature(out MethodSignature? decoded);
```

Recognizes static nongeneric methods with Int32, Int64, Boolean or String scalar or
one-dimensional zero-based vector parameters/results, plus Void only as a scalar
result. Success returns an immutable copied `MethodSignature`; malformed or unsupported
encodings return false with null. Parameter counts are canonical CLI compressed integers,
bounded to 256. Exact consumption rejects trailing/truncated bytes, Void elements,
jagged/multidimensional arrays, byrefs, nominal/generic types and other conventions.
This reads declarations only; it neither resolves types nor validates bodies.

`AssemblyBuilder.ImportReference` and `MemberReference.ResolveMethod` now use this
contract and distinguish full vector element signatures when resolving overloads.
The existing explicit dependency/core identity, ownership and resolver restrictions
remain. For example, a read-only `Identity(int[]) -> int[]` definition can be imported,
called with `MethodBuilder.Call`, and emitted as ordinary CLI AssemblyRef/TypeRef/MemberRef
metadata or as a native assembly. Primitive-only recognition APIs remain narrower.

Vectors retain .NET CLI `SZARRAY` signatures; no extension or semantic divergence is
introduced. Imported nominal and generic signatures still need a separate identity
contract. The C# vector checks execute ordinary CLR library/application images and
check exact overload resolution, native declaration projection and malformed encodings.
The Raven [runtime probe](../docs/experiments/extended-cli-metadata/vector-library-validation.json)
executes independently emitted native library/application images with the neoCLR profile.


## Imported generic methods (development, 2026-10-01)

```csharp
bool MethodDefinition.TryGetStaticGenericValueSignature(out MethodSignature? decoded);
ImportedGenericMethodReference ImportedMethodReference.MakeGenericInstance(params SignatureType[] typeArguments);

public sealed class ImportedGenericMethodReference {
    public ImportedMethodReference Definition { get; }
    public IReadOnlyList<SignatureType> TypeArguments { get; }
    public MethodSignature Signature { get; }
}
void MethodBuilder.Call(ImportedGenericMethodReference method);
void MethodBuilder.Emit(OpCode opCode, ImportedGenericMethodReference operand);
```

The recognizer returns an immutable signature for static unconstrained methods with
1–32 method parameters, primitives, method parameters (`MVAR`) and single vectors of
those types. Generic parameter names in this callable view are positional `T0`, `T1`,
etc.; the original image remains unchanged. The generic arity must agree with the
GenericParam table, whose indices must be consecutive and whose flags/constraints
must be empty. Nongeneric, constrained, out-of-scope, noncanonical or malformed
signatures return false/null. Declaring-type parameters and nominal types are outside
this subset. Existing nongeneric recognizers keep their narrower behavior.

ImportReference retains this declaration contract; MemberReference.ResolveMethod
also matches its generic arity and exact parameter/result types. Instantiation copies
one non-Void consumer-scoped signature type per method parameter, including owned or
imported reference types, constructions, supported vectors and caller generic parameters.
Null arrays throw ArgumentNullException; wrong arity, foreign type ownership, a
nongeneric definition or substitutions producing a nested vector throw ArgumentException.
Definition exposes the immutable imported reference, never a mutable dependency builder.
TypeArguments is the copied ordered sequence; Signature substitutes method parameters
and may retain caller-scoped parameters until emission.

Call and raw Emit(Call, reference) append the same generic call. Null references throw
ArgumentNullException; wrong opcodes, consuming owners or caller generic scopes throw
ArgumentException. Method/declaring-type parameter indices are checked against the
caller at emission. An uninstantiated generic ImportedMethodReference cannot be called. Instruction limits
and stack/type mismatches continue to throw InvalidDataException on append/write.
CLI output uses a standard MethodSpec targeting an external MemberRef. Native output
uses the existing generic_arguments call contract; there is no format or opcode change.

For example, import `First<T>(T[]) -> T`, instantiate with PrimitiveType.Int32, then
pass an Int32 vector to Call. The C# tests execute the resulting separate library and
application on the CLR. Raven's native-profile [generic library probe](../docs/experiments/extended-cli-metadata/generic-library-validation.json)
verifies and runs the equivalent binary boundary in neoCLR, including consumer-owned
nominal arguments, alias mutation, external constructions and caller method/owner
parameter forwarding. Imported generic declaring types, cross-dependency nominal types in the imported
definition signature and constraints remain unsupported; this is not general collection
import support.

### Lifecycle direction reaffirmed 2026-10-01

.NET metadata remains the baseline; neoCLR extensions are explicit. This Cecil-like
library is intended to support inspection, modification, creation from nothing and
writing through familiar definition/reference concepts. Current loaded snapshots are
immutable and preserve original bytes; producer graphs are editable. General editing
of arbitrary loaded assemblies through one model is not implemented yet. The bounded
import APIs described above must not be read as a permanent split or a new metadata
format. Assembly-level functions already exist, with a CLI `<Module>` compatibility
representation; new generic imports add no further format extension.


## Imported type signatures (development, 2026-10-01)

```csharp
ImportedTypeReference AssemblyBuilder.ImportReference(TypeDefinition definition, AssemblyIdentity dependencyCoreLibrary);

public sealed class ImportedTypeReference : IEquatable<ImportedTypeReference> {
    public AssemblyBuilder Owner { get; }
    public AssemblyIdentity AssemblyIdentity { get; }
    public string Namespace { get; }
    public string Name { get; }
    public int GenericArity { get; }
    public IReadOnlyList<SignatureType> TypeArguments { get; }
    public ImportedTypeReference MakeGenericInstance(params SignatureType[] arguments);
    public bool Equals(ImportedTypeReference? other);
    public override bool Equals(object? obj);
    public override int GetHashCode();
    public override string ToString();
}
ImportedTypeReference? SignatureType.ImportedType { get; }
public static implicit operator SignatureType(ImportedTypeReference type);
```

ImportReference copies the exact assembly identity, namespace, metadata name and arity
from a public top-level reference class/interface declaration. It interns definition
references per consumer and rejects conflicting dependency MVIDs. The explicit core
identity must match the consumer. Value types, nested/static/nonpublic types, signed
or flagged dependencies, self imports, constrained/variant generic definitions and
arities above 32 are unsupported. Generic indices must be consecutive. Unsupported
contracts and limits throw InvalidDataException; null inputs throw ArgumentNullException.
The consumer admits at most 4096 imported type names and 256 dependency identities.
The overload addition means a null literal in ImportReference needs an explicit
MethodDefinition or TypeDefinition cast to select the intended overload.

A nongeneric reference converts directly to SignatureType. A generic definition first
needs MakeGenericInstance, which copies one non-Void consumer-scoped argument per
parameter. Arguments may include the consumer's own classes, other imported reference
types and scoped generic parameters; scope is checked at signature use. Depth is limited
to 16. Null arrays throw ArgumentNullException. Wrong arity, already constructed/open
signature use, foreign ownership, Void/null arguments and excessive depth throw
ArgumentException. Generic substitution recurses into imported constructions.

Owner identifies the consumer; AssemblyIdentity/Namespace/Name identify the external
definition. GenericArity belongs to the definition; TypeArguments is empty for definition
references and copied/read-only for constructions. Equality compares consumer ownership,
exact external identity and ordered arguments; GetHashCode agrees. ToString is diagnostic
only. No mutable dependency TypeBuilder is exposed. Imported constructions are accessed
through SignatureType.ImportedType, while GenericInstance still describes owned types.
LocalDefinition.SignatureType also carries these types; ClassType remains null for them.

Example: import a dependency's Box<T> TypeDefinition, create an Order class in the
consumer, and call `box.MakeGenericInstance(order)`. Use the resulting reference in
MethodSignature, fields, locals, vectors and default-value initialization. Standard CLI
output uses AssemblyRef/TypeRef and CLASS/GENERICINST signatures (TypeSpec for typed IL
operands). The native writer uses existing Named/Constructed identities scoped to the
dependency manifest. Its reference projection preserves external scope rather than
inventing local definitions. Unknown scopes, noncanonical names and arity mismatches
reject. No native format, instruction or schema extension was added. Earlier experimental
metadata readers may reject these external signatures; rebuild consumers with this API.

Native dependencies must use the metadata writer's existing format-5 naming contract;
importing an arbitrary CLI declaration does not translate its executable implementation.
Imported references are recorded as native dependencies even when not subsequently used.
The reader resolves no dependencies and verifies no external constraints/layouts.
Method/constructor imports containing these signatures and mappings to translated
System's original native identities are **not yet implemented**. This slice supplies
the signature contract required for that work; Raven's collections sample remains blocked.

Validation: C# tests inspect actual CLR generic scopes and execute the output, then
check native declaration projection and malformed dependency rejection. The
`--imported-type-integration <neoclr> <fresh-output>` test mode loads a separate native
library/application, verifies and returns 42, including external interface and
Box<consumer Order> fields/defaults and generic forwarding.

## Imported nominal method signatures (development, 2026-10-01)

The existing `AssemblyBuilder.ImportReference(MethodDefinition, AssemblyIdentity)`
overload decodes dependency-local CLASS and GENERICINST signatures into consumer-owned
`ImportedTypeReference` values, including supported vectors and method parameters. A
method can return `Box<T>`, accept it, or accept a vector of a dependency interface.
The method owner still must be nongeneric and top-level, and the method static. The
exact dependency definition snapshot supplies type name, arity and reference category;
module-local tokens are never compared across assemblies.

The immutable `ImportedMethodReference.Signature` exposes the remapped contract. Import
interning still checks exact identity/MVID/token and signature equality. Scoped generic
arguments and substituted signatures use the existing `MakeGenericInstance` API.
Malformed/truncated/noncanonical encodings, missing definitions, excessive nesting,
value-type encodings, nested vectors and unsupported TypeRef signatures throw
`InvalidDataException`. Null inputs retain `ArgumentNullException` behavior.

Standard CLI TypeRef/MemberRef/MethodSpec output and native dependency-scoped named
signatures are retained, without runtime format changes. Direct calls through foreign
mutable producer methods do not gain nominal support; use the explicit snapshot import.
Primitive-only recognizers and `MemberReference.ResolveMethod` retain their documented
narrower subset. General nominal MemberRef resolution is still pending.

The C# contract tests execute a library factory returning an object and a library reader
accepting it on the CLR. Raven's native probe additionally tests a generic factory,
consumer-owned payload aliases, interface-vector overload matching and missing-method
rejection. Nullable annotations are not retained by the current declaration projection;
these cross-assembly factory contracts use nonnullable references. Imported value/union
contracts, constructors/instance calls and translated-System native identities remain
unsupported. [Native evidence](../docs/experiments/extended-cli-metadata/nominal-method-validation.json).

## Owned value types (development, 2026-10-01)

```csharp
TypeBuilder AssemblyBuilder.AddValueType(string @namespace, string name,
    TypeVisibility visibility = TypeVisibility.Public);
TypeBuilder AssemblyBuilder.AddGenericValueType(string @namespace, string name,
    IEnumerable<string> genericParameterNames,
    TypeVisibility visibility = TypeVisibility.Public);
bool TypeBuilder.IsValueType { get; }
bool TypeDefinition.IsValueType { get; }
```

AddValueType creates an owned, sealed sequential-layout CLI type deriving from the
configured core's System.ValueType. AddGenericValueType adds one through 32 invariant,
unconstrained parameters and appends the normal metadata arity suffix to the name.
Names/visibility/duplicate and resource checks match AddClass/AddGenericClass; invalid
inputs throw ArgumentException and null generic parameter collections throw
ArgumentNullException. Neither method creates an instance constructor. The definition
supports static methods, primitive or declaring-parameter instance fields, defaults, parameters/results,
locals, vectors and generic construction/arguments. Public/internal visibility is
preserved in both output formats.

TypeBuilder.IsValueType reports the producer category. TypeDefinition.IsValueType
reports whether the input directly extends a type named System.ValueType or System.Enum;
this is metadata classification, not runtime loading or validation of the base assembly.
SignatureType.ClassType is retained as the compatibility property name for an owned
nongeneric nominal identity, including a value type. GenericTypeInstance.Definition
likewise distinguishes classes, interfaces and value types using the definition flags.
CLI encoding uses VALUETYPE and GENERICINST VALUETYPE for these types. Native output
uses the existing non-reference, sealed type shape; native reading and CLI projection
preserve it without a new instruction or format revision.

ValueType and DefaultConstructor generic requirements admit these definitions;
ReferenceType requirements and nominal class bounds reject them. Instance methods,
constructors and interface implementation are not yet supported for producer value
types (InvalidOperationException). Direct nominal field storage is rejected with ArgumentException, avoiding unsupported
recursive inline layouts. Declaring-type parameters are allowed, with types substituted
at construction. StoreField/Emit(Stfld, ...) on a value-type owner require an address
obtained from LoadLocalAddress for an initialized local of the exact owner type.
Reads accept either that address or a value copy. Uninitialized, mismatched-owner and
value-copy stores throw InvalidDataException on write; partial field assignment does
not initialize a local. Readonly field restrictions remain unchanged. Static properties may use static accessors as elsewhere.

Imported value-type definitions still reject: importing them as reference types would
corrupt signature/category semantics. Native reader checks the bounded shape and rejects
unsupported field storage before projection. These are current producer limits, not
permanent neoCLR rules; runtime value types already exist beyond this subset.

The C# checks execute defaults, field reads, static/generic forwarding and vector
storage on CLR and native neoCLR, both returning 42. They also test CLR reflection,
read snapshots, native projection, constraints and invalid shape/storage rejection.
[Evidence](../docs/experiments/extended-cli-metadata/value-type-validation.json).
This is a prerequisite for Raven union emission, not a completed collections gate.
Generic payload storage and addressed local field mutation are now tested: value
payloads and arrays retain copies, while reference payloads retain their aliases.
The legacy native managed-reference store returns an inhabited Void value; the native
writer appends a pop (and adjusts branch offsets) to preserve CLI stfld stack behavior.
No runtime opcode or format change is introduced.

## Definition-first model (author direction, 2026-10-01)

**Planned refactor, not implemented API:** definitions are the canonical editable
metadata model. Builders are optional convenience facades over those exact objects;
writers consume definitions. A caller must be able to construct an assembly/module,
a TypeDefinition with attributes and an explicit base-type reference, and field/method
members, then write it without using a builder. A builder must expose its definition
and preserve object identity: editing that definition affects subsequent output without
copying or synchronizing a second graph.

The current implementation instead uses immutable loaded definition snapshots and a
separate mutable producer graph. This is an architectural gap. The earlier bounded
builder APIs are compatibility surfaces, not the long-term source of truth. The
Cecil comparison is about the editable graph and ownership model, not dependence on
host reflection or an identical API spelling. Value-type classification follows the
base-type/category contract; sealed/layout attributes alone do not establish that a
type is a value type. Explicit target core identities remain essential for neoCLR.

The next implementation slices are:

1. Establish constructible assembly/module/type/field definitions and explicit type
   references with ownership-aware member collections. Use the author's manual struct
   example as a C# contract test. Encode a sealed sequential value type based on the
   configured core's ValueType and inspect/load the produced CLI/native assembly.
2. Move existing producer state into definitions and adapt builders into facades.
   Keep existing Raven-facing calls compatible where practical. Check that a builder
   and direct edits see the same definition objects and encode equivalent output;
   temporary adapters must not become a second authoritative graph.
3. Move method signatures, locals and instructions into definition/body objects, retain
   type-safe/raw Emit helpers, and make both writers consume that representation.
   Preserve symbolic branch targets and recalculate offsets on encoding; an ILProcessor
   can then provide insertion helpers without requiring a separate body model.
4. Materialize supported read metadata and bodies into the same definitions for
   read–edit–write. Preserve opaque unchanged input where possible and explicitly reject
   edits that would discard unsupported metadata. Do not claim arbitrary loaded assembly
   editing until preservation and executable roundtrip tests establish it.

Validation must cover manual construction, ownership/identity, edits made through both
surfaces, repeated writing and supported loaded roundtrips, plus existing C# tests and
Raven-to-native consumers. The roadmap's unchanged collections/runtime-library acceptance
remains the overall goal; this author-directed refactor precedes more builder-only
feature expansion. No new constructor or mutation API in this plan is shipped yet.

### Cecil alignment reference (reviewed 2026-10-01)

The author endorsed alignment with [Mono.Cecil](https://github.com/jbevain/cecil)
where it matters and supplied that repository as the reference. Review of its current
source supports the following direction; these are refactor requirements, not claims
that our API already implements them:

- AssemblyDefinition creation, reading and writing address the same object model.
  Follow that lifecycle and familiar definition/reference terminology.
  [AssemblyDefinition source](https://github.com/jbevain/cecil/blob/master/Mono.Cecil/AssemblyDefinition.cs).
- Type definitions expose attributes, base types and editable members. Follow that
  shape; derive value-type classification from the base/category contract rather than
  a collection of independent flags. Cecil's IsValueType checks System.ValueType/Enum
  and does not support setting that property directly.
  [TypeDefinition source](https://github.com/jbevain/cecil/blob/master/Mono.Cecil/TypeDefinition.cs).
- Member collections maintain declaring-owner relationships when adding/removing members.
  Adopt explicit ownership invariants to prevent accidentally sharing a definition
  between modules/types. Exact reparenting behavior remains a design/validation choice.
  [MemberDefinitionCollection source](https://github.com/jbevain/cecil/blob/master/Mono.Cecil/MemberDefinitionCollection.cs).
- Module-scoped importing and type-system access provide reference context. Preserve
  this separation from runtime loading; neoCLR must retain explicit core/dependency
  identity rather than infer its target from host reflection.
  [ModuleDefinition source](https://github.com/jbevain/cecil/blob/master/Mono.Cecil/ModuleDefinition.cs).
- An ILProcessor operates on a method body's instruction collection and supports typed
  raw emission plus insertion/removal/replacement. Adopt this relationship; convenient
  Call/LoadArgument helpers may sit above it. Retargeting branches and maintaining valid
  bodies need explicit rules and tests, not an assumption that list replacement does all
  repairs automatically.
  [ILProcessor source](https://github.com/jbevain/cecil/blob/master/Mono.Cecil.Cil/ILProcessor.cs).

The benefit is one familiar editable graph for manual callers, compiler emission and
inspection. The costs are migration of existing builder-owned state, ownership/cache
invalidation rules and preservation testing for loaded content. We are aligning the
model, not promising drop-in Cecil API/binary compatibility or adding a Cecil runtime
dependency. neoCLR assembly-level functions and future metadata categories remain
explicit extensions to that shared model. Different encoders consume it and validate
supported target capabilities; they do not determine the public graph's shape.


## Authored definitions: first migration slice

Development API, 2026-10-01. CLI metadata and CIL remain the baseline; this slice
changes the authoring model without changing the wire format or instruction set.
It implements only the assembly/type/field part of the definition-first plan above.

| API | Contract |
| --- | --- |
| `AssemblyDefinition.CreateAssembly(AssemblyIdentity identity, AssemblyIdentity coreLibrary)` | Creates an authored assembly with explicit target identity and core contract. Existing producer identity validation applies. |
| `AssemblyDefinition.Write()` | Encodes authored definitions; loaded snapshots still return owned original bytes. |
| `AssemblyDefinition.WriteNativeAssembly()` | Encodes authored definitions through the existing native writer. Loaded snapshots throw `InvalidOperationException`. |
| `AssemblyBuilder.Definition` / `AssemblyBuilder.ForDefinition(AssemblyDefinition)` | Returns the same assembly/facade without copying declarations. Null input throws `ArgumentNullException`; loaded input throws `InvalidOperationException`. |
| `ModuleDefinition.ImportReference(AssemblyIdentity scope, string namespace, string name)` | Creates an explicitly scoped reference; does not load a dependency or consult host reflection. Invalid/null arguments reject. Explicit resolution requires matching assembly identity. |
| `TypeDefinition(string namespace, string name, uint attributes, TypeReference? baseType)` | Creates a detached declaration. Attachment admits nongeneric root classes, static classes or sealed sequential value types with the same module's explicitly imported core Object/ValueType base. Unsupported shape, foreign ownership or duplicate name throws `ArgumentException`. |
| `TypeDefinition.BaseType` | Authored base reference; loaded base decoding is pending, so loaded null is not proof of no base. |
| `FieldDefinition(string name, ushort attributes, SignatureType fieldType)` | Creates a detached instance field. Supports Private, Assembly or Public access and optional InitOnly; nonvoid signature required. Value-type storage retains existing primitive/generic-payload restrictions. |
| `FieldDefinition.FieldType` | Authored signature; null for opaque loaded signatures. `GetSignature()` rejects authored fields until encoded/read; use FieldType instead. |
| `FieldDefinition.Name` | Authored fields can be renamed with Unicode/name and duplicate checks. Loaded edits throw `InvalidOperationException`. |
| `TypeBuilder.Definition` / `FieldBuilder.Definition` | Exact declaration object consumed by the facade. Builder field creation appends to the definition collection. |

`ModuleDefinition.Types` and `TypeDefinition.Fields` now return `IList<T>` instead
of `IReadOnlyList<T>`: a development API source compatibility change. Authored lists
support appending only. Insertion elsewhere, replacement, removal and clearing throw
`NotSupportedException`. Ownership is exclusive; attach a type to its module and fields
to their type. Detached Module properties are unset until attachment. Physical tokens
remain zero until encoding and rereading; token lookup is a loaded-image operation.
`ModuleDefinition.Fields` reflects current authored fields.

Method bodies, property associations and generic parameter declarations still use the
existing builder representation. Authored method/function views and EntryPoint now expose canonical declarations;
use the builder facade to create type methods and edit bodies. Other
snapshot metadata views are not materialized authored views. Loaded definitions remain
read-only, byte-preserving snapshots. Writers currently use builder encoding adapters;
this is not completion of canonical method/body or reader/editor migration.

```csharp
var assembly = AssemblyDefinition.CreateAssembly(identity, targetCoreIdentity);
var module = assembly.MainModule;
var valueType = module.ImportReference(targetCoreIdentity, "System", "ValueType");
var type = new TypeDefinition("Example", "MyStruct", 0x109, valueType);
type.Fields.Add(new FieldDefinition("MyField", 0x6, PrimitiveType.Int32));
module.Types.Add(type);
byte[] pe = assembly.Write();
```

Validation: `AuthoredDefinitionChecks` covers exact facade identity, direct declarations,
renaming, ownership rejection, CLR shape/execution and native verify/run (42).
`--authored-definition-integration <runtime> <fresh-directory>` reproduces the native
case. The 76 C# contract groups and Raven external-signature/generic-library runtime
probes pass. The broader collections Option<Order> gate remains unchanged.


### Shared method declarations

The next migration slice adds `MethodBuilder.Definition`, with the declaration owning
its name, authored namespace, access/static flags and immutable signature.
`MethodDefinition.AuthoredSignature` returns that same `MethodSignature` (null for
loaded declarations); `MethodDefinition.Namespace` returns the authored namespace
(null for loaded physical rows). `ModuleDefinition.Methods`, `.Functions`,
`TypeDefinition.Methods` and `AssemblyDefinition.EntryPoint` return those exact
objects. Global functions retain a null declaring type. Type-method and aggregate method collections remain read-only views. The following
slice enables direct assembly-function construction; mutable body definitions remain pending.

`MethodDefinition.Attributes` includes context-derived CLI flags for constructors,
property accessors, interface contracts and implementations. Inspection and PE writing
share flag calculation; writer-side accessor classification stays precomputed.
Authored `GetSignature()` throws `InvalidOperationException`; raw signature recognizers
remain loaded-image operations. Write and reread for physical tokens/signatures.
EntryPointToken remains zero before serialization. Existing typed helpers and raw
Emit overloads continue to operate through the method builder.

Validated by 76 C# groups, including declaration identity and accessor/interface flag
round trips; direct-struct CLR/native execution returns 42. Raven's external-signature
native probe also passes with the rebuilt metadata dependency.


### Direct assembly-level function construction

`new MethodDefinition(string name, MethodSignature signature,
MethodVisibility visibility = MethodVisibility.Public, string @namespace = "")`
creates a detached static function. Public/Internal access is supported; other access
throws `ArgumentOutOfRangeException`. Null signatures throw `ArgumentNullException`;
invalid/reserved names and namespaces throw `ArgumentException`. Final Unicode/body
validation remains at writing, preserving existing builder behavior.

Append to `assembly.MainModule.Functions` to attach and validate signature ownership,
uniqueness and the 256-function limit. This collection now returns `IList<MethodDefinition>`
instead of `IReadOnlyList<MethodDefinition>` (development API change). Loaded collections
reject appends; authored collections reject replacement/removal/reordering. Attached or
loaded declarations cannot be attached again. A failed foreign-signature attachment does
not transfer ownership. Type-owned methods still require TypeBuilder construction.

`MethodBuilder.ForDefinition(MethodDefinition)` returns the existing body helper facade.
Null throws `ArgumentNullException`; detached or loaded input throws
`InvalidOperationException`. `AssemblyDefinition.EntryPoint` can now be set on authored
assemblies, including null for a library. Foreign/detached entries throw
`ArgumentException`; loaded assignment throws `InvalidOperationException`. Writing
still validates the entry signature/body. Detached Module is unset until attachment.

```csharp
var answer = new MethodDefinition("Answer", PrimitiveMethodSignature.Int32(0, true),
    @namespace: "Example");
assembly.MainModule.Functions.Add(answer);
var answerBody = MethodBuilder.ForDefinition(answer);
answerBody.LoadConstant(42);
answerBody.Return();
var main = new MethodDefinition("Main", PrimitiveMethodSignature.Int32(0, true));
assembly.MainModule.Functions.Add(main);
assembly.EntryPoint = main;
var mainBody = MethodBuilder.ForDefinition(main);
mainBody.Call(answerBody);
mainBody.Return();
byte[] image = assembly.Write();
```

This follows Cecil's declaration/body-helper separation while preserving neoCLR's
assembly-level function category. Existing CLI global-function projection and native
encoding remain unchanged. The executable test combines this call with direct struct
field storage; CLR/native return 42. All 76 C# contract groups pass. Arbitrary instruction
editing, canonical body definitions and loaded read/edit/write remain future slices.


### Direct static type-method construction

`new MethodDefinition(string name, ushort attributes, MethodSignature signature)`
creates a detached static type method. Attributes accept Static plus Public, Assembly
or Private, and optional HideBySig. The subsequent instance/constructor extension is described below. Abstract, virtual
and other flags remain unsupported by this constructor; existing builders retain
their previous bounded contracts. Null signatures throw `ArgumentNullException`; unsupported
flags, empty/overlong names and `.cctor` throw `ArgumentException`.

Attach the declaring type to its authored module, then append the method to
`TypeDefinition.Methods`. This property now returns `IList<MethodDefinition>` instead
of `IReadOnlyList<MethodDefinition>` (development source compatibility change). It is
append-only for authored types and read-only for loaded types. Adding to a detached
type throws `InvalidOperationException`. Attachment checks signature ownership and
scope, duplicate name/parameter/generic-arity signatures and the 256-method limit.
Foreign or already attached declarations reject without moving ownership. Explicit
function declarations and type-method declarations cannot be interchanged. Interface
methods continue to require the abstract-contract builder API.

`MethodBuilder.ForDefinition` supplies existing typed helpers and raw Emit overloads.
The method's namespace follows its declaring type, and its declaring type/module are
the exact authored objects. Existing builder-created instance methods, constructors
and interface contracts also enter this same collection. Method bodies remain in the
builder representation. CLI attributes and encodings are unchanged.

```csharp
var answer = new MethodDefinition("Answer", 0x96,
    PrimitiveMethodSignature.Int32(0, true)); // Public | Static | HideBySig
attachedType.Methods.Add(answer);
var body = MethodBuilder.ForDefinition(answer);
body.LoadConstant(42);
body.Return();
```

Validation extends the manual executable to call a directly declared static type method
through an assembly-level helper. CLR and neoCLR return 42. C# contracts cover shared
identity, access flags, duplicate/foreign attachment, detached-owner rejection and
unsupported method categories. The broader collections Option<Order> gate is unchanged.


### Direct instance methods and constructors

The CLI-attribute `MethodDefinition(string, ushort, MethodSignature)` constructor now
also accepts nonstatic instance methods and `.ctor` declarations. Public/Assembly/Private
and optional HideBySig apply as before. A `.ctor` must be nonstatic, return Void and
declare no method generic parameters. It may specify SpecialName and RTSpecialName
together; writing supplies both flags as it does for existing builder constructors.
Those flags on ordinary methods, partial constructor flag pairs, `.cctor`, arbitrary
virtual/abstract flags and invalid constructor signatures throw `ArgumentException`.

Attachment requires a reference-class owner for instance methods/constructors; static
and value-type owners throw `InvalidOperationException` before ownership transfer.
Direct interface contracts are covered by the subsequent interface slice below. Signature ownership, generic scopes,
access and readonly-field rules reuse the existing method/body contracts. Detached
method bodies still require attachment before `MethodBuilder.ForDefinition` succeeds.

```csharp
var constructor = new MethodDefinition(".ctor", 0x1806,
    new MethodSignature(PrimitiveType.Void, new[] { PrimitiveType.Int32 }));
attachedClass.Methods.Add(constructor);
var body = MethodBuilder.ForDefinition(constructor);
body.LoadArgument(0); // receiver
body.LoadArgument(1);
body.StoreField(ownedFieldBuilder);
body.Return();
```

As with the existing producer, root-class CLI constructors initialize System.Object;
native construction uses the existing root-object path. Constructor chaining, value-type
instance methods and class virtual dispatch are not added by this authoring change.
No wire format or CIL changes are introduced. This uses the Cecil-like definition/body
helper separation while retaining the current bounded execution contract.

The C# executable fixture constructs a manually declared reference class, initializes
its private readonly Int32 field in a manual constructor, calls a manual instance reader,
and carries the result through the function/type-method/struct path. CLR and neoCLR
return 42. Tests also compare emitted constructor/instance flags and reject invalid
constructor signatures and owner categories. Canonical body definitions, direct interface
contracts and loaded editing remain pending; the collections Option<Order> gate is unchanged.


### Direct interface declarations and dispatch

A nongeneric `TypeDefinition` with Interface | Abstract and optional Public attributes
can now be attached directly. Its base reference must be null and it cannot declare
fields. Unsupported bases/storage reject before module ownership is transferred.

The CLI-attribute `MethodDefinition` constructor admits public instance contracts with
Abstract | Virtual | NewSlot together (optional HideBySig). Partial flag combinations,
static/private contracts, constructors and method-generic contracts reject with
`ArgumentException`. Attach these declarations to an authored interface; an abstract
contract on a class or concrete declaration on an interface throws
`InvalidOperationException`. These bounds match existing interface builder support.

```csharp
var contract = new TypeDefinition("Example", "IRead", 0xa1, null);
assembly.MainModule.Types.Add(contract);
var read = new MethodDefinition("Read", 0x5c6,
    PrimitiveMethodSignature.Int32(0, true));
contract.Methods.Add(read);
// Use MethodBuilder.ForDefinition(read) as the target of CallVirtual.
```

The contract remains bodyless. Writing rejects instructions or locals on it through
existing abstract-method validation. Relationships (base interfaces and class
implementations) still use TypeBuilder.AddBaseInterface/AddInterfaceImplementation;
they have not yet moved to directly mutable definition collections. Generic interface
definitions still require builders. No class virtual dispatch, static interface methods,
default implementations or new instruction encodings are introduced.

The manual executable declares IRead and Read directly, registers the class implementation
through the existing helper and calls it with CallVirtual. CLR and neoCLR return 42.
All 76 C# groups pass, including existing multi-implementation/inherited interface tests
and new invalid-owner/flags/interface-storage checks. Raven's rebuilt external-signature
probe also passes. Canonical bodies, relationship definitions and loaded editing remain
pending; this does not resolve the collections Option<Order> import gap.


### Authored interface relationships

`InterfaceImplementation(TypeReference interfaceType)` creates an unattached relationship;
null throws `ArgumentNullException`. `InterfaceType` retains the exact immutable reference.
`DeclaringType` is null until successful attachment. Append to the new
`TypeDefinition.Interfaces : IList<InterfaceImplementation>` on an attached authored type.
For interfaces this declares inheritance; for nongeneric root classes it declares implicit
implementation. Builders add to this same collection; their existing handle lists are
encoding caches over these immutable edges.

Targets must resolve locally to attached, same-assembly nongeneric interfaces. Foreign,
noninterface, detached, unresolved, duplicate or cyclic targets and reused edge objects
throw `ArgumentException`; unsupported/detached owners throw `InvalidOperationException`.
Replacement, removal and clearing throw `NotSupportedException`. Loaded `Interfaces`
access also throws `NotSupportedException` until reader materialization is implemented.
No resolver/host loading occurs during attachment. Existing implementation completeness,
256-edge limits and cycle checks remain in force.

```csharp
derivedInterface.Interfaces.Add(new InterfaceImplementation(baseInterface.ToReference()));
implementingClass.Interfaces.Add(new InterfaceImplementation(derivedInterface.ToReference()));
```

The manual inherited-interface dispatch case executes on CLR/neoCLR (42). All 76 C#
groups pass, including duplicate/cycle/category rejection. CLI InterfaceImpl and native
encoding remain unchanged. Generic relationships and loaded editing remain unsupported.


### Definition-owned method bodies

`MethodDefinition.Body : MethodBodyDefinition` now owns authored instruction, local
and symbolic-label storage. Existing MethodBuilder helpers and raw Emit overloads
operate on this same storage, as do the CLI/native writer adapters. The definition
returns one stable body object. Loaded Body access throws `NotSupportedException`;
loaded-body materialization and editing remain pending. An abstract declaration may
have an empty Body, but writing rejects instructions or locals on it.

| Member | Contract |
| --- | --- |
| `MethodBodyDefinition.Method` | Exact owning MethodDefinition. |
| `Locals : IReadOnlyList<LocalDefinition>` | Cached, live read-only view of declared slots; existing builder-owned handles remain compatible. |
| `Labels : IReadOnlyList<BranchLabel>` | Cached, live read-only view of allocated label handles. |
| `ClearInstructions()` | Removes instructions and label marks, preserving locals and label handles. Referenced labels must be marked again. Writers revalidate the result. |

Use `MethodBuilder.ForDefinition(method)` for emission after attachment. ClearBody
now delegates to Body.ClearInstructions. Read-only local views no longer allocate a
wrapper on each getter call; no broader performance improvement is claimed. This
moves canonical storage without introducing a second instruction graph or changing
CLI/CIL/native lowering. Public arbitrary instruction lists, insert-before/after,
body decoding, exception regions and independent local construction are not added.

C# tests verify method/body/local/label identity, clear-and-rebuild with a retained
branch label and locals, CLR execution (42), and rejection of loaded Body access.
The native manual object/interface/struct case replaces a body through the definition
API before emission and returns 42. All 76 contract groups and the rebuilt Raven
external-signature probe pass. Property/generic-definition migration and loaded editing
remain open, as does the collections Option<Order> target gap.


### Shared authored property declarations

`PropertyDefinition(string name, SignatureType propertyType, MethodDefinition? getter = null,
MethodDefinition? setter = null)` creates a detached association. It requires a nonnull
value type and at least one authored accessor; null type throws `ArgumentNullException`,
missing/loaded accessors throw `ArgumentException`. Append to the attached owner's
`TypeDefinition.Properties : IList<PropertyDefinition>` after attaching accessors.
The collection is append-only; loaded collections remain read-only. Changing its former
IReadOnlyList return type is a development API compatibility change.

Attachment reuses name/Unicode, non-Void signature, scope/ownership, accessor reuse,
getter/setter shape, index parameters, duplicate and limit validation. Invalid associations
throw `ArgumentException` without attaching; detached owners throw `InvalidOperationException`.
Accessors must be nongeneric ordinary methods on that owner. Their access and staticness
remain on method definitions. Authored property flags are zero; custom attributes, defaults
and arbitrary PropertyAttributes authoring are not introduced.

`PropertyType` and `ParameterTypes` expose authored value/index signatures (null for loaded
opaque signatures). GetMethod/SetMethod return the exact authored declarations. Accessing
DeclaringType before attachment throws `InvalidOperationException`. Authored GetSignature
throws `InvalidOperationException`; encode and read for physical blobs. Raw primitive
recognition remains a loaded-signature operation. PropertyBuilder.Definition returns this
same object; the builder delegates its state. ModuleDefinition.Properties includes authored
properties. Loaded raw rows and their preservation behavior remain unchanged.

```csharp
var property = new PropertyDefinition("Value", PrimitiveType.Int32, getterDefinition);
attachedType.Properties.Add(property);
```

The manual executable associates its instance reader with Value. CLR reflection constructs
the class and reads Value as 42; native execution still returns 42. All 76 contract groups,
including existing getter/setter/indexer tests, pass; Raven's rebuilt external-signature
probe passes. No property wire encoding or compiler admission change is introduced.

### Builder direction clarified by the author (2026-10-01)

Definitions/references and inspection/manipulation remain Cecil-like. Builders are a
separate convenience layer over those same definitions and should increasingly follow
Reflection.Emit-style generation patterns. They are not a replacement or drop-in
implementation of System.Reflection.Emit, and this direction does not require a host
Reflection.Emit dependency. Typed helpers and raw emits remain complementary.

This is an architectural direction, not a completed API rename or compatibility claim.
Future Define-style construction and body-helper decisions should follow that direction
while preserving target-specific capability checks and explicit identities. The property
migration here shares declarations; existing Add-style entry points remain supported.


### Direct generic types and shared constraint storage

`TypeDefinition(string namespace, string name, uint attributes, TypeReference? baseType,
IEnumerable<string> genericParameterNames)` creates a detached generic root class,
value type or interface. Supply a simple name without backtick/arity; the constructor
appends the CLI arity suffix. One through 32 unique valid names are copied using the
existing signature-name rules. Null input throws `ArgumentNullException`; invalid names,
empty/duplicate/over-limit parameters throw `ArgumentException`. Attachment applies the
existing category/base/ownership restrictions; generic static classes remain unsupported.
Pending fields validate VAR references against the declaring arity, including supported
generic value-type payload fields.

`TypeDefinition.GenericParameterNames` is an immutable authored list shared by the builder;
it is null for loaded snapshots whose names are not materialized. Existing GenericArity
continues to report loaded arity. `SpecialConstraints` and `GenericConstraints` expose
cached live read-only views of definition-owned storage. Loaded access throws
`NotSupportedException`. Existing TypeBuilder.SetSpecialConstraints/AddBaseTypeConstraint
helpers validate and update that storage. Nominal constraints retain their existing
GenericTypeConstraint builder bound handles; independent generic-parameter/constraint
reference objects and loaded constraint editing are future work.

The manual executable declares Identity<T> directly, adds Pass(T):T directly, constrains T
to a value type through the helper, and calls Identity<Int32>.Pass. CLR reflection confirms
the parameter name and special constraint; CLR/native execution returns 42. A String
instantiation rejects. All 76 C# groups and Raven's rebuilt native probe pass. Existing
GenericParam/GenericParamConstraint and native encodings are unchanged.

### Layered reader/writer direction (author clarification, 2026-10-01)

The intended write pipeline is **builders → definitions → encoded metadata → PE**.
Builders are optional generation helpers over the shared definitions. Metadata encoding
owns CLI tables/heaps, signatures, method bodies and explicit neoCLR extensions; the PE
layer packages the result. The read path reverses these boundaries: PE extraction,
metadata decoding and definition materialization. Builders may then wrap editable loaded
definitions. Readers and writers should expose complementary responsibilities at these
boundaries, rather than making the object model depend on a particular packaging writer.

This is the target architecture, not a claim that the separation is complete. Current
AssemblyBuilder.WriteImage still combines metadata/body encoding and PE packaging;
AssemblyDefinition reading returns bounded immutable snapshots. Native container handling
also has its existing adapter path. No format change or new binary layer API is claimed
by this record. The definition migration provides shared authoring state; extracting the
encoding/packaging boundary and materializing editable loaded definitions remain work.


### Integration priority after architecture checkpoint

Author direction, 2026-10-01: the current API is good enough to resume the working
Raven/native case. Further object-model migration, builder naming and encoding/PE
separation are deferred unless required by an observed integration blocker. This
supersedes the earlier migration-first sequencing, not the architectural direction.
The current blocker remains imported Option<Order> value-category support; the
reference-only import contract must not be widened by treating it as a class.


### Imported value signatures (integration slice, 2026-10-01)

AssemblyBuilder.ImportReference(TypeDefinition, core) now admits public top-level value
definitions in addition to reference classes/interfaces. Values must directly extend
System.ValueType through an AssemblyRef matching the explicit dependency core identity.
Enums, local/fake ValueType bases, nested/nonpublic declarations and constrained/variant
generic definitions remain unsupported. ImportedTypeReference.IsValueType is immutable
and participates in identity, interning consistency and generic substitution. Special
constraint checks distinguish imported values from references.

CLI signatures encode imported values with VALUETYPE and GENERICINST VALUETYPE. The
static method signature decoder accepts dependency-local value TypeDefs and rejects
CLASS/VALUETYPE disagreement with the declaration. Cross-dependency TypeRef signatures,
imported instance/constructor calls and generic declaring-owner member calls remain
unsupported. Importing a type does not import its members or implement Raven unions.

Native format-5 signatures still use Named/Constructed types resolved by the loader.
To retain the CLI category in metadata-only projection, the assembly manifest now has
an optional `value_type_references` array of canonical external native definition names.
The writer emits it only for imported values. The reader bounds it to 4096 unique names,
requires declared dependency scopes and restores value-category signatures in the PE
projection. Native validation requires each listed name to resolve to a nonreference,
noninterface loaded type, rejecting missing declarations/category mismatches.

This is a temporary native projection annotation for information represented directly
in CLI signatures. It adds no opcode or runtime representation. Existing payloads without
the field retain their reference-only import interpretation. Payloads containing it
require the updated runtime; older strict readers reject the new manifest field.
The metadata library owns emission/projection and neoCLR owns dependency validation.
The future native indexed metadata reader/writer should encode the category directly
in signatures, removing the need for this annotation.

C# tests (77 groups) cover cross-assembly nominal/generic value signatures, CLR factory/read
and default/forward execution (42), projection category retention, category disagreement
and explicit core rejection. `--imported-value-integration <runtime> <fresh-directory>`
emits separate native library/consumer containers, verifies/runs (42), and checks that a
reference-class substitution for an imported value rejects. Raven's dedicated consumer
probe also returns 42 with imported returns, parameters, locals and forwarding. The
unchanged collections sample advances to an unsupported lowered invocation; it does not
execute yet.

## Imported member dispatch and closed generic interfaces (2026-10-02)

These are development host C# APIs in `NeoCLR.Metadata.Experimental.Model`. They
use the existing CLI/CIL format: HAS_THIS and VAR in signatures, TypeSpec parents
for constructed MemberRefs and callvirt for interface/final virtual dispatch. Native
format-5 uses its existing constructed-owner function references and interface
relationships. No new opcode is introduced. Reuse the CLI/Cecil comparison and
research above; the benefit is one exact imported contract for both writers, with
a bounded supported subset instead of host reflection loading. The cost is explicit
dependency registration and rejection of currently unsupported declarations.

| Type/member | Contract |
| --- | --- |
| `AssemblyBuilder.ImportReference(MethodDefinition, AssemblyIdentity)` | Also imports public nongeneric instance methods of public top-level reference classes/interfaces. Concrete class methods must be nonvirtual or final; interface methods must be abstract/virtual. Owners may be invariant, unconstrained generic reference types. Core/identity/MVID, access/category and scoped signature checks remain mandatory. |
| `ImportedMethodReference.IsStatic : bool` | Whether the reference excludes a receiver. |
| `ImportedMethodReference.IsInterfaceMethod : bool` | Whether the declaring contract is an interface. |
| `ImportedMethodReference.RequiresVirtualDispatch : bool` | Whether Callvirt is required, including final virtual class members; Call is rejected for these references. |
| `ImportedMethodReference.MakeConstructedReference(IEnumerable<SignatureType> declaringTypeArguments, IEnumerable<SignatureType>? methodArguments = null)` | Returns `ImportedConstructedMethodReference`. Copies arguments, requires exact arities and consumer ownership; caller VAR/MVAR scope is checked when emitted. Instance method generics remain unsupported; static generics may have method arguments. |
| `ImportedConstructedMethodReference.Definition` | The consumer-owned open imported member. |
| `ImportedConstructedMethodReference.DeclaringType` | Consumer-owned constructed `ImportedTypeReference`. |
| `ImportedConstructedMethodReference.Signature` | Substituted `MethodSignature`, with declaring and method parameters replaced simultaneously. |
| `ImportedConstructedMethodReference.MethodArguments` | Read-only copied method argument list. |
| `MethodBuilder.Call(ImportedConstructedMethodReference)` | Emits a direct Call for a concrete/static contract. |
| `MethodBuilder.CallVirtual(ImportedMethodReference)` and `CallVirtual(ImportedConstructedMethodReference)` | Emit Callvirt for an imported interface/final virtual contract. |
| `MethodBuilder.Emit(OpCode, ImportedMethodReference)` and `Emit(OpCode, ImportedConstructedMethodReference)` | Raw typed operands retain the same ownership, dispatch and generic-scope checks as helpers. No unrestricted integer/token escape hatch. |
| `InterfaceImplementation(TypeReference, IEnumerable<SignatureType>)` | Definition-owned relationship with copied arguments; attachment validates ownership and shape. |
| `InterfaceImplementation.TypeArguments : IReadOnlyList<SignatureType>` | Empty for nongeneric relationships, copied ordered arguments for a construction. |
| `TypeBuilder.AddInterfaceImplementation(GenericTypeInstance)` | Appends the same definition relationship. Supports nongeneric root classes implementing closed owned generic interfaces with no inherited interface edges. Existing nongeneric overload is unchanged. |

Null inputs throw `ArgumentNullException`; invalid construction/ownership/opcodes
throw `ArgumentException` (an invalid implementation owner throws
`InvalidOperationException`). Unsupported imported declarations, signature bytes,
identity conflicts, limits, missing public interface implementations and invalid
body stacks throw `InvalidDataException`. Instructions retain the existing count
limits. Generic arities are at most 32; constructed relationships are bounded to
256. The target interface must be in the same assembly as an authored implementation;
imported dispatch references themselves can cross assemblies. Neither API infers
implementation conformance from matching method names alone.

```csharp
var imported = consumer.ImportReference(interfaceMethodDefinition, explicitCore);
var call = imported.MakeConstructedReference(new SignatureType[] { PrimitiveType.Int32 });
body.Call(factoryReference); // Returns exactly the imported constructed interface.
body.LoadConstant(42);
body.CallVirtual(call);
body.Return();
```

Readers preserve closed generic interface relationships in reference projections;
older experimental host readers reject this expanded subset. The existing native
runtime already supports these relationships. Imported constructors/value-instance
members, generic instance methods, nonfinal virtual class slots, generic interface
inheritance, cross-dependency TypeRef signatures and native System identity mapping
remain outside this increment. `ImportedInterfaceChecks` executes a separately
written library/consumer on CLR and neoCLR (42), checks native null-receiver failure,
wrong arity/opcode/owner rejection and reference projection. Raven's corresponding
probe executes constructed interface and final class calls (42); the unchanged
collections sample advances to propagation-expression lowering.


### Writable managed-reference parameters (development, 2026-10-02)

`SignatureType.ByReference(SignatureType elementType)` creates a managed-reference
parameter signature; `SignatureType.ByReferenceElement` returns its target or null.
The target is a non-Void supported type, including vectors and scoped generic parameters.
Null throws ArgumentNullException; Void, nested byrefs or exceeding the 16-level nesting
bound throws ArgumentException. Ownership and generic scope are checked on use.
Only method parameters admit this type: byref returns, fields, locals, array elements
and generic arguments reject. ToString appends `&` to the target diagnostic name.

MethodSignature, owned and imported method calls, generic substitution and native CLI
projection preserve BYREF. CLI uses ELEMENT_TYPE_BYREF in parameter signatures; native
uses its existing ByRef type. The bounded static value/generic signature readers recognize
byref primitive/vector/MVAR parameters; primitive-only readers continue to return false
for those signatures. Out flags are emitted only for explicit OutParameters contracts described below; no In flag or readonly modifier is inferred.

Call consumes an initialized exact local address or a matching forwarded ref parameter.
It cannot establish assignment for an uninitialized caller local. `LoadArgument` loads
the reference; `LoadObject`/`StoreObject` (or raw Ldobj/Stobj) read/write the referenced
value. Argument rebinding with Starg rejects; Initobj remains local-address-only.
The body validator rejects mismatched targets, non-address operands and uninitialized
ref calls when either writer runs. References cannot escape via results or value locals.
This slice does not admit ref receiver methods or Raven `out` propagation calls. Explicit metadata out calls are described below.

`ByReferenceChecks.cs` demonstrates generic replacement and reference forwarding, with
independent library/consumer execution from both CLI snapshots and native projections.
Run `--byref-integration <runtime> <fresh-output>` for native binary verification and
execution (42); the regular C# tests include positive and rejection checks.


### Output parameter contracts (development, 2026-10-02)

`MethodSignature(SignatureType returnType, IEnumerable<SignatureType> parameterTypes,
IEnumerable<string>? genericParameterNames = null, IEnumerable<int>? outParameters = null)`
accepts optional zero-based declared parameter indices (excluding the instance receiver).
`IReadOnlyList<int> OutParameters` exposes a copied, sorted list. Every index must be
unique, in bounds and designate a ByReference parameter; invalid or oversized lists
throw ArgumentException. Omission means ordinary ref semantics. The distinction is a
parameter contract, not an overload identity; methods cannot overload ref versus out.
Interface implementations must match the declared out contract.

Callers may pass uninitialized local addresses to declared outputs. On normal return,
those locals become definitely assigned. Ref inputs, including aliases of outputs,
are checked before any output assignment is published. Callees cannot read an output
before writing it and must assign every output on every normal return path; forwarding
an output to another out parameter also establishes assignment. Ldobj/Stobj use the
same parameter addresses as ref calls. Invalid flow raises InvalidDataException when
writing either representation. Initobj still accepts only local addresses.

CLI output uses ordinary BYREF signatures plus Param Out flags. Snapshot signature
readers and imports retain those flags in OutParameters. Native output uses the existing
out_parameters function member, which runtime loading checks and execution enforces;
no wire-format extension or runtime opcode is added. Native-to-CLI projection writes
matching Param rows. Generic method/owner substitution retains output indices.
C# definite-assignment is a language rule; the CLI flag alone does not prove an external
body assigns its outputs. Imported contracts require a trusted/validated implementation;
the metadata reader does not inspect method bodies.

`OutParameterChecks.cs` covers generic assignment, output forwarding, ordinary CLR Out
reflection, native projection/import and library/consumer execution. Negative checks
cover invalid indices, missing/partial assignment, reads before writes, interface
contract mismatch and aliased ref/out preconditions. Run `--out-integration <runtime>
<fresh-output>` for native binary verify/run (42). Conditional out_when_true,
readonly/in contracts and Raven admission remain unsupported by this producer API.
