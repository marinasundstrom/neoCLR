using Mono.Cecil;
using System.Text;

// Current reflection API catalog. Placeholder metadata bodies never execute.
static class ReflectionBindings
{
    public static readonly string[] ReferenceTypes = ["System.Type", "System.Introspection.TypeInfo", "System.Introspection.ParameterInfo", "System.Introspection.MemberInfo", "System.Introspection.FieldInfo", "System.Introspection.MethodInfo", "System.Introspection.PropertyInfo"];
    static readonly (string Owner, string Name, string[] Args, string Result, bool Static)[] Members = [
        ("System.Type", "GetTypeFromHandle", ["System.RuntimeTypeHandle"], "System.Type", true),
        ("System.Type", "get_Info", [], "System.Introspection.TypeInfo", false),
        ("System.Type", "get_Name", [], "String", false),
        ("System.Type", "get_GenericArgumentCount", [], "Int32", false),
        ("System.Type", "GetGenericArgument", ["Int32"], "System.Type", false),
        ("System.Type", "Equals", ["System.Type"], "Boolean", false),
        ("System.Introspection.TypeInfo", "GetFields", [], "System.Introspection.FieldInfo[]", false),
        ("System.Introspection.TypeInfo", "GetFields", ["System.Introspection.BindingFlags"], "System.Introspection.FieldInfo[]", false),
        ("System.Introspection.TypeInfo", "GetMethods", [], "System.Introspection.MethodInfo[]", false),
        ("System.Introspection.TypeInfo", "GetMethods", ["System.Introspection.BindingFlags"], "System.Introspection.MethodInfo[]", false),
        ("System.Introspection.TypeInfo", "GetProperties", [], "System.Introspection.PropertyInfo[]", false),
        ("System.Introspection.TypeInfo", "GetProperties", ["System.Introspection.BindingFlags"], "System.Introspection.PropertyInfo[]", false),
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
        ("System.Introspection.ParameterInfo", "get_Name", [], "String", false),
        ("System.Introspection.ParameterInfo", "get_Position", [], "Int32", false),
        ("System.Introspection.ParameterInfo", "get_ParameterType", [], "System.Type", false),
        ("System.Introspection.ParameterInfo", "get_IsOut", [], "Boolean", false),
        ("System.Introspection.ParameterInfo", "get_IsOutWhenTrue", [], "Boolean", false),
        ("System.Introspection.ParameterInfo", "get_IsReadOnly", [], "Boolean", false),
        ("System.Introspection.MemberInfo", "get_Name", [], "String", false),
        ("System.Introspection.MemberInfo", "get_DeclaringType", [], "System.Type", false),
        ("System.Introspection.FieldInfo", "get_FieldType", [], "System.Type", false),
        ("System.Introspection.FieldInfo", "get_IsPublic", [], "Boolean", false),
        ("System.Introspection.FieldInfo", "get_IsPrivate", [], "Boolean", false),
        ("System.Introspection.FieldInfo", "get_IsAssembly", [], "Boolean", false),
        ("System.Introspection.FieldInfo", "get_IsStatic", [], "Boolean", false),
        ("System.Introspection.FieldInfo", "get_DefinitionIndex", [], "Int32", false),
        ("System.Introspection.MethodInfo", "get_ReturnType", [], "System.Type", false),
        ("System.Introspection.MethodInfo", "get_IsStatic", [], "Boolean", false),
        ("System.Introspection.MethodInfo", "get_IsPublic", [], "Boolean", false),
        ("System.Introspection.MethodInfo", "get_IsPrivate", [], "Boolean", false),
        ("System.Introspection.MethodInfo", "get_IsAssembly", [], "Boolean", false),
        ("System.Introspection.MethodInfo", "get_IsReceiverByRef", [], "Boolean", false),
        ("System.Introspection.MethodInfo", "get_DefinitionIndex", [], "Int32", false),
        ("System.Introspection.MethodInfo", "GetParameters", [], "System.Introspection.ParameterInfo[]", false),
        ("System.Introspection.MethodInfo", "get_IsReadOnly", [], "Boolean", false),
        ("System.Introspection.MethodInfo", "get_IsVirtual", [], "Boolean", false),
        ("System.Introspection.MethodInfo", "get_IsOverride", [], "Boolean", false),
        ("System.Introspection.MethodInfo", "get_IsAbstract", [], "Boolean", false),
        ("System.Introspection.PropertyInfo", "get_PropertyType", [], "System.Type", false),
        ("System.Introspection.PropertyInfo", "get_IsStatic", [], "Boolean", false),
        ("System.Introspection.PropertyInfo", "get_CanRead", [], "Boolean", false),
        ("System.Introspection.PropertyInfo", "get_CanWrite", [], "Boolean", false),
        ("System.Introspection.PropertyInfo", "get_DefinitionIndex", [], "Int32", false),
        ("System.Introspection.PropertyInfo", "GetIndexParameters", [], "System.Introspection.ParameterInfo[]", false),
        ("System.Introspection.PropertyInfo", "GetGetMethod", [], "System.Option<System.Introspection.MethodInfo>", false),
        ("System.Introspection.PropertyInfo", "GetGetMethod", ["Boolean"], "System.Option<System.Introspection.MethodInfo>", false),
        ("System.Introspection.PropertyInfo", "GetSetMethod", [], "System.Option<System.Introspection.MethodInfo>", false),
        ("System.Introspection.PropertyInfo", "GetSetMethod", ["Boolean"], "System.Option<System.Introspection.MethodInfo>", false),
        ("System.Introspection.TypeInfo", "GetInterfaces", [], "System.Type[]", false),
        ("System.Introspection.TypeInfo", "GetEnumNames", [], "String[]", false),
        ("System.Introspection.TypeInfo", "GetEnumUnderlyingType", [], "System.Type", false),
        ("System.Introspection.TypeInfo", "get_BaseType", [], "System.Option<System.Type>", false),
    ];
    public const string Declarations = """
        public struct RuntimeTypeHandle { }
        public class Type { internal Type() { } public System.Introspection.TypeInfo Info => default; public string Name => default; public int GenericArgumentCount => default; public bool IsEnum => default; public bool IsArray => default; public bool IsAbstract => default; public bool IsReadOnly => default; public bool IsByRef => default; public bool IsPointer => default; public bool IsValueType => default; public bool IsInterface => default; public string FullName => default; public string Namespace => default; public static System.Type GetTypeFromHandle(System.RuntimeTypeHandle handle) => default; public System.Type GetGenericArgument(int index) => default; public bool Equals(System.Type other) => default; public System.Type[] GetGenericArguments() => default; public System.Option<System.Type> GetElementType() => default; }
        namespace Introspection { public interface TypeInfo { System.Option<System.Type> BaseType { get; } System.Type[] GetInterfaces(); string[] GetEnumNames(); System.Type GetEnumUnderlyingType(); FieldInfo[] GetFields(); FieldInfo[] GetFields(BindingFlags flags); MethodInfo[] GetMethods(); MethodInfo[] GetMethods(BindingFlags flags); PropertyInfo[] GetProperties(); PropertyInfo[] GetProperties(BindingFlags flags); } }
        namespace Introspection { public interface ParameterInfo { string Name { get; } int Position { get; } System.Type ParameterType { get; } bool IsOut { get; } bool IsOutWhenTrue { get; } bool IsReadOnly { get; } } }
        namespace Introspection { public interface MemberInfo { string Name { get; } System.Type DeclaringType { get; } } }
        namespace Introspection { public interface FieldInfo : MemberInfo { System.Type FieldType { get; } bool IsPublic { get; } bool IsPrivate { get; } bool IsAssembly { get; } bool IsStatic { get; } int DefinitionIndex { get; } } }
        namespace Introspection { public interface MethodInfo : MemberInfo { System.Type ReturnType { get; } bool IsStatic { get; } bool IsPublic { get; } bool IsPrivate { get; } bool IsAssembly { get; } bool IsReceiverByRef { get; } int DefinitionIndex { get; } bool IsReadOnly { get; } bool IsVirtual { get; } bool IsOverride { get; } bool IsAbstract { get; } System.Introspection.ParameterInfo[] GetParameters(); } }
        namespace Introspection { public interface PropertyInfo : MemberInfo { System.Type PropertyType { get; } bool IsStatic { get; } bool CanRead { get; } bool CanWrite { get; } int DefinitionIndex { get; } System.Introspection.ParameterInfo[] GetIndexParameters(); System.Option<System.Introspection.MethodInfo> GetGetMethod(); System.Option<System.Introspection.MethodInfo> GetGetMethod(bool arg0); System.Option<System.Introspection.MethodInfo> GetSetMethod(); System.Option<System.Introspection.MethodInfo> GetSetMethod(bool arg0); } }
        namespace Introspection { [Flags] public enum BindingFlags { Default = 0, DeclaredOnly = 2, Instance = 4, Static = 8, Public = 16, NonPublic = 32 } }
        """;
    public const string ProviderDeclarations = """
        namespace Introspection { internal sealed class RuntimeTypeInfo : TypeInfo { private RuntimeTypeInfo() { } internal static TypeInfo FromHandle(System.RuntimeTypeHandle handle) => default; public System.Option<System.Type> BaseType => default; public System.Type[] GetInterfaces() => default; public string[] GetEnumNames() => default; public System.Type GetEnumUnderlyingType() => default; public FieldInfo[] GetFields() => default; public FieldInfo[] GetFields(BindingFlags flags) => default; public MethodInfo[] GetMethods() => default; public MethodInfo[] GetMethods(BindingFlags flags) => default; public PropertyInfo[] GetProperties() => default; public PropertyInfo[] GetProperties(BindingFlags flags) => default; } }
        namespace Introspection { internal sealed class RuntimeParameterInfo : ParameterInfo { private RuntimeParameterInfo() { } public string Name => default; public int Position => default; public System.Type ParameterType => default; public bool IsOut => default; public bool IsOutWhenTrue => default; public bool IsReadOnly => default; } }
        namespace Introspection { internal abstract class RuntimeMemberInfo { protected RuntimeMemberInfo() { } public string Name => default; public System.Type DeclaringType => default; } }
        namespace Introspection { internal sealed class RuntimeFieldInfo : RuntimeMemberInfo, FieldInfo { private RuntimeFieldInfo() { } public System.Type FieldType => default; public bool IsPublic => default; public bool IsPrivate => default; public bool IsAssembly => default; public bool IsStatic => default; public int DefinitionIndex => default; } }
        namespace Introspection { internal sealed class RuntimeMethodInfo : RuntimeMemberInfo, MethodInfo { private RuntimeMethodInfo() { } public System.Type ReturnType => default; public bool IsStatic => default; public bool IsPublic => default; public bool IsPrivate => default; public bool IsAssembly => default; public bool IsReceiverByRef => default; public int DefinitionIndex => default; public bool IsReadOnly => default; public bool IsVirtual => default; public bool IsOverride => default; public bool IsAbstract => default; public System.Introspection.ParameterInfo[] GetParameters() => default; } }
        namespace Introspection { internal sealed class RuntimePropertyInfo : RuntimeMemberInfo, PropertyInfo { private RuntimePropertyInfo() { } public System.Type PropertyType => default; public bool IsStatic => default; public bool CanRead => default; public bool CanWrite => default; public int DefinitionIndex => default; public System.Introspection.ParameterInfo[] GetIndexParameters() => default; public System.Option<System.Introspection.MethodInfo> GetGetMethod() => default; public System.Option<System.Introspection.MethodInfo> GetGetMethod(bool arg0) => default; public System.Option<System.Introspection.MethodInfo> GetSetMethod() => default; public System.Option<System.Introspection.MethodInfo> GetSetMethod(bool arg0) => default; } }
        """;
    static readonly Dictionary<string, string> Helpers = new();
    public static void Reset() => Helpers.Clear();
    public static string Adapters => string.Join("\n", Helpers.Values);
    public static bool IsReference(string type) => ReferenceTypes.Contains(type);
    public static bool IsArray(string type) => ReferenceTypes.Any(t => type == $"arrayref<{t}>");
    public static bool IsType(string type) => IsReference(type) || IsArray(type) || type is "System.RuntimeTypeHandle" or "System.Introspection.BindingFlags";
    public static bool Assignable(string source, string target) => source == target || target == "System.Introspection.MemberInfo" && source is "System.Introspection.FieldInfo" or "System.Introspection.MethodInfo" or "System.Introspection.PropertyInfo";
    public static string? Type(TypeReference type)
    {
        if (type is ArrayType { IsVector: true } array && ReferenceTypes.Contains(array.ElementType.FullName)) return $"arrayref<{array.ElementType.FullName}>";
        if (!RuntimeSignatures.IsCore(type.Scope)) return null;
        if (ReferenceTypes.Contains(type.FullName) && !type.IsValueType) return type.FullName;
        return type.IsValueType && type.FullName is "System.RuntimeTypeHandle" or "System.Introspection.BindingFlags" ? type.FullName : null;
    }
    public static void Validate(ModuleDefinition module)
    {
        EnumBindings.Validate(module);
        foreach (var name in ReferenceTypes)
        {
            var type = module.GetType(name) ?? throw new InvalidDataException("Missing reflection type: " + name);
            IntrospectionHierarchy.Validate(type);
            var isInfo = name != "System.Type";
            var parents = name is "System.Introspection.FieldInfo" or "System.Introspection.MethodInfo" or "System.Introspection.PropertyInfo"
                ? new[] { "System.Introspection.MemberInfo" } : Array.Empty<string>();
            if (!type.IsPublic || type.IsValueType || type.IsInterface != isInfo || type.HasGenericParameters
                || type.BaseType?.FullName != (isInfo ? null : "System.Object")
                || type.IsAbstract != isInfo || isInfo && !type.Interfaces.Select(i => i.InterfaceType.FullName).SequenceEqual(parents))
                throw new InvalidDataException("Unsupported reflection contract: " + name);
        }
    }
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        var owner = Type(reference.DeclaringType);
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
            for (var i = 0; i < inputs.Length; i++)
            {
                body.AppendLine("ldarg arg" + i);
            }
            body.AppendLine($"{(definition.DeclaringType.IsInterface ? "callvirt" : "call")} {(reference.HasThis ? "instance " : "")}{owner}::{reference.Name}({string.Join(',', args)})");
            body.AppendLine("ret\n.end"); Helpers.Add(key, body.ToString());
        }
        return new(name, inputs, result);
    }
}
