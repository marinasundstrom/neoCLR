using Mono.Cecil;

// Raven's closed-interface contract is metadata, not the CLI Sealed bit.
static class IntrospectionHierarchy
{
    const string Prefix = "System.Introspection.";
    const string AttributeName = "System.Runtime.CompilerServices.ClosedHierarchyAttribute";
    static readonly Dictionary<string, string[]> Cases = new()
    {
        ["TypeInfo"] = ["RuntimeTypeInfo"],
        ["ParameterInfo"] = ["RuntimeParameterInfo"],
        ["MemberInfo"] = ["FieldInfo", "MethodInfo", "PropertyInfo"],
        ["FieldInfo"] = ["RuntimeFieldInfo"],
        ["MethodInfo"] = ["RuntimeMethodInfo"],
        ["PropertyInfo"] = ["RuntimePropertyInfo"],
        ["RuntimeMemberInfo"] = ["RuntimeFieldInfo", "RuntimeMethodInfo", "RuntimePropertyInfo"],
    };
    public static bool IsRoot(TypeReference type) => type.Namespace == "System.Introspection" && Cases.ContainsKey(type.Name);
    public static void Project(ModuleDefinition module)
    {
        // C# marks inherited implicit interface implementations virtual/final.
        // Raven keeps these shared storage accessors ordinary methods; only the
        // concrete leaves implement the public interfaces.
        foreach (var method in module.GetType(Prefix + "RuntimeMemberInfo").Methods.Where(m => !m.IsConstructor))
        {
            method.IsVirtual = false;
            method.IsFinal = false;
            method.IsNewSlot = false;
        }
        var attribute = new TypeDefinition("System.Runtime.CompilerServices", "ClosedHierarchyAttribute",
            TypeAttributes.NotPublic | TypeAttributes.Class | TypeAttributes.Sealed, module.GetType("System.Attribute"));
        var vector = new ArrayType(module.GetType("System.Type"));
        var constructor = new MethodDefinition(".ctor", MethodAttributes.Public | MethodAttributes.HideBySig
            | MethodAttributes.SpecialName | MethodAttributes.RTSpecialName, module.TypeSystem.Void);
        constructor.Parameters.Add(new ParameterDefinition("permittedTypes", ParameterAttributes.None, vector));
        attribute.Methods.Add(constructor);
        module.Types.Add(attribute);
        foreach (var (name, cases) in Cases)
        {
            var root = module.GetType(Prefix + name);
            var marker = new CustomAttribute(constructor);
            marker.ConstructorArguments.Add(new CustomAttributeArgument(vector, cases.Select(c =>
                new CustomAttributeArgument(module.GetType("System.Type"), module.GetType(Prefix + c))).ToArray()));
            root.CustomAttributes.Add(marker);
        }
    }
    public static void RejectExternalProviders(IEnumerable<ModuleDefinition> modules)
    {
        foreach (var type in modules.SelectMany(m => m.GetTypes()))
            foreach (var parent in type.Interfaces.Select(i => i.InterfaceType).Concat(type.BaseType is { } b ? new[] { b } : []))
                if (IsRoot(parent) && RuntimeSignatures.IsCore(parent.Scope))
                    throw new InvalidDataException("External introspection implementations are not permitted: " + type.FullName);
    }
    public static void Validate(TypeDefinition type)
    {
        if (!IsRoot(type)) return;
        var markers = type.CustomAttributes.Where(a => a.AttributeType.FullName == AttributeName).ToArray();
        // Raven serializes permitted types with its assembly's simple name. Resolve
        // those names only within this explicitly supplied definition, without
        // weakening the ordinary assembly-reference resolver's exact identities.
        if (markers.Length != 1 || markers[0].ConstructorArguments.Count != 1
            || markers[0].ConstructorArguments[0].Value is not CustomAttributeArgument[] cases
            || cases.Any(c => c.Value is not TypeReference reference || type.Module.GetType(reference.FullName) is null
                || (reference.Scope is AssemblyNameReference assembly ? assembly.Name != type.Module.Assembly.Name.Name
                    : reference.Scope != type.Module))
            || !cases.Select(c => (c.Value as TypeReference)?.FullName).Order()
                .SequenceEqual(Cases[type.Name].Select(c => Prefix + c).Order()))
            throw new InvalidDataException("Introspection sealed hierarchy does not match its permitted providers: " + type.FullName);
    }
}
