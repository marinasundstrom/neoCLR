using System.Globalization;
using System.Text.Json;

namespace NeoCLR.Metadata.Experimental.Model;

// Float bits are strings so JSON tools cannot round them or reject non-finite values.
internal static class LiteralConstant
{
    internal static PrimitiveType Type(object value) => value switch
    {
        int => PrimitiveType.Int32,
        float => PrimitiveType.Single,
        double => PrimitiveType.Double,
        _ => throw new InvalidDataException("unsupported literal constant")
    };
    internal static object Encode(object value) => value switch
    {
        int => value,
        float number => new { single_bits = BitConverter.SingleToUInt32Bits(number).ToString("x8", CultureInfo.InvariantCulture) },
        double number => new { double_bits = BitConverter.DoubleToUInt64Bits(number).ToString("x16", CultureInfo.InvariantCulture) },
        _ => throw new InvalidDataException("unsupported literal constant")
    };
    internal static object Decode(JsonElement value)
    {
        if (value.ValueKind == JsonValueKind.Number && value.TryGetInt32(out var integer)) return integer;
        if (value.ValueKind != JsonValueKind.Object || value.EnumerateObject().Count() != 1)
            throw new InvalidDataException("invalid literal constant");
        var part = value.EnumerateObject().Single();
        if (part.Value.ValueKind != JsonValueKind.String) throw new InvalidDataException("invalid literal bits");
        var bits = part.Value.GetString()!;
        if (bits.Any(c => !char.IsAsciiHexDigit(c))) throw new InvalidDataException("invalid literal bits");
        if (part.Name == "single_bits" && bits.Length == 8 && uint.TryParse(bits, NumberStyles.HexNumber, CultureInfo.InvariantCulture, out var single))
            return BitConverter.UInt32BitsToSingle(single);
        if (part.Name == "double_bits" && bits.Length == 16 && ulong.TryParse(bits, NumberStyles.HexNumber, CultureInfo.InvariantCulture, out var wide))
            return BitConverter.UInt64BitsToDouble(wide);
        throw new InvalidDataException("invalid literal bits");
    }
}
