using System.Text.Json;

namespace NeoCLR.Metadata.Experimental;

/// <summary>Translates existing format-5 JSON modules into standalone binary NEOX containers.</summary>
/// <remarks>Preserves native values without synthesizing CLI declarations. This is a transport API,
/// not a semantic metadata reader: neoCLR must validate declarations, dependencies and bodies.</remarks>
public static class NativeModuleContainer
{
    private static readonly IReadOnlyDictionary<ushort, ushort> Schemas = new Dictionary<ushort, ushort> { [256] = 2 };

    /// <summary>Encodes a native JSON module using required execution schema 2.</summary>
    /// <param name="nativeImage">Format-5 JSON, at most 4 MiB, with integer numeric values only.</param>
    /// <returns>Owned NEOX bytes, at most 1 MiB, without PE or CLI metadata.</returns>
    /// <exception cref="InvalidDataException">Invalid module header, unsupported binary values or exceeded bounds.</exception>
    public static byte[] WriteBinary(ReadOnlySpan<byte> nativeImage)
    {
        var payload = NativeBinaryCodec.Encode(nativeImage);
        ValidateHeader(NativeBinaryCodec.Decode(payload));
        return MetadataEnvelope.Write([new MetadataSection(256, 2, true, payload)], Schemas);
    }

    /// <summary>Encodes a standalone library using execution schema 3, including UInt64 operands.</summary>
    /// <param name="nativeImage">Format-5 JSON up to 32 MiB; Int64 negatives and UInt64 nonnegative numbers.</param>
    /// <returns>Owned NEOX bytes up to 8 MiB, without CLI metadata.</returns>
    /// <exception cref="InvalidDataException">Invalid header, unsupported values or exceeded bounds.</exception>
    /// <remarks>Depth remains 64; the item budget is 2,097,152. Older readers reject required schema 3.</remarks>
    public static byte[] WriteLibraryBinary(ReadOnlySpan<byte> nativeImage)
    {
        var payload = NativeBinaryCodec.Encode(nativeImage, library: true);
        ValidateHeader(NativeBinaryCodec.Decode(payload, library: true));
        return MetadataEnvelope.WriteCore([new MetadataSection(256, 3, true, payload, 8 * 1024 * 1024)],
            new Dictionary<ushort, ushort> { [256] = 3 }, 8 * 1024 * 1024);
    }

    /// <summary>Validates transport and the module header, then reconstructs owned native JSON values.</summary>
    /// <param name="image">Complete standalone NEOX image: schema 2 up to 1 MiB or schema 3 up to 8 MiB.</param>
    /// <returns>Equivalent JSON values; original whitespace and lexical spellings are not retained.</returns>
    /// <exception cref="InvalidDataException">Malformed framing, required schema, binary values or module header.</exception>
    /// <remarks>Unknown optional sections are admitted but are not returned in the JSON view.
    /// Read, WriteBinary and WriteLibraryBinary do not validate member schemas or executable bodies.</remarks>
    public static byte[] Read(ReadOnlySpan<byte> image)
    {
        IReadOnlyList<MetadataSection> sections;
        try { sections = MetadataEnvelope.Read(image, Schemas); }
        catch (InvalidDataException)
        { sections = MetadataEnvelope.ReadCore(image, new Dictionary<ushort, ushort> { [256] = 3 }, 8 * 1024 * 1024); }
        var execution = sections.SingleOrDefault(s => s.Kind == 256);
        if (execution is null || !execution.Required || execution.Version is not (2 or 3))
            throw new InvalidDataException("required binary native execution section missing");
        var json = NativeBinaryCodec.Decode(execution.Payload, library: execution.Version == 3);
        ValidateHeader(json);
        return json;
    }

    private static void ValidateHeader(byte[] json)
    {
        using var document = JsonDocument.Parse(json);
        var root = document.RootElement;
        if (root.ValueKind != JsonValueKind.Object ||
            !root.TryGetProperty("format", out var format) || format.ValueKind != JsonValueKind.Number || !format.TryGetInt32(out var version) || version != 5 ||
            !root.TryGetProperty("name", out var name) || name.ValueKind != JsonValueKind.String || string.IsNullOrWhiteSpace(name.GetString()) ||
            !root.TryGetProperty("functions", out var functions) || functions.ValueKind != JsonValueKind.Array)
            throw new InvalidDataException("expected format-5 native module header");
    }
}
