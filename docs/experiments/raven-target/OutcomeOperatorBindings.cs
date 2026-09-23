using Mono.Cecil;

// Raven.Core-style composition over the existing neoCLR union carriers.
static class OutcomeOperatorBindings
{
    public const string Declarations = """
        namespace Tasks {
            public static class TaskResultOperators {
                public static Task<Result<U,E>> MapResult<T,E,U>(this Task<Result<T,E>> self, Func<T,U> transform) => default;
            }
            public static class TaskOperators {
                public static Task<U> Map<T,U>(this Task<T> self, Func<T,U> transform) => default;
                public static Task<U> Then<T,U>(this Task<T> self, Func<T,Task<U>> continuation) => default;
            }
        }
        public static class OptionOperators {
            public static Option<U> Map<T,U>(this Option<T> self, Func<T,U> mapper) => default;
            public static Option<U> Then<T,U>(this Option<T> self, Func<T,Option<U>> binder) => default;
            public static Option<T> Filter<T>(this Option<T> self, Func<T,bool> predicate) => default;
            public static Option<T> OrElse<T>(this Option<T> self, Func<Option<T>> alternative) => default;
            public static T UnwrapOrElse<T>(this Option<T> self, Func<T> factory) => default;
            public static T UnwrapOr<T>(this Option<T> self, T defaultValue) => default;
            public static U Match<T,U>(this Option<T> self, Func<T,U> some, Func<U> none) => default;
            public static Option<T> Tap<T>(this Option<T> self, Func<T,PropagationUnit> action) => default;
            public static Option<T> TapNone<T>(this Option<T> self, Func<PropagationUnit> action) => default;
            public static Collections.Iterable<T> ToIterable<T>(this Option<T> self) => default;
            public static Result<U,E> ThenResult<T,U,E>(this Option<T> self, Func<T,Result<U,E>> binder, Func<E> noneError) => default;
            public static Result<U,E> MapResult<T,U,E>(this Option<T> self, Func<T,U> mapper, Func<E> noneError) => default;
            public static Result<T,E> OkOr<T,E>(this Option<T> self, E error) => default;
            public static Result<T,E> OkOr<T,E>(this Option<T> self, Func<E> errorFactory) => default;
        }
        public static class ResultOperators {
            public static Result<U,E> Map<T,E,U>(this Result<T,E> self, Func<T,U> mapper) => default;
            public static Result<U,E> Then<T,E,U>(this Result<T,E> self, Func<T,Result<U,E>> binder) => default;
            public static Result<T,F> MapError<T,E,F>(this Result<T,E> self, Func<E,F> mapper) => default;
            public static U Match<T,E,U>(this Result<T,E> self, Func<T,U> ok, Func<E,U> error) => default;
            public static Result<T,E> Tap<T,E>(this Result<T,E> self, Func<T,PropagationUnit> action) => default;
            public static Result<T,E> TapError<T,E>(this Result<T,E> self, Func<E,PropagationUnit> action) => default;
            public static Result<T,E> OrElse<T,E>(this Result<T,E> self, Func<E,Result<T,E>> recover) => default;
            public static T UnwrapOrElse<T,E>(this Result<T,E> self, Func<T> factory) => default;
            public static T UnwrapOr<T,E>(this Result<T,E> self, T defaultValue) => default;
            public static Collections.Iterable<T> ToIterable<T,E>(this Result<T,E> self) => default;
        }
        public static class OptionNestedOperators {
            public static Option<T> Flatten<T>(this Option<Option<T>> self) => default;
        }
        """;

    sealed record Contract(string Owner, string Name, int Arity, string[] Arguments, string Result);
    static readonly Contract[] Contracts = [
        new("System.Tasks.TaskResultOperators", "MapResult", 3, ["System.Tasks.Task<System.Result<@0,@1>>", "System.Func<@0,@2>"], "System.Tasks.Task<System.Result<@2,@1>>"),
        new("System.Tasks.TaskOperators", "Map", 2, ["System.Tasks.Task<@0>", "System.Func<@0,@1>"], "System.Tasks.Task<@1>"),
        new("System.Tasks.TaskOperators", "Then", 2, ["System.Tasks.Task<@0>", "System.Func<@0,System.Tasks.Task<@1>>"], "System.Tasks.Task<@1>"),
        new("System.OptionOperators", "Map", 2, ["System.Option<@0>", "System.Func<@0,@1>"], "System.Option<@1>"),
        new("System.OptionOperators", "Then", 2, ["System.Option<@0>", "System.Func<@0,System.Option<@1>>"], "System.Option<@1>"),
        new("System.OptionOperators", "Filter", 1, ["System.Option<@0>", "System.Func<@0,Boolean>"], "System.Option<@0>"),
        new("System.OptionOperators", "OrElse", 1, ["System.Option<@0>", "System.Func<System.Option<@0>>"], "System.Option<@0>"),
        new("System.OptionOperators", "UnwrapOrElse", 1, ["System.Option<@0>", "System.Func<@0>"], "@0"),
        new("System.OptionOperators", "UnwrapOr", 1, ["System.Option<@0>", "@0"], "@0"),
        new("System.OptionOperators", "Match", 2, ["System.Option<@0>", "System.Func<@0,@1>", "System.Func<@1>"], "@1"),
        new("System.OptionOperators", "Tap", 1, ["System.Option<@0>", "System.Func<@0,Void>"], "System.Option<@0>"),
        new("System.OptionOperators", "TapNone", 1, ["System.Option<@0>", "System.Func<Void>"], "System.Option<@0>"),
        new("System.OptionOperators", "ToIterable", 1, ["System.Option<@0>"], "System.Collections.Iterable<@0>"),
        new("System.OptionOperators", "ThenResult", 3, ["System.Option<@0>", "System.Func<@0,System.Result<@1,@2>>", "System.Func<@2>"], "System.Result<@1,@2>"),
        new("System.OptionOperators", "MapResult", 3, ["System.Option<@0>", "System.Func<@0,@1>", "System.Func<@2>"], "System.Result<@1,@2>"),
        new("System.OptionOperators", "OkOr", 2, ["System.Option<@0>", "@1"], "System.Result<@0,@1>"),
        new("System.OptionOperators", "OkOr", 2, ["System.Option<@0>", "System.Func<@1>"], "System.Result<@0,@1>"),
        new("System.OptionNestedOperators", "Flatten", 1, ["System.Option<System.Option<@0>>"], "System.Option<@0>"),
        new("System.ResultOperators", "Map", 3, ["System.Result<@0,@1>", "System.Func<@0,@2>"], "System.Result<@2,@1>"),
        new("System.ResultOperators", "Then", 3, ["System.Result<@0,@1>", "System.Func<@0,System.Result<@2,@1>>"], "System.Result<@2,@1>"),
        new("System.ResultOperators", "MapError", 3, ["System.Result<@0,@1>", "System.Func<@1,@2>"], "System.Result<@0,@2>"),
        new("System.ResultOperators", "Match", 3, ["System.Result<@0,@1>", "System.Func<@0,@2>", "System.Func<@1,@2>"], "@2"),
        new("System.ResultOperators", "Tap", 2, ["System.Result<@0,@1>", "System.Func<@0,Void>"], "System.Result<@0,@1>"),
        new("System.ResultOperators", "TapError", 2, ["System.Result<@0,@1>", "System.Func<@1,Void>"], "System.Result<@0,@1>"),
        new("System.ResultOperators", "OrElse", 2, ["System.Result<@0,@1>", "System.Func<@1,System.Result<@0,@1>>"], "System.Result<@0,@1>"),
        new("System.ResultOperators", "UnwrapOrElse", 2, ["System.Result<@0,@1>", "System.Func<@0>"], "@0"),
        new("System.ResultOperators", "UnwrapOr", 2, ["System.Result<@0,@1>", "@0"], "@0"),
        new("System.ResultOperators", "ToIterable", 2, ["System.Result<@0,@1>"], "System.Collections.Iterable<@0>"),
    ];

    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool callvirt)
    {
        if (reference.DeclaringType.FullName is not ("System.OptionOperators" or "System.ResultOperators" or "System.OptionNestedOperators" or "System.Tasks.TaskOperators" or "System.Tasks.TaskResultOperators")) return null;
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || reference.HasThis || callvirt
            || reference is not GenericInstanceMethod method
            || definition.GenericParameters.Any(p => p.HasConstraints || p.Attributes != GenericParameterAttributes.NonVariant))
            throw new InvalidDataException("Unsupported outcome operator signature.");
        var types = method.GenericArguments.Select(GenericUnionBindings.Type).ToArray();
        if (types.Any(t => t is null)) throw new InvalidDataException("Unsupported outcome operator type arguments.");
        string Close(string template) {
            for (var i = 0; i < types.Length; i++) template = template.Replace("@" + i, types[i]);
            return template;
        }
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type,
            allowOpenMethodParameters: GenericUnionBindings.ParameterMap is not null);
        if (!Contracts.Any(c => c.Owner == reference.DeclaringType.FullName && c.Name == reference.Name
            && c.Arity == types.Length && args.SequenceEqual(c.Arguments.Select(Close)) && result == Close(c.Result)))
            throw new InvalidDataException($"Unsupported outcome operator contract: {reference.FullName}; {string.Join(',', args)} -> {result}.");
        return new(reference.DeclaringType.FullName + "::" + reference.Name + "<" + string.Join(',', types) + ">", args, result);
    }
}
