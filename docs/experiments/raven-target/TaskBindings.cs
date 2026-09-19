using Mono.Cecil;

// Provisional single-invocation completion API. No exception or async lowering policy.
static class TaskBindings
{
    const string Prefix = "System.Threading.Tasks.";
    const string Queue = Prefix + "TaskQueue";
    static readonly Dictionary<string, (string Kind, string Payload)> Shapes = new();
    public static void Reset() => Shapes.Clear();
    public static bool IsType(string type) => Shapes.ContainsKey(type);
    public const string Declarations = """
        namespace Threading.Tasks {
            public sealed class TaskQueue {
                public TaskQueue() { }
                public void Post(Func<PropagationUnit> callback) { }
                public void Drain() { }
            }
            public sealed class Task<T> {
                public Task(TaskCompletionSource<T> source) { }
                public bool IsCompleted => default;
                public T GetResult() => default;
                public void OnCompleted(Func<PropagationUnit> callback) { }
            }
            public sealed class TaskCompletionSource<T> {
                public TaskCompletionSource(TaskQueue queue) { }
                public Task<T> Task => default;
                public bool TrySetResult(T value) => default;
                public bool Completed() => default;
                public T Read() => default;
                public void Register(Func<PropagationUnit> callback) { }
            }
        }
        """;

    public static void Project(ModuleDefinition module)
    {
        foreach (var method in module.GetType(Prefix + "Task`1").Methods.Where(m => m.IsConstructor)
            .Concat(module.GetType(Prefix + "TaskCompletionSource`1").Methods.Where(m => m.Name is "Completed" or "Read" or "Register")))
            method.Attributes = (method.Attributes & ~MethodAttributes.MemberAccessMask) | MethodAttributes.Assembly;
    }

    public static bool SameType(TypeReference left, TypeReference right) =>
        left.FullName == right.FullName && left.FullName is Prefix + "Task`1" or Prefix + "TaskCompletionSource`1" or Queue
        && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);

    public static string? Type(TypeReference type)
    {
        if (type.IsValueType || !RuntimeSignatures.IsCore(type.Scope)) return null;
        if (type.FullName == Queue) { Shapes[Queue] = ("TaskQueue", ""); return Queue; }
        if (type is not GenericInstanceType g || g.GenericArguments.Count != 1) return null;
        var kind = g.ElementType.FullName switch
        {
            Prefix + "Task`1" => "Task",
            Prefix + "TaskCompletionSource`1" => "TaskCompletionSource",
            _ => null
        };
        if (kind is null) return null;
        var payload = GenericUnionBindings.Type(g.GenericArguments[0]);
        if (payload is null) return null;
        var owner = Prefix + kind + "<" + payload + ">";
        Shapes[owner] = (kind, payload);
        return owner;
    }

    public static CollectionBindings.Binding? Bind(MethodReference reference, MethodDefinition definition,
        bool construct, bool library)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var (kind, payload) = Shapes[owner];
        var internalMember = kind == "Task" && definition.IsConstructor
            || kind == "TaskCompletionSource" && definition.Name is "Completed" or "Read" or "Register";
        if (internalMember ? !library || !definition.IsAssembly : !definition.IsPublic)
            throw new InvalidDataException("Invalid Task member visibility.");
        if (!definition.DeclaringType.IsSealed || definition.DeclaringType.IsInterface
            || definition.DeclaringType.HasInterfaces
            || definition.DeclaringType.GenericParameters.Any(p => p.HasConstraints || p.Attributes != GenericParameterAttributes.NonVariant)
            || !reference.HasThis || definition.IsStatic || definition.HasGenericParameters
            || !definition.IsPublic && !(library && definition.IsAssembly)
            || definition.IsConstructor != construct)
            throw new InvalidDataException("Unsupported provisional Task contract.");
        var expected = (kind, definition.Name) switch
        {
            ("TaskQueue", ".ctor") => ("", "noresult"),
            ("TaskQueue", "Post") => ("System.Func<Void>", "noresult"),
            ("TaskQueue", "Drain") => ("", "noresult"),
            ("Task", ".ctor") => (Prefix + "TaskCompletionSource<" + payload + ">", "noresult"),
            ("Task", "get_IsCompleted") => ("", "Boolean"),
            ("Task", "GetResult") => ("", payload),
            ("Task", "OnCompleted") => ("System.Func<Void>", "noresult"),
            ("TaskCompletionSource", ".ctor") => (Queue, "noresult"),
            ("TaskCompletionSource", "get_Task") => ("", Prefix + "Task<" + payload + ">"),
            ("TaskCompletionSource", "TrySetResult") => (payload, "Boolean"),
            ("TaskCompletionSource", "Completed") when library => ("", "Boolean"),
            ("TaskCompletionSource", "Read") when library => ("", payload),
            ("TaskCompletionSource", "Register") when library => ("System.Func<Void>", "noresult"),
            _ => throw new InvalidDataException("Unsupported Task member: " + definition.FullName)
        };
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type,
            allowOpenMethodParameters: library);
        if (string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported Task signature.");
        return construct
            ? new(args, owner, $"newobj instance {owner}::.ctor({string.Join(',', args)})")
            : new(new[] { owner }.Concat(args).ToArray(), result, $"call instance {owner}::{definition.Name}({string.Join(',', args)})");
    }
}
