using Mono.Cecil;

// Experimental map surface. Storage and hashing algorithms live in the IL library.
static class MapBindings
{
    static readonly Dictionary<string, (string Kind, string Key, string Value)> Shapes = new();
    public static void Reset() => Shapes.Clear();
    public static bool IsType(string type) => Shapes.ContainsKey(type);
    public static string? Type(TypeReference type)
    {
        if (type.IsValueType || type is not GenericInstanceType g || g.GenericArguments.Count != 2
            || !RuntimeSignatures.IsCore(g.ElementType.Scope)) return null;
        var kind = g.ElementType.FullName switch {
            "System.Collections.Map`2" => "Map", "System.Collections.MutableMap`2" => "MutableMap",
            "System.Collections.HashMap`2" => "HashMap", _ => null
        };
        if (kind is null) return null;
        var key = GenericUnionBindings.Type(g.GenericArguments[0]);
        var value = GenericUnionBindings.Type(g.GenericArguments[1]);
        if (key is null || value is null) return null;
        var owner = $"System.Collections.{kind}<{key},{value}>";
        Shapes[owner] = (kind, key, value);
        return owner;
    }
    public static bool Assignable(string source, string target)
    {
        if (!Shapes.TryGetValue(source, out var shape)) return false;
        return (shape.Kind == "HashMap" && target == $"System.Collections.MutableMap<{shape.Key},{shape.Value}>")
            || (shape.Kind is "HashMap" or "MutableMap" && target == $"System.Collections.Map<{shape.Key},{shape.Value}>");
    }
    public static void Validate(ModuleDefinition module)
    {
        foreach (var kind in new[] { "Map", "MutableMap", "HashMap" })
        {
            var type = module.GetType("System.Collections." + kind + "`2") ?? throw new InvalidDataException("Missing map contract.");
            var parents = kind switch {
                "Map" => Array.Empty<string>(),
                "MutableMap" => new[] { "System.Collections.Map`2<K,V>" },
                _ => new[] { "System.Collections.Map`2<K,V>", "System.Collections.MutableMap`2<K,V>" }
            };
            if (type.IsValueType || type.IsInterface != (kind != "HashMap") || type.GenericParameters.Count != 2
                || type.GenericParameters.Any(p => p.Attributes != GenericParameterAttributes.NonVariant || p.HasConstraints)
                || !type.Interfaces.Select(i => i.InterfaceType.FullName).Order().SequenceEqual(parents.Order())
                || type.IsInterface && type.Methods.Any(m => !m.IsPublic || !m.IsAbstract || !m.IsVirtual || !m.IsNewSlot || !m.HasThis || m.HasBody))
                throw new InvalidDataException("Incompatible map contract: " + kind);
        }
    }
    public static CollectionBindings.Binding? Construct(MethodReference reference, MethodDefinition definition)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var shape = Shapes[owner];
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var expected = new[] { $"System.Func<{shape.Key},{shape.Key},Boolean>", $"System.Func<{shape.Key},Int32>" };
        if (shape.Kind != "HashMap" || !definition.IsConstructor || !reference.HasThis || result != "noresult" || !args.SequenceEqual(expected))
            throw new InvalidDataException("Unsupported map constructor.");
        return new(args, owner, $"newobj instance {owner}::.ctor({string.Join(',', args)})");
    }
    public static CollectionBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool callvirt)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var shape = Shapes[owner];
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var expected = (shape.Kind, definition.Name) switch {
            ("Map" or "HashMap", "get_Count") => ("", "Int32"),
            ("Map" or "HashMap", "get_Keys") => ("", $"System.Collections.Sequence<{shape.Key}>"),
            ("Map" or "HashMap", "Find") => (shape.Key, $"System.Option<{shape.Value}>"),
            ("Map" or "HashMap", "ContainsKey") => (shape.Key, "Boolean"),
            ("MutableMap" or "HashMap", "TryAdd") => ($"{shape.Key},{shape.Value}", "Boolean"),
            ("MutableMap" or "HashMap", "Set") => ($"{shape.Key},{shape.Value}", "noresult"),
            _ => throw new InvalidDataException("Unsupported map member: " + definition.FullName)
        };
        if (!reference.HasThis || !callvirt || string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported map call shape.");
        return new(new[] { owner }.Concat(args).ToArray(), result, $"callvirt instance {owner}::{definition.Name}({string.Join(',', args)})");
    }
}
