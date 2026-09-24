using Mono.Cecil;

// Bounded executable catalog. This is not Raven's future language-contract configuration.
static class CollectionBindings
{
    public const string List = "System.Collections.List<Int32>";
    public const string ArrayList = "System.Collections.ArrayList<Int32>";
    public const string Iterable = "System.Collections.Iterable<Int32>";
    public const string Iterator = "System.Collections.Iterator<Int32>";
    public const string Disposable = "System.Disposable";
    static readonly Dictionary<string, (string Kind, string Element)> Shapes = new();
    public static void Reset() { Shapes.Clear(); MapBindings.Reset(); }
    public static bool IsReference(string type) => type == Disposable || Shapes.ContainsKey(type) || MapBindings.IsType(type)
        || type is List or ArrayList or Iterable or Iterator;
    public static bool IsArrayList(string? type) => type is not null && type.StartsWith("System.Collections.ArrayList<", StringComparison.Ordinal);
    public static bool Assignable(string source, string target)
    {
        if (source == "String" && target is "System.Collections.Iterable<Char>" or "System.Collections.Collection<Char>" or "System.Collections.Sequence<Char>") return true;
        if (source == target || MapBindings.Assignable(source, target)) return true;
        if (ManagedArrayBindings.IsType(source))
            return new[] { "Iterable", "Collection", "Sequence", "MutableSequence" }
                .Any(kind => target == $"System.Collections.{kind}<{source[9..^1]}>");
        if (!Shapes.TryGetValue(source, out var from))
            from = source switch { ArrayList => ("ArrayList", "Int32"), List => ("List", "Int32"), Iterator => ("Iterator", "Int32"), _ => default };
        if (from.Kind == "Iterator") return target == Disposable;
        var hierarchy = new[] { "ArrayList", "List", "MutableSequence", "Sequence", "Collection", "Iterable" };
        var index = Array.IndexOf(hierarchy, from.Kind);
        return index >= 0 && hierarchy.Skip(index + 1).Any(kind => target == $"System.Collections.{kind}<{from.Element}>");
    }

    public static string? Type(TypeReference type) => Type(type, null);

    public static string? Type(TypeReference type, Func<TypeReference, string?>? parameterMap)
    {
        if (MapBindings.Type(type) is { } map) return map;
        if (type.IsValueType) return null;
        if (type.FullName == Disposable && RuntimeSignatures.IsCore(type.Scope)) return Disposable;
        if (type is not GenericInstanceType g || g.GenericArguments.Count != 1
            || !RuntimeSignatures.IsCore(g.ElementType.Scope)) return null;
        var kind = g.ElementType.FullName switch {
            "System.Collections.Collection`1" => "Collection", "System.Collections.Sequence`1" => "Sequence",
            "System.Collections.MutableSequence`1" => "MutableSequence",
            "System.Collections.List`1" => "List", "System.Collections.ArrayList`1" => "ArrayList",
            "System.Collections.Iterable`1" => "Iterable", "System.Collections.Iterator`1" => "Iterator",
            _ => null
        };
        if (kind is null) return null;
        var element = parameterMap?.Invoke(g.GenericArguments[0]) ?? GenericUnionBindings.Type(g.GenericArguments[0]);
        if (element is null || !((parameterMap is not null || GenericUnionBindings.ParameterMap is not null) && g.GenericArguments[0] is GenericParameter || element is "Int32" or "Double" or "Boolean" or "String" or "Void"
            || UriBindings.IsName(element) || HttpBindings.IsName(element) || ApplicationTypes.IsType(element) || GenericUnionBindings.IsType(element) || ErrorBindings.IsType(element) || DelegateBindings.IsType(element) || ReflectionBindings.IsReference(element) || IsReference(element) || InterfaceBindings.IsInterface(element) || element.StartsWith("arrayref<", StringComparison.Ordinal)
            || PrimitiveBindings.Types.Contains(element) || CalendarBindings.Types.Contains(element) || ErrorBindings.IsEmpty(element))) return null;
        var owner = $"System.Collections.{kind}<{element}>";
        Shapes[owner] = (kind, element);
        return owner;
    }

    public static void Validate(ModuleDefinition module)
    {
        MapBindings.Validate(module);
        foreach (var (name, parent) in new[] {
            ("System.Disposable", ""), ("System.Collections.Iterable`1", ""),
            ("System.Collections.Iterator`1", "System.Disposable"),
            ("System.Collections.Collection`1", "System.Collections.Iterable`1<T>"),
            ("System.Collections.Sequence`1", "System.Collections.Collection`1<T>|System.Collections.Iterable`1<T>"),
            ("System.Collections.MutableSequence`1", "System.Collections.Sequence`1<T>|System.Collections.Collection`1<T>|System.Collections.Iterable`1<T>"),
            ("System.Collections.List`1", "System.Collections.MutableSequence`1<T>|System.Collections.Sequence`1<T>|System.Collections.Collection`1<T>|System.Collections.Iterable`1<T>"),
            ("System.Collections.ArrayList`1", "System.Collections.List`1<T>|System.Collections.MutableSequence`1<T>|System.Collections.Sequence`1<T>|System.Collections.Collection`1<T>|System.Collections.Iterable`1<T>") })
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
    public static Binding? Bind(MethodReference reference, MethodDefinition definition, bool callvirt, Func<TypeReference, string?>? parameterMap = null)
    {
        if (MapBindings.Bind(reference, definition, callvirt) is { } map) return map;
        var owner = Type(reference.DeclaringType, parameterMap);
        if (owner is null) return null;
        var (parameters, result) = RuntimeSignatures.Match(reference, definition, t => parameterMap?.Invoke(t) ?? ApplicationTypes.Type(t) ?? Type(t) ?? DelegateBindings.Type(t) ?? GenericUnionBindings.Type(t), allowOpenMethodParameters: parameterMap is not null);
        var (kind, element) = owner == Disposable ? ("Disposable", "") : Shapes[owner];
        var expected = (kind, definition.Name) switch {
            ("List" or "ArrayList", "Add") => (element, "noresult", true),
            ("ArrayList", "get_Capacity") => ("", "Int32", true),
            ("Collection" or "ArrayList", "get_Count") => ("", "Int32", true),
            ("Sequence" or "MutableSequence" or "ArrayList", "get_Item") => ("Int32", element, true),
            ("MutableSequence" or "ArrayList", "set_Item") => ("Int32," + element, "noresult", true),
            ("ArrayList", "FindIndex" or "FindLastIndex") => ($"System.Func<{element},Boolean>", "System.Option<Int32>", true),
            ("ArrayList", "Exists" or "TrueForAll") => ($"System.Func<{element},Boolean>", "Boolean", true),
            ("ArrayList", "Find" or "FindLast") => ($"System.Func<{element},Boolean>", $"System.Option<{element}>", true),
            ("ArrayList", "FindAll") => ($"System.Func<{element},Boolean>", owner, true),
            ("ArrayList", "Copy") => ("", owner, true),
            ("Iterable" or "ArrayList", "GetIterator") => ("", $"System.Collections.Iterator<{element}>", true),
            ("Iterator", "MoveNext") => ("", "Boolean", true),
            ("Iterator", "get_Current") => ("", element, true),
            ("Disposable", "Dispose") => ("", "noresult", true),
            _ => throw new InvalidDataException("Unsupported collection member: " + definition.FullName)
        };
        if (string.Join(',', parameters) != expected.Item1 || result != expected.Item2
            || reference.HasThis != expected.Item3 || callvirt != expected.Item3)
            throw new InvalidDataException("Unsupported collection call shape.");
        var args = reference.HasThis ? new[] { owner }.Concat(parameters).ToArray() : parameters;
        return new(args, result, $"{(callvirt ? "callvirt instance" : "call")} {owner}::{definition.Name}({string.Join(',', parameters)})");
    }
}
