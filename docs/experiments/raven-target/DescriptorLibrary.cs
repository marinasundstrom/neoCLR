using Mono.Cecil;

static class DescriptorLibrary
{
    static readonly Dictionary<string, (string Name, string Type)[]> Layouts = new()
    {
        ["MemberInfo"] = [("Name", "System.String"), ("DeclaringType", "System.Type")],
        ["FieldInfo"] = [("FieldType", "System.Type"), ("IsPublic", "System.Boolean"), ("IsPrivate", "System.Boolean"), ("IsAssembly", "System.Boolean"), ("IsStatic", "System.Boolean"), ("DefinitionIndex", "System.Int32")],
        ["MethodInfo"] = [("ReturnType", "System.Type"), ("IsStatic", "System.Boolean"), ("IsPublic", "System.Boolean"), ("IsPrivate", "System.Boolean"), ("IsAssembly", "System.Boolean"), ("IsReceiverByRef", "System.Boolean"), ("DefinitionIndex", "System.Int32"), ("Parameters", "System.Runtime.CompilerServices.ParameterSnapshot"), ("IsReadOnly", "System.Boolean"), ("IsVirtual", "System.Boolean"), ("IsOverride", "System.Boolean"), ("IsAbstract", "System.Boolean")],
        ["PropertyInfo"] = [("PropertyType", "System.Type"), ("IsStatic", "System.Boolean"), ("CanRead", "System.Boolean"), ("CanWrite", "System.Boolean"), ("DefinitionIndex", "System.Int32"), ("IndexParameters", "System.Runtime.CompilerServices.ParameterSnapshot"), ("Getter", "System.Option`1<System.Introspection.MethodInfo>"), ("Setter", "System.Option`1<System.Introspection.MethodInfo>")],
    };
    public static bool IsDescriptor(TypeReference type) => type.Namespace == "System.Introspection" && Layouts.ContainsKey(type.Name);
    public static bool IsBaseConstructor(MethodDefinition method) => IsDescriptor(method.DeclaringType)
        && ApplicationTypes.IsLibrary(method.DeclaringType) && method.DeclaringType.Name == "MemberInfo"
        && method.IsConstructor && method.IsFamily && !method.IsStatic;
    public static MethodDefinition[] Roots(ModuleDefinition source, ModuleDefinition core,
        Func<TypeDefinition, TypeDefinition, string, MethodDefinition[]> roots)
    {
        var result = new List<MethodDefinition>();
        foreach (var name in Layouts.Keys)
        {
            var fullName = "System.Introspection." + name;
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
        return result.ToArray();
    }
    static bool CoreStorage(TypeReference type, ModuleDefinition source)
    {
        if (type is GenericInstanceType generic)
            return RuntimeSignatures.IsCore(generic.ElementType.Scope) && generic.GenericArguments.All(t => CoreStorage(t, source));
        return type.MetadataType is MetadataType.String or MetadataType.Int32 or MetadataType.Boolean
            || RuntimeSignatures.IsCore(type.Scope) || type.FullName == "System.Introspection.MethodInfo" && type.Resolve()?.Module == source;
    }
    public static string Base(TypeDefinition type) => type.Name == "MemberInfo" ? "System.Object" : "System.Introspection.MemberInfo";
    public static bool SameType(TypeReference left, TypeReference right) => IsDescriptor(left)
        && left.FullName == right.FullName && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);
}
