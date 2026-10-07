using System.Text;
using System.Text.Json;

namespace NeoCLR.Metadata.Experimental;

// NEOX execution schemas 2/3/4: bounded CBOR, definite containers, text keys, Int64;
// library schema 3 additionally admits UInt64 and larger explicit budgets.
// Schema 4 doubles only the library envelope byte budget; all other bounds remain.
// booleans/null; no tags, byte strings, floating point or indefinite lengths.
internal static class NativeBinaryCodec
{
    private static readonly UTF8Encoding Utf8 = new(false, true);
    internal static byte[] Encode(ReadOnlySpan<byte> json, bool library = false, bool expandedLibrary = false)
    {
        if (json.Length > (library ? 32 * 1024 * 1024 : 4 * 1024 * 1024)) throw Invalid("native JSON exceeds limit");
        try { return EncodeCore(json, library, expandedLibrary); }
        catch (Exception error) when (error is JsonException or InvalidOperationException or EncoderFallbackException)
        { throw new InvalidDataException("invalid native binary input", error); }
    }

    private static byte[] EncodeCore(ReadOnlySpan<byte> json, bool library, bool expandedLibrary)
    {
        using var document = JsonDocument.Parse(json.ToArray(), new JsonDocumentOptions { MaxDepth = 64 });
        using var stream = new MemoryStream();
        void Head(int major, ulong value)
        {
            Span<byte> bytes = stackalloc byte[9];
            if (value < 24) { stream.WriteByte((byte)(major * 32 + (int)value)); return; }
            int size = value <= byte.MaxValue ? 1 : value <= ushort.MaxValue ? 2 : value <= uint.MaxValue ? 4 : 8;
            bytes[0] = (byte)(major * 32 + (size == 1 ? 24 : size == 2 ? 25 : size == 4 ? 26 : 27));
            for (int i = size; i > 0; i--) { bytes[i] = (byte)value; value >>= 8; }
            stream.Write(bytes[..(size + 1)]);
        }
        void Text(string text) { var bytes = Utf8.GetBytes(text); Head(3, (ulong)bytes.Length); stream.Write(bytes); }
        void Value(JsonElement value)
        {
            switch (value.ValueKind)
            {
                case JsonValueKind.Object:
                    var properties = value.EnumerateObject().ToArray(); Head(5, (ulong)properties.Length);
                    var names = new HashSet<string>(StringComparer.Ordinal);
                    foreach (var property in properties)
                    {
                        if (!names.Add(property.Name)) throw Invalid("duplicate map key");
                        Text(property.Name); Value(property.Value);
                    }
                    break;
                case JsonValueKind.Array:
                    Head(4, (ulong)value.GetArrayLength()); foreach (var item in value.EnumerateArray()) Value(item); break;
                case JsonValueKind.String: Text(value.GetString()!); break;
                case JsonValueKind.Number:
                    if (library && value.TryGetUInt64(out ulong unsigned)) { Head(0, unsigned); break; }
                    if (!value.TryGetInt64(out long integer)) throw Invalid("unsupported integer range or non-integer number");
                    Head(integer >= 0 ? 0 : 1, (ulong)(integer >= 0 ? integer : -(integer + 1))); break;
                case JsonValueKind.True: stream.WriteByte(0xf5); break;
                case JsonValueKind.False: stream.WriteByte(0xf4); break;
                case JsonValueKind.Null: stream.WriteByte(0xf6); break;
                default: throw Invalid("unsupported value");
            }
            if (stream.Length > (expandedLibrary ? 16 * 1024 * 1024 : library ? 8 * 1024 * 1024 : MetadataEnvelope.MaxImageSize) - 32) throw Invalid("binary payload exceeds envelope limit");
        }
        Value(document.RootElement);
        var result = stream.ToArray();
        // Apply the same node/depth limits as consumers before publishing bytes.
        _ = Decode(result, library, expandedLibrary);
        return result;
    }

    internal static byte[] Decode(ReadOnlySpan<byte> input, bool library = false, bool expandedLibrary = false)
    {
        if (input.Length > (expandedLibrary ? 16 * 1024 * 1024 : library ? 8 * 1024 * 1024 : MetadataEnvelope.MaxImageSize) - 32) throw Invalid("binary payload exceeds envelope limit");
        var data = input.ToArray(); int position = 0, nodes = 0;
        using var stream = new MemoryStream();
        using var json = new Utf8JsonWriter(stream);
        byte[] Take(int size)
        {
            if (size < 0 || size > data.Length - position) throw Invalid("truncated binary payload");
            var bytes = data.AsSpan(position, size).ToArray(); position += size; return bytes;
        }
        (int Major, ulong Value) Head()
        {
            byte first = Take(1)[0]; int major = first >> 5, argument = first & 31;
            if (argument < 24) return (major, (ulong)argument);
            int size = argument switch { 24 => 1, 25 => 2, 26 => 4, 27 => 8, _ => throw Invalid("indefinite or reserved CBOR item") };
            ulong value = 0; foreach (byte b in Take(size)) value = (value << 8) | b;
            if (value < (size == 1 ? 24UL : size == 2 ? 256UL : size == 4 ? 65536UL : 4294967296UL))
                throw Invalid("nonminimal CBOR argument");
            return (major, value);
        }
        int Length(ulong length)
        {
            if (length > (ulong)(data.Length - position)) throw Invalid("binary length exceeds remaining bytes");
            return (int)length;
        }
        string Text(ulong length)
        {
            try { return Utf8.GetString(Take(Length(length))); }
            catch (DecoderFallbackException error) { throw new InvalidDataException("invalid binary UTF-8", error); }
        }
        void Count(int depth)
        {
            if (depth > 64 || ++nodes > (library ? 2097152 : 262144)) throw Invalid("binary depth/node limit exceeded");
        }
        void Value(int depth)
        {
            Count(depth); var (major, argument) = Head();
            switch (major)
            {
                case 0 when library: json.WriteNumberValue(argument); break;
                case 0: case 1:
                    if (argument > long.MaxValue) throw Invalid("integer outside Int64 range");
                    json.WriteNumberValue(major == 0 ? (long)argument : -1 - (long)argument); break;
                case 3: json.WriteStringValue(Text(argument)); break;
                case 4:
                    int count = Length(argument); json.WriteStartArray();
                    for (int i = 0; i < count; i++) Value(depth + 1);
                    json.WriteEndArray(); break;
                case 5:
                    int pairs = Length(argument);
                    if (pairs > (data.Length - position) / 2) throw Invalid("map count exceeds remaining bytes");
                    var names = new HashSet<string>(StringComparer.Ordinal); json.WriteStartObject();
                    for (int i = 0; i < pairs; i++)
                    {
                        Count(depth + 1); var key = Head();
                        if (key.Major != 3) throw Invalid("map key must be text");
                        string name = Text(key.Value); if (!names.Add(name)) throw Invalid("duplicate map key");
                        json.WritePropertyName(name); Value(depth + 1);
                    }
                    json.WriteEndObject(); break;
                case 7 when argument == 20: json.WriteBooleanValue(false); break;
                case 7 when argument == 21: json.WriteBooleanValue(true); break;
                case 7 when argument == 22: json.WriteNullValue(); break;
                default: throw Invalid("unsupported CBOR kind");
            }
        }
        Value(0);
        if (position != data.Length) throw Invalid("trailing binary bytes");
        json.Flush();
        if (stream.Length > (library ? 32 * 1024 * 1024 : MetadataArtifactReader.MaxImageSize)) throw Invalid("decoded JSON exceeds limit");
        return stream.ToArray();
    }
    private static InvalidDataException Invalid(string message) => new(message);
}
