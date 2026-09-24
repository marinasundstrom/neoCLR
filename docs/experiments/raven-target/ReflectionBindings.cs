using Mono.Cecil;
using System.Text;

// Current reflection API catalog. Placeholder metadata bodies never execute.
static class ReflectionBindings
{
    public static readonly string[] ReferenceTypes = ["System.Introspection.AssemblyInfo", "System.Introspection.ModuleInfo", "System.Runtime.RuntimeContext", "System.Introspection.TypeInfo", "System.Introspection.ParameterInfo", "System.Introspection.MemberInfo", "System.Introspection.FieldInfo", "System.Introspection.MethodInfo", "System.Introspection.PropertyInfo"];
    static readonly (string Owner, string Name, string[] Args, string Result, bool Static)[] Members = [
        ("System.Introspection.ParameterInfo", "get_MetadataToken", [], "Int32", false),
        ("System.Introspection.ParameterInfo", "get_Module", [], "System.Introspection.ModuleInfo", false),
        ("System.Introspection.MemberInfo", "get_MetadataToken", [], "Int32", false),
        ("System.Introspection.MemberInfo", "get_Module", [], "System.Introspection.ModuleInfo", false),
        ("System.Runtime.RuntimeContext", "get_ExecutingAssembly", [], "System.Introspection.AssemblyInfo", false),
        ("System.Introspection.AssemblyInfo", "get_Name", [], "String", false),
        ("System.Introspection.AssemblyInfo", "get_FullName", [], "String", false),
        ("System.Introspection.AssemblyInfo", "get_MetadataToken", [], "Int32", false),
        ("System.Introspection.AssemblyInfo", "get_ReferencedAssemblies", [], "System.Collections.Sequence<System.Introspection.AssemblyInfo>", false),
        ("System.Introspection.AssemblyInfo", "GetModules", [], "System.Collections.Sequence<System.Introspection.ModuleInfo>", false),
        ("System.Introspection.AssemblyInfo", "GetTypes", [], "System.Collections.Sequence<System.Introspection.TypeInfo>", false),
        ("System.Introspection.ModuleInfo", "get_Name", [], "String", false),
        ("System.Introspection.ModuleInfo", "get_Assembly", [], "System.Introspection.AssemblyInfo", false),
        ("System.Introspection.ModuleInfo", "get_MetadataToken", [], "Int32", false),
        ("System.Introspection.ModuleInfo", "GetTypes", [], "System.Collections.Sequence<System.Introspection.TypeInfo>", false),
        ("System.Object", "GetType", [], "System.Introspection.TypeInfo", false),
        ("System.Object", "ToString", [], "String", false),
        ("System.Object", "Equals", ["System.Object"], "Boolean", false),
        ("System.Object", "GetHashCode", [], "Int32", false),
        ("System.Object", "ReferenceEquals", ["System.Object", "System.Object"], "Boolean", true),
        ("System.Runtime.RuntimeContext", "get_Current", [], "System.Runtime.RuntimeContext", true),
        ("System.Runtime.RuntimeContext", "GetTypeInfoFromHandle", ["System.RuntimeTypeHandle"], "System.Introspection.TypeInfo", false),
        ("System.Introspection.TypeInfo", "get_GenericArgumentCount", [], "Int32", false),
        ("System.Introspection.TypeInfo", "GetGenericArgument", ["Int32"], "System.Introspection.TypeInfo", false),
        ("System.Introspection.TypeInfo", "Equals", ["System.Introspection.TypeInfo"], "Boolean", false),
        ("System.Introspection.TypeInfo", "GetFields", [], "System.Collections.Sequence<System.Introspection.FieldInfo>", false),
        ("System.Introspection.TypeInfo", "GetFields", ["System.Introspection.BindingFlags"], "System.Collections.Sequence<System.Introspection.FieldInfo>", false),
        ("System.Introspection.TypeInfo", "GetMethods", [], "System.Collections.Sequence<System.Introspection.MethodInfo>", false),
        ("System.Introspection.TypeInfo", "GetMethods", ["System.Introspection.BindingFlags"], "System.Collections.Sequence<System.Introspection.MethodInfo>", false),
        ("System.Introspection.TypeInfo", "GetProperties", [], "System.Collections.Sequence<System.Introspection.PropertyInfo>", false),
        ("System.Introspection.TypeInfo", "GetProperties", ["System.Introspection.BindingFlags"], "System.Collections.Sequence<System.Introspection.PropertyInfo>", false),
        ("System.Introspection.TypeInfo", "GetGenericArguments", [], "System.Collections.Sequence<System.Introspection.TypeInfo>", false),
        ("System.Introspection.TypeInfo", "GetElementType", [], "System.Option<System.Introspection.TypeInfo>", false),
        ("System.Introspection.TypeInfo", "get_IsValueType", [], "Boolean", false),
        ("System.Introspection.TypeInfo", "get_IsEnum", [], "Boolean", false),
        ("System.Introspection.TypeInfo", "get_IsArray", [], "Boolean", false),
        ("System.Introspection.TypeInfo", "get_IsAbstract", [], "Boolean", false),
        ("System.Introspection.TypeInfo", "get_IsOpen", [], "Boolean", false),
        ("System.Introspection.TypeInfo", "get_IsClosedHierarchy", [], "Boolean", false),
        ("System.Introspection.TypeInfo", "get_IsUnion", [], "Boolean", false),
        ("System.Introspection.TypeInfo", "get_IsReadOnly", [], "Boolean", false),
        ("System.Introspection.TypeInfo", "get_IsByRef", [], "Boolean", false),
        ("System.Introspection.TypeInfo", "get_IsPointer", [], "Boolean", false),
        ("System.Introspection.TypeInfo", "get_IsInterface", [], "Boolean", false),
        ("System.Introspection.TypeInfo", "get_FullName", [], "String", false),
        ("System.Introspection.TypeInfo", "get_Namespace", [], "String", false),
        ("System.Introspection.ParameterInfo", "get_Name", [], "String", false),
        ("System.Introspection.ParameterInfo", "get_Position", [], "Int32", false),
        ("System.Introspection.ParameterInfo", "get_ParameterType", [], "System.Introspection.TypeInfo", false),
        ("System.Introspection.ParameterInfo", "get_IsOut", [], "Boolean", false),
        ("System.Introspection.ParameterInfo", "get_IsOutWhenTrue", [], "Boolean", false),
        ("System.Introspection.ParameterInfo", "get_IsReadOnly", [], "Boolean", false),
        ("System.Introspection.MemberInfo", "get_Name", [], "String", false),
        ("System.Introspection.MemberInfo", "get_DeclaringType", [], "System.Option<System.Introspection.TypeInfo>", false),
        ("System.Introspection.FieldInfo", "get_FieldType", [], "System.Introspection.TypeInfo", false),
        ("System.Introspection.FieldInfo", "get_IsPublic", [], "Boolean", false),
        ("System.Introspection.FieldInfo", "get_IsPrivate", [], "Boolean", false),
        ("System.Introspection.FieldInfo", "get_IsAssembly", [], "Boolean", false),
        ("System.Introspection.FieldInfo", "get_IsStatic", [], "Boolean", false),
        ("System.Introspection.FieldInfo", "get_DefinitionIndex", [], "Int32", false),
        ("System.Introspection.MethodInfo", "get_ReturnType", [], "System.Introspection.TypeInfo", false),
        ("System.Introspection.MethodInfo", "get_IsStatic", [], "Boolean", false),
        ("System.Introspection.MethodInfo", "get_IsPublic", [], "Boolean", false),
        ("System.Introspection.MethodInfo", "get_IsPrivate", [], "Boolean", false),
        ("System.Introspection.MethodInfo", "get_IsAssembly", [], "Boolean", false),
        ("System.Introspection.MethodInfo", "get_IsReceiverByRef", [], "Boolean", false),
        ("System.Introspection.MethodInfo", "get_DefinitionIndex", [], "Int32", false),
        ("System.Introspection.MethodInfo", "GetParameters", [], "System.Collections.Sequence<System.Introspection.ParameterInfo>", false),
        ("System.Introspection.MethodInfo", "get_IsReadOnly", [], "Boolean", false),
        ("System.Introspection.MethodInfo", "get_IsVirtual", [], "Boolean", false),
        ("System.Introspection.MethodInfo", "get_IsOverride", [], "Boolean", false),
        ("System.Introspection.MethodInfo", "get_IsAbstract", [], "Boolean", false),
        ("System.Introspection.PropertyInfo", "get_PropertyType", [], "System.Introspection.TypeInfo", false),
        ("System.Introspection.PropertyInfo", "get_IsStatic", [], "Boolean", false),
        ("System.Introspection.PropertyInfo", "get_CanRead", [], "Boolean", false),
        ("System.Introspection.PropertyInfo", "get_CanWrite", [], "Boolean", false),
        ("System.Introspection.PropertyInfo", "get_DefinitionIndex", [], "Int32", false),
        ("System.Introspection.PropertyInfo", "GetIndexParameters", [], "System.Collections.Sequence<System.Introspection.ParameterInfo>", false),
        ("System.Introspection.PropertyInfo", "GetGetMethod", [], "System.Option<System.Introspection.MethodInfo>", false),
        ("System.Introspection.PropertyInfo", "GetGetMethod", ["Boolean"], "System.Option<System.Introspection.MethodInfo>", false),
        ("System.Introspection.PropertyInfo", "GetSetMethod", [], "System.Option<System.Introspection.MethodInfo>", false),
        ("System.Introspection.PropertyInfo", "GetSetMethod", ["Boolean"], "System.Option<System.Introspection.MethodInfo>", false),
        ("System.Introspection.TypeInfo", "GetInterfaces", [], "System.Collections.Sequence<System.Introspection.TypeInfo>", false),
        ("System.Introspection.TypeInfo", "GetEnumNames", [], "System.Collections.Sequence<String>", false),
        ("System.Introspection.TypeInfo", "GetEnumUnderlyingType", [], "System.Introspection.TypeInfo", false),
        ("System.Introspection.TypeInfo", "get_BaseType", [], "System.Option<System.Introspection.TypeInfo>", false),
    ];
    public const string Declarations = """
        namespace Introspection { public interface ModuleInfo { string Name { get; } AssemblyInfo Assembly { get; } int MetadataToken { get; } System.Collections.Sequence<TypeInfo> GetTypes(); } }
        namespace Introspection { public interface AssemblyInfo { string Name { get; } string FullName { get; } int MetadataToken { get; } System.Collections.Sequence<AssemblyInfo> ReferencedAssemblies { get; } System.Collections.Sequence<ModuleInfo> GetModules(); System.Collections.Sequence<TypeInfo> GetTypes(); } }
        public struct RuntimeTypeHandle { }
        internal class Type { } // CLI custom-attribute type tokens only; no runtime API.
        namespace Runtime { public sealed class RuntimeContext { private RuntimeContext() { } public static RuntimeContext Current => default; public System.Introspection.AssemblyInfo ExecutingAssembly => default; public System.Introspection.TypeInfo GetTypeInfoFromHandle(System.RuntimeTypeHandle handle) => default; } }
        namespace Introspection { public interface TypeInfo : MemberInfo, System.Equatable<TypeInfo> { string FullName { get; } string Namespace { get; } int GenericArgumentCount { get; } bool IsArray { get; } bool IsByRef { get; } bool IsPointer { get; } bool IsInterface { get; } bool IsReadOnly { get; } bool IsAbstract { get; } bool IsOpen { get; } bool IsClosedHierarchy { get; } bool IsUnion { get; } bool IsEnum { get; } bool IsValueType { get; } TypeInfo GetGenericArgument(int index); bool Equals(TypeInfo other); System.Collections.Sequence<TypeInfo> GetGenericArguments(); System.Option<TypeInfo> GetElementType();  System.Option<System.Introspection.TypeInfo> BaseType { get; } System.Collections.Sequence<System.Introspection.TypeInfo> GetInterfaces(); System.Collections.Sequence<string> GetEnumNames(); System.Introspection.TypeInfo GetEnumUnderlyingType(); System.Collections.Sequence<FieldInfo> GetFields(); System.Collections.Sequence<FieldInfo> GetFields(BindingFlags flags); System.Collections.Sequence<MethodInfo> GetMethods(); System.Collections.Sequence<MethodInfo> GetMethods(BindingFlags flags); System.Collections.Sequence<PropertyInfo> GetProperties(); System.Collections.Sequence<PropertyInfo> GetProperties(BindingFlags flags); } }
        namespace Introspection { public interface ParameterInfo { int MetadataToken { get; } ModuleInfo Module { get; } string Name { get; } int Position { get; } System.Introspection.TypeInfo ParameterType { get; } bool IsOut { get; } bool IsOutWhenTrue { get; } bool IsReadOnly { get; } } }
        namespace Introspection { public interface MemberInfo { int MetadataToken { get; } ModuleInfo Module { get; } string Name { get; } System.Option<System.Introspection.TypeInfo> DeclaringType { get; } } }
        namespace Introspection { public interface FieldInfo : MemberInfo { System.Introspection.TypeInfo FieldType { get; } bool IsPublic { get; } bool IsPrivate { get; } bool IsAssembly { get; } bool IsStatic { get; } int DefinitionIndex { get; } } }
        namespace Introspection { public interface MethodInfo : MemberInfo { System.Introspection.TypeInfo ReturnType { get; } bool IsStatic { get; } bool IsPublic { get; } bool IsPrivate { get; } bool IsAssembly { get; } bool IsReceiverByRef { get; } int DefinitionIndex { get; } bool IsReadOnly { get; } bool IsVirtual { get; } bool IsOverride { get; } bool IsAbstract { get; } System.Collections.Sequence<System.Introspection.ParameterInfo> GetParameters(); } }
        namespace Introspection { public interface PropertyInfo : MemberInfo { System.Introspection.TypeInfo PropertyType { get; } bool IsStatic { get; } bool CanRead { get; } bool CanWrite { get; } int DefinitionIndex { get; } System.Collections.Sequence<System.Introspection.ParameterInfo> GetIndexParameters(); System.Option<System.Introspection.MethodInfo> GetGetMethod(); System.Option<System.Introspection.MethodInfo> GetGetMethod(bool arg0); System.Option<System.Introspection.MethodInfo> GetSetMethod(); System.Option<System.Introspection.MethodInfo> GetSetMethod(bool arg0); } }
        namespace Introspection { [Flags] public enum BindingFlags { Default = 0, DeclaredOnly = 2, Instance = 4, Static = 8, Public = 16, NonPublic = 32 } }
        """;
    public const string ProviderDeclarations = "\n" + """
        #nullable enable annotations
        namespace Introspection { internal sealed class RuntimeModuleInfo : ModuleInfo { private RuntimeModuleInfo() { } public string Name => default; public AssemblyInfo Assembly => default; public int MetadataToken => default; public System.Collections.Sequence<TypeInfo> GetTypes() => default; public override bool Equals(object? other) => default; public override int GetHashCode() => default; public override string ToString() => default; } }
        namespace Introspection { internal sealed class RuntimeAssemblyInfo : AssemblyInfo { private RuntimeAssemblyInfo() { } public string Name => default; public string FullName => default; public int MetadataToken => default; public System.Collections.Sequence<AssemblyInfo> ReferencedAssemblies => default; public System.Collections.Sequence<ModuleInfo> GetModules() => default; public System.Collections.Sequence<TypeInfo> GetTypes() => default; public override bool Equals(object? other) => default; public override int GetHashCode() => default; public override string ToString() => default; } }
        namespace Introspection { internal sealed class RuntimeTypeInfo : TypeInfo { public System.Option<TypeInfo> DeclaringType => default; public int MetadataToken => default; public ModuleInfo Module => default; public string Name => default; public string FullName => default; public string Namespace => default; public int GenericArgumentCount => default; public bool IsArray => default; public bool IsByRef => default; public bool IsPointer => default; public bool IsInterface => default; public bool IsReadOnly => default; public bool IsAbstract => default; public bool IsOpen => default; public bool IsClosedHierarchy => default; public bool IsUnion => default; public bool IsEnum => default; public bool IsValueType => default; public TypeInfo GetGenericArgument(int index) => default; public bool Equals(TypeInfo other) => default; public override bool Equals(object? other) => default; public override int GetHashCode() => default; public override string ToString() => default; public System.Collections.Sequence<TypeInfo> GetGenericArguments() => default; public System.Option<TypeInfo> GetElementType() => default;  private RuntimeTypeInfo() { } internal static TypeInfo FromHandle(System.RuntimeTypeHandle handle) => default; public System.Option<System.Introspection.TypeInfo> BaseType => default; public System.Collections.Sequence<System.Introspection.TypeInfo> GetInterfaces() => default; public System.Collections.Sequence<string> GetEnumNames() => default; public System.Introspection.TypeInfo GetEnumUnderlyingType() => default; public System.Collections.Sequence<FieldInfo> GetFields() => default; public System.Collections.Sequence<FieldInfo> GetFields(BindingFlags flags) => default; public System.Collections.Sequence<MethodInfo> GetMethods() => default; public System.Collections.Sequence<MethodInfo> GetMethods(BindingFlags flags) => default; public System.Collections.Sequence<PropertyInfo> GetProperties() => default; public System.Collections.Sequence<PropertyInfo> GetProperties(BindingFlags flags) => default; } }
        namespace Introspection { internal sealed class RuntimeParameterInfo : ParameterInfo { public int MetadataToken => default; public ModuleInfo Module => default; private RuntimeParameterInfo() { } public string Name => default; public int Position => default; public System.Introspection.TypeInfo ParameterType => default; public bool IsOut => default; public bool IsOutWhenTrue => default; public bool IsReadOnly => default; public override bool Equals(object? other) => default; public override int GetHashCode() => default; public override string ToString() => default; } }
        namespace Introspection { internal abstract class RuntimeMemberInfo { public int MetadataToken => default; public ModuleInfo Module => default; protected RuntimeMemberInfo() { } public string Name => default; public System.Option<System.Introspection.TypeInfo> DeclaringType => default; public override string ToString() => default; } }
        namespace Introspection { internal sealed class RuntimeFieldInfo : RuntimeMemberInfo, FieldInfo { private RuntimeFieldInfo() { } public System.Introspection.TypeInfo FieldType => default; public bool IsPublic => default; public bool IsPrivate => default; public bool IsAssembly => default; public bool IsStatic => default; public int DefinitionIndex => default; public override bool Equals(object? other) => default; public override int GetHashCode() => default; } }
        namespace Introspection { internal sealed class RuntimeMethodInfo : RuntimeMemberInfo, MethodInfo { private RuntimeMethodInfo() { } public System.Introspection.TypeInfo ReturnType => default; public bool IsStatic => default; public bool IsPublic => default; public bool IsPrivate => default; public bool IsAssembly => default; public bool IsReceiverByRef => default; public int DefinitionIndex => default; public bool IsReadOnly => default; public bool IsVirtual => default; public bool IsOverride => default; public bool IsAbstract => default; public System.Collections.Sequence<System.Introspection.ParameterInfo> GetParameters() => default; public override bool Equals(object? other) => default; public override int GetHashCode() => default; } }
        namespace Introspection { internal sealed class RuntimePropertyInfo : RuntimeMemberInfo, PropertyInfo { private RuntimePropertyInfo() { } public System.Introspection.TypeInfo PropertyType => default; public bool IsStatic => default; public bool CanRead => default; public bool CanWrite => default; public int DefinitionIndex => default; public System.Collections.Sequence<System.Introspection.ParameterInfo> GetIndexParameters() => default; public System.Option<System.Introspection.MethodInfo> GetGetMethod() => default; public System.Option<System.Introspection.MethodInfo> GetGetMethod(bool arg0) => default; public System.Option<System.Introspection.MethodInfo> GetSetMethod() => default; public System.Option<System.Introspection.MethodInfo> GetSetMethod(bool arg0) => default; public override bool Equals(object? other) => default; public override int GetHashCode() => default; } }
        #nullable restore annotations
        """ + "\n";
    static readonly Dictionary<string, string> Helpers = new();
    public static void Reset() => Helpers.Clear();
    public static string Adapters => string.Join("\n", Helpers.Values);
    public static bool IsReference(string type) => ReferenceTypes.Contains(type);
    public static bool IsArray(string type) => ReferenceTypes.Any(t => type == $"arrayref<{t}>");
    public static bool IsType(string type) => IsReference(type) || IsArray(type) || type is "System.RuntimeTypeHandle" or "System.Introspection.BindingFlags";
    public static bool Assignable(string source, string target) => source == target || target == "System.Object" && ManagedArrayBindings.IsReference(source) || target == "System.Introspection.MemberInfo" && source is "System.Introspection.FieldInfo" or "System.Introspection.MethodInfo" or "System.Introspection.PropertyInfo" or "System.Introspection.TypeInfo";
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
        EnumBindings.Validate(module, EnumBindings.TaskState);
        EnumBindings.Validate(module, EnumBindings.EntryKind);
        foreach (var name in ReferenceTypes)
        {
            var type = module.GetType(name) ?? throw new InvalidDataException("Missing reflection type: " + name);
            IntrospectionHierarchy.Validate(type);
            var isInfo = name != "System.Runtime.RuntimeContext";
            var parents = name is "System.Introspection.FieldInfo" or "System.Introspection.MethodInfo" or "System.Introspection.PropertyInfo"
                ? new[] { "System.Introspection.MemberInfo" } : name == "System.Introspection.TypeInfo" ? new[] { "System.Introspection.MemberInfo", "System.Equatable`1<System.Introspection.TypeInfo>" } : Array.Empty<string>();
            if (!type.IsPublic || type.IsValueType || type.IsInterface != isInfo || type.HasGenericParameters
                || type.BaseType?.FullName != (isInfo ? null : "System.Object")
                || type.IsAbstract != isInfo || isInfo && !type.Interfaces.Select(i => i.InterfaceType.FullName).SequenceEqual(parents))
                throw new InvalidDataException("Unsupported reflection contract: " + name);
        }
    }
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool virtualCall = false)
    {
        var owner = reference.DeclaringType.FullName == "System.Object" && reference.Name is "GetType" or "ToString" or "Equals" or "GetHashCode" or "ReferenceEquals" && RuntimeSignatures.IsCore(reference.DeclaringType.Scope) ? "System.Object" : Type(reference.DeclaringType);
        if (owner is null || IsArray(owner) || owner == "System.RuntimeTypeHandle") return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, t => t.FullName == "System.Object" && (t.MetadataType == MetadataType.Object || RuntimeSignatures.IsCore(t.Scope)) ? "System.Object" : Type(t) ?? CollectionBindings.Type(t) ?? ProcessBindings.ArrayType(t) ?? GenericUnionBindings.Type(t));
        string Project(string t) => t.EndsWith("[]") ? "arrayref<" + t[..^2] + ">" : t;
        if (!Members.Any(m => m.Owner == owner && m.Name == reference.Name && m.Static == !reference.HasThis
            && m.Args.Select(Project).SequenceEqual(args) && Project(m.Result) == result))
            throw new InvalidDataException("Unsupported reflection signature: " + reference.FullName);
        var inputs = reference.HasThis ? new[] { owner }.Concat(args).ToArray() : args;
        var key = reference.FullName + (owner == "System.Object" && definition.IsVirtual && virtualCall ? "#virtual" : "");
        var name = "RuntimeReflection" + Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(Encoding.UTF8.GetBytes(key)))[..16];
        if (!Helpers.ContainsKey(key))
        {
            var body = new StringBuilder($".function {name}({string.Join(',', inputs.Select((t,i) => t + " arg" + i))}) -> {result}\n");
            for (var i = 0; i < inputs.Length; i++)
            {
                body.AppendLine("ldarg arg" + i);
            }
            body.AppendLine($"{(definition.DeclaringType.IsInterface || definition.IsVirtual && virtualCall ? "callvirt" : "call")} {(reference.HasThis ? "instance " : "")}{owner}::{reference.Name}({string.Join(',', args)})");
            body.AppendLine("ret\n.end"); Helpers.Add(key, body.ToString());
        }
        return new(name, inputs, result);
    }
}
