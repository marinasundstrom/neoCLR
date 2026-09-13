using Mono.Cecil;
using System.Text;
using System.Text.RegularExpressions;

// Per-import application metadata. No host type loading or per-application API catalog.
static class ApplicationTypes
{
    static ModuleDefinition? Module;
    static readonly Dictionary<string, TypeDefinition> Types = new();
    static readonly HashSet<string> Expanded = new();
    static readonly Dictionary<string, Dictionary<string, string>> Adapters = new();
    public static void Reset(ModuleDefinition module) { Module = module; Types.Clear(); Expanded.Clear(); Adapters.Clear(); }
    public static bool IsType(string name) => Types.ContainsKey(name);
    public static bool IsReference(string name) => Types.TryGetValue(name, out var type) && !type.IsValueType;
    public static string? Type(TypeReference reference)
    {
        if (reference is ByReferenceType byref) return Type(byref.ElementType) is { } element ? element + "&" : null;
        if (reference is TypeSpecification || reference.Scope is AssemblyNameReference assembly && assembly.FullName != Module?.Assembly.Name.FullName) return null;
        var type = reference.Resolve();
        if (type is null || type.Module != Module || type.FullName == "System.Unit" || type.Name == "<Module>") return null;
        if (type.HasGenericParameters || type.IsEnum
            || type.IsExplicitLayout || (type.DeclaringType?.HasGenericParameters ?? false) || type.IsValueType && type.HasInterfaces
            || (!type.IsInterface && type.BaseType?.FullName is not ("System.Object" or "System.ValueType") && type.BaseType?.Resolve()?.Module != Module)
            || type.Fields.Any(f => f.IsStatic || f.HasMarshalInfo || f.IsInitOnly)
            || type.Methods.Any(m => m.IsConstructor && m.IsStatic))
            throw new InvalidDataException("Unsupported application type: " + type.FullName);
        var name = $"Application.Type_{type.MetadataToken.ToUInt32():x8}";
        Types[name] = type;
        if (Types.Count > 128) throw new InvalidDataException("Application type limit exceeded.");
        return name;
    }
    public static bool IsInterface(string name) => Types.TryGetValue(name, out var type) && type.IsInterface;
    public static bool Assignable(string from, string to)
    {
        if (from == to) return true;
        if (!Types.TryGetValue(from, out var type) || type.IsValueType) return false;
        var visited = new HashSet<TypeDefinition>();
        bool Visit(TypeDefinition current)
        {
            if (!visited.Add(current)) return false;
            if (current.BaseType is { } parent && parent.Resolve()?.Module == Module
                && (Type(parent) == to || Visit(parent.Resolve()))) return true;
            return current.Interfaces.Any(i => GenericUnionBindings.Type(i.InterfaceType) == to ||
                i.InterfaceType.Resolve()?.Module == Module && Visit(i.InterfaceType.Resolve()));
        }
        return Visit(type);
    }
    public static void Expand(Func<TypeReference, bool, string> map, Queue<MethodDefinition> pending)
    {
        while (Types.Any(t => !Expanded.Contains(t.Key)))
        {
            var (name, type) = Types.First(t => !Expanded.Contains(t.Key)); Expanded.Add(name);
            if (type.BaseType?.Resolve()?.Module == Module) map(type.BaseType, false);
            foreach (var contract in type.Interfaces) map(contract.InterfaceType, false);
            foreach (var field in type.Fields) map(field.FieldType, false);
            foreach (var method in type.Methods.Where(m => !m.IsStatic))
            {
                CheckMethod(method);
                if (method.Overrides.Any(o => !method.IsPublic || o.Name != method.Name || o.DeclaringType.Resolve()?.IsInterface != true
                    || !o.Parameters.Select(p => map(RuntimeSignatures.Close(p.ParameterType, o.DeclaringType), false)).SequenceEqual(method.Parameters.Select(p => map(p.ParameterType, false)))
                    || map(RuntimeSignatures.Close(o.ReturnType, o.DeclaringType), true) != map(method.ReturnType, true)) || method.IsFinal && !method.IsNewSlot) throw new InvalidDataException("Explicit implementations and sealed overrides are not admitted yet.");
                foreach (var parameter in method.Parameters) map(parameter.ParameterType, false);
                map(method.ReturnType, true);
                if (type.IsInterface)
                {
                    if (!method.IsAbstract || !method.IsPublic || !method.IsVirtual || method.HasBody)
                        throw new InvalidDataException("Only abstract public interface contracts are admitted.");
                }
                else if (!method.IsAbstract) pending.Enqueue(method);
            }
        }
    }
    public static string Modifiers(MethodDefinition method) => method.IsAbstract ? "abstract " :
        method.IsVirtual && !method.IsFinal ? (method.IsNewSlot ? "virtual " : "override ") : "";
    public static string MethodName(MethodDefinition method)
    {
        if (method.IsConstructor) return ".ctor";
        if (!Regex.IsMatch(method.Name, @"^[A-Za-z_][A-Za-z0-9_]*$")) return $"Generated_{method.MetadataToken.ToUInt32():x8}";
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
    public static void AddAdapter(string owner, string name, string body)
    {
        if (!Adapters.TryGetValue(owner, out var methods)) Adapters[owner] = methods = new();
        methods[name] = body;
    }
    public static string Declarations(Func<TypeReference, bool, string> map, Dictionary<MethodDefinition, string> bodies)
    {
        var output = new StringBuilder();
        var emitted = new HashSet<string>();
        while (Types.Any(t => !emitted.Contains(t.Key)))
        {
            var (name, type) = Types.First(t => !emitted.Contains(t.Key)); emitted.Add(name);
            output.AppendLine(type.IsInterface ? $".interface {name}" : $".type {(type.IsValueType ? "" : "class ")}{(type.IsAbstract ? "abstract " : "")}{name}");
            if (type.BaseType?.Resolve()?.Module == Module) output.AppendLine(".extends " + map(type.BaseType, false));
            foreach (var contract in type.Interfaces) output.AppendLine(".implements " + map(contract.InterfaceType, false));
            foreach (var method in type.Methods.Where(m => m.IsAbstract))
                output.AppendLine($".method instance {(type.IsInterface ? "" : "abstract ")}{MethodName(method)}({string.Join(',', method.Parameters.Select(p => map(p.ParameterType, false)))}) -> {map(method.ReturnType, true)}\n.end");
            foreach (var field in type.Fields)
                output.AppendLine($".field Field_{field.MetadataToken.ToUInt32():x8} {map(field.FieldType, false)}");
            foreach (var body in bodies.Where(p => p.Key.DeclaringType == type)) output.Append(body.Value);
            if (Adapters.TryGetValue(name, out var adapters)) foreach (var body in adapters.Values) output.Append(body);
            output.AppendLine(".end");
        }
        return output.ToString();
    }
}
