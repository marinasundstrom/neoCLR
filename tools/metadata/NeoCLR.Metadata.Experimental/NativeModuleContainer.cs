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

    /// <summary>Validates transport and the module header, then reconstructs owned native JSON values.</summary>
    /// <param name="image">Complete standalone schema-2 NEOX image, at most 1 MiB.</param>
    /// <returns>Equivalent JSON values; original whitespace and lexical spellings are not retained.</returns>
    /// <exception cref="InvalidDataException">Malformed framing, required schema, binary values or module header.</exception>
    /// <remarks>Unknown optional sections are admitted but are not returned in the JSON view.
    /// Read and WriteBinary do not validate member schemas or executable bodies.</remarks>
    public static byte[] Read(ReadOnlySpan<byte> image)
    {
        var sections = MetadataEnvelope.Read(image, Schemas);
        var execution = sections.SingleOrDefault(s => s.Kind == 256);
        if (execution is null || !execution.Required || execution.Version != 2)
            throw new InvalidDataException("required binary native execution section missing");
        var json = NativeBinaryCodec.Decode(execution.Payload);
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
