using Mono.Cecil;

// Target library policies expressed through ordinary CLI classes and interfaces.
static class ComparerBindings
{
    const string Prefix = "System.Collections.";
    public const string StringOwner = "System.StringComparer";
    static readonly string[] Kinds = ["Comparer", "EqualityComparer", "DelegateComparer", "DelegateEqualityComparer"];
    public const string Declarations = """
        namespace Collections {
            public interface Comparer<T> { int Compare(T left, T right); }
            public interface EqualityComparer<T> { bool Equals(T left, T right); int GetHashCode(T value); }
            public sealed class DelegateComparer<T> : Comparer<T> {
                public DelegateComparer(Func<T,T,int> compare) { }
                public int Compare(T left, T right) => default;
            }
            public sealed class DelegateEqualityComparer<T> : EqualityComparer<T> {
                public DelegateEqualityComparer(Func<T,T,bool> equal, Func<T,int> hash) { }
                public bool Equals(T left, T right) => default;
                public int GetHashCode(T value) => default;
            }
        }
        public enum StringComparison { Ordinal = 0, OrdinalIgnoreCase = 1 }
        public sealed class StringComparer : Collections.EqualityComparer<string>, Collections.Comparer<string> {
            private StringComparer() { }
            public static StringComparer Ordinal => default;
            public static StringComparer OrdinalIgnoreCase => default;
            public bool Equals(string left, string right) => default;
            public int GetHashCode(string value) => default;
            public int Compare(string left, string right) => default;
        }
        """;
    public static bool IsName(string name) => name == StringOwner || Kinds.Any(k => name.StartsWith(Prefix + k + "<", StringComparison.Ordinal));
    public static bool IsInterface(string name) => name.StartsWith(Prefix + "Comparer<", StringComparison.Ordinal) || name.StartsWith(Prefix + "EqualityComparer<", StringComparison.Ordinal);
    public static string? Type(TypeReference type)
    {
        if (!RuntimeSignatures.IsCore(type.Scope) || type.IsValueType) return null;
        if (type.FullName == StringOwner) return StringOwner;
        if (type is not GenericInstanceType g || g.GenericArguments.Count != 1) return null;
        var kind = Kinds.SingleOrDefault(k => g.ElementType.FullName == Prefix + k + "`1");
        if (kind is null) return null;
        var element = GenericUnionBindings.Type(g.GenericArguments[0]);
        return element is null ? null : Prefix + kind + "<" + element + ">";
    }
    public static bool Converts(string source, string target) => source == StringOwner
        ? target is "System.Collections.Comparer<String>" or "System.Collections.EqualityComparer<String>"
        : source.StartsWith(Prefix + "Delegate", StringComparison.Ordinal) && IsName(source)
            && target == Prefix + source[(Prefix.Length + "Delegate".Length)..];

    public static void Validate(ModuleDefinition module)
    {
        foreach (var kind in Kinds.Append("StringComparer"))
        {
            var name = kind == "StringComparer" ? StringOwner : Prefix + kind + "`1";
            var type = module.GetType(name) ?? throw new InvalidDataException("Missing comparer contract: " + name);
            var contract = kind is "Comparer" or "EqualityComparer";
            var parents = kind switch {
                "StringComparer" => new[] { Prefix + "EqualityComparer`1<System.String>", Prefix + "Comparer`1<System.String>" },
                "DelegateComparer" => new[] { Prefix + "Comparer`1<T>" },
                "DelegateEqualityComparer" => new[] { Prefix + "EqualityComparer`1<T>" },
                _ => Array.Empty<string>()
            };
            if (!type.IsPublic || type.IsValueType || type.IsInterface != contract || type.IsAbstract != contract
                || type.IsSealed == contract || type.GenericParameters.Count != (kind == "StringComparer" ? 0 : 1)
                || type.GenericParameters.Any(p => p.Attributes != GenericParameterAttributes.NonVariant || p.HasConstraints)
                || !type.Interfaces.Select(i => i.InterfaceType.FullName).Order().SequenceEqual(parents.Order()))
                throw new InvalidDataException("Unsupported comparer type contract: " + name);
        }
    }

    public static CollectionBindings.Binding? Construct(MethodReference reference, MethodDefinition definition)
    {
        var call = Bind(reference, definition, true);
        return call is null ? null : new(call.Arguments, call.Result, call.Instruction!);
    }

    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool construct = false)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var element = owner == StringOwner ? "String" : owner[(owner.IndexOf('<') + 1)..^1];
        var kind = owner == StringOwner ? "StringComparer" : owner[Prefix.Length..owner.IndexOf('<')];
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type, allowOpenMethodParameters: GenericUnionBindings.ParameterMap is not null);
        (string Args, string Result, bool Static) expected = (kind, reference.Name) switch {
            ("DelegateComparer", ".ctor") => ($"System.Func<{element},{element},Int32>", "noresult", false),
            ("DelegateEqualityComparer", ".ctor") => ($"System.Func<{element},{element},Boolean>,System.Func<{element},Int32>", "noresult", false),
            ("StringComparer", "get_Ordinal" or "get_OrdinalIgnoreCase") => ("", StringOwner, true),
            ("StringComparer" or "Comparer" or "DelegateComparer", "Compare") => ($"{element},{element}", "Int32", false),
            ("StringComparer" or "EqualityComparer" or "DelegateEqualityComparer", "Equals") => ($"{element},{element}", "Boolean", false),
            ("StringComparer" or "EqualityComparer" or "DelegateEqualityComparer", "GetHashCode") => (element, "Int32", false),
            _ => throw new InvalidDataException("Unsupported comparer member: " + reference.FullName)
        };
        var contract = IsInterface(owner);
        var ordinaryInstance = !expected.Static && !definition.IsConstructor;
        if (!definition.IsPublic || definition.HasGenericParameters || definition.IsStatic != expected.Static
            || reference.HasThis == expected.Static || construct != definition.IsConstructor
            || definition.DeclaringType.IsInterface != contract || definition.IsAbstract != contract
            || definition.IsVirtual != ordinaryInstance || definition.IsNewSlot != ordinaryInstance
            || definition.IsFinal != (ordinaryInstance && !contract)
            || string.Join(',', args) != expected.Args || result != expected.Result)
            throw new InvalidDataException("Unsupported comparer signature: " + reference.FullName);
        var inputs = construct || expected.Static ? args : new[] { owner }.Concat(args).ToArray();
        return new("", inputs, construct ? owner : result,
            Instruction: $"{(construct ? "newobj instance " : expected.Static ? "call " : "callvirt instance ")}{owner}::{reference.Name}({string.Join(',', args)})");
    }
}
