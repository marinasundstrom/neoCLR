using Mono.Cecil;

static class DescriptorLibrary
{
    static readonly Dictionary<string, (string Name, string Type)[]> Layouts = new()
    {
        ["MemberInfo"] = [("Name", "System.String"), ("DeclaringType", "System.Introspection.TypeInfo"), ("MetadataToken", "System.Int32")],
        ["FieldInfo"] = [("FieldType", "System.Introspection.TypeInfo"), ("IsPublic", "System.Boolean"), ("IsPrivate", "System.Boolean"), ("IsAssembly", "System.Boolean"), ("IsStatic", "System.Boolean"), ("DefinitionIndex", "System.Int32")],
        ["MethodInfo"] = [("ReturnType", "System.Introspection.TypeInfo"), ("IsStatic", "System.Boolean"), ("IsPublic", "System.Boolean"), ("IsPrivate", "System.Boolean"), ("IsAssembly", "System.Boolean"), ("IsReceiverByRef", "System.Boolean"), ("DefinitionIndex", "System.Int32"), ("Parameters", "System.Runtime.CompilerServices.ParameterSnapshot"), ("IsReadOnly", "System.Boolean"), ("IsVirtual", "System.Boolean"), ("IsOverride", "System.Boolean"), ("IsAbstract", "System.Boolean")],
        ["PropertyInfo"] = [("PropertyType", "System.Introspection.TypeInfo"), ("IsStatic", "System.Boolean"), ("CanRead", "System.Boolean"), ("CanWrite", "System.Boolean"), ("DefinitionIndex", "System.Int32"), ("IndexParameters", "System.Runtime.CompilerServices.ParameterSnapshot"), ("Getter", "System.Option`1<System.Introspection.MethodInfo>"), ("Setter", "System.Option`1<System.Introspection.MethodInfo>")],
    };
    public static bool IsDescriptor(TypeReference type) => type.Namespace == "System.Introspection" && type.Name.StartsWith("Runtime") && Layouts.ContainsKey(type.Name[7..]);
    public static bool IsBaseConstructor(MethodDefinition method) => IsDescriptor(method.DeclaringType)
        && ApplicationTypes.IsLibrary(method.DeclaringType) && method.DeclaringType.Name == "RuntimeMemberInfo"
        && method.IsConstructor && method.IsFamily && !method.IsStatic;
    public static MethodDefinition[] Roots(ModuleDefinition source, ModuleDefinition core,
        Func<TypeDefinition, TypeDefinition, string, MethodDefinition[]> roots,
        Func<TypeDefinition, TypeDefinition, string, MethodDefinition[]> interfaces)
    {
        var result = new List<MethodDefinition>();
        var contracts = Layouts.Keys.Append("TypeInfo").ToArray();
        // The closed family is authored together; bind the explicit source/core
        // pairs before checking its mutually referring signatures.
        foreach (var name in contracts)
            ApplicationTypes.BindLibrary(source.GetType("System.Introspection." + name), "System.Introspection." + name);
        foreach (var name in contracts)
            interfaces(source.GetType("System.Introspection." + name), core.GetType("System.Introspection." + name), "System.Introspection." + name);
        foreach (var name in Layouts.Keys)
        {
            var fullName = "System.Introspection.Runtime" + name;
            var type = source.GetType(fullName) ?? throw new InvalidDataException("Missing descriptor " + name);
            var contract = core.GetType(fullName) ?? throw new InvalidDataException("Missing descriptor contract " + name);
            var layout = Layouts[name];
            if (type.Fields.Count != layout.Length || type.Fields.Zip(layout).Any(pair =>
                pair.First.Name != "Stored" + pair.Second.Name || pair.First.FieldType.FullName != pair.Second.Type
                || !CoreStorage(pair.First.FieldType, source))
                || type.Methods.Count(m => m.IsConstructor) != 1
                || type.Methods.Single(m => m.IsConstructor).IsFamily != (name == "MemberInfo")
                || name != "MemberInfo" && !type.Methods.Single(m => m.IsConstructor).IsPrivate)
                throw new InvalidDataException("Descriptor storage or construction contract mismatch: " + name);
            result.AddRange(roots(type, contract, fullName));
        }
        result.AddRange(roots(source.GetType("System.Introspection.RuntimeTypeInfo"),
            core.GetType("System.Introspection.RuntimeTypeInfo"), "System.Introspection.RuntimeTypeInfo"));
        result.AddRange(LibraryImplementation.Roots(source, core, "System.Runtime.Reflection.TypeReflectionExtensions"));
        result.AddRange(LibraryImplementation.Roots(source, core, "System.Runtime.Reflection.PropertyReflectionExtensions"));
        return result.ToArray();
    }
    static bool CoreStorage(TypeReference type, ModuleDefinition source)
    {
        if (type is GenericInstanceType generic)
            return RuntimeSignatures.IsCore(generic.ElementType.Scope) && generic.GenericArguments.All(t => CoreStorage(t, source));
        return type.MetadataType is MetadataType.String or MetadataType.Int32 or MetadataType.Boolean
            || RuntimeSignatures.IsCore(type.Scope) || type.FullName is "System.Introspection.MethodInfo" or "System.Introspection.TypeInfo" && type.Resolve()?.Module == source;
    }
    public static string Base(TypeDefinition type) => type.Name == "RuntimeMemberInfo" ? "System.Object" : "System.Introspection.RuntimeMemberInfo";
    public static bool IsProvider(TypeReference type) => type.Namespace == "System.Introspection"
        && (IsDescriptor(type) || type.Name is "RuntimeTypeInfo" or "RuntimeParameterInfo" or "RuntimeAssemblyInfo" or "RuntimeModuleInfo");
    public static bool SameType(TypeReference left, TypeReference right) => (IsProvider(left)
        || left.Namespace == "System.Introspection" && left.Name is "MemberInfo" or "FieldInfo" or "MethodInfo" or "PropertyInfo" or "TypeInfo" or "ParameterInfo" or "AssemblyInfo" or "ModuleInfo")
        && left.FullName == right.FullName && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);
}
