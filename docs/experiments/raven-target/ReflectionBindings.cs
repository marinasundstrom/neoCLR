using Mono.Cecil;
using System.Text;

// Current reflection API catalog. Placeholder metadata bodies never execute.
static class ReflectionBindings
{
    public static readonly string[] Classes = ["System.Type", "System.Reflection.TypeInfo", "System.Reflection.ParameterInfo", "System.Reflection.MemberInfo", "System.Reflection.FieldInfo", "System.Reflection.MethodInfo", "System.Reflection.PropertyInfo"];
    static readonly (string Owner, string Name, string[] Args, string Result, bool Static)[] Members = [
        ("System.Type", "GetTypeFromHandle", ["System.RuntimeTypeHandle"], "System.Type", true),
        ("System.Type", "get_Info", [], "System.Reflection.TypeInfo", false),
        ("System.Type", "get_Name", [], "String", false),
        ("System.Type", "get_GenericArgumentCount", [], "Int32", false),
        ("System.Type", "GetGenericArgument", ["Int32"], "System.Type", false),
        ("System.Type", "Equals", ["System.Type"], "Boolean", false),
        ("System.Reflection.TypeInfo", "GetFields", [], "System.Reflection.FieldInfo[]", false),
        ("System.Reflection.TypeInfo", "GetFields", ["System.Reflection.BindingFlags"], "System.Reflection.FieldInfo[]", false),
        ("System.Reflection.TypeInfo", "GetMethods", [], "System.Reflection.MethodInfo[]", false),
        ("System.Reflection.TypeInfo", "GetMethods", ["System.Reflection.BindingFlags"], "System.Reflection.MethodInfo[]", false),
        ("System.Reflection.TypeInfo", "GetProperties", [], "System.Reflection.PropertyInfo[]", false),
        ("System.Reflection.TypeInfo", "GetProperties", ["System.Reflection.BindingFlags"], "System.Reflection.PropertyInfo[]", false),
        ("System.Type", "GetGenericArguments", [], "System.Type[]", false),
        ("System.Type", "GetElementType", [], "System.Option<System.Type>", false),
        ("System.Type", "get_IsValueType", [], "Boolean", false),
        ("System.Type", "get_IsEnum", [], "Boolean", false),
        ("System.Type", "get_IsArray", [], "Boolean", false),
        ("System.Type", "get_IsAbstract", [], "Boolean", false),
        ("System.Type", "get_IsReadOnly", [], "Boolean", false),
        ("System.Type", "get_IsByRef", [], "Boolean", false),
        ("System.Type", "get_IsPointer", [], "Boolean", false),
        ("System.Type", "get_IsInterface", [], "Boolean", false),
        ("System.Type", "get_FullName", [], "String", false),
        ("System.Type", "get_Namespace", [], "String", false),
        ("System.TypeOf<T>", "Of", ["T"], "System.Type", true),
        ("System.Reflection.ParameterInfo", "get_Name", [], "String", false),
        ("System.Reflection.ParameterInfo", "get_Position", [], "Int32", false),
        ("System.Reflection.ParameterInfo", "get_ParameterType", [], "System.Type", false),
        ("System.Reflection.ParameterInfo", "get_IsOut", [], "Boolean", false),
        ("System.Reflection.ParameterInfo", "get_IsOutWhenTrue", [], "Boolean", false),
        ("System.Reflection.ParameterInfo", "get_IsReadOnly", [], "Boolean", false),
        ("System.Reflection.MemberInfo", "get_Name", [], "String", false),
        ("System.Reflection.MemberInfo", "get_DeclaringType", [], "System.Type", false),
        ("System.Reflection.FieldInfo", "get_FieldType", [], "System.Type", false),
        ("System.Reflection.FieldInfo", "get_IsPublic", [], "Boolean", false),
        ("System.Reflection.FieldInfo", "get_IsPrivate", [], "Boolean", false),
        ("System.Reflection.FieldInfo", "get_IsAssembly", [], "Boolean", false),
        ("System.Reflection.FieldInfo", "get_IsStatic", [], "Boolean", false),
        ("System.Reflection.FieldInfo", "get_DefinitionIndex", [], "Int32", false),
        ("System.Reflection.MethodInfo", "get_ReturnType", [], "System.Type", false),
        ("System.Reflection.MethodInfo", "get_IsStatic", [], "Boolean", false),
        ("System.Reflection.MethodInfo", "get_IsPublic", [], "Boolean", false),
        ("System.Reflection.MethodInfo", "get_IsPrivate", [], "Boolean", false),
        ("System.Reflection.MethodInfo", "get_IsAssembly", [], "Boolean", false),
        ("System.Reflection.MethodInfo", "get_IsReceiverByRef", [], "Boolean", false),
        ("System.Reflection.MethodInfo", "get_DefinitionIndex", [], "Int32", false),
        ("System.Reflection.MethodInfo", "GetParameters", [], "System.Reflection.ParameterInfo[]", false),
        ("System.Reflection.MethodInfo", "get_IsReadOnly", [], "Boolean", false),
        ("System.Reflection.MethodInfo", "get_IsVirtual", [], "Boolean", false),
        ("System.Reflection.MethodInfo", "get_IsOverride", [], "Boolean", false),
        ("System.Reflection.MethodInfo", "get_IsAbstract", [], "Boolean", false),
        ("System.Reflection.PropertyInfo", "get_PropertyType", [], "System.Type", false),
        ("System.Reflection.PropertyInfo", "get_IsStatic", [], "Boolean", false),
        ("System.Reflection.PropertyInfo", "get_CanRead", [], "Boolean", false),
        ("System.Reflection.PropertyInfo", "get_CanWrite", [], "Boolean", false),
        ("System.Reflection.PropertyInfo", "get_DefinitionIndex", [], "Int32", false),
        ("System.Reflection.PropertyInfo", "GetIndexParameters", [], "System.Reflection.ParameterInfo[]", false),
        ("System.Reflection.PropertyInfo", "GetGetMethod", [], "System.Option<System.Reflection.MethodInfo>", false),
        ("System.Reflection.PropertyInfo", "GetGetMethod", ["Boolean"], "System.Option<System.Reflection.MethodInfo>", false),
        ("System.Reflection.PropertyInfo", "GetSetMethod", [], "System.Option<System.Reflection.MethodInfo>", false),
        ("System.Reflection.PropertyInfo", "GetSetMethod", ["Boolean"], "System.Option<System.Reflection.MethodInfo>", false),
        ("System.Reflection.TypeInfo", "GetInterfaces", [], "System.Type[]", false),
        ("System.Reflection.TypeInfo", "GetEnumNames", [], "String[]", false),
        ("System.Reflection.TypeInfo", "GetEnumUnderlyingType", [], "System.Type", false),
        ("System.Reflection.TypeInfo", "get_BaseType", [], "System.Option<System.Type>", false),
    ];
    public const string Declarations = """
        public struct RuntimeTypeHandle { }
        public class Type { internal Type() { } public System.Reflection.TypeInfo Info => default; public string Name => default; public int GenericArgumentCount => default; public bool IsEnum => default; public bool IsArray => default; public bool IsAbstract => default; public bool IsReadOnly => default; public bool IsByRef => default; public bool IsPointer => default; public bool IsValueType => default; public bool IsInterface => default; public string FullName => default; public string Namespace => default; public static System.Type GetTypeFromHandle(System.RuntimeTypeHandle handle) => default; public System.Type GetGenericArgument(int index) => default; public bool Equals(System.Type other) => default; public System.Type[] GetGenericArguments() => default; public System.Option<System.Type> GetElementType() => default; }
        public static class TypeOf<T> { public static System.Type Of(T arg0) => default; }
        namespace Reflection { public class TypeInfo { internal TypeInfo() { } public System.Option<System.Type> BaseType => default; public System.Type[] GetInterfaces() => default; public string[] GetEnumNames() => default; public System.Type GetEnumUnderlyingType() => default; public FieldInfo[] GetFields() => default; public FieldInfo[] GetFields(BindingFlags arg0) => default; public MethodInfo[] GetMethods() => default; public MethodInfo[] GetMethods(BindingFlags arg0) => default; public PropertyInfo[] GetProperties() => default; public PropertyInfo[] GetProperties(BindingFlags arg0) => default; } }
        namespace Reflection { public class ParameterInfo { internal ParameterInfo() { } public string Name => default; public int Position => default; public System.Type ParameterType => default; public bool IsOut => default; public bool IsOutWhenTrue => default; public bool IsReadOnly => default; } }
        namespace Reflection { public abstract class MemberInfo { internal MemberInfo() { } public string Name => default; public System.Type DeclaringType => default; } }
        namespace Reflection { public class FieldInfo : System.Reflection.MemberInfo { internal FieldInfo() { } public System.Type FieldType => default; public bool IsPublic => default; public bool IsPrivate => default; public bool IsAssembly => default; public bool IsStatic => default; public int DefinitionIndex => default; } }
        namespace Reflection { public class MethodInfo : System.Reflection.MemberInfo { internal MethodInfo() { } public System.Type ReturnType => default; public bool IsStatic => default; public bool IsPublic => default; public bool IsPrivate => default; public bool IsAssembly => default; public bool IsReceiverByRef => default; public int DefinitionIndex => default; public bool IsReadOnly => default; public bool IsVirtual => default; public bool IsOverride => default; public bool IsAbstract => default; public System.Reflection.ParameterInfo[] GetParameters() => default; } }
        namespace Reflection { public class PropertyInfo : System.Reflection.MemberInfo { internal PropertyInfo() { } public System.Type PropertyType => default; public bool IsStatic => default; public bool CanRead => default; public bool CanWrite => default; public int DefinitionIndex => default; public System.Reflection.ParameterInfo[] GetIndexParameters() => default; public System.Option<System.Reflection.MethodInfo> GetGetMethod() => default; public System.Option<System.Reflection.MethodInfo> GetGetMethod(bool arg0) => default; public System.Option<System.Reflection.MethodInfo> GetSetMethod() => default; public System.Option<System.Reflection.MethodInfo> GetSetMethod(bool arg0) => default; } }
        namespace Reflection { [Flags] public enum BindingFlags { Default = 0, DeclaredOnly = 2, Instance = 4, Static = 8, Public = 16, NonPublic = 32 } }
        """;
    static readonly Dictionary<string, string> Helpers = new();
    public static void Reset() => Helpers.Clear();
    public static string Adapters => string.Join("\n", Helpers.Values);
    public static bool IsReference(string type) => Classes.Contains(type);
    public static bool IsArray(string type) => Classes.Any(t => type == $"arrayref<{t}>");
    public static bool IsType(string type) => IsReference(type) || IsArray(type) || type is "System.RuntimeTypeHandle" or "System.Reflection.BindingFlags";
    public static bool Assignable(string source, string target) => source == target || target == "System.Reflection.MemberInfo" && source is "System.Reflection.FieldInfo" or "System.Reflection.MethodInfo" or "System.Reflection.PropertyInfo";
    public static string? Type(TypeReference type)
    {
        if (type is ArrayType { IsVector: true } array && Classes.Contains(array.ElementType.FullName)) return $"arrayref<{array.ElementType.FullName}>";
        if (!RuntimeSignatures.IsCore(type.Scope)) return null;
        if (Classes.Contains(type.FullName) && !type.IsValueType) return type.FullName;
        return type.IsValueType && type.FullName is "System.RuntimeTypeHandle" or "System.Reflection.BindingFlags" ? type.FullName : null;
    }
    public static void Validate(ModuleDefinition module)
    {
        EnumBindings.Validate(module);
        foreach (var name in Classes)
        {
            var type = module.GetType(name) ?? throw new InvalidDataException("Missing reflection type: " + name);
            var parent = name is "System.Reflection.FieldInfo" or "System.Reflection.MethodInfo" or "System.Reflection.PropertyInfo" ? "System.Reflection.MemberInfo" : "System.Object";
            if (type.IsValueType || type.IsInterface || type.HasGenericParameters || type.BaseType?.FullName != parent
                || type.IsAbstract != (name == "System.Reflection.MemberInfo"))
                throw new InvalidDataException("Unsupported reflection class contract: " + name);
        }
    }
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        var owner = Type(reference.DeclaringType);
        if (reference.DeclaringType is GenericInstanceType { GenericArguments.Count: 1 } generic
            && generic.ElementType.FullName == "System.TypeOf`1" && RuntimeSignatures.IsCore(generic.Scope))
        {
            var element = GenericUnionBindings.Type(generic.GenericArguments[0]);
            if (element is not null && reference.Name == "Of" && !reference.HasThis)
            {
                var signature = RuntimeSignatures.Match(reference, definition, t => Type(t) ?? GenericUnionBindings.Type(t));
                if (signature.Result == "System.Type" && signature.Args.SequenceEqual(new[] { element }))
                    return new($"System.TypeOf<{element}>::Of", signature.Args, signature.Result);
            }
            throw new InvalidDataException("Unsupported TypeOf signature.");
        }
        if (owner is null || IsArray(owner) || owner == "System.RuntimeTypeHandle") return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, t => Type(t) ?? ProcessBindings.ArrayType(t) ?? GenericUnionBindings.Type(t));
        string Project(string t) => t.EndsWith("[]") ? "arrayref<" + t[..^2] + ">" : t;
        if (!Members.Any(m => m.Owner == owner && m.Name == reference.Name && m.Static == !reference.HasThis
            && m.Args.Select(Project).SequenceEqual(args) && Project(m.Result) == result))
            throw new InvalidDataException("Unsupported reflection signature: " + reference.FullName);
        var inputs = reference.HasThis ? new[] { owner }.Concat(args).ToArray() : args;
        var key = reference.FullName;
        var name = "RuntimeReflection" + Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(Encoding.UTF8.GetBytes(key)))[..16];
        if (!Helpers.ContainsKey(key))
        {
            var body = new StringBuilder($".function {name}({string.Join(',', inputs.Select((t,i) => t + " arg" + i))}) -> {result}\n");
            // Raven-authored Type already returns managed arrays. The remaining
            // NeoIL descriptors still return snapshot arrays which need copying.
            var vector = owner != "System.Type" && result.StartsWith("arrayref<", StringComparison.Ordinal);
            var element = vector ? result[9..^1] : "";
            if (vector) body.AppendLine($".local {element}[] source\n.local {result} destination\n.local Int32 index");
            for (var i = 0; i < inputs.Length; i++)
            {
                body.AppendLine("ldarg arg" + i);
            }
            body.AppendLine($"call {(reference.HasThis ? "instance " : "")}{owner}::{reference.Name}({string.Join(',', args)})");
            if (vector) body.AppendLine($"stloc source\nldloc source\nldlen\nconv.i4\nnewarr {element}\nstloc destination\nldc.i4 0\nstloc index\nbr Test\nCopy:\nldloc destination\nldloc index\nldloc source\nldloc index\nldelem {element}\nstelem {element}\nldloc index\nldc.i4 1\nadd\nstloc index\nTest:\nldloc index\nldloc source\nldlen\nconv.i4\nblt Copy\nldloc destination");
            body.AppendLine("ret\n.end"); Helpers.Add(key, body.ToString());
        }
        return new(name, inputs, result);
    }
}
