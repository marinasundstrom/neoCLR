namespace NeoCLR.Metadata.Experimental.Model;

/// <summary>An owned physical MethodDef declaration, including type-independent global functions.</summary>
/// <remarks>Signature bytes preserve CLI encodings; general signatures are opaque until a decoder supports them.
/// No body is decoded and no runtime assembly is loaded.</remarks>
public sealed class MethodDefinition
{
    private readonly byte[] signature;
    private readonly uint declaringToken;
    internal MethodDefinition(ModuleDefinition module, AssemblyDefinition.MethodRow row)
    {
        Module = module;
        MetadataToken = row.Token;
        Name = row.Name;
        Attributes = row.Attributes;
        ImplementationAttributes = row.ImplementationAttributes;
        GenericArity = row.Arity;
        declaringToken = row.DeclaringToken;
        signature = row.Signature;
    }
    /// <summary>Gets the owning module snapshot.</summary>
    public ModuleDefinition Module { get; }
    /// <summary>Gets the owned declaring type, or null for a global function.</summary>
    public TypeDefinition? DeclaringType => Module.GetTypeDefinition(declaringToken);
    /// <summary>Gets the physical MethodDef token, meaningful only within this module.</summary>
    public uint MetadataToken { get; }
    /// <summary>Gets the declared metadata name.</summary>
    public string Name { get; }
    /// <summary>Gets raw CLI MethodAttributes bits, without target-specific reinterpretation.</summary>
    public ushort Attributes { get; }
    /// <summary>Gets raw CLI MethodImplAttributes bits.</summary>
    public ushort ImplementationAttributes { get; }
    /// <summary>Gets the number of declared method GenericParam rows.</summary>
    public int GenericArity { get; }
    /// <summary>Gets whether the MethodAttributes.Static bit is set.</summary>
    public bool IsStatic => (Attributes & 0x10) != 0;
    /// <summary>Copies the CLI signature blob without resolving its type references.</summary>
    /// <returns>New owned bytes; unsupported encodings remain opaque rather than being simplified.</returns>
    public byte[] GetSignature() => (byte[])signature.Clone();

    /// <summary>Recognizes the writer's static, nongeneric Int32 parameter/result or no-result signature subset.</summary>
    /// <param name="parameterCount">On success, Int32 parameter count (0–256); otherwise zero.</param>
    /// <param name="returnsValue">On success, true for Int32 and false for CLI void; otherwise false.</param>
    /// <returns>True only for the exact supported encoding; false for other or malformed signatures.</returns>
    /// <remarks>Does not treat CLI void as an inhabited value. Recognition is not body verification or general CLI signature validation.</remarks>
    public bool TryGetStaticInt32Signature(out int parameterCount, out bool returnsValue)
    {
        parameterCount = 0;
        returnsValue = false;
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

    /// <summary>Recognizes static nongeneric Int32/Int64/Boolean parameters and Int32/Int64/Boolean/void results.</summary>
    /// <param name="decoded">An owned immutable signature on success; otherwise null.</param>
    /// <returns>False for unsupported or malformed encodings; does not verify method bodies.</returns>
    public bool TryGetStaticPrimitiveSignature(out PrimitiveMethodSignature? decoded)
    {
        decoded = null;
        return IsStatic && GenericArity == 0 && TryDecodeStaticPrimitiveSignature(signature, out decoded);
    }

    internal static bool TryDecodeStaticPrimitiveSignature(ReadOnlySpan<byte> signature, out PrimitiveMethodSignature? decoded)
    {
        decoded = null;
        if (signature.Length < 3 || signature[0] != 0) return false;
        int position = 1;
        int count = signature[position++];
        if ((count & 0x80) != 0)
        {
            if ((count & 0xc0) != 0x80 || position >= signature.Length) return false;
            count = ((count & 0x3f) << 8) | signature[position++];
            if (count < 128) return false;
        }
        if (count > 256 || position >= signature.Length) return false;
        byte result = signature[position++];
        if (result is not (0x01 or 0x02 or 0x08 or 0x0a) || signature.Length - position != count) return false;
        var parameters = new PrimitiveType[count];
        for (int i = 0; i < count; i++)
        {
            byte type = signature[position++];
            if (type is not (0x02 or 0x08 or 0x0a)) return false;
            parameters[i] = type == 0x02 ? PrimitiveType.Boolean : type == 0x0a ? PrimitiveType.Int64 : PrimitiveType.Int32;
        }
        decoded = new(result == 0x01 ? PrimitiveType.Void : result == 0x02 ? PrimitiveType.Boolean : result == 0x0a ? PrimitiveType.Int64 : PrimitiveType.Int32, parameters);
        return true;
    }
}
