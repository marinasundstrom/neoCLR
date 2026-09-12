using Mono.Cecil;

// Bounded executable catalog. This is not Raven's future language-contract configuration.
static class CollectionBindings
{
    public const string List = "System.Collections.List<Int32>";
    public const string ArrayList = "System.Collections.ArrayList<Int32>";
    public const string Iterable = "System.Collections.Iterable<Int32>";
    public const string Iterator = "System.Collections.Iterator<Int32>";
    public const string Disposable = "System.Disposable";
    public static bool IsReference(string type) => type is List or ArrayList or Iterable or Iterator or Disposable;
    public static bool Assignable(string source, string target) => source == target
        || source == ArrayList && target is List or Iterable
        || source == List && target == Iterable
        || source == Iterator && target == Disposable;

    public static string? Type(TypeReference type)
    {
        if (type.IsValueType) return null;
        if (type.FullName == Disposable && type.Scope.Name == CoreDeclarations.Identity) return Disposable;
        if (type is not GenericInstanceType g || g.GenericArguments.Count != 1
            || g.GenericArguments[0].MetadataType != MetadataType.Int32
            || g.ElementType.Scope.Name != CoreDeclarations.Identity) return null;
        return g.ElementType.FullName switch {
            "System.Collections.List`1" => List, "System.Collections.ArrayList`1" => ArrayList,
            "System.Collections.Iterable`1" => Iterable, "System.Collections.Iterator`1" => Iterator,
            _ => null
        };
    }

    public static void Validate(ModuleDefinition module)
    {
        foreach (var (name, parent) in new[] {
            ("System.Disposable", ""), ("System.Collections.Iterable`1", ""),
            ("System.Collections.Iterator`1", "System.Disposable"),
            ("System.Collections.List`1", "System.Collections.Iterable`1<T>"),
            ("System.Collections.ArrayList`1", "System.Collections.List`1<T>|System.Collections.Iterable`1<T>") })
        {
            var type = module.GetType(name) ?? throw new InvalidDataException("Missing collection contract: " + name);
            if (type.IsInterface && type.Methods.Any(m => !m.IsPublic || !m.IsAbstract || !m.IsVirtual || !m.IsNewSlot || !m.HasThis || m.HasBody))
                throw new InvalidDataException("Incompatible collection interface members: " + name);
            if (type.IsValueType || type.IsInterface != (name != "System.Collections.ArrayList`1")
                || type.GenericParameters.Count != (name.Contains('`') ? 1 : 0)
                || type.GenericParameters.Any(p => p.Attributes != GenericParameterAttributes.NonVariant || p.HasConstraints)
                || !type.Interfaces.Select(i => i.InterfaceType.FullName).Order().SequenceEqual((parent == "" ? Array.Empty<string>() : parent.Split('|')).Order()))
                throw new InvalidDataException("Incompatible collection contract: " + name + " parents=" + string.Join(",", type.Interfaces.Select(i => i.InterfaceType.FullName)));
        }
    }

    public sealed record Binding(string[] Arguments, string Result, string Instruction);
    public static Binding? Bind(MethodReference reference, MethodDefinition definition, bool callvirt)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        if (!definition.IsPublic || reference.ExplicitThis || reference is GenericInstanceMethod
            || reference.HasGenericParameters || reference.CallingConvention != MethodCallingConvention.Default
            || reference.HasThis != definition.HasThis || definition.HasGenericParameters)
            throw new InvalidDataException("Unsupported collection signature.");
        // Resolve both definition and reference under the same closed owner arguments.
        string Closed(TypeReference t) => t switch {
            GenericParameter { Type: GenericParameterType.Type, Position: 0 } when reference.DeclaringType is GenericInstanceType => "Int32",
            _ when t.MetadataType == MetadataType.Void => "noresult",
            _ when t.MetadataType == MetadataType.Int32 => "Int32",
            _ when t.MetadataType == MetadataType.Boolean => "Boolean",
            GenericInstanceType g when g.GenericArguments.Count == 1 && g.GenericArguments[0] is GenericParameter { Type: GenericParameterType.Type, Position: 0 } =>
                g.ElementType.FullName switch { "System.Collections.Iterator`1" => Iterator, "System.Collections.ArrayList`1" => ArrayList,
                    _ => throw new InvalidDataException("Unsupported collection generic signature.") },
            _ => Type(t) ?? throw new InvalidDataException("Unsupported collection signature type.")
        };
        var parameters = reference.Parameters.Select(p => Closed(p.ParameterType)).ToArray();
        var result = Closed(reference.ReturnType);
        if (!parameters.SequenceEqual(definition.Parameters.Select(p => Closed(p.ParameterType))) || result != Closed(definition.ReturnType))
            throw new InvalidDataException("Collection reference/definition signature mismatch.");
        var expected = (owner, definition.Name) switch {
            (ArrayList, "Allocate") => ("Int32", ArrayList, false),
            (List or ArrayList, "Add") => ("Int32", "noresult", true),
            (List or ArrayList, "get_Count") => ("", "Int32", true),
            (List or ArrayList, "get_Item") => ("Int32", "Int32", true),
            (List or ArrayList, "set_Item") => ("Int32,Int32", "noresult", true),
            (Iterable or ArrayList, "GetIterator") => ("", Iterator, true),
            (Iterator, "MoveNext") => ("", "Boolean", true),
            (Iterator, "get_Current") => ("", "Int32", true),
            (Disposable, "Dispose") => ("", "noresult", true),
            _ => throw new InvalidDataException("Unsupported collection member: " + definition.FullName)
        };
        if (string.Join(',', parameters) != expected.Item1 || result != expected.Item2
            || reference.HasThis != expected.Item3 || callvirt != expected.Item3)
            throw new InvalidDataException("Unsupported collection call shape.");
        var args = reference.HasThis ? new[] { owner }.Concat(parameters).ToArray() : parameters;
        return new(args, result, $"{(callvirt ? "callvirt instance" : "call")} {owner}::{definition.Name}({string.Join(',', parameters)})");
    }
}
