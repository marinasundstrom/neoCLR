using Mono.Cecil;
using Mono.Cecil.Cil;

// Project a bounded Raven union family into a CLI compiler reference. Update existing
// definitions in place so other core signatures retain the same type identities.
// Stub bodies deliberately throw; native execution always imports source bodies.
static class StandardUnionReference
{
    public static void WriteReference(string implementation, string corePath, string owner, string output)
    {
        if (new[] { implementation, corePath }.Any(path => Path.GetFullPath(path) == Path.GetFullPath(output)))
            throw new InvalidDataException("Union projection output must differ from its inputs.");
        using var source = AssemblyDefinition.ReadAssembly(implementation);
        using var core = AssemblyDefinition.ReadAssembly(corePath);
        var module = core.MainModule;
        var originals = module.AssemblyReferences.ToHashSet();
        var root = source.MainModule.GetType(owner) ?? throw new InvalidDataException("Missing source union.");
        if (!ApplicationTypes.IsEmptyCaseUnion(root))
            throw new InvalidDataException("Only standard empty-case unions can be projected.");
        RavenUnionMetadata.ValidateEmptyCases(root);
        var supportNames = new[] { StandardUnionLibrary.ProtocolName, RavenUnionMetadata.CaseAttribute };
        var selected = new[] { root }.Concat(root.NestedTypes).Concat(supportNames
            .Where(name => module.GetType(name) is null).Select(name => source.MainModule.GetType(name)
                ?? throw new InvalidDataException("Missing union reference support type: " + name))).ToArray();
        if (selected.Any(type => type.HasGenericParameters || type.HasEvents
            || type.Properties.Any(p => p.HasParameters)
            || type.Methods.Any(m => m.HasGenericParameters || m.IsPInvokeImpl || m.ExplicitThis
                || m.CallingConvention != MethodCallingConvention.Default
                || m.Parameters.Any(p => p.HasConstant || p.HasMarshalInfo))))
            throw new InvalidDataException("Unsupported union reference member metadata.");
        var types = new Dictionary<string, TypeDefinition>();
        foreach (var type in selected)
        {
            var clone = module.GetType(type.FullName);
            if (clone is not null)
            {
                if (type == root && (!clone.IsValueType || clone.HasGenericParameters
                    || !clone.NestedTypes.Select(t => t.Name).Order().SequenceEqual(root.NestedTypes.Select(t => t.Name).Order())))
                    throw new InvalidDataException("Existing union reference case identities do not match.");
                clone.Attributes = type.Attributes;
                clone.Fields.Clear();
                clone.Methods.Clear();
                clone.Properties.Clear();
                clone.Interfaces.Clear();
                clone.CustomAttributes.Clear();
            }
            else
            {
                clone = new TypeDefinition(type.Namespace, type.Name, type.Attributes);
                if (type.DeclaringType is null) module.Types.Add(clone);
                else types[type.DeclaringType.FullName].NestedTypes.Add(clone);
            }
            clone.PackingSize = type.PackingSize;
            clone.ClassSize = type.ClassSize;
            types.Add(type.FullName, clone);
        }
        TypeReference Map(TypeReference type)
        {
            if (type is ByReferenceType byref) return new ByReferenceType(Map(byref.ElementType));
            if (types.TryGetValue(type.FullName, out var owned)) return owned;
            // Import primitive encodings without adding a host framework dependency.
            if (type.MetadataType is MetadataType.Void or MetadataType.Boolean or MetadataType.Byte
                or MetadataType.Int32 or MetadataType.String or MetadataType.Object)
                return module.GetTypes().SelectMany(t => t.Methods)
                    .SelectMany(m => m.Parameters.Select(p => p.ParameterType).Append(m.ReturnType))
                    .First(t => t.MetadataType == type.MetadataType);
            if (!RuntimeSignatures.IsCore(type.Scope))
                throw new InvalidDataException("Foreign union reference type: " + type.FullName);
            return module.GetType(type.FullName)
                ?? throw new InvalidDataException("Unsupported union reference type: " + type.FullName);
        }
        foreach (var type in selected)
        {
            var clone = types[type.FullName];
            clone.BaseType = type.BaseType is null ? null : Map(type.BaseType);
            foreach (var implemented in type.Interfaces)
                clone.Interfaces.Add(new InterfaceImplementation(Map(implemented.InterfaceType)));
            foreach (var field in type.Fields)
                clone.Fields.Add(new FieldDefinition(field.Name, field.Attributes, Map(field.FieldType)) { Offset = field.Offset });
            var methods = new Dictionary<MethodDefinition, MethodDefinition>();
            foreach (var method in type.Methods)
            {
                var copy = new MethodDefinition(method.Name, method.Attributes, Map(method.ReturnType));
                foreach (var parameter in method.Parameters)
                    copy.Parameters.Add(new ParameterDefinition(parameter.Name, parameter.Attributes, Map(parameter.ParameterType)));
                if (!method.IsAbstract)
                {
                    copy.Body.Instructions.Add(Instruction.Create(OpCodes.Ldnull));
                    copy.Body.Instructions.Add(Instruction.Create(OpCodes.Throw));
                }
                clone.Methods.Add(copy);
                methods.Add(method, copy);
            }
            foreach (var property in type.Properties)
                clone.Properties.Add(new PropertyDefinition(property.Name, property.Attributes, Map(property.PropertyType)) {
                    GetMethod = property.GetMethod is null ? null : methods[property.GetMethod],
                    SetMethod = property.SetMethod is null ? null : methods[property.SetMethod]
                });
        }
        foreach (var type in selected)
            for (var index = 0; index < type.Methods.Count; index++)
                foreach (var implemented in type.Methods[index].Overrides)
                {
                    var declaration = new MethodReference(implemented.Name, Map(implemented.ReturnType), Map(implemented.DeclaringType))
                    { HasThis = implemented.HasThis, CallingConvention = implemented.CallingConvention };
                    foreach (var parameter in implemented.Parameters)
                        declaration.Parameters.Add(new ParameterDefinition(Map(parameter.ParameterType)));
                    types[type.FullName].Methods[index].Overrides.Add(declaration);
                }
        _ = StandardUnionLibrary.ProtocolType(module.GetType(StandardUnionLibrary.ProtocolName));
        var caseConstructor = module.GetType(RavenUnionMetadata.CaseAttribute).Methods.Single(m => m.IsConstructor);
        if (!caseConstructor.IsPublic || caseConstructor.IsStatic
            || !caseConstructor.Parameters.Select(p => p.ParameterType.MetadataType)
                .SequenceEqual(new[] { MetadataType.String, MetadataType.String, MetadataType.Int32 }))
            throw new InvalidDataException("Unsupported Raven union case attribute constructor.");
        types[root.FullName].CustomAttributes.Add(new CustomAttribute(module.GetType(
            "System.Runtime.CompilerServices.UnionAttribute").Methods.Single(m => m.IsConstructor)));
        foreach (var attribute in root.CustomAttributes.Where(a => a.AttributeType.FullName == RavenUnionMetadata.CaseAttribute))
        {
            var copy = new CustomAttribute(caseConstructor);
            foreach (var argument in attribute.ConstructorArguments)
                copy.ConstructorArguments.Add(new CustomAttributeArgument(Map(argument.Type), argument.Value));
            types[root.FullName].CustomAttributes.Add(copy);
        }
        foreach (var method in module.GetTypes().SelectMany(t => t.Methods).Where(m => m.HasBody))
        { _ = method.Body.Instructions.Count; _ = method.Body.Variables.Count; }
        foreach (var reference in module.AssemblyReferences.Where(r => !originals.Contains(r)).ToArray())
        {
            if (module.GetTypeReferences().Any(t => ReferenceEquals(t.Scope, reference)))
                throw new InvalidDataException("Union reference introduced an external scope.");
            module.AssemblyReferences.Remove(reference);
        }
        core.Write(output);
    }
}
