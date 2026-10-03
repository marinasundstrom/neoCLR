namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An owned callable declaration, including type-independent global functions.</summary>
/// <remarks>Signature bytes preserve CLI encodings; general signatures are opaque until a decoder supports them.
/// No body is decoded and no runtime assembly is loaded.</remarks>
public sealed partial class MethodDefinition
{
    private readonly byte[] signature;
    private readonly int[] outParameters = [];
    private readonly uint declaringToken;
    private readonly bool unsupportedGenericParameters;
    private readonly bool unsupportedParameterModes;
    internal MethodDefinition(ModuleDefinition module, AssemblyDefinition.MethodRow row)
    {
        Module = module;
        MetadataToken = row.Token;
        Name = row.Name;
        declarationAttributes = row.Attributes;
        ImplementationAttributes = row.ImplementationAttributes;
        GenericArity = row.Arity;
        unsupportedGenericParameters = row.UnsupportedGenericParameters;
        unsupportedParameterModes = row.UnsupportedParameterModes;
        declaringToken = row.DeclaringToken;
        signature = row.Signature;
        outParameters = row.OutParameters;
        LoadParameterNames(row.ParameterNames);
        nativeSignature = row.NativeSignature?.Materialize(module);
        nativeNamespace = row.NativeNamespace;
    }
    /// <summary>Gets the owning module snapshot.</summary>
    public ModuleDefinition Module { get; internal set; } = null!;
    /// <summary>Gets the owned declaring type, or null for a global function.</summary>
    public TypeDefinition? DeclaringType => Producer is { } producer ? producer.DeclaringType?.Definition : AuthoredSignature is not null ? null : Module.GetTypeDefinition(declaringToken);
    /// <summary>Gets the module-local MethodDef token, or validated native origin token.</summary>
    public uint MetadataToken { get; }
    /// <summary>Gets the declared metadata name.</summary>
    public string Name { get; }
    /// <summary>Gets raw CLI MethodAttributes bits, without target-specific reinterpretation.</summary>
    private readonly ushort declarationAttributes;
    internal ushort DeclarationAttributes => declarationAttributes;
    /// <summary>Gets CLI attributes, including authored accessor and interface implementation flags.</summary>
    public ushort Attributes => Producer?.GetAttributes() ?? declarationAttributes;
    /// <summary>Gets raw CLI MethodImplAttributes bits.</summary>
    public ushort ImplementationAttributes { get; }
    /// <summary>Gets the number of declared method GenericParam rows.</summary>
    public int GenericArity { get; }
    /// <summary>Gets whether the MethodAttributes.Static bit is set.</summary>
    public bool IsStatic => (declarationAttributes & 0x10) != 0;
    /// <summary>Copies a loaded CLI signature blob without resolving its type references.</summary>
    /// <returns>New owned bytes; unsupported encodings remain opaque rather than being simplified.</returns>
    /// <exception cref="NotSupportedException">Native declarations have no CLI signature blob; use TryGetSignature.</exception>
    public byte[] GetSignature() => nativeSignature is not null ? throw new NotSupportedException("native declarations have no CLI signature blob; use TryGetSignature") : AuthoredSignature is null ? (byte[])signature.Clone() : throw new InvalidOperationException("authored signature tokens are assigned when writing; use AuthoredSignature");

    /// <summary>Recognizes the writer's static, nongeneric Int32 parameter/result or no-result signature subset.</summary>
    /// <param name="parameterCount">On success, Int32 parameter count (0–256); otherwise zero.</param>
    /// <param name="returnsValue">On success, true for Int32 and false for CLI void; otherwise false.</param>
    /// <returns>True only for the exact supported encoding; false for other or malformed signatures.</returns>
    /// <remarks>Does not treat CLI void as an inhabited value. Recognition is not body verification or general CLI signature validation.</remarks>
    public bool TryGetStaticInt32Signature(out int parameterCount, out bool returnsValue)
    {
        parameterCount = 0;
        returnsValue = false;
        if (nativeSignature is { } native)
        {
            if (!IsStatic || GenericArity != 0 || native.ReturnType.Primitive is not (PrimitiveType.Int32 or PrimitiveType.Void) || native.ParameterTypes.Any(p => p.Primitive != PrimitiveType.Int32)) return false;
            parameterCount = native.ParameterTypes.Count; returnsValue = native.ReturnType.Primitive == PrimitiveType.Int32; return true;
        }
        if (!IsStatic || GenericArity != 0) return false;
        return TryDecodeStaticInt32Signature(signature, out parameterCount, out returnsValue);
    }
    internal static bool TryDecodeStaticInt32Signature(ReadOnlySpan<byte> signature, out int parameterCount, out bool returnsValue)
    {
        parameterCount = 0;
        returnsValue = false;
        if (!TryDecodeStaticPrimitiveSignature(signature, out var decoded) ||
            decoded!.ReturnType is not (PrimitiveType.Int32 or PrimitiveType.Void) ||
            decoded.ParameterTypes.Any(p => p != PrimitiveType.Int32)) return false;
        parameterCount = decoded.ParameterTypes.Count;
        returnsValue = decoded.ReturnType != PrimitiveType.Void;
        return true;
    }

    /// <summary>Recognizes static nongeneric Int32/Int64/Boolean/String parameters and Int32/Int64/Boolean/String/void results.</summary>
    /// <param name="decoded">An owned immutable signature on success; otherwise null.</param>
    /// <returns>False for unsupported or malformed encodings; does not verify method bodies.</returns>
    public bool TryGetStaticPrimitiveSignature(out PrimitiveMethodSignature? decoded)
    {
        decoded = null;
        if (nativeSignature is { } native)
        {
            if (!IsStatic || GenericArity != 0 || native.ReturnType.Primitive is null || native.ParameterTypes.Any(p => p.Primitive is null)) return false;
            decoded = new PrimitiveMethodSignature(native.ReturnType.Primitive!.Value, native.ParameterTypes.Select(p => p.Primitive!.Value));
            return true;
        }
        return IsStatic && GenericArity == 0 && TryDecodeStaticPrimitiveSignature(signature, out decoded);
    }

    /// <summary>Recognizes static nongeneric primitive and zero-based primitive-vector signatures.</summary>
    /// <param name="decoded">An immutable signature on success; otherwise null.</param>
    /// <returns>False for malformed or unsupported signatures, including nested arrays, nominal types and byrefs.</returns>
    /// <remarks>Void is allowed only as a result. No type resolution, body validation or code loading occurs.</remarks>
    public bool TryGetStaticValueSignature(out MethodSignature? decoded)
    {
        static bool Value(SignatureType type) => type.Primitive is not null || type.ArrayElement?.Primitive is not null;
        decoded = IsStatic && GenericArity == 0 && nativeSignature is { } native && Value(native.ReturnType) && native.ParameterTypes.All(Value) ? native : null;
        if (decoded is not null) return true;
        return IsStatic && GenericArity == 0 && TryDecodeStaticValueSignature(signature, out decoded) && ApplyOutputs(ref decoded);
    }

    /// <summary>Recognizes unconstrained static generic primitive/vector signatures with scoped method parameters.</summary>
    /// <param name="decoded">Immutable signature with preserved native or positional CLI parameter names on success; null otherwise.</param>
    /// <returns>False for nongeneric, constrained, malformed or unsupported declarations.</returns>
    /// <remarks>At most 32 method parameters. No nominal types, declaring-type parameters or nested vectors.</remarks>
    public bool TryGetStaticGenericValueSignature(out MethodSignature? decoded)
    {
        static bool Scalar(SignatureType type) => type.Primitive is not null || type.MethodParameterIndex is not null;
        static bool Value(SignatureType type) => Scalar(type) || type.ArrayElement is { } element && Scalar(element);
        decoded = IsStatic && GenericArity is > 0 and <= 32 && nativeSignature is { } native &&
            Value(native.ReturnType) && native.ParameterTypes.All(Value) ? native : null;
        if (decoded is not null) return true;
        return IsStatic && !unsupportedGenericParameters && GenericArity is > 0 and <= 32 &&
            TryDecodeStaticValueSignature(signature, out decoded, GenericArity) && ApplyOutputs(ref decoded);
    }

    private bool ApplyOutputs(ref MethodSignature? decoded)
    {
        try { decoded = new(decoded!.ReturnType, decoded.ParameterTypes, decoded.GenericParameterNames, outParameters); return true; }
        catch (ArgumentException) { decoded = null; return false; }
    }

    internal MethodSignature DecodeImportedSignature(AssemblyBuilder consumer, AssemblyIdentity core, IAssemblyResolver? resolver = null)
    {
        if (nativeSignature is not null)
        {
            if (!IsStatic && (DeclaringType is null || GenericArity != 0))
                throw new InvalidDataException("unsupported native callable import");
            return new MethodSignature(consumer.ImportNativeSignatureType(nativeSignature.ReturnType, core, resolver),
                nativeSignature.ParameterTypes.Select(type => consumer.ImportNativeSignatureType(type, core, resolver)), nativeSignature.GenericParameterNames, nativeSignature.OutParameters);
        }
        if (unsupportedParameterModes || unsupportedGenericParameters || GenericArity is < 0 or > 32)
            throw new InvalidDataException("unsupported imported method declaration");
        try
        {
            var decoded = new ImportedSignatureDecoder(signature, Module, consumer, core, GenericArity, DeclaringType?.GenericArity ?? 0, !IsStatic).Read();
            return new(decoded.ReturnType, decoded.ParameterTypes, decoded.GenericParameterNames, outParameters);
        }
        catch (ArgumentException error) { throw new InvalidDataException("invalid imported signature", error); }
    }

    internal static bool TryDecodeStaticPrimitiveSignature(ReadOnlySpan<byte> signature, out PrimitiveMethodSignature? decoded)
    {
        decoded = null;
        if (!TryDecodeStaticValueSignature(signature, out var value) || value!.ReturnType.Primitive is not { } result ||
            value.ParameterTypes.Any(p => p.Primitive is null)) return false;
        decoded = new(result, value.ParameterTypes.Select(p => p.Primitive!.Value));
        return true;
    }

    internal static bool TryDecodeStaticValueSignature(ReadOnlySpan<byte> signature, out MethodSignature? decoded, int genericArity = 0)
    {
        decoded = null;
        if (signature.Length < 3 || signature[0] != (genericArity == 0 ? 0 : 0x10)) return false;
        int position = 1;
        if (genericArity > 0 && signature[position++] != genericArity) return false;
        int count = signature[position++];
        if ((count & 0x80) != 0)
        {
            if ((count & 0xc0) != 0x80 || position >= signature.Length) return false;
            count = ((count & 0x3f) << 8) | signature[position++];
            if (count < 128) return false;
        }
        if (count > 256 || !ReadType(signature, ref position, true, out var result, genericArity)) return false;
        var parameters = new SignatureType[count];
        for (int i = 0; i < count; i++)
        {
            if (!ReadType(signature, ref position, false, out var parameter, genericArity, allowByReference: true)) return false;
            parameters[i] = parameter!;
        }
        if (position != signature.Length) return false;
        decoded = new(result!, parameters, Enumerable.Range(0, genericArity).Select(i => "T" + i));
        return true;
    }

    private static bool ReadType(ReadOnlySpan<byte> signature, ref int position, bool allowVoid, out SignatureType? type, int genericArity, bool allowByReference = false)
    {
        type = null;
        if (position >= signature.Length) return false;
        var code = signature[position++];
        if (code == 0x10 && allowByReference)
        {
            if (!ReadType(signature, ref position, false, out var target, genericArity)) return false;
            type = SignatureType.ByReference(target!);
            return true;
        }
        var vector = code == 0x1d;
        if (vector)
        {
            if (position >= signature.Length) return false;
            code = signature[position++];
        }
        if (code == 0x1e)
        {
            if (position >= signature.Length || signature[position] >= genericArity) return false;
            var parameter = SignatureType.MethodParameter(signature[position++]);
            type = vector ? SignatureType.ArrayOf(parameter) : parameter;
            return true;
        }
        PrimitiveType? primitive = code switch
        {
            0x01 when allowVoid && !vector => PrimitiveType.Void,
            0x02 => PrimitiveType.Boolean,
            0x08 => PrimitiveType.Int32,
            0x05 => PrimitiveType.Byte, 0x0a => PrimitiveType.Int64,
            0x0e => PrimitiveType.String,
            _ => null
        };
        if (primitive is null) return false;
        type = vector ? SignatureType.ArrayOf(primitive.Value) : primitive.Value;
        return true;
    }
}
