namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyDefinition
{
    /// <summary>Reads authoritative native namespace-function and bounded class/value/interface declarations directly from PE/#Neo or standalone NEOX.</summary>
    /// <param name="image">Complete schema-1/2/3 PE runtime container or schema-2/3 standalone NEOX with an assembly manifest.</param>
    /// <returns>An owned immutable declaration snapshot, without generating or importing a CLI projection.</returns>
    /// <exception cref="InvalidDataException">Invalid container or unsupported declarations, including constrained and generic static owners and generic instance methods, unsupported field types and signatures beyond the bounded native profile.</exception>
    /// <remarks>This materialization profile admits nongeneric methods and static generic methods/functions with supported nongeneric interface bounds and scoped method/type parameters, primitive, nominal value/reference, local or external generic construction, vector or bounded function signatures, interface-scoped Self member signatures, and unconstrained classes, values and interfaces (including supported nested declarations under nongeneric owners) with primitive, nominal value/reference or vector fields/properties and exact dependency identities.
    /// Public value-type ToString overrides retain their inherited CLI slot flags and native call name.
    /// Writable ref/out parameters retain their element signature and output indices; readonly modes and byref constructors remain unsupported.
    /// Bodies remain opaque. Write copies the original image; editing remains pending. Supported method definitions can be imported for native calls.
    /// Mvid is empty because the native manifest declares none. Tokens retain module-local native origin identifiers.</remarks>
    public static AssemblyDefinition ReadNativeAssembly(ReadOnlySpan<byte> image)
    {
        if (image.Length > RuntimeAssemblyContainer.MaxLibraryImageSize) throw new InvalidDataException("image exceeds limit");
        var owned = image.ToArray();
        var native = NativeAssemblyDefinition.ReadLibraryAssembly(owned.AsSpan().StartsWith("NEOX"u8)
            ? NativeModuleContainer.Read(owned) : RuntimeAssemblyContainer.Read(owned));
        return native.MaterializeDeclarations(owned);
    }

    internal static AssemblyDefinition NativeDeclarations(AssemblyIdentity identity, TypeRow[] types, FieldRow[] fields, MethodRow[] methods, PropertyRow[] properties,
        ReferenceRow[] references, TypeReferenceRow[] typeReferences, byte[] image, uint entryPoint)
        => new(identity, identity.Name + ".dll", Guid.Empty, types, fields, properties, methods, [], references, typeReferences, null, image, entryPoint) { IsNative = true };
}

public sealed partial class NativeAssemblyDefinition
{
    internal AssemblyDefinition MaterializeDeclarations(byte[] image)
    {
        static bool SupportedArgument(SignatureType type) => type.MethodParameterIndex is not null ||
            (type.ArrayElement is { } element ? SupportedArgument(element) : SupportedScalar(type));
        static bool SupportedScalar(SignatureType type) => type.FunctionSignature is { } function && SupportedArgument(function.ReturnType) && function.ParameterTypes.All(SupportedArgument) || type.IsSelf || type.GenericInstance is { } instance && instance.TypeArguments.All(SupportedArgument) || type.Primitive is not null || type.TypeParameterIndex is not null ||
            type.ClassType is { IsStatic: false } ||
            type.ImportedType is { } imported &&
            imported.GenericArity == imported.TypeArguments.Count && imported.TypeArguments.All(SupportedArgument);
        static bool SupportedMethod(SignatureType type) => type.MethodParameterIndex is not null ||
            type.ArrayElement is { } element && SupportedMethod(element) || Supported(type);
        static bool Supported(SignatureType type) => SupportedScalar(type) || type.ArrayElement is { } element && Supported(element);
        // Fail closed rather than returning a partial assembly with silently missing types.
        if (properties.Any(p => p.Parameters.Any(parameter => !Supported(parameter)) || !Supported(p.Type)) || types.Any(t => (t.GenericNames.Length != 0 && t.IsStatic) || t.Fields.Any(f => !Supported(f.Signature!)) || t.InterfaceSignatures.Any(i => !Supported(i)) || t.Constraints.Length != 0 || t.SpecialConstraints.Count != 0) ||
            methods.Any(m => (m.Owner < 0 && m.Visibility is not (MethodVisibility.Public or MethodVisibility.Internal)) ||
            (m.Name == ".ctor" && m.Signature.ParameterTypes.Any(p => p.ByReferenceElement is not null)) ||
            (m.Instance && m.Signature.GenericParameterNames.Count != 0) || !SupportedMethod(m.Signature.ReturnType) ||
            m.Signature.ParameterTypes.Any(p => !SupportedMethod(p.ByReferenceElement ?? p))))
            throw new InvalidDataException("native definition materialization requires bounded static generic or nongeneric signatures and unconstrained class/value/interface declarations");
        var nominalTokens = types.Select((type, index) => (type, token: 0x02000002u + (uint)index))
            .ToDictionary(item => (item.type.DeclaringType < 0 ? 0u : 0x02000002u + (uint)item.type.DeclaringType, item.type.Namespace, item.type.Name), item => item.token);
        uint LocalToken(TypeBuilder type) => nominalTokens[(type.Definition.DeclaringType is { } parent ? LocalToken(parent.Producer!) : 0u, type.Namespace, type.Name)];
        var referenceTokens = References.Select((identity, index) => (identity, token: 0x23000001u + (uint)index)).ToDictionary(item => item.identity, item => item.token);
        var externalRows = new List<AssemblyDefinition.TypeReferenceRow>();
        var externalTokens = new Dictionary<(AssemblyIdentity, string, string, uint), uint>();
        AssemblyDefinition.NativeSignatureTypeRow Copy(SignatureType type)
        {
            if (type.FunctionSignature is { } function) return new(null, 0, Function: new(Copy(function.ReturnType), function.ParameterTypes.Select(Copy).ToArray()));
            if (type.ByReferenceElement is { } target) return new(null, 0, Copy(target), IsByReference: true);
            if (type.IsSelf) return new(null, 0, IsSelf: true);
            if (type.GenericInstance is { } instance) return new(null, LocalToken(instance.Definition), Arguments: instance.TypeArguments.Select(Copy).ToArray());
            if (type.TypeParameterIndex is { } typeParameter) return new(null, 0, TypeParameter: typeParameter);
            if (type.MethodParameterIndex is { } parameter) return new(null, 0, MethodParameter: parameter);
            if (type.ArrayElement is { } element) return new(null, 0, Copy(element));
            if (type.Primitive is { } primitive) return new(primitive, 0);
            if (type.ClassType is { } local) return new(null, LocalToken(local));
            var imported = type.ImportedType!;
            return new(null, ExternalToken(imported), Arguments: imported.TypeArguments.Count == 0 ? null : imported.TypeArguments.Select(Copy).ToArray());
        }
        uint ExternalToken(ImportedTypeReference imported)
        {
            var parent = imported.DeclaringType is { } declaring ? ExternalToken(declaring) : 0u;
            var key = (imported.AssemblyIdentity, imported.Namespace, imported.Name, parent);
            if (externalTokens.TryGetValue(key, out var token)) return token;
            if (!referenceTokens.TryGetValue(imported.AssemblyIdentity, out var scope)) throw new InvalidDataException("native signature dependency is not declared");
            token = 0x01000001u + (uint)externalRows.Count;
            externalTokens.Add(key, token);
            externalRows.Add(new(token, imported.Namespace, imported.Name, parent == 0 ? scope : parent));
            return token;
        }
        var typeRows = types.Select((type, index) => new AssemblyDefinition.TypeRow(
            0x02000002u + (uint)index, type.Namespace, type.Name, type.GenericNames.Length, type.DeclaringType < 0 ? 0u : 0x02000002u + (uint)type.DeclaringType,
            (uint)(0x100000 | (type.IsInterface ? 0xa0 : type.IsStatic ? 0x180 : type.IsClosedHierarchy ? 0x80 : type.EnumMembers is not null ? 0x100 : type.IsValueType ? 0x108 : 0) | (type.DeclaringType < 0 ? type.Visibility == TypeVisibility.Public ? 1 : 0 : type.Visibility == TypeVisibility.Public ? 2 : 5)), !type.IsStatic, type.IsValueType, null, type.InterfaceSignatures.Select(Copy).ToArray(), type.GenericNames, IsEnum: type.EnumMembers is not null, NativePrimitive: type.NativePrimitive, NativeGrapheme: type.NativeGrapheme, BaseTypeToken: type.BaseIndex < 0 ? 0u : 0x02000002u + (uint)type.BaseIndex, IsClosedHierarchy: type.IsClosedHierarchy, IsFlagsEnum: type.IsFlagsEnum)).ToArray();
        var fieldRows = new List<AssemblyDefinition.FieldRow>();
        for (int owner = 0; owner < types.Length; owner++)
        {
            foreach (var field in types[owner].Fields)
                fieldRows.Add(new(0x04000001u + (uint)fieldRows.Count, 0x02000002u + (uint)owner, field.Name,
                    (ushort)((types[owner].EnumMembers is not null ? 0x600 : 0) | (field.IsReadOnly ? 0x20 : 0) | (field.Visibility == FieldVisibility.Public ? 6 : field.Visibility == FieldVisibility.Internal ? 3 : 1)),
                    [], Copy(field.Signature!)));
            if (types[owner].EnumMembers is { } members)
                foreach (var member in members)
                    fieldRows.Add(new(0x04000001u + (uint)fieldRows.Count, 0x02000002u + (uint)owner, member.Name,
                        0x8056, [], new(null, 0x02000002u + (uint)owner), member.Value));
        }
        var accessors = properties.SelectMany(p => new[] { p.Getter, p.Setter }).Where(index => index >= 0).ToHashSet();
        var rows = methods.Select((method, index) => new AssemblyDefinition.MethodRow(
            0x06000001u + (uint)index, method.Owner < 0 ? 0 : 0x02000002u + (uint)method.Owner, method.Name,
            (ushort)((method.ExplicitInterfaces.Length != 0 ? 0x160 : 0) | (method.Override ? 0xc0 : 0) | (method.Owner >= 0 && types[method.Owner].IsInterface ? 0x5c0 : 0) | (accessors.Contains(index) ? 0x800 : 0) | (method.Instance ? 0 : 0x10) | (method.Instance && method.Name == ".ctor" ? 0x1800 : 0) | (method.Visibility == MethodVisibility.Public ? 6 : method.Visibility == MethodVisibility.Internal ? 3 : method.Visibility == MethodVisibility.Protected ? 4 : 1)), method.ImplementationAttributes, method.Signature.GenericParameterNames.Count, [], false, [],
            new AssemblyDefinition.NativeMethodSignatureRow(Copy(method.Signature.ReturnType), method.Signature.ParameterTypes.Select(Copy).ToArray(), method.Signature.GenericParameterNames.ToArray(), method.Signature.OutParameters.ToArray()), method.Namespace, ParameterNames: method.ParameterNames, ParameterArrayIndex: method.ParameterArrayIndex, InterfaceConstraints: method.InterfaceConstraints.Select(c => new AssemblyDefinition.MethodConstraintRow(c.Parameter, Copy(c.Type).TypeToken)).ToArray())).ToArray();
        var propertyRows = properties.Select((property, index) => new AssemblyDefinition.PropertyRow(
            0x17000001u + (uint)index, 0x02000002u + (uint)property.Owner, property.Name, 0, [],
            property.Getter < 0 ? 0 : 0x06000001u + (uint)property.Getter,
            property.Setter < 0 ? 0 : 0x06000001u + (uint)property.Setter, [], Copy(property.Type), property.Parameters.Select(Copy).ToArray())).ToArray();
        var references = References.Select((identity, index) => new AssemblyDefinition.ReferenceRow(0x23000001u + (uint)index, identity)).ToArray();
        var attributes = types.Select(type => type.Attributes.Select(a => (Owner: Copy(a.Owner), a.Arguments)).ToArray()).ToArray();
        var explicitRows = methods.Select(m => m.ExplicitInterfaces.Select(e => (Owner: Copy(e.Owner), e.Name)).ToArray()).ToArray();
        var result = AssemblyDefinition.NativeDeclarations(Identity, typeRows, fieldRows.ToArray(), rows, propertyRows, references, externalRows.ToArray(), image, entryPointToken);
        for (int i = 0; i < types.Length; i++)
            result.MainModule.GetTypeDefinition(0x02000002u + (uint)i)!.SetLoadedAttributes(attributes[i].Select(a =>
                new CustomAttributeDefinition(a.Owner.Materialize(result.MainModule).ReferencedType!, a.Arguments)));
        for (int i = 0; i < methods.Length; i++)
            result.MainModule.GetMethodDefinition(0x06000001u + (uint)i)!.SetLoadedExplicitInterfaces(explicitRows[i].Select(mapping =>
            {
                var owner = mapping.Owner.Materialize(result.MainModule);
                var relationship = owner.ReferencedGenericInstance is { } constructed
                    ? new InterfaceImplementation(constructed.Definition, constructed.TypeArguments)
                    : new InterfaceImplementation(owner.ReferencedType!);
                return new ExplicitInterfaceImplementation(relationship, mapping.Name);
            }));
        return result;
    }
}

public sealed partial class MethodDefinition
{
    private readonly MethodSignature? nativeSignature;
    private readonly string? nativeNamespace;

    /// <summary>Reads the supported logical signature without serializing a CLI blob or resolving dependencies.</summary>
    /// <param name="decoded">The immutable signature on success; otherwise null.</param>
    /// <returns>True for authored signatures, native bounded generic or nongeneric functions/methods, and the existing bounded static CLI value/generic profiles.</returns>
    /// <remarks>False does not mean an absent signature: other loaded CLI signatures require contextual decoding.
    /// Native signatures retain native no-result semantics. This operation never materializes a method body.</remarks>
    public bool TryGetSignature(out MethodSignature? decoded)
    {
        if (unsupportedParameterModes) { decoded = null; return false; }
        decoded = nativeSignature ?? AuthoredSignature;
        if (decoded is not null) return true;
        return GenericArity == 0 ? TryGetStaticValueSignature(out decoded) : TryGetStaticGenericValueSignature(out decoded);
    }
}
