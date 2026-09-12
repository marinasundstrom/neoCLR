using Mono.Cecil;

// Target adapter: distinguish a generic Void type from a no-result return signature.
// No guest code runs, and no source-level compiler or host framework is modified.
static class VoidProjection
{
    public static void Write(string source, string core, string destination)
    {
        foreach (var path in new[] { source, core })
            if (new FileInfo(path).Length > 16 * 1024 * 1024) throw new InvalidDataException("Image exceeds profile limit.");
        using var library = AssemblyDefinition.ReadAssembly(core);
        var definition = library.MainModule.GetType("System.Void");
        if (definition is null || !definition.IsValueType || definition.Fields.Any(f => !f.IsStatic))
            throw new InvalidDataException("The target must declare an empty System.Void value.");
        using var assembly = AssemblyDefinition.ReadAssembly(source);
        var module = assembly.MainModule;
        var originalReferences = module.AssemblyReferences.ToHashSet();
        var scope = module.AssemblyReferences.Single(r => r.FullName == library.Name.FullName);
        var visited = new HashSet<TypeReference>();
        void Visit(TypeReference? type)
        {
            if (type is null || !visited.Add(type)) return;
            if (type is GenericInstanceType generic)
                for (var i = 0; i < generic.GenericArguments.Count; i++)
                {
                    if (generic.GenericArguments[i].MetadataType == MetadataType.Void)
                        generic.GenericArguments[i] = new TypeReference("System", "Void", module, scope, true);
                    else Visit(generic.GenericArguments[i]);
                }
            if (type is Mono.Cecil.TypeSpecification specification) Visit(specification.ElementType);
            Visit(type.DeclaringType);
        }
        TypeReference Storage(TypeReference type)
        {
            if (type.MetadataType == MetadataType.Void) return new TypeReference("System", "Void", module, scope, true);
            if (type is ByReferenceType byref) return new ByReferenceType(Storage(byref.ElementType));
            Visit(type);
            return type;
        }
        void Method(MethodReference method)
        {
            Visit(method.DeclaringType); Visit(method.ReturnType);
            foreach (var parameter in method.Parameters) parameter.ParameterType = Storage(parameter.ParameterType);
        }
        foreach (var member in module.GetMemberReferences().OfType<MethodReference>()) Method(member);
        foreach (var type in module.GetTypes())
        {
            Visit(type.BaseType);
            foreach (var field in type.Fields) Visit(field.FieldType);
            foreach (var method in type.Methods)
            {
                Method(method);
                if (!method.HasBody) continue;
                foreach (var variable in method.Body.Variables) variable.VariableType = Storage(variable.VariableType);
                foreach (var instruction in method.Body.Instructions)
                    switch (instruction.Operand)
                    {
                        case MethodReference called: Method(called); break;
                        case FieldReference field: Visit(field.DeclaringType); Visit(field.FieldType); break;
                        case TypeReference operand: Visit(operand); break;
                    }
            }
        }
        // Cecil may synthesize a core reference while decoding primitive signatures.
        // Drop only newly synthesized, unreferenced table entries, never input dependencies.
        foreach (var reference in module.AssemblyReferences.Where(r => !originalReferences.Contains(r)).ToArray())
        {
            if (module.GetTypeReferences().Any(t => ReferenceEquals(t.Scope, reference)))
                throw new InvalidDataException("Projection introduced a target-external type scope.");
            module.AssemblyReferences.Remove(reference);
        }
        assembly.Write(destination);
    }
}
