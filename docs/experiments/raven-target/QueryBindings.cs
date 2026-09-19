using Mono.Cecil;

// Generic library operators, not generic application-body importing.
static class QueryBindings
{
    public const string Declarations = """
        namespace Linq {
            public static class Operators {
                public static Collections.Iterable<T> Filter<T>(this Collections.Iterable<T> source, Func<T, bool> predicate) => default;
                public static Collections.Iterable<U> Map<T,U>(this Collections.Iterable<T> source, Func<T,U> selector) => default;
                public static Option<T> First<T>(this Collections.Iterable<T> source) => default;
                public static Option<T> First<T>(this Collections.Iterable<T> source, Func<T, bool> predicate) => default;
                public static Option<T> Last<T>(this Collections.Iterable<T> source) => default;
                public static Option<T> Last<T>(this Collections.Iterable<T> source, Func<T, bool> predicate) => default;
                public static Result<T, SingleError> Single<T>(this Collections.Iterable<T> source) => default;
                public static Result<T, SingleError> Single<T>(this Collections.Iterable<T> source, Func<T, bool> predicate) => default;
                public static Collections.ArrayList<T> ToList<T>(this Collections.Iterable<T> source) => default;
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
        var arity = reference.Name == "Map" ? 2 : 1;
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
