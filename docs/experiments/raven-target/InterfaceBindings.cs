using Mono.Cecil;

static class InterfaceBindings
{
    public const string Declarations = """
        public interface Equatable<T> { bool Equals(T other); }
        public interface Comparable<T> { int CompareTo(T other); }
        public interface Clonable<T> { T Clone(); }
        public interface Closable<E> { Result<PropagationUnit,E> Close(); }
        """;
    static readonly HashSet<string> Contracts = new() { "System.Equatable", "System.Comparable", "System.Clonable", "System.Closable" };
    public static bool IsInterface(string type) => (type == "System.Clock" || StreamBindings.IsCapability(type) || type == StorageProviderBindings.Name) || Contracts.Any(c => type.StartsWith(c + "<", StringComparison.Ordinal));
    public static string? Type(TypeReference type, Func<TypeReference, string>? parameterMap = null)
    {
        if (!RuntimeSignatures.IsCore(type.Scope)) return null;
        if ((type.FullName == "System.Clock" || StreamBindings.IsCapability(type.FullName) || type.FullName == StorageProviderBindings.Name) && !type.IsValueType) return type.FullName;
        if (type.FullName == "System.Object") return "System.Object";
        if (type is not GenericInstanceType g || g.IsValueType || g.GenericArguments.Count != 1) return null;
        var name = g.ElementType.FullName.Split('`')[0];
        if (!Contracts.Contains(name)) return null;
        var element = parameterMap?.Invoke(g.GenericArguments[0]) ?? ReflectionBindings.Type(g.GenericArguments[0]) ?? GenericUnionBindings.Type(g.GenericArguments[0]);
        return element is null ? null : name + "<" + element + ">";
    }
    public static bool Converts(string source, string target) => StreamBindings.Assignable(source, target) || source == "String" && target == "System.Collections.Iterable<Char>" || IsInterface(target)
        && (source == "System.Object" || source == "String" || ReflectionBindings.IsReference(source) || CalendarBindings.IsReference(source));
    public static string Convert(string source, string target) => Converts(source,target) ? "castclass " + target + "\n" : "";
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (reference.DeclaringType.FullName == "System.Clock") return CalendarBindings.Bind(reference, definition);
        if (reference.DeclaringType.FullName == StorageProviderBindings.Name) return StorageProviderBindings.Bind(reference, definition);
        if (StreamBindings.IsCapability(reference.DeclaringType.FullName)) return StreamBindings.BindCapability(reference, definition);
        var owner = Type(reference.DeclaringType);
        if (owner is null || !IsInterface(owner)) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, t => Type(t) ?? ReflectionBindings.Type(t) ?? GenericUnionBindings.Type(t));
        var element = owner[(owner.IndexOf('<')+1)..^1];
        var name = owner[..owner.IndexOf('<')];
        var expected = name switch {
            "System.Equatable" => ("Equals", element, "Boolean"),
            "System.Comparable" => ("CompareTo", element, "Int32"),
            "System.Clonable" => ("Clone", "", element),
            "System.Closable" => ("Close", "", "System.Result<Void," + element + ">"),
            _ => throw new InvalidDataException("Unsupported interface")
        };
        if (!definition.DeclaringType.IsInterface || !definition.IsAbstract || !definition.IsVirtual
            || !reference.HasThis || reference.Name != expected.Item1 || string.Join(',',args) != expected.Item2 || result != expected.Item3)
            throw new InvalidDataException("Unsupported interface member contract");
        return new(owner + "::" + reference.Name, new[]{owner}.Concat(args).ToArray(), result,
            Instruction: $"callvirt instance {owner}::{reference.Name}({string.Join(',',args)})");
    }
    public static void Validate(ModuleDefinition module)
    {
        StreamBindings.ValidateCapabilities(module);
        StorageProviderBindings.Validate(module);
        foreach (var name in Contracts) {
            var type = module.GetType(name + "`1");
            if (type is null || !type.IsInterface || type.GenericParameters.Count != 1
                || type.GenericParameters[0].Attributes != GenericParameterAttributes.NonVariant
                || type.GenericParameters[0].HasConstraints || type.Interfaces.Count != 0 || type.Fields.Count != 0
                || type.Methods.Count != 1 || type.Methods.Any(m => !m.IsPublic || !m.IsAbstract || !m.IsVirtual || !m.IsNewSlot || m.HasBody))
                throw new InvalidDataException("Unsupported fundamental interface metadata: " + name);
        }
    }
    public static string Project(string source)
    {
        foreach (var name in new[]{"SByte","Byte","Int16","UInt16","Char","Int32","UInt32","Int64","UInt64","IntPtr","UIntPtr","Single","Double","Boolean"})
            source = source.Replace("public struct " + name + " {", "public struct " + name + " : Comparable<" + name + ">" + (name == "Int32" ? ", Equatable<Int32>" : "") + " {");
        foreach (var name in new[]{"Date","Time","Instant","Duration"})
            source = source.Replace("public struct " + name + " {", "public struct " + name + " : Comparable<" + name + ">, Equatable<" + name + "> {");
        source = source.Replace("public sealed class String {", "public sealed class String : Equatable<String>, Collections.Iterable<char> {");
        source = source.Replace("public class Type {", "public class Type : Equatable<Type> {");
        return source;
    }
}
