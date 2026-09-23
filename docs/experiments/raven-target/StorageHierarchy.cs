using Mono.Cecil;

// Close item kinds while allowing providers to implement either public branch.
static class StorageHierarchy
{
    const string Marker = "System.Runtime.CompilerServices.ClosedHierarchyAttribute";
    static readonly string[] Branches = ["System.Storage.File", "System.Storage.Directory"];
    public static void Project(ModuleDefinition module)
    {
        var attribute = module.GetType(Marker);
        if (attribute is null)
        {
            attribute = new TypeDefinition("System.Runtime.CompilerServices", "ClosedHierarchyAttribute",
                TypeAttributes.NotPublic | TypeAttributes.Class | TypeAttributes.Sealed, module.GetType("System.Attribute"));
            var ctor = new MethodDefinition(".ctor", MethodAttributes.Public | MethodAttributes.HideBySig
                | MethodAttributes.SpecialName | MethodAttributes.RTSpecialName, module.TypeSystem.Void);
            ctor.Parameters.Add(new ParameterDefinition("permittedTypes", ParameterAttributes.None, new ArrayType(module.GetType("System.Type"))));
            attribute.Methods.Add(ctor);
            module.Types.Add(attribute);
        }
        var constructor = attribute.Methods.Single(m => m.IsConstructor);
        var vector = constructor.Parameters[0].ParameterType;
        var marker = new CustomAttribute(constructor);
        marker.ConstructorArguments.Add(new CustomAttributeArgument(vector, Branches.Select(name =>
            new CustomAttributeArgument(module.GetType("System.Type"), module.GetType(name))).ToArray()));
        module.GetType(StorageItemBindings.Root).CustomAttributes.Add(marker);
    }
    public static void Validate(TypeDefinition type)
    {
        if (!StorageItemBindings.IsName(type.FullName)) return;
        var markers = type.CustomAttributes.Where(a => a.AttributeType.FullName == Marker).ToArray();
        if (type.FullName != StorageItemBindings.Root)
        {
            if (markers.Length != 0) throw new InvalidDataException("Storage branches must remain provider-implementable.");
            return;
        }
        if (markers.Length != 1 || markers[0].ConstructorArguments.Count != 1
            || markers[0].ConstructorArguments[0].Value is not CustomAttributeArgument[] cases
            || cases.Any(c => c.Value is not TypeReference reference || type.Module.GetType(reference.FullName) is null
                || (reference.Scope is AssemblyNameReference assembly ? assembly.Name != type.Module.Assembly.Name.Name : reference.Scope != type.Module))
            || !cases.Select(c => (c.Value as TypeReference)?.FullName).Order().SequenceEqual(Branches.Order()))
            throw new InvalidDataException("StorageItem must be closed to File and Directory.");
    }
    public static void RejectExternalBranches(IEnumerable<ModuleDefinition> modules)
    {
        bool HasBranch(TypeDefinition type, HashSet<TypeDefinition> visited)
        {
            if (!visited.Add(type)) return false;
            return type.Interfaces.Any(i => RuntimeSignatures.IsCore(i.InterfaceType.Scope)
                ? Branches.Contains(i.InterfaceType.FullName)
                : HasBranch(i.InterfaceType.Resolve(), visited))
                || type.BaseType is { } parent && !RuntimeSignatures.IsCore(parent.Scope) && HasBranch(parent.Resolve(), visited);
        }
        foreach (var type in modules.SelectMany(m => m.GetTypes()))
            // Raven emits transitive InterfaceImpl rows too. A redundant root row
            // on a File/Directory implementation is not a new item kind.
            if (type.Interfaces.Any(i => i.InterfaceType.FullName == StorageItemBindings.Root && RuntimeSignatures.IsCore(i.InterfaceType.Scope))
                && !HasBranch(type, new()))
                throw new InvalidDataException("External direct StorageItem branches are not permitted: " + type.FullName);
    }
}
