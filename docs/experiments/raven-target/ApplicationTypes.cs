using Mono.Cecil;
using System.Text;
using System.Text.RegularExpressions;

// Per-import application metadata. No host type loading or per-application API catalog.
static class ApplicationTypes
{
    static ModuleDefinition? Module;
    static readonly Dictionary<string, TypeDefinition> Types = new();
    public static void Reset(ModuleDefinition module) { Module = module; Types.Clear(); }
    public static bool IsType(string name) => Types.ContainsKey(name);
    public static bool IsReference(string name) => Types.TryGetValue(name, out var type) && !type.IsValueType;
    public static string? Type(TypeReference reference)
    {
        if (reference is ByReferenceType byref) return Type(byref.ElementType) is { } element ? element + "&" : null;
        if (reference is TypeSpecification || reference.Scope is AssemblyNameReference assembly && assembly.FullName != Module?.Assembly.Name.FullName) return null;
        var type = reference.Resolve();
        if (type is null || type.Module != Module || type.FullName == "System.Unit" || type.Name == "<Module>") return null;
        if (type.HasGenericParameters || type.IsInterface || type.IsEnum || type.IsAbstract
            || type.IsExplicitLayout || type.IsNested || type.HasInterfaces
            || type.BaseType?.FullName is not ("System.Object" or "System.ValueType")
            || type.Fields.Any(f => f.IsStatic || f.HasMarshalInfo || f.IsInitOnly)
            || type.Methods.Any(m => m.IsVirtual || m.HasOverrides || m.IsConstructor && m.IsStatic))
            throw new InvalidDataException("Unsupported application type: " + type.FullName);
        var name = $"Application.Type_{type.MetadataToken.ToUInt32():x8}";
        Types[name] = type;
        if (Types.Count > 128) throw new InvalidDataException("Application type limit exceeded.");
        return name;
    }
    public static string MethodName(MethodDefinition method)
    {
        if (method.IsConstructor) return ".ctor";
        if (!Regex.IsMatch(method.Name, @"^[A-Za-z_][A-Za-z0-9_]*$")) throw new InvalidDataException("Unsupported application member name.");
        return method.Name;
    }
    public static void CheckMethod(MethodReference method)
    {
        if (method.ExplicitThis || method.HasGenericParameters || method is GenericInstanceMethod
            || method.DeclaringType.HasGenericParameters || method.DeclaringType is GenericInstanceType
            || method.CallingConvention != MethodCallingConvention.Default)
            throw new InvalidDataException("Unsupported application signature: " + method.FullName);
        if (method.HasThis && Type(method.DeclaringType) is null) throw new InvalidDataException("Unsupported application receiver.");
    }
    public static string Receiver(MethodDefinition method) => Type(method.DeclaringType)! + (method.DeclaringType.IsValueType ? "&" : "");
    public sealed record FieldShape(string Owner, string Type, string Name, bool ValueOwner);
    public static FieldShape? Field(FieldReference reference, MethodDefinition caller, Func<TypeReference, bool, string> map)
    {
        var field = reference.Resolve();
        if (field is null || field.Module != Module) return null;
        var owner = Type(field.DeclaringType)!;
        if (field.IsStatic || reference.FullName != field.FullName || (!field.IsPublic && caller.DeclaringType != field.DeclaringType))
            throw new InvalidDataException("Unsupported application field access.");
        return new(owner, map(field.FieldType, false), $"Field_{field.MetadataToken.ToUInt32():x8}", field.DeclaringType.IsValueType);
    }
    public static string Declarations(Func<TypeReference, bool, string> map, Dictionary<MethodDefinition, string> bodies)
    {
        var output = new StringBuilder();
        var emitted = new HashSet<string>();
        while (Types.Any(t => !emitted.Contains(t.Key)))
        {
            var (name, type) = Types.First(t => !emitted.Contains(t.Key)); emitted.Add(name);
            output.AppendLine($".type {(type.IsValueType ? "" : "class ")}{name}");
            foreach (var field in type.Fields)
                output.AppendLine($".field Field_{field.MetadataToken.ToUInt32():x8} {map(field.FieldType, false)}");
            foreach (var body in bodies.Where(p => p.Key.DeclaringType == type)) output.Append(body.Value);
            output.AppendLine(".end");
        }
        return output.ToString();
    }
}
