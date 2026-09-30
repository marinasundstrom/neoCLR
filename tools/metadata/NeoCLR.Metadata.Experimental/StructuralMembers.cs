using System.Buffers.Binary;

namespace NeoCLR.Metadata.Experimental;

/// <summary>A synthesized operation on signature root 1, not a CLI MethodDef reference.</summary>
/// <param name="Operation">array_length, tuple_element, tuple_deconstruct or function_invoke.</param>
/// <param name="Owner">Must be 1 in this bounded profile.</param>
/// <param name="Element">Zero-based tuple element; zero for other operations.</param>
public sealed record StructuralMemberReference(string Operation, int Owner = 1, int Element = 0);

/// <summary>Catalog-scoped synthesized-member identity, independent of local reference numbering.</summary>
/// <param name="Owner">Resolved structural owner.</param>
/// <param name="Operation">Synthesized operation name.</param>
/// <param name="Element">Tuple element ordinal, or zero.</param>
/// <remarks>Schema 1 only. Value equality is not a persistent identifier or execution capability.</remarks>
public sealed record ResolvedMemberIdentity(ResolvedTypeIdentity Owner, string Operation, int Element);

/// <summary>An immutable derived operation contract; no invocation or dispatch target.</summary>
public sealed class StructuralMemberDescriptor
{
    internal StructuralMemberDescriptor(ResolvedMemberIdentity identity, IEnumerable<ResolvedTypeIdentity> parameters,
        IEnumerable<string> modes, ResolvedTypeIdentity result, bool noResult)
    {
        Identity = identity;
        Parameters = Array.AsReadOnly(parameters.ToArray());
        Modes = Array.AsReadOnly(modes.ToArray());
        Result = result;
        NoResult = noResult;
    }
    /// <summary>Gets the resolved owner/operation/ordinal identity.</summary>
    public ResolvedMemberIdentity Identity { get; }
    /// <summary>Gets ordered parameter type identities, excluding the receiver.</summary>
    public IReadOnlyList<ResolvedTypeIdentity> Parameters { get; }
    /// <summary>Gets one passing mode per parameter.</summary>
    public IReadOnlyList<string> Modes { get; }
    /// <summary>Gets the result identity; unit when NoResult is true.</summary>
    public ResolvedTypeIdentity Result { get; }
    /// <summary>Gets whether the operation has no result, distinct from an inhabited unit result.</summary>
    public bool NoResult { get; }
}

/// <summary>Section-4/schema-1 synthesized member codec and explicit-catalog contract derivation.</summary>
public static class StructuralMembers
{
    private static readonly string[] Operations = ["array_length", "tuple_element", "tuple_deconstruct", "function_invoke"];
    /// <summary>Gets the intrinsic native unsigned result identity used by array length.</summary>
    /// <remarks>This descriptor identity is not a new serializable signature opcode.</remarks>
    public static ResolvedTypeIdentity NativeUnsignedResult { get; } = StructuralIdentity.NativeUnsignedResult();

    /// <summary>Reads a complete bounded table into owned, read-only storage.</summary>
    /// <param name="payload">Section-4/schema-1 payload.</param>
    /// <returns>Ordered, unique operation references; owner shapes are not yet validated.</returns>
    /// <exception cref="InvalidDataException">Invalid count/length, operation, flags, owner, operand or duplicates.</exception>
    public static IReadOnlyList<StructuralMemberReference> Read(ReadOnlySpan<byte> payload)
    {
        if (payload.Length < 2) throw new InvalidDataException("truncated structural member table");
        int count = U16(payload);
        if (count > 256 || payload.Length != 2 + count * 6) throw new InvalidDataException("invalid structural member table length/count");
        var members = new StructuralMemberReference[count];
        for (int i = 0; i < count; i++)
        {
            var row = payload.Slice(2 + i * 6, 6);
            if (row[2] is < 1 or > 4 || row[3] != 0) throw new InvalidDataException("unknown structural operation or flags");
            members[i] = new(Operations[row[2] - 1], U16(row), U16(row[4..]));
        }
        Validate(members);
        return Array.AsReadOnly(members);
    }

    /// <summary>Writes validated references to a new owned payload, preserving order.</summary>
    /// <param name="members">At most 256 unique references.</param>
    /// <returns>New section-4/schema-1 bytes.</returns>
    /// <exception cref="ArgumentNullException">Members is null.</exception>
    /// <exception cref="InvalidDataException">Invalid count, null/invalid rows or duplicates.</exception>
    public static byte[] Write(IReadOnlyList<StructuralMemberReference> members)
    {
        ArgumentNullException.ThrowIfNull(members);
        Validate(members);
        var bytes = new byte[2 + members.Count * 6];
        Put(bytes, members.Count);
        for (int i = 0; i < members.Count; i++)
        {
            var row = bytes.AsSpan(2 + i * 6, 6);
            Put(row, members[i].Owner);
            row[2] = (byte)(Array.IndexOf(Operations, members[i].Operation) + 1);
            Put(row[4..], members[i].Element);
        }
        return bytes;
    }

    /// <summary>Validates the owner and derives catalog-scoped member contracts.</summary>
    /// <param name="members">Ordered member references.</param>
    /// <param name="root">Section-3 signature root, owner handle 1.</param>
    /// <param name="context">Signature binder context.</param>
    /// <param name="bindings">Nominal references and declaring owners.</param>
    /// <param name="catalog">Authoritative host declarations; do not mutate inputs during resolution.</param>
    /// <returns>Owned read-only descriptors in input order.</returns>
    /// <exception cref="ArgumentNullException">Any argument is null.</exception>
    /// <exception cref="InvalidDataException">Invalid table, unresolved owner, incompatible shape or tuple ordinal outside arity.</exception>
    /// <remarks>Does not validate envelope profile composition, load assemblies or grant execution access.</remarks>
    public static IReadOnlyList<StructuralMemberDescriptor> Resolve(IReadOnlyList<StructuralMemberReference> members,
        TypeExpression root, SignatureContext context, ReferenceBindings bindings,
        IReadOnlyDictionary<MetadataReference, MetadataDefinition> catalog)
    {
        ArgumentNullException.ThrowIfNull(members);
        Validate(members);
        var owner = StructuralIdentity.Resolve(root, context, bindings, catalog);
        // Resolve each child once; tuple projections share identities without repeated per-member traversal.
        var children = root.Children.Select(child => StructuralIdentity.Resolve(child, context, bindings, catalog)).ToArray();
        var unit = StructuralIdentity.Resolve(new TypeExpression("unit"), context, bindings, catalog);
        var result = new List<StructuralMemberDescriptor>();
        foreach (var member in members)
        {
            bool valid = member.Operation switch
            {
                "array_length" => root.Kind is "array" or "array_ref",
                "tuple_element" => root.Kind == "tuple" && member.Element < children.Length,
                "tuple_deconstruct" => root.Kind == "tuple",
                "function_invoke" => root.Kind == "function",
                _ => false
            };
            if (!valid) throw new InvalidDataException("structural operation is incompatible with owner shape or tuple ordinal");
            var identity = new ResolvedMemberIdentity(owner, member.Operation, member.Element);
            result.Add(member.Operation switch
            {
                "array_length" => new(identity, [], [], NativeUnsignedResult, false),
                "tuple_element" => new(identity, [], [], children[member.Element], false),
                "tuple_deconstruct" => new(identity, children, Enumerable.Repeat("out", children.Length), unit, true),
                _ => new(identity, children[..^1], root.Modes, children[^1], root.NoResult)
            });
        }
        return result.AsReadOnly();
    }

    private static void Validate(IReadOnlyList<StructuralMemberReference> members)
    {
        if (members.Count > 256) throw new InvalidDataException("too many structural members");
        var unique = new HashSet<StructuralMemberReference>();
        foreach (var member in members)
        {
            if (member is null || Array.IndexOf(Operations, member.Operation) < 0 || member.Owner != 1 ||
                member.Element is < 0 or > 255 || (member.Operation != "tuple_element" && member.Element != 0))
                throw new InvalidDataException("invalid structural operation reference");
            if (!unique.Add(member)) throw new InvalidDataException("duplicate structural member reference");
        }
    }
    private static ushort U16(ReadOnlySpan<byte> bytes) => BinaryPrimitives.ReadUInt16LittleEndian(bytes);
    private static void Put(Span<byte> bytes, int value) => BinaryPrimitives.WriteUInt16LittleEndian(bytes, (ushort)value);
}
