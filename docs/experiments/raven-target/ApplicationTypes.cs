using Mono.Cecil;
using System.Text;

// Per-import application metadata. No host type loading or per-application API catalog.
static class ApplicationTypes
{
    static readonly HashSet<ModuleDefinition> Modules = new();
    static readonly Dictionary<string, TypeDefinition> Types = new();
    static readonly HashSet<string> Expanded = new();
    static readonly Dictionary<string, Dictionary<string, string>> Adapters = new();
    static readonly Dictionary<TypeDefinition, string> LibraryNames = new();
    public static void BindLibrary(TypeDefinition type, string name) => LibraryNames.Add(type, name);
    public static Func<TypeReference, string>? LibraryMap;
    static readonly Dictionary<string, TypeReference> LibraryReferenceTypes = new();
    static readonly Dictionary<string, TypeDefinition> LibraryReferences = new();
    static ModuleDefinition? LibraryModule;
    static string? LibraryScope;
    static readonly HashSet<TypeDefinition> LibraryDependencies = new();
    public static void SetLibraryScope(ModuleDefinition module, string scope) { LibraryModule = module; LibraryScope = scope; }
    static void RegisterLibraryDependency(TypeReference reference)
    {
        if (LibraryModule is null || reference is GenericParameter) return;
        var element = reference.GetElementType();
        if (element is GenericParameter || element.Scope is ModuleDefinition module && module != LibraryModule
            || element.Scope is AssemblyNameReference assembly && assembly.FullName != LibraryModule.Assembly.Name.FullName) return;
        var type = element.Resolve();
        if (type is null || type.Module != LibraryModule || type.IsPublic || LibraryNames.ContainsKey(type)) return;
        if (type.IsNested || type.IsValueType || type.IsInterface || type.IsAbstract || type.IsExplicitLayout
            || type.BaseType?.FullName != "System.Object" || type.HasEvents
            || type.GenericParameters.Any(p => p.HasConstraints || p.Attributes != GenericParameterAttributes.NonVariant)
            || type.Fields.Any(f => !f.IsPrivate || f.IsStatic || f.IsInitOnly || f.HasMarshalInfo)
            || type.Methods.Any(m => !m.IsPublic || !m.HasThis || !m.HasBody || m.HasGenericParameters
                || m.Parameters.Any(p => p.IsOut || p.ParameterType.IsByReference)))
            throw new InvalidDataException("Unsupported private library dependency: " + type.FullName);
        LibraryNames.Add(type, "neoCLR.Library." + LibraryScope + ".Type_" + Convert.ToHexString(Encoding.UTF8.GetBytes(type.FullName)));
        LibraryDependencies.Add(type);
    }
    public static bool IsLibraryDependency(TypeReference type) { RegisterLibraryDependency(type); return LibraryDependencies.Contains(type.Resolve()); }
    public static bool IsLibrary(TypeReference type) { RegisterLibraryDependency(type); return LibraryNames.ContainsKey(type.Resolve()); }
    public static bool IsLibraryParameter(GenericParameter parameter) => parameter.Type == GenericParameterType.Type
        && parameter.Owner is TypeDefinition owner && LibraryNames.ContainsKey(owner);
    public static bool OnlyLibraryTypes => Types.Values.All(LibraryNames.ContainsKey);
    public static void Reset(params ModuleDefinition[] modules) { LibraryModule = null; LibraryScope = null; LibraryDependencies.Clear(); LibraryMap = null; LibraryReferenceTypes.Clear(); LibraryReferences.Clear(); LibraryNames.Clear(); Modules.Clear(); Modules.UnionWith(modules); Types.Clear(); Expanded.Clear(); Adapters.Clear(); }
    public static object[] IdentityMap() => Types.Select(p => (object)new {
        AssemblyIdentity = p.Value.Module.Assembly.Name.FullName, MetadataName = p.Value.FullName, RuntimeName = p.Key,
        Fields = p.Value.Fields.Where(_ => !PrimitiveLibrary.IsMatched(p.Value)).Select(f => new { MetadataName = f.Name, RuntimeName = MetadataIdentity.MemberName(f.Name) }).ToArray()
    }).ToArray();
    public static bool IsModule(ModuleDefinition? module) => module is not null && Modules.Contains(module);
    public static void CheckAccess(TypeReference reference, ModuleDefinition caller)
    {
        if (reference is GenericInstanceType generic)
            foreach (var argument in generic.GenericArguments) CheckAccess(argument, caller);
        if (reference is TypeSpecification specification) { CheckAccess(specification.ElementType, caller); return; }
        if (reference is GenericParameter) return;
        if (reference.Scope is AssemblyNameReference assembly && !Modules.Any(m => m.Assembly.Name.FullName == assembly.FullName)) return;
        var type = reference.Resolve();
        if (type is null || !IsModule(type.Module) || type.Module == caller) return;
        for (var owner = type; owner is not null; owner = owner.DeclaringType)
            if (!(owner.IsPublic || owner.IsNestedPublic))
                throw new InvalidDataException("Nonpublic imported type access unsupported: " + type.FullName);
    }
    public static bool IsType(string name) => Types.ContainsKey(name) || LibraryReferences.ContainsKey(name);
    public static bool IsReference(string name) => (Types.TryGetValue(name, out var type) || LibraryReferences.TryGetValue(name, out type)) && !type.IsValueType;
    public static string? Type(TypeReference reference)
    {
        RegisterLibraryDependency(reference);
        if (reference is ByReferenceType byref) return Type(byref.ElementType) is { } element ? element + "&" : null;
        if (reference is GenericInstanceType instance && LibraryNames.TryGetValue(instance.ElementType.Resolve(), out var libraryName))
        {
            var definition = instance.ElementType.Resolve();
            if (instance.GenericArguments.Count != definition.GenericParameters.Count) throw new InvalidDataException("Invalid library type arity.");
            _ = Type(definition);
            var constructed = libraryName + "<" + string.Join(',', instance.GenericArguments.Select(t => LibraryMap!(t))) + ">";
            LibraryReferences[constructed] = definition;
            LibraryReferenceTypes[constructed] = instance;
            return constructed;
        }
        if (reference is TypeSpecification || reference.Scope is AssemblyNameReference assembly && !Modules.Any(m => m.Assembly.Name.FullName == assembly.FullName)) return null;
        var type = reference.Resolve();
        if (type is null || !Modules.Contains(type.Module) || type.FullName == "System.Unit" || type.Name == "<Module>") return null;
        if (type.HasGenericParameters && !LibraryNames.ContainsKey(type) || type.IsEnum
            || type.IsExplicitLayout || (type.DeclaringType?.HasGenericParameters ?? false) || type.IsValueType && type.HasInterfaces && !LibraryNames.ContainsKey(type)
            || (!type.IsInterface && type.BaseType?.FullName is not ("System.Object" or "System.ValueType") && !IsModule(type.BaseType?.Resolve()?.Module))
            || type.Fields.Any(f => f.IsStatic || f.HasMarshalInfo || f.IsInitOnly)
            || type.Methods.Any(m => m.IsConstructor && m.IsStatic))
            throw new InvalidDataException("Unsupported application type: " + type.FullName);
        var name = LibraryNames.TryGetValue(type, out var libraryOwner)
            ? libraryOwner + (type.HasGenericParameters ? "<" + string.Join(',', type.GenericParameters.Select(p => "T" + p.Position)) + ">" : "")
            : MetadataIdentity.TypeName(type);
        Types[name] = type;
        if (Types.Count > 128) throw new InvalidDataException("Application type limit exceeded.");
        return name;
    }
    public static bool IsInterface(string name) => Types.TryGetValue(name, out var type) && type.IsInterface;
    public static bool Assignable(string from, string to)
    {
        if (from == to) return true;
        if (!(Types.TryGetValue(from, out var type) || LibraryReferences.TryGetValue(from, out type)) || type.IsValueType) return false;
        if (LibraryNames.ContainsKey(type))
        {
            var owner = LibraryReferenceTypes.GetValueOrDefault(from) ?? type;
            return type.Interfaces.Any(i =>
            {
                var shape = LibraryMap!(Close(i.InterfaceType, owner));
                return shape == to || CollectionBindings.Assignable(shape, to);
            });
        }
        var visited = new HashSet<TypeDefinition>();
        bool Visit(TypeDefinition current)
        {
            if (!visited.Add(current)) return false;
            if (current.BaseType is { } parent && IsModule(parent.Resolve()?.Module)
                && (Type(parent) == to || Visit(parent.Resolve()))) return true;
            return current.Interfaces.Any(i => GenericUnionBindings.Type(i.InterfaceType) == to ||
                IsModule(i.InterfaceType.Resolve()?.Module) && Visit(i.InterfaceType.Resolve()));
        }
        return Visit(type);
    }
    public static void Expand(Func<TypeReference, bool, string> map, Queue<MethodDefinition> pending)
    {
        while (Types.Any(t => !Expanded.Contains(t.Key)))
        {
            var (name, type) = Types.First(t => !Expanded.Contains(t.Key)); Expanded.Add(name);
            if (IsModule(type.BaseType?.Resolve()?.Module)) { CheckAccess(type.BaseType!, type.Module); map(type.BaseType!, false); }
            foreach (var contract in type.Interfaces) { CheckAccess(contract.InterfaceType, type.Module); map(contract.InterfaceType, false); }
            foreach (var field in type.Fields) map(field.FieldType, false);
            foreach (var method in type.Methods.Where(m => !m.IsStatic && !(PrimitiveLibrary.IsMatched(type) && PrimitiveLibrary.IsDefaultConstructor(m))))
            {
                CheckMethod(method);
                if (method.Overrides.Any(o => !method.IsPublic || o.Name != method.Name || o.DeclaringType.Resolve()?.IsInterface != true
                    || !o.Parameters.Select(p => map(RuntimeSignatures.Close(p.ParameterType, o.DeclaringType, allowOpenMethodParameters: LibraryNames.ContainsKey(type)), false)).SequenceEqual(method.Parameters.Select(p => map(p.ParameterType, false)))
                    || map(RuntimeSignatures.Close(o.ReturnType, o.DeclaringType, allowOpenMethodParameters: LibraryNames.ContainsKey(type)), true) != map(method.ReturnType, true)) || method.IsFinal && !method.IsNewSlot) throw new InvalidDataException("Explicit implementations and sealed overrides are not admitted yet.");
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
        return MetadataIdentity.MemberName(method.Name);
    }
    public static void CheckMethod(MethodReference method)
    {
        if (method.ExplicitThis || method.HasGenericParameters || method is GenericInstanceMethod
            || (method.DeclaringType.HasGenericParameters || method.DeclaringType is GenericInstanceType) && !IsLibrary(method.DeclaringType)
            || method.CallingConvention != MethodCallingConvention.Default)
            throw new InvalidDataException("Unsupported application signature: " + method.FullName);
        if (method.HasThis && Type(method.DeclaringType) is null) throw new InvalidDataException("Unsupported application receiver.");
    }
    public static string Receiver(MethodReference method) => Type(method.DeclaringType)! + (method.DeclaringType.IsValueType && !LibraryImplementation.IsByValueReceiver(method) ? "&" : "");
    public sealed record FieldShape(string Owner, string Type, string Name, bool ValueOwner);
    public static FieldShape? Field(FieldReference reference, MethodDefinition caller, Func<TypeReference, bool, string> map)
    {
        var field = reference.Resolve();
        if (field is null || !Modules.Contains(field.Module)) return null;
        var owner = Type(reference.DeclaringType)!;
        if (field.IsStatic || (!IsLibrary(field.DeclaringType) && reference.FullName != field.FullName) || (!field.IsPublic && caller.DeclaringType != field.DeclaringType))
            throw new InvalidDataException("Unsupported application field access.");
        if (IsLibrary(field.DeclaringType) && !LibraryImplementation.SameType(
            Close(reference.FieldType, reference.DeclaringType), Close(field.FieldType, reference.DeclaringType)))
            throw new InvalidDataException("Invalid constructed library field signature.");
        return new(owner, map(Close(field.FieldType, reference.DeclaringType), false), MetadataIdentity.MemberName(field.Name), field.DeclaringType.IsValueType);
    }
    public static bool Matches(MethodReference reference, MethodDefinition definition)
    {
        if (!IsLibrary(definition.DeclaringType)) return reference.FullName == definition.FullName;
        var owner = reference.DeclaringType;
        return owner.Resolve() == definition.DeclaringType && reference.Name == definition.Name
            && reference.HasThis == definition.HasThis && reference.ExplicitThis == definition.ExplicitThis
            && reference.CallingConvention == definition.CallingConvention
            && reference.Parameters.Count == definition.Parameters.Count
            && LibraryImplementation.SameType(Close(reference.ReturnType, owner), Close(definition.ReturnType, owner))
            && reference.Parameters.Zip(definition.Parameters).All(p =>
                LibraryImplementation.SameType(Close(p.First.ParameterType, owner), Close(p.Second.ParameterType, owner)));
    }
    public static TypeReference Close(TypeReference type, TypeReference owner, int depth = 0)
    {
        if (depth > 32) throw new InvalidDataException("Library signature nesting limit exceeded.");
        if (owner is not GenericInstanceType instance || !IsLibrary(owner)) return type;
        if (type is GenericParameter p && p.Type == GenericParameterType.Type
            && p.Owner == instance.ElementType.Resolve()) return instance.GenericArguments[p.Position];
        if (type is ArrayType { IsVector: true } array) return new ArrayType(Close(array.ElementType, owner, depth + 1));
        if (type is GenericInstanceType generic)
        {
            var result = new GenericInstanceType(generic.ElementType);
            foreach (var argument in generic.GenericArguments) result.GenericArguments.Add(Close(argument, owner, depth + 1));
            return result;
        }
        return type;
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
            output.AppendLine(type.IsInterface ? $".interface {name}" : $".type {(LibraryDependencies.Contains(type) ? "internal " : "")}{(type.IsValueType ? "" : "class ")}{(type.IsAbstract ? "abstract " : "")}{name}");
            if (IsModule(type.BaseType?.Resolve()?.Module)) output.AppendLine(".extends " + map(type.BaseType, false));
            foreach (var contract in type.Interfaces) output.AppendLine(".implements " + map(contract.InterfaceType, false));
            foreach (var method in type.Methods.Where(m => m.IsAbstract))
                output.AppendLine($".method instance {(type.IsInterface ? "" : "abstract ")}{MethodName(method)}({string.Join(',', method.Parameters.Select(p => map(p.ParameterType, false) + (LibraryNames.ContainsKey(type) ? " " + p.Name : "")))}) -> {map(method.ReturnType, true)}\n.end");
            foreach (var field in type.Fields.Where(_ => !PrimitiveLibrary.IsMatched(type)))
                output.AppendLine($".field {(LibraryNames.ContainsKey(type) && field.IsPrivate ? "private " : "")}{MetadataIdentity.MemberName(field.Name)} {map(field.FieldType, false)}");
            if (LibraryNames.ContainsKey(type))
                foreach (var property in type.Properties)
                {
                    output.AppendLine($".property instance {MetadataIdentity.MemberName(property.Name)}({string.Join(',', property.Parameters.Select(p => map(p.ParameterType, false)))}) -> {map(property.PropertyType, false)}");
                    if (property.GetMethod is { } getter) output.AppendLine($".get instance {name}::{MethodName(getter)}({string.Join(',', getter.Parameters.Select(p => map(p.ParameterType, false)))})");
                    if (property.SetMethod is { } setter) output.AppendLine($".set instance {name}::{MethodName(setter)}({string.Join(',', setter.Parameters.Select(p => map(p.ParameterType, false)))})");
                    output.AppendLine(".end");
                }
            foreach (var body in bodies.Where(p => p.Key.DeclaringType == type)) output.Append(body.Value);
            if (Adapters.TryGetValue(name, out var adapters)) foreach (var body in adapters.Values) output.Append(body);
            output.AppendLine(".end");
        }
        return output.ToString();
    }
}
