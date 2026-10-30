using System.Collections.ObjectModel;
using System.Globalization;
using System.Text.Json;

namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An explicitly supplied, immutable parameter constant; null differs from no default.</summary>
public sealed class ParameterDefaultValue
{
    /// <summary>The scalar value, or null for an explicit null default.</summary>
    public object? Value { get; }
    /// <summary>Constructs a supported scalar default without invoking application code.</summary>
    public ParameterDefaultValue(object? value)
    {
        if (value is not (null or bool or sbyte or byte or short or ushort or int or uint or long or ulong or float or double or string))
            throw new ArgumentException("unsupported parameter default", nameof(value));
        Value = value;
    }
    internal string Kind => Value?.GetType().Name ?? "Null";
    internal string Text => Value switch
    {
        null => "",
        string text => text,
        bool boolean => boolean ? "true" : "false",
        float number => BitConverter.SingleToUInt32Bits(number).ToString("x8", CultureInfo.InvariantCulture),
        double number => BitConverter.DoubleToUInt64Bits(number).ToString("x16", CultureInfo.InvariantCulture),
        IFormattable number => number.ToString(null, CultureInfo.InvariantCulture),
        _ => throw new InvalidDataException("invalid parameter default")
    };
    internal object Encode() => new { kind = Kind, value = Text };
    internal static ParameterDefaultValue Decode(JsonElement element)
    {
        if (element.ValueKind != JsonValueKind.Object || element.EnumerateObject().Count() != 2 ||
            !element.TryGetProperty("kind", out var kind) || !element.TryGetProperty("value", out var text) ||
            kind.ValueKind != JsonValueKind.String || text.ValueKind != JsonValueKind.String)
            throw new InvalidDataException("invalid parameter default");
        var value = text.GetString()!;
        try
        {
            object? result = kind.GetString() switch
            {
                "Null" when value == "" => null,
                "String" => value,
                "Boolean" when value is "true" or "false" => value == "true",
                "SByte" => sbyte.Parse(value, CultureInfo.InvariantCulture),
                "Byte" => byte.Parse(value, CultureInfo.InvariantCulture),
                "Int16" => short.Parse(value, CultureInfo.InvariantCulture),
                "UInt16" => ushort.Parse(value, CultureInfo.InvariantCulture),
                "Int32" => int.Parse(value, CultureInfo.InvariantCulture),
                "UInt32" => uint.Parse(value, CultureInfo.InvariantCulture),
                "Int64" => long.Parse(value, CultureInfo.InvariantCulture),
                "UInt64" => ulong.Parse(value, CultureInfo.InvariantCulture),
                "Single" when value.Length == 8 => BitConverter.UInt32BitsToSingle(uint.Parse(value, NumberStyles.HexNumber, CultureInfo.InvariantCulture)),
                "Double" when value.Length == 16 => BitConverter.UInt64BitsToDouble(ulong.Parse(value, NumberStyles.HexNumber, CultureInfo.InvariantCulture)),
                _ => throw new InvalidDataException("unsupported parameter default kind")
            };
            var decoded = new ParameterDefaultValue(result);
            if (decoded.Text != value) throw new InvalidDataException("noncanonical parameter default");
            return decoded;
        }
        catch (Exception error) when (error is FormatException or OverflowException)
        { throw new InvalidDataException("invalid parameter default value", error); }
    }
    internal void Validate(SignatureType type)
    {
        bool valid = Value is null
            ? type.Primitive == PrimitiveType.String || type.ArrayElement is not null ||
                type.ImportedType is { IsValueType: false } || type.ClassType is { IsValueType: false } || type.GenericInstance?.Definition is { IsValueType: false } ||
                type.ReferencedType?.Resolve() is { IsValueType: false } ||
                type.ReferencedGenericInstance?.Definition.Resolve() is { IsValueType: false } || type.FunctionSignature is not null
            : type.Primitive?.ToString() == Kind;
        if (!valid) throw new InvalidDataException("parameter default does not match its by-value type");
    }
}

public sealed partial class MethodDefinition
{
    private readonly Dictionary<int, ParameterDefaultValue> parameterDefaults = [];
    private IReadOnlyDictionary<int, ParameterDefaultValue>? parameterDefaultView;
    /// <summary>Explicit defaults keyed by zero-based position; absent entries are required parameters.</summary>
    public IReadOnlyDictionary<int, ParameterDefaultValue> ParameterDefaults => parameterDefaultView ??= new ReadOnlyDictionary<int, ParameterDefaultValue>(parameterDefaults);
    /// <summary>Sets or clears a scalar default on an authored parameter without changing call arity.</summary>
    public void SetParameterDefault(int position, ParameterDefaultValue? value)
    {
        if (Producer is null) throw new InvalidOperationException("loaded parameter metadata is read-only");
        if (position < 0 || position >= Producer.ParameterCount) throw new ArgumentOutOfRangeException(nameof(position));
        value?.Validate(Producer.Signature.ParameterTypes[position]);
        if (value is null) parameterDefaults.Remove(position); else parameterDefaults[position] = value;
    }
    internal void LoadParameterDefaults(IReadOnlyDictionary<int, ParameterDefaultValue>? values)
    {
        foreach (var pair in values ?? new Dictionary<int, ParameterDefaultValue>()) parameterDefaults.Add(pair.Key, pair.Value);
    }
}

public sealed partial class MethodBuilder
{
    /// <summary>Sets or clears an explicit parameter default through its owned definition.</summary>
    public void SetParameterDefault(int position, ParameterDefaultValue? value) => Definition.SetParameterDefault(position, value);
}
