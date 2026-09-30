using System.Buffers.Binary;

namespace NeoCLR.Metadata.Experimental;

/// <summary>A definition key in an explicit host catalog, not a persistent CLI assembly identity.</summary>
/// <param name="Assembly">Nonempty host-assigned assembly scope UUID.</param>
/// <param name="Module">Nonempty host-assigned module scope UUID.</param>
/// <param name="Token">Nonzero-row TypeDef or MethodDef token in that catalog module.</param>
public readonly record struct MetadataReference(Guid Assembly, Guid Module, uint Token);

/// <summary>An authoritative host-catalog declaration used by the experimental resolver.</summary>
/// <param name="Kind">type, interface or method.</param>
/// <param name="Arity">Generic arity, from zero through 256.</param>
/// <param name="Owner">Declaring type for a method; otherwise null.</param>
public sealed record MetadataDefinition(string Kind, int Arity = 0, MetadataReference? Owner = null);

/// <summary>Immutable local references and one-based owner indices; zero owner indices mean absent.</summary>
public sealed class ReferenceBindings
{
    /// <summary>Copies the reference list. Wire and owner validation occurs in ReferenceTable.</summary>
    /// <param name="references">At most 256 catalog references.</param>
    /// <param name="typeOwner">Declaring type index, or zero.</param>
    /// <param name="methodOwner">Declaring method index, or zero.</param>
    /// <param name="selfOwner">Self contract index, or zero.</param>
    /// <exception cref="ArgumentNullException">References is null.</exception>
    /// <exception cref="ArgumentException">The list exceeds 256 entries.</exception>
    public ReferenceBindings(IReadOnlyList<MetadataReference> references, int typeOwner = 0, int methodOwner = 0, int selfOwner = 0)
    {
        ArgumentNullException.ThrowIfNull(references);
        if (references.Count > 256) throw new ArgumentException("too many references", nameof(references));
        References = Array.AsReadOnly(references.ToArray());
        TypeOwner = typeOwner;
        MethodOwner = methodOwner;
        SelfOwner = selfOwner;
    }
    /// <summary>Gets copied references in local index order.</summary>
    public IReadOnlyList<MetadataReference> References { get; }
    /// <summary>Gets the declaring-type index, or zero.</summary>
    public int TypeOwner { get; }
    /// <summary>Gets the declaring-method index, or zero.</summary>
    public int MethodOwner { get; }
    /// <summary>Gets the Self-contract index, or zero.</summary>
    public int SelfOwner { get; }
}

/// <summary>Codec for the section-2/schema-1 reference payload, independent of PE tables.</summary>
public static class ReferenceTable
{
    /// <summary>Reads bounded references with network-order UUIDs and little-endian tokens.</summary>
    /// <param name="payload">One complete reference payload.</param>
    /// <returns>Owned bindings with validated kinds, indices and uniqueness.</returns>
    /// <exception cref="InvalidDataException">Malformed data or invalid references/owners.</exception>
    public static ReferenceBindings Read(ReadOnlySpan<byte> payload)
    {
        if (payload.Length < 8) throw new InvalidDataException("truncated reference table");
        int count = U16(payload, 0);
        if (count > 256 || payload.Length != 8 + 36 * count) throw new InvalidDataException("invalid reference table length/count");
        var refs = new MetadataReference[count];
        for (int i = 0; i < count; i++)
        {
            int offset = 8 + i * 36;
            refs[i] = new(new Guid(payload.Slice(offset, 16), bigEndian: true),
                new Guid(payload.Slice(offset + 16, 16), bigEndian: true),
                BinaryPrimitives.ReadUInt32LittleEndian(payload[(offset + 32)..]));
        }
        var result = new ReferenceBindings(refs, U16(payload, 2), U16(payload, 4), U16(payload, 6));
        Validate(result);
        return result;
    }

    /// <summary>Validates and writes a new payload, preserving local reference order.</summary>
    /// <param name="bindings">Reference list and owner indices.</param>
    /// <returns>A new caller-owned byte array.</returns>
    /// <exception cref="ArgumentNullException">Bindings is null.</exception>
    /// <exception cref="InvalidDataException">Invalid/duplicate references or owner indices/kinds.</exception>
    public static byte[] Write(ReferenceBindings bindings)
    {
        ArgumentNullException.ThrowIfNull(bindings);
        Validate(bindings);
        var result = new byte[8 + 36 * bindings.References.Count];
        Put(result, 0, bindings.References.Count);
        Put(result, 2, bindings.TypeOwner); Put(result, 4, bindings.MethodOwner); Put(result, 6, bindings.SelfOwner);
        for (int i = 0; i < bindings.References.Count; i++)
        {
            var reference = bindings.References[i];
            int offset = 8 + i * 36;
            reference.Assembly.TryWriteBytes(result.AsSpan(offset, 16), bigEndian: true, out _);
            reference.Module.TryWriteBytes(result.AsSpan(offset + 16, 16), bigEndian: true, out _);
            BinaryPrimitives.WriteUInt32LittleEndian(result.AsSpan(offset + 32), reference.Token);
        }
        return result;
    }

    internal static void Validate(ReferenceBindings bindings)
    {
        var unique = new HashSet<MetadataReference>();
        foreach (var reference in bindings.References)
        {
            ValidateReference(reference);
            if (!unique.Add(reference)) throw new InvalidDataException("duplicate reference");
        }
        foreach (var (index, kind) in new[] { (bindings.TypeOwner, 2u), (bindings.MethodOwner, 6u), (bindings.SelfOwner, 2u) })
        {
            if (index < 0 || index > bindings.References.Count) throw new InvalidDataException("owner reference outside table");
            if (index != 0 && bindings.References[index - 1].Token >> 24 != kind) throw new InvalidDataException("wrong owner reference kind");
        }
    }
    internal static void ValidateReference(MetadataReference reference)
    {
        if (reference.Assembly == Guid.Empty || reference.Module == Guid.Empty ||
            reference.Token >> 24 is not (2 or 6) || (reference.Token & 0xffffff) == 0)
            throw new InvalidDataException("invalid catalog definition reference");
    }
    private static ushort U16(ReadOnlySpan<byte> data, int offset) => BinaryPrimitives.ReadUInt16LittleEndian(data[offset..]);
    private static void Put(Span<byte> data, int offset, int value) => BinaryPrimitives.WriteUInt16LittleEndian(data[offset..], (ushort)value);
}
