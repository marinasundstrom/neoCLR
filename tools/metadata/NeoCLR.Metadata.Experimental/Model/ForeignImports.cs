using System.Text.Json;
using System.Text.Json.Serialization;

namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>ABI for a synchronous, call-scoped scalar callback. Native code must not retain it.</summary>
public sealed class NativeCallbackDefinition
{
    /// <summary>Creates a scalar callback ABI with an explicit supported convention.</summary>
    public NativeCallbackDefinition(string callingConvention = "Cdecl")
    {
        if (callingConvention is not ("Cdecl" or "System")) throw new ArgumentException("unsupported callback calling convention");
        CallingConvention = callingConvention;
    }
    /// <summary>Gets the callback calling convention, independent of the containing import.</summary>
    [JsonPropertyName("calling_convention")]
    public string CallingConvention { get; }

    internal void Validate(SignatureType type)
    {
        var shape = type.FunctionSignature;
        bool Scalar(SignatureType slot) => slot.Primitive is
            PrimitiveType.SByte or PrimitiveType.Byte or PrimitiveType.Int16 or PrimitiveType.UInt16 or
            PrimitiveType.Int32 or PrimitiveType.UInt32 or PrimitiveType.Int64 or PrimitiveType.UInt64 or
            PrimitiveType.IntPtr or PrimitiveType.UIntPtr or PrimitiveType.Single or PrimitiveType.Double;
        if (shape is null || shape.ParameterTypes.Any(slot => !Scalar(slot)) ||
            !(Scalar(shape.ReturnType) || shape.NoResult))
            throw new InvalidDataException($"native callbacks require a scalar Function signature: {type}");
    }

    internal static NativeCallbackDefinition Read(JsonElement value)
    {
        if (value.ValueKind != JsonValueKind.Object || value.EnumerateObject().Count() != 1 ||
            !value.TryGetProperty("calling_convention", out var convention))
            throw new InvalidDataException("unsupported native callback ABI metadata");
        return new(convention.GetString()!);
    }
}

/// <summary>Explicit C ABI import metadata. No native library is loaded by this object.</summary>
public sealed class NativeImportDefinition
{
    /// <summary>Creates a native import with exact export spelling and optional input string encoding.</summary>
    /// <exception cref="ArgumentException">Names are empty/contain NUL, or encoding is unsupported.</exception>
    public NativeImportDefinition(string library, string entryPoint, string? stringMarshalling = null, string callingConvention = "Cdecl", IReadOnlyDictionary<int, string>? parameterMarshalling = null, string? returnMarshalling = null, IReadOnlyDictionary<int, NativeCallbackDefinition>? callbackParameters = null)
    {
        if (string.IsNullOrEmpty(library) || library.Contains('\0') || string.IsNullOrEmpty(entryPoint) || entryPoint.Contains('\0'))
            throw new ArgumentException("native library and entry point must be nonempty and contain no NUL");
        if (stringMarshalling is not (null or "Utf8" or "Utf16"))
            throw new ArgumentException("native string marshalling must be Utf8 or Utf16");
        if (callingConvention is not ("Cdecl" or "System")) throw new ArgumentException("unsupported native calling convention");
        if (parameterMarshalling?.Any(p => p.Key < 0 || !Valid(p.Value)) == true || returnMarshalling is not null && !Valid(returnMarshalling))
            throw new ArgumentException("unsupported native marshalling descriptor");
        Library = library; EntryPoint = entryPoint; StringMarshalling = stringMarshalling;
        CallingConvention = callingConvention;
        ParameterMarshalling = new System.Collections.ObjectModel.ReadOnlyDictionary<int, string>(parameterMarshalling is null ? [] : new Dictionary<int, string>(parameterMarshalling));
        ReturnMarshalling = returnMarshalling;
        if (callbackParameters?.Any(p => p.Key < 0 || p.Value is null) == true)
            throw new ArgumentException("invalid callback parameter descriptor");
        CallbackParameters = new System.Collections.ObjectModel.ReadOnlyDictionary<int, NativeCallbackDefinition>(
            callbackParameters is null ? [] : new Dictionary<int, NativeCallbackDefinition>(callbackParameters));
    }
    /// <summary>Gets the native loader name or path.</summary>
    [JsonPropertyName("library")]
    public string Library { get; }
    /// <summary>Gets the exact exported symbol.</summary>
    [JsonPropertyName("entry_point")]
    public string EntryPoint { get; }
    /// <summary>Gets the supported platform C ABI.</summary>
    [JsonPropertyName("calling_convention")]
    public string CallingConvention { get; }
    /// <summary>Gets call-scoped input string encoding, or null to reject String arguments.</summary>
    [JsonPropertyName("string_marshalling"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public string? StringMarshalling { get; }

    /// <summary>Gets zero-based parameter conversion overrides.</summary>
    [JsonPropertyName("parameter_marshalling")]
    public IReadOnlyDictionary<int, string> ParameterMarshalling { get; }
    /// <summary>Gets the explicit result conversion, or null for a direct ABI result.</summary>
    [JsonPropertyName("return_marshalling"), JsonIgnore(Condition = JsonIgnoreCondition.WhenWritingNull)]
    public string? ReturnMarshalling { get; }
    /// <summary>Gets zero-based call-scoped scalar callback ABI descriptors.</summary>
    [JsonPropertyName("callback_parameters")]
    public IReadOnlyDictionary<int, NativeCallbackDefinition> CallbackParameters { get; }
    private static bool Valid(string value) => value is "Utf8" or "Utf16" or "Bool8" or "Bool32";

    internal void Validate(MethodSignature signature)
    {
        bool Scalar(SignatureType type) => type.PointerElement is not null || type.Primitive is
            PrimitiveType.SByte or PrimitiveType.Byte or PrimitiveType.Int16 or PrimitiveType.UInt16 or
            PrimitiveType.Int32 or PrimitiveType.UInt32 or PrimitiveType.Int64 or PrimitiveType.UInt64 or
            PrimitiveType.IntPtr or PrimitiveType.UIntPtr or PrimitiveType.Single or PrimitiveType.Double;
        bool Slot(SignatureType type, string? conversion, bool result) => conversion switch
        {
            "Utf8" or "Utf16" => !result && type == PrimitiveType.String,
            "Bool8" or "Bool32" => type == PrimitiveType.Boolean,
            null => Scalar(type) || result && type == PrimitiveType.Void,
            _ => false,
        };
        foreach (var (index, callback) in CallbackParameters)
        {
            if (index >= signature.ParameterTypes.Count || ParameterMarshalling.ContainsKey(index))
                throw new InvalidDataException("callback index outside signature or conflicting marshalling");
            callback.Validate(signature.ParameterTypes[index]);
        }
        if (signature.GenericParameterNames.Count != 0 || signature.OutParameters.Count != 0 ||
            ParameterMarshalling.Keys.Any(i => i >= signature.ParameterTypes.Count) ||
            !Slot(signature.ReturnType, ReturnMarshalling, true) ||
            signature.ParameterTypes.Where((p, i) => !CallbackParameters.ContainsKey(i) && !Slot(p, ParameterMarshalling.GetValueOrDefault(i) ?? (p == PrimitiveType.String ? StringMarshalling : null), false)).Any())
            throw new InvalidDataException("unsupported native import signature or marshalling");
    }

    internal static NativeImportDefinition Read(JsonElement value)
    {
        if (value.ValueKind != JsonValueKind.Object) throw new InvalidDataException("invalid native import");
        var fields = value.EnumerateObject().Select(p => p.Name).ToArray();
        if (fields.Distinct().Count() != fields.Length || fields.Any(p => p is not ("library" or "entry_point" or "calling_convention" or "string_marshalling" or "parameter_marshalling" or "return_marshalling" or "callback_parameters")))
            throw new InvalidDataException("unsupported native import metadata");
        try
        {
            var parameters = new Dictionary<int, string>();
            if (value.TryGetProperty("parameter_marshalling", out var entries))
                foreach (var entry in entries.EnumerateObject())
                {
                    if (!int.TryParse(entry.Name, out var index) || entry.Name != index.ToString(System.Globalization.CultureInfo.InvariantCulture) || !parameters.TryAdd(index, entry.Value.GetString()!))
                        throw new InvalidDataException("invalid native marshalling index");
                }
            var callbacks = new Dictionary<int, NativeCallbackDefinition>();
            if (value.TryGetProperty("callback_parameters", out var callbackEntries))
                foreach (var entry in callbackEntries.EnumerateObject())
                {
                    if (!int.TryParse(entry.Name, out var index) || entry.Name != index.ToString(System.Globalization.CultureInfo.InvariantCulture) ||
                        !callbacks.TryAdd(index, NativeCallbackDefinition.Read(entry.Value)))
                        throw new InvalidDataException("invalid native callback index");
                }
            return new(value.GetProperty("library").GetString()!, value.GetProperty("entry_point").GetString()!,
                value.TryGetProperty("string_marshalling", out var encoding) ? encoding.GetString() : null,
                value.GetProperty("calling_convention").GetString()!, parameters,
                value.TryGetProperty("return_marshalling", out var result) ? result.GetString() : null, callbacks);
        }
        catch (Exception error) when (error is ArgumentException or InvalidOperationException or KeyNotFoundException)
        { throw new InvalidDataException("invalid native import metadata", error); }
    }
}

public sealed partial class MethodDefinition
{
    /// <summary>Gets an explicit native import, or null for managed/runtime methods.</summary>
    public NativeImportDefinition? NativeImport { get; private set; }

    /// <summary>Marks an empty, nongeneric static declaration as a foreign C ABI function.</summary>
    /// <exception cref="InvalidOperationException">The declaration has a body, runtime flags, generic owner, or incompatible method kind.</exception>
    /// <exception cref="InvalidDataException">The signature has unsupported native ABI types.</exception>
    public void SetNativeImport(NativeImportDefinition import)
    {
        ArgumentNullException.ThrowIfNull(import);
        if (AuthoredSignature is null || !IsStatic || GenericArity != 0 || ImplementationAttributes != 0 ||
            (DeclarationAttributes & 0x540) != 0 || Name.StartsWith('.') ||
            Producer?.DeclaringType?.GenericParameterNames.Count > 0 ||
            Body.Instructions.Count != 0 || Body.LocalStorage.Count != 0 || Body.LabelStorage.Count != 0)
            throw new InvalidOperationException("native imports require empty nongeneric static declarations");
        import.Validate(AuthoredSignature);
        NativeImport = import;
    }
}

public sealed partial class MethodBuilder
{
    /// <summary>Marks this empty static declaration as a foreign C ABI function.</summary>
    public void SetNativeImport(NativeImportDefinition import) => Definition.SetNativeImport(import);
}
