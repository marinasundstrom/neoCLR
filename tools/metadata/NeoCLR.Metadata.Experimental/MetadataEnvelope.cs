using System.Buffers.Binary;

namespace NeoCLR.Metadata.Experimental;

/// <summary>Reads and writes bounded NEOX 0.1 envelopes, without interpreting section payloads.</summary>
/// <remarks>Experimental host tooling, not a PE reader or a guest runtime API.</remarks>
public static class MetadataEnvelope
{
    /// <summary>Maximum encoded image size in bytes (1 MiB).</summary>
    public const int MaxImageSize = 1024 * 1024;
    /// <summary>Maximum number of sections in an envelope.</summary>
    public const int MaxSections = 64;
    private const int HeaderSize = 16;
    private const int EntrySize = 16;

    /// <summary>Validates framing and schema requirements and copies sections into owned storage.</summary>
    /// <param name="image">Complete image; the caller must not modify it during this call.</param>
    /// <param name="supportedSchemas">Kind/version pairs admitted by the caller. Unknown optional sections are retained.</param>
    /// <returns>A read-only list of immutable sections in input order, independent of the input buffer.</returns>
    /// <exception cref="ArgumentNullException">The schema dictionary is null.</exception>
    /// <exception cref="InvalidDataException">The image is malformed, exceeds limits, or requires an unsupported schema.</exception>
    /// <remarks>Admission does not validate a schema's payload, resolve references or authorize execution.</remarks>
    public static IReadOnlyList<MetadataSection> Read(
        ReadOnlySpan<byte> image, IReadOnlyDictionary<ushort, ushort> supportedSchemas)
        => ReadCore(image, supportedSchemas, MaxImageSize);

    internal static IReadOnlyList<MetadataSection> ReadCore(ReadOnlySpan<byte> image,
        IReadOnlyDictionary<ushort, ushort> supportedSchemas, int maxImageSize)
    {
        ArgumentNullException.ThrowIfNull(supportedSchemas);
        if (image.Length < HeaderSize || image.Length > maxImageSize) throw Invalid("invalid image size");
        if (!image[..4].SequenceEqual("NEOX"u8) || U16(image, 4) != 0 || U16(image, 6) != 1)
            throw Invalid("unsupported envelope version or magic");
        uint count = U32(image, 8);
        if (count > MaxSections || U32(image, 12) != image.Length) throw Invalid("invalid size or section count");
        int end = HeaderSize + (int)count * EntrySize;
        if (end > image.Length) throw Invalid("truncated directory");
        var seen = new HashSet<ushort>();
        var sections = new List<MetadataSection>((int)count);
        for (int i = 0; i < count; i++)
        {
            int entry = HeaderSize + i * EntrySize;
            ushort kind = U16(image, entry), version = U16(image, entry + 2);
            uint flags = U32(image, entry + 4), offset = U32(image, entry + 8), length = U32(image, entry + 12);
            if (kind == 0 || version == 0 || !seen.Add(kind)) throw Invalid("invalid or duplicate section kind/version");
            if ((flags & ~1u) != 0) throw Invalid("unknown section flags");
            if (offset != end || length > image.Length - end) throw Invalid("invalid section range");
            bool required = (flags & 1) != 0;
            if (required && (!supportedSchemas.TryGetValue(kind, out ushort supported) || supported != version))
                throw Invalid($"unsupported required section {kind} schema {version}");
            sections.Add(new MetadataSection(kind, version, required, image.Slice(end, (int)length), maxImageSize));
            end += (int)length;
        }
        if (end != image.Length) throw Invalid("trailing bytes");
        return sections.AsReadOnly();
    }

    /// <summary>Writes sections in supplied order with canonical contiguous directory/payload framing.</summary>
    /// <param name="sections">Immutable sections to encode; duplicate kinds are invalid.</param>
    /// <param name="supportedSchemas">The writer's admitted kind/version pairs for required sections.</param>
    /// <returns>A new caller-owned complete NEOX image.</returns>
    /// <exception cref="ArgumentNullException">Either argument is null.</exception>
    /// <exception cref="InvalidDataException">Sections are null, duplicated, unsupported or exceed envelope limits.</exception>
    /// <remarks>The caller owns payload validation and reference remapping. Opaque preservation is not semantic rewriting.</remarks>
    public static byte[] Write(
        IReadOnlyList<MetadataSection> sections, IReadOnlyDictionary<ushort, ushort> supportedSchemas)
        => WriteCore(sections, supportedSchemas, MaxImageSize);

    internal static byte[] WriteCore(IReadOnlyList<MetadataSection> sections,
        IReadOnlyDictionary<ushort, ushort> supportedSchemas, int maxImageSize)
    {
        ArgumentNullException.ThrowIfNull(sections);
        ArgumentNullException.ThrowIfNull(supportedSchemas);
        if (sections.Count > MaxSections) throw Invalid("too many sections");
        int size = HeaderSize + sections.Count * EntrySize;
        var seen = new HashSet<ushort>();
        foreach (var section in sections)
        {
            if (section is null) throw Invalid("null section");
            if (!seen.Add(section.Kind)) throw Invalid("duplicate section kind");
            if (section.Required && (!supportedSchemas.TryGetValue(section.Kind, out ushort version) || version != section.Version))
                throw Invalid("unsupported required section");
            if (section.PayloadLength > maxImageSize - size) throw Invalid("image too large");
            size += section.PayloadLength;
        }
        byte[] result = new byte[size];
        "NEOX"u8.CopyTo(result);
        Put16(result, 6, 1);
        Put32(result, 8, (uint)sections.Count);
        Put32(result, 12, (uint)size);
        int offset = HeaderSize + sections.Count * EntrySize;
        for (int i = 0; i < sections.Count; i++)
        {
            var section = sections[i];
            int entry = HeaderSize + i * EntrySize;
            Put16(result, entry, section.Kind);
            Put16(result, entry + 2, section.Version);
            Put32(result, entry + 4, section.Required ? 1u : 0u);
            Put32(result, entry + 8, (uint)offset);
            Put32(result, entry + 12, (uint)section.PayloadLength);
            section.Payload.CopyTo(result.AsSpan(offset));
            offset += section.PayloadLength;
        }
        return result;
    }

    private static InvalidDataException Invalid(string message) => new(message);
    private static ushort U16(ReadOnlySpan<byte> data, int offset) => BinaryPrimitives.ReadUInt16LittleEndian(data[offset..]);
    private static uint U32(ReadOnlySpan<byte> data, int offset) => BinaryPrimitives.ReadUInt32LittleEndian(data[offset..]);
    private static void Put16(Span<byte> data, int offset, ushort value) => BinaryPrimitives.WriteUInt16LittleEndian(data[offset..], value);
    private static void Put32(Span<byte> data, int offset, uint value) => BinaryPrimitives.WriteUInt32LittleEndian(data[offset..], value);
}
