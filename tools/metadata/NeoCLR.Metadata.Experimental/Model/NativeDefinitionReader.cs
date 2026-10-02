namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyDefinition
{
    /// <summary>Reads authoritative native namespace-function and bounded class declarations directly from PE/#Neo.</summary>
    /// <param name="image">Complete API-produced schema-1/2 runtime container.</param>
    /// <returns>An owned immutable declaration snapshot, without generating or importing a CLI projection.</returns>
    /// <exception cref="InvalidDataException">Invalid container or unsupported declarations, including value/interface/generic types, unsupported field types, properties and signatures beyond primitives/nominal classes and their vectors.</exception>
    /// <remarks>This first materialization profile admits nongeneric functions/methods with primitive, nominal class or vector signatures, top-level classes with primitive, nominal class or vector fields and exact dependency identities.
    /// Bodies remain opaque. Write copies the original image; editing remains pending. Supported method definitions can be imported for native calls.
    /// Mvid is empty because the native manifest declares none. Tokens retain module-local native origin identifiers.</remarks>
    public static AssemblyDefinition ReadNativeAssembly(ReadOnlySpan<byte> image)
    {
        if (image.Length > MetadataArtifactReader.MaxImageSize) throw new InvalidDataException("image exceeds limit");
        var owned = image.ToArray();
        var native = NativeAssemblyDefinition.ReadAssembly(RuntimeAssemblyContainer.Read(owned));
        return native.MaterializeDeclarations(owned);
    }

    internal static AssemblyDefinition NativeDeclarations(AssemblyIdentity identity, TypeRow[] types, FieldRow[] fields, MethodRow[] methods,
        ReferenceRow[] references, TypeReferenceRow[] typeReferences, byte[] image, uint entryPoint)
        => new(identity, identity.Name + ".dll", Guid.Empty, types, fields, [], methods, [], references, typeReferences, null, image, entryPoint) { IsNative = true };
}

public sealed partial class NativeAssemblyDefinition
{
    internal AssemblyDefinition MaterializeDeclarations(byte[] image)
    {
        static bool SupportedScalar(SignatureType type) => type.Primitive is not null ||
            type.ClassType is { IsStatic: false, IsInterface: false, IsValueType: false } ||
            type.ImportedType is { IsValueType: false, GenericArity: 0, DeclaringType: null };
        static bool Supported(SignatureType type) => SupportedScalar(type) || type.ArrayElement is { } element && SupportedScalar(element);
        // Fail closed rather than returning a partial assembly with silently missing types.
        if (properties.Length != 0 || types.Any(t => t.IsInterface || t.IsValueType || t.DeclaringType >= 0 ||
            t.GenericNames.Length != 0 || t.Fields.Any(f => !Supported(f.Signature!)) || t.BaseInterfaces.Length != 0 || t.Constraints.Length != 0 || t.SpecialConstraints.Count != 0) ||
            methods.Any(m => (m.Owner < 0 && m.Visibility is not (MethodVisibility.Public or MethodVisibility.Internal)) ||
            m.Signature.GenericParameterNames.Count != 0 || !Supported(m.Signature.ReturnType) ||
            m.Signature.ParameterTypes.Any(p => !Supported(p))))
            throw new InvalidDataException("native definition materialization requires nongeneric primitive/nominal-class/vector signatures and top-level classes with primitive, nominal class or vector fields");
        var nominalTokens = types.Select((type, index) => (type, token: 0x02000002u + (uint)index))
            .ToDictionary(item => (item.type.Namespace, item.type.Name), item => item.token);
        var referenceTokens = References.Select((identity, index) => (identity, token: 0x23000001u + (uint)index)).ToDictionary(item => item.identity, item => item.token);
        var externalRows = new List<AssemblyDefinition.TypeReferenceRow>();
        var externalTokens = new Dictionary<(AssemblyIdentity, string, string), uint>();
        AssemblyDefinition.NativeSignatureTypeRow Copy(SignatureType type)
        {
            if (type.ArrayElement is { } element) return new(null, 0, Copy(element));
            if (type.Primitive is { } primitive) return new(primitive, 0);
            if (type.ClassType is { } local) return new(null, nominalTokens[(local.Namespace, local.Name)]);
            var imported = type.ImportedType!;
            var key = (imported.AssemblyIdentity, imported.Namespace, imported.Name);
            if (!externalTokens.TryGetValue(key, out var token))
            {
                if (!referenceTokens.TryGetValue(imported.AssemblyIdentity, out var scope)) throw new InvalidDataException("native signature dependency is not declared");
                token = 0x01000001u + (uint)externalRows.Count;
                externalTokens.Add(key, token);
                externalRows.Add(new(token, imported.Namespace, imported.Name, scope));
            }
            return new(null, token);
        }
        var typeRows = types.Select((type, index) => new AssemblyDefinition.TypeRow(
            0x02000002u + (uint)index, type.Namespace, type.Name, 0, 0,
            (uint)(0x100000 | (type.IsStatic ? 0x180 : 0) | (type.Visibility == TypeVisibility.Public ? 1 : 0)), !type.IsStatic, false, null)).ToArray();
        var fieldRows = new List<AssemblyDefinition.FieldRow>();
        for (int owner = 0; owner < types.Length; owner++)
            foreach (var field in types[owner].Fields)
                fieldRows.Add(new(0x04000001u + (uint)fieldRows.Count, 0x02000002u + (uint)owner, field.Name,
                    (ushort)((field.IsReadOnly ? 0x20 : 0) | (field.Visibility == FieldVisibility.Public ? 6 : field.Visibility == FieldVisibility.Internal ? 3 : 1)),
                    [], Copy(field.Signature!)));
        var rows = methods.Select((method, index) => new AssemblyDefinition.MethodRow(
            0x06000001u + (uint)index, method.Owner < 0 ? 0 : 0x02000002u + (uint)method.Owner, method.Name,
            (ushort)((method.Instance ? 0 : 0x10) | (method.Instance && method.Name == ".ctor" ? 0x1800 : 0) | (method.Visibility == MethodVisibility.Public ? 6 : method.Visibility == MethodVisibility.Internal ? 3 : 1)), 0, 0, [], false, [],
            new AssemblyDefinition.NativeMethodSignatureRow(Copy(method.Signature.ReturnType), method.Signature.ParameterTypes.Select(Copy).ToArray()), method.Namespace)).ToArray();
        var references = References.Select((identity, index) => new AssemblyDefinition.ReferenceRow(0x23000001u + (uint)index, identity)).ToArray();
        return AssemblyDefinition.NativeDeclarations(Identity, typeRows, fieldRows.ToArray(), rows, references, externalRows.ToArray(), image, entryPointToken);
    }
}

public sealed partial class MethodDefinition
{
    private readonly MethodSignature? nativeSignature;
    private readonly string? nativeNamespace;

    /// <summary>Reads the supported logical signature without serializing a CLI blob or resolving dependencies.</summary>
    /// <param name="decoded">The immutable signature on success; otherwise null.</param>
    /// <returns>True for authored signatures, native primitive/nominal-class/vector functions/methods, and the existing bounded static CLI value/generic profiles.</returns>
    /// <remarks>False does not mean an absent signature: other loaded CLI signatures require contextual decoding.
    /// Native signatures retain native no-result semantics. This operation never materializes a method body.</remarks>
    public bool TryGetSignature(out MethodSignature? decoded)
    {
        decoded = nativeSignature ?? AuthoredSignature;
        if (decoded is not null) return true;
        return GenericArity == 0 ? TryGetStaticValueSignature(out decoded) : TryGetStaticGenericValueSignature(out decoded);
    }
}
