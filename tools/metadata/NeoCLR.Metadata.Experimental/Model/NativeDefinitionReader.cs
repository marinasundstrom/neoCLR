namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyDefinition
{
    /// <summary>Reads authoritative native namespace-function declarations directly from PE/#Neo.</summary>
    /// <param name="image">Complete API-produced schema-1/2 runtime container.</param>
    /// <returns>An owned immutable declaration snapshot, without generating or importing a CLI projection.</returns>
    /// <exception cref="InvalidDataException">Invalid container or unsupported declarations, including nominal types and nonprimitive signatures.</exception>
    /// <remarks>This first materialization profile admits primitive nongeneric namespace functions and exact dependency identities.
    /// Bodies remain opaque. Write copies the original image; editing remains pending. Primitive function definitions can be imported for native calls.
    /// Mvid is empty because the native manifest declares none. Tokens retain module-local native origin identifiers.</remarks>
    public static AssemblyDefinition ReadNativeAssembly(ReadOnlySpan<byte> image)
    {
        if (image.Length > MetadataArtifactReader.MaxImageSize) throw new InvalidDataException("image exceeds limit");
        var owned = image.ToArray();
        var native = NativeAssemblyDefinition.ReadAssembly(RuntimeAssemblyContainer.Read(owned));
        return native.MaterializeFunctions(owned);
    }

    internal static AssemblyDefinition NativeFunctions(AssemblyIdentity identity, MethodRow[] methods,
        ReferenceRow[] references, byte[] image, uint entryPoint)
        => new(identity, identity.Name + ".dll", Guid.Empty, [], [], [], methods, [], references, [], null, image, entryPoint) { IsNative = true };
}

public sealed partial class NativeAssemblyDefinition
{
    internal AssemblyDefinition MaterializeFunctions(byte[] image)
    {
        // Fail closed rather than returning a partial assembly with silently missing types.
        if (types.Length != 0 || methods.Any(m => m.Owner >= 0 || m.Instance || m.Visibility is not (MethodVisibility.Public or MethodVisibility.Internal) ||
            m.Signature.GenericParameterNames.Count != 0 || m.Signature.ReturnType.Primitive is null ||
            m.Signature.ParameterTypes.Any(p => p.Primitive is null)))
            throw new InvalidDataException("native definition materialization currently requires primitive nongeneric namespace functions");
        var rows = methods.Select((method, index) => new AssemblyDefinition.MethodRow(
            0x06000001u + (uint)index, 0, method.Name,
            (ushort)(0x10 | (method.Visibility == MethodVisibility.Public ? 6 : 3)), 0, 0, [], false, [],
            new MethodSignature(method.Signature.ReturnType, method.Signature.ParameterTypes), method.Namespace)).ToArray();
        var references = References.Select((identity, index) => new AssemblyDefinition.ReferenceRow(0x23000001u + (uint)index, identity)).ToArray();
        return AssemblyDefinition.NativeFunctions(Identity, rows, references, image, entryPointToken);
    }
}

public sealed partial class MethodDefinition
{
    private readonly MethodSignature? nativeSignature;
    private readonly string? nativeNamespace;

    /// <summary>Reads the supported logical signature without serializing a CLI blob or resolving dependencies.</summary>
    /// <param name="decoded">The immutable signature on success; otherwise null.</param>
    /// <returns>True for authored signatures, native primitive functions, and the existing bounded static CLI value/generic profiles.</returns>
    /// <remarks>False does not mean an absent signature: other loaded CLI signatures require contextual decoding.
    /// Native signatures retain native no-result semantics. This operation never materializes a method body.</remarks>
    public bool TryGetSignature(out MethodSignature? decoded)
    {
        decoded = nativeSignature ?? AuthoredSignature;
        if (decoded is not null) return true;
        return GenericArity == 0 ? TryGetStaticValueSignature(out decoded) : TryGetStaticGenericValueSignature(out decoded);
    }
}
