# Experimental .NET metadata library

**Feature-branch development only; not part of Preview 11 or the neoCLR guest API.**
Assembly and namespace: `NeoCLR.Metadata.Experimental`. Target: .NET 10.
Source project: `tools/metadata/NeoCLR.Metadata.Experimental`.
This is the first reusable reader/writer slice intended for Raven's future symbol
loader and code-generation adapters. It reads/writes **NEOX 0.1 framing, structural signatures, reference tables and synthesized-member tables**,
and derives structural identities/member contracts against an explicitly supplied host catalog.
No PE/CLI loading, artifact recognition, physical CLI declaration resolution,
Introspection assembly loading or assembly emission is implemented by this library yet.

## Namespace and types

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
