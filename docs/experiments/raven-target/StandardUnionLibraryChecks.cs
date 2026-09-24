using Mono.Cecil;
using Mono.Cecil.Cil;

// Test reference generation from emitted source metadata. Stub bodies deliberately
// fault; validation must import source bodies rather than execute the reference.
static class StandardUnionLibraryChecks
{
    public static void WriteReference(string implementation, string corePath, string owner, string output, string? mutation = null)
    {
        using var source = AssemblyDefinition.ReadAssembly(implementation);
        using var core = AssemblyDefinition.ReadAssembly(corePath);
        var module = core.MainModule;
        var originals = module.AssemblyReferences.ToHashSet();
        var root = source.MainModule.GetType(owner) ?? throw new InvalidDataException("Missing probe union.");
        var protocol = source.MainModule.GetType("System.Runtime.CompilerServices.IUnion");
        var caseAttribute = source.MainModule.GetType(RavenUnionMetadata.CaseAttribute)
            ?? throw new InvalidDataException("Missing source Raven union case attribute definition.");
        var selected = new[] { root, protocol, caseAttribute }.Concat(root.NestedTypes).ToArray();
        var types = new Dictionary<string, TypeDefinition>();
        foreach (var type in selected)
        {
            if (type is null || module.GetType(type.FullName) is not null)
                throw new InvalidDataException("Missing or conflicting probe type.");
            var clone = new TypeDefinition(type.Namespace, type.Name, type.Attributes)
            { PackingSize = type.PackingSize, ClassSize = type.ClassSize };
            types.Add(type.FullName, clone);
            if (type.DeclaringType is null) module.Types.Add(clone);
            else types[type.DeclaringType.FullName].NestedTypes.Add(clone);
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
            return module.GetType(type.FullName)
                ?? throw new InvalidDataException("Unsupported probe reference type: " + type.FullName);
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
        types[root.FullName].CustomAttributes.Add(new CustomAttribute(module.GetType(
            "System.Runtime.CompilerServices.UnionAttribute").Methods.Single(m => m.IsConstructor)));
        foreach (var attribute in root.CustomAttributes.Where(a => a.AttributeType.FullName == RavenUnionMetadata.CaseAttribute))
        {
            var copy = new CustomAttribute(types[RavenUnionMetadata.CaseAttribute].Methods.Single(m => m.IsConstructor));
            foreach (var argument in attribute.ConstructorArguments)
                copy.ConstructorArguments.Add(new CustomAttributeArgument(Map(argument.Type), argument.Value));
            types[root.FullName].CustomAttributes.Add(copy);
        }
        var carrier = types[root.FullName];
        switch (mutation)
        {
            case null: break;
            case "wrong-protocol":
                types["System.Runtime.CompilerServices.IUnion"].Methods.Single().ReturnType =
                    Map(root.Methods.Single(m => m.Name == "get_IsHeader").ReturnType); break;
            case "wrong-case": carrier.NestedTypes[0].Name += "Changed"; break;
            case "wrong-return": carrier.Methods.Single(m => m.Name == "get_IsHeader").ReturnType = Map(
                source.MainModule.GetType(owner).Methods.Single(m => m.Name == "ToString").ReturnType); break;
            case "wrong-output": carrier.Methods.First(m => m.Name == "TryGetValue").Parameters[0].IsOut = false; break;
            case "nonempty-case": carrier.NestedTypes[0].Fields.Add(new FieldDefinition("Extra", FieldAttributes.Private,
                carrier.Fields.Single(f => f.Name == "<Tag>").FieldType)); break;
            case "missing-case-metadata":
                carrier.CustomAttributes.Remove(carrier.CustomAttributes.First(a => a.AttributeType.FullName == RavenUnionMetadata.CaseAttribute)); break;
            case "wrong-case-ordinal":
                carrier.CustomAttributes.First(a => a.AttributeType.FullName == RavenUnionMetadata.CaseAttribute)
                    .ConstructorArguments[2] = new CustomAttributeArgument(Map(root.CustomAttributes.First(a => a.AttributeType.FullName == RavenUnionMetadata.CaseAttribute).ConstructorArguments[2].Type), 99); break;
            case "wrong-case-name":
                carrier.CustomAttributes.First(a => a.AttributeType.FullName == RavenUnionMetadata.CaseAttribute)
                    .ConstructorArguments[0] = new CustomAttributeArgument(Map(root.CustomAttributes.First(a => a.AttributeType.FullName == RavenUnionMetadata.CaseAttribute).ConstructorArguments[0].Type), "Probe.Missing+Headers"); break;
            case "unmarked": carrier.CustomAttributes.Clear(); break;
            default: throw new InvalidDataException("Unknown union reference mutation.");
        }
        foreach (var method in module.GetTypes().SelectMany(t => t.Methods).Where(m => m.HasBody))
        { _ = method.Body.Instructions.Count; _ = method.Body.Variables.Count; }
        foreach (var reference in module.AssemblyReferences.Where(r => !originals.Contains(r)).ToArray())
        {
            if (module.GetTypeReferences().Any(t => ReferenceEquals(t.Scope, reference)))
                throw new InvalidDataException("Probe reference introduced an external scope.");
            module.AssemblyReferences.Remove(reference);
        }
        core.Write(output);
    }
}
