using Mono.Cecil;

// Bounded bootstrap path for standard-syntax unions with empty cases. The selected
// reference family must match; application namespaces alone never authorize aliasing.
static class StandardUnionLibrary
{
    public static bool IsCandidate(TypeDefinition type) => type.IsExplicitLayout
        && type.CustomAttributes.Any(a => a.AttributeType.FullName == "System.Runtime.CompilerServices.UnionAttribute"
            && RuntimeSignatures.IsCore(a.AttributeType.Scope));

    public static MethodDefinition[] Roots(TypeDefinition source, TypeDefinition reference)
    {
        if (!ApplicationTypes.IsEmptyCaseUnion(source) || !ApplicationTypes.IsEmptyCaseUnion(reference)
            || source.FullName != reference.FullName)
            throw new InvalidDataException("Unsupported standard union library shape.");
        if (!RavenUnionMetadata.ValidateEmptyCases(source).SequenceEqual(RavenUnionMetadata.ValidateEmptyCases(reference)))
            throw new InvalidDataException("Raven union case metadata does not match reference contract.");
        const string protocolName = "System.Runtime.CompilerServices.IUnion";
        var protocol = source.Module.GetType(protocolName)
            ?? throw new InvalidDataException("Standard union bootstrap requires its source protocol declaration.");
        var contractProtocol = reference.Module.GetType(protocolName)
            ?? throw new InvalidDataException("Missing standard union reference protocol.");
        foreach (var candidate in new[] { protocol, contractProtocol })
            if (!candidate.IsPublic || !candidate.IsInterface || candidate.HasFields || candidate.HasInterfaces
                || candidate.HasGenericParameters || candidate.Methods.Count != 1
                || candidate.Methods[0] is not { Name: "get_Value", IsAbstract: true, HasThis: true, HasParameters: false } getter
                || getter.ReturnType.MetadataType != MetadataType.Object)
                throw new InvalidDataException("Unsupported standard union protocol.");
        var sources = new[] { source, protocol }.Concat(source.NestedTypes).ToArray();
        var contracts = new[] { reference, contractProtocol }.Concat(reference.NestedTypes).ToArray();
        var ownedNames = sources.Select(t => t.FullName).ToHashSet();
        string TypeKey(TypeReference type)
        {
            if (type is ByReferenceType byref) return TypeKey(byref.ElementType) + "&";
            if (type is TypeSpecification || type is GenericParameter)
                throw new InvalidDataException("Unsupported standard union signature.");
            if (type.MetadataType is MetadataType.Void or MetadataType.Boolean or MetadataType.Byte
                or MetadataType.Int32 or MetadataType.String or MetadataType.Object)
                return type.MetadataType.ToString();
            if (ownedNames.Contains(type.FullName)
                && (type.Resolve()?.Module == source.Module || type.Resolve()?.Module == reference.Module))
                return type.FullName;
            if (RuntimeSignatures.IsCore(type.Scope)) return "core:" + type.FullName;
            throw new InvalidDataException("Foreign standard union signature: " + type.FullName);
        }
        string MethodKey(MethodDefinition method) => method.Name + ":" + method.Attributes + ":"
            + TypeKey(method.ReturnType) + "(" + string.Join(',', method.Parameters.Select(p =>
                p.Name + ":" + p.Attributes + ":" + TypeKey(p.ParameterType))) + ")"
            + ":overrides=" + string.Join(',', method.Overrides.Select(o => TypeKey(o.DeclaringType)
                + ":" + o.Name + ":" + TypeKey(o.ReturnType)).Order());
        string PropertyKey(PropertyDefinition property) => property.Name + ":" + TypeKey(property.PropertyType)
            + ":" + property.GetMethod?.Name + ":" + property.SetMethod?.Name;
        foreach (var type in sources)
        {
            var contract = contracts.SingleOrDefault(t => t.FullName == type.FullName);
            if (contract is null || type.Attributes != contract.Attributes || type.HasGenericParameters
                || type.HasEvents || type.Properties.Any(p => p.HasParameters)
                || type.PackingSize != contract.PackingSize || type.ClassSize != contract.ClassSize
                || type.Interfaces.Select(i => TypeKey(i.InterfaceType)).Order().SequenceEqual(
                    contract.Interfaces.Select(i => TypeKey(i.InterfaceType)).Order()) == false
                || !type.Fields.Select(f => (f.Name, f.Attributes, f.Offset, TypeKey(f.FieldType))).SequenceEqual(
                    contract.Fields.Select(f => (f.Name, f.Attributes, f.Offset, TypeKey(f.FieldType))))
                || type.Methods.Any(m => m.HasGenericParameters || m.ExplicitThis
                    || m.CallingConvention != MethodCallingConvention.Default || m.IsPInvokeImpl
                    || m.IsStatic && m.IsConstructor || m.Overrides.Any(o => o.DeclaringType.FullName != protocolName
                        || o.Name != m.Name || o.HasParameters || TypeKey(o.ReturnType) != TypeKey(m.ReturnType))
                    || !type.IsInterface && !m.HasBody)
                || !type.Methods.Select(MethodKey).Order().SequenceEqual(contract.Methods.Select(MethodKey).Order())
                || !type.Properties.Select(PropertyKey).Order().SequenceEqual(contract.Properties.Select(PropertyKey).Order()))
                throw new InvalidDataException("Standard union library does not match reference contract: " + type.FullName
                    + "; source methods=[" + string.Join(';', type.Methods.Select(m => MethodKey(m) + ":overrides=" + m.HasOverrides))
                    + "]; reference methods=[" + string.Join(';', contract?.Methods.Select(MethodKey) ?? []) + "]");
        }
        foreach (var type in sources) ApplicationTypes.BindLibrary(type, type.FullName.Replace('/', '.'));
        foreach (var type in sources) _ = ApplicationTypes.Type(type);
        return sources.Where(t => !t.IsInterface).SelectMany(t => t.Methods).ToArray();
    }
}
