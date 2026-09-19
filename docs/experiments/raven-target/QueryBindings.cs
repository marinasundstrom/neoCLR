using Mono.Cecil;

// Generic library operators, not generic application-body importing.
static class QueryBindings
{
    public const string Declarations = """
        namespace Linq {
            public static class Operators {
                public static Collections.Iterable<T> Filter<T>(this Collections.Iterable<T> self, Func<T, bool> predicate) => default;
                public static Collections.Iterable<U> Map<T,U>(this Collections.Iterable<T> self, Func<T,U> selector) => default;
                public static bool Any<T>(this Collections.Iterable<T> self) => default;
                public static bool Any<T>(this Collections.Iterable<T> self, Func<T,bool> predicate) => default;
                public static bool All<T>(this Collections.Iterable<T> self, Func<T,bool> predicate) => default;
                public static int Count<T>(this Collections.Iterable<T> self) => default;
                public static int Count<T>(this Collections.Iterable<T> self, Func<T,bool> predicate) => default;
                public static U Fold<T,U>(this Collections.Iterable<T> self, U seed, Func<U,T,U> accumulator) => default;
                public static Collections.Iterable<T> Take<T>(this Collections.Iterable<T> self, int count) => default;
                public static Collections.Iterable<T> Skip<T>(this Collections.Iterable<T> self, int count) => default;
                public static Collections.Iterable<T> Concat<T>(this Collections.Iterable<T> self, Collections.Iterable<T> second) => default;
                public static Collections.Iterable<U> FlatMap<T,U>(this Collections.Iterable<T> self, Func<T,Collections.Iterable<U>> selector) => default;
                public static Option<T> First<T>(this Collections.Iterable<T> self) => default;
                public static Option<T> First<T>(this Collections.Iterable<T> self, Func<T, bool> predicate) => default;
                public static Option<T> Last<T>(this Collections.Iterable<T> self) => default;
                public static Option<T> Last<T>(this Collections.Iterable<T> self, Func<T, bool> predicate) => default;
                public static Result<T, SingleError> Single<T>(this Collections.Iterable<T> self) => default;
                public static Result<T, SingleError> Single<T>(this Collections.Iterable<T> self, Func<T, bool> predicate) => default;
                public static Collections.ArrayList<T> ToList<T>(this Collections.Iterable<T> self) => default;
            }
        }
        """;

    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool callvirt)
    {
        if (reference.DeclaringType.FullName != "System.Linq.Operators") return null;
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || reference.HasThis || callvirt
            || reference is not GenericInstanceMethod method
            || definition.GenericParameters.Any(p => p.HasConstraints || p.Attributes != GenericParameterAttributes.NonVariant))
            throw new InvalidDataException("Unsupported query signature.");
        var types = method.GenericArguments.Select(GenericUnionBindings.Type).ToArray();
        var arity = reference.Name is "Map" or "FlatMap" or "Fold" ? 2 : 1;
        if (types.Length != arity || types.Any(t => t is null))
            throw new InvalidDataException("Unsupported query type arguments.");
        var source = $"System.Collections.Iterable<{types[0]}>";
        var terminalArguments = definition.Parameters.Count == 2
            ? new[] { source, $"System.Func<{types[0]},Boolean>" } : new[] { source };
        var (expected, returns) = reference.Name switch {
            "Filter" => (new[] { source, $"System.Func<{types[0]},Boolean>" }, source),
            "Map" => (new[] { source, $"System.Func<{types[0]},{types[1]}>" }, $"System.Collections.Iterable<{types[1]}>"),
            "First" or "Last" => (terminalArguments, $"System.Option<{types[0]}>"),
            "Single" => (terminalArguments, $"System.Result<{types[0]},System.Linq.SingleError>"),
            "Any" => (terminalArguments, "Boolean"),
            "All" => (new[] { source, $"System.Func<{types[0]},Boolean>" }, "Boolean"),
            "Count" => (terminalArguments, "Int32"),
            "Fold" => (new[] { source, types[1]!, $"System.Func<{types[1]},{types[0]},{types[1]}>" }, types[1]!),
            "Take" or "Skip" => (new[] { source, "Int32" }, source),
            "Concat" => (new[] { source, source }, source),
            "FlatMap" => (new[] { source, $"System.Func<{types[0]},System.Collections.Iterable<{types[1]}>>" }, $"System.Collections.Iterable<{types[1]}>"),
            "ToList" => (new[] { source }, $"System.Collections.ArrayList<{types[0]}>"),
            _ => throw new InvalidDataException("Unsupported query operator.")
        };
        var (args, result) = RuntimeSignatures.Match(reference, definition,
            t => CollectionBindings.Type(t) ?? DelegateBindings.Type(t) ?? GenericUnionBindings.Type(t));
        if (!args.SequenceEqual(expected) || result != returns)
            throw new InvalidDataException("Unsupported query contract.");
        return new($"System.Linq.Operators::{reference.Name}<{string.Join(',', types)}>", args, result);
    }
}
