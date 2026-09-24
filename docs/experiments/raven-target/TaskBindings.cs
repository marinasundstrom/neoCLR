using Mono.Cecil;

// Provisional single-invocation completion API. No exception or async lowering policy.
static class TaskBindings
{
    const string Prefix = "System.Tasks.";
    const string Queue = Prefix + "TaskQueue";
    static readonly Dictionary<string, (string Kind, string Payload)> Shapes = new();
    public static void Reset() => Shapes.Clear();
    public static bool IsType(string type) => Shapes.ContainsKey(type);
    public const string Declarations = """
        namespace Tasks {
            public enum TaskState { Pending = 0, Completed = 1, Cancelled = 2 }
            public static class TaskOutcome {
                public struct Cancelled { public Cancelled() { } }
                public struct Completed<T> {
                    public Completed(T value) { Value = value; }
                    public T Value { get; set; }
                    public void Deconstruct(out T value) { value = Value; }
                }
            }
            [System.Runtime.CompilerServices.Union]
            public struct TaskOutcome<T> {
                public TaskOutcome(TaskOutcome.Completed<T> value) { }
                public TaskOutcome(TaskOutcome.Cancelled value) { }
                public bool IsCompleted => false;
                public bool IsCancelled => false;
                public TaskOutcome.Completed<T> GetCompletedCase() => default;
                public TaskOutcome.Cancelled GetCancelledCase() => default;
                public bool TryGet(out TaskOutcome.Completed<T> value) { value = default; return false; }
                public bool TryGet(out TaskOutcome.Cancelled value) { value = default; return false; }
                public object Value => default;
                public bool TryGetValue(out TaskOutcome.Completed<T> value) { value = default; return false; }
                public bool TryGetValue(out TaskOutcome.Cancelled value) { value = default; return false; }
            }
            public sealed class TaskQueue {
                public TaskQueue() { }
                public static TaskQueue Default => default;
                public static TaskQueue Current => default;
                public void Post(Func<PropagationUnit> callback) { }
                public void Drain() { }
                public void Run(Func<PropagationUnit> callback) { }
            }
            public sealed class Task<T> : Runtime.CompilerServices.ITaskAwaiter {
                public Task(Promise<T> source) { }
                public TaskState State => default;
                public Option<TaskOutcome<T>> Outcome => default;
                public bool IsCompleted => default;
                public bool IsCancelled => default;
                public T GetResult() => default;
                public TaskQueue Dispatcher() => default;
                public Task<T> GetAwaiter() => default;
                public void OnCompleted(Func<PropagationUnit> callback) { }
            }
            public sealed class Promise<T> {
                public Promise() { }
                public Promise(TaskQueue queue) { }
                public Task<T> Task => default;
                public bool Complete(T value) => default;
                public bool Cancel() => default;
                public TaskQueue Dispatcher() => default;
                public bool Cancelled() => default;
                public bool Completed() => default;
                public T Read() => default;
                public void Register(Func<PropagationUnit> callback) { }
            }
        }
        namespace Runtime.CompilerServices {
            public interface IAsyncStateMachine {
                void MoveNext();
                void SetStateMachine(IAsyncStateMachine stateMachine);
            }
            public interface ITaskAwaiter { void OnCompleted(Func<PropagationUnit> callback); }
            public sealed class AsyncTaskMethodBuilder<T> {
                public AsyncTaskMethodBuilder(Tasks.TaskQueue queue) { }
                public static AsyncTaskMethodBuilder<T> Create() => default;
                public Tasks.Task<T> Task => default;
                public void Start<TState>(ref TState stateMachine) where TState : IAsyncStateMachine { }
                public void SetStateMachine(IAsyncStateMachine stateMachine) { }
                public bool HasStateMachine() => false;
                public IAsyncStateMachine GetStateMachine() => default;
                public void SetResult(T value) { }
                public void SetCancelled() { }
                public void AwaitOnCompleted<TAwaiter, TState>(ref TAwaiter awaiter, ref TState stateMachine) where TAwaiter : ITaskAwaiter where TState : IAsyncStateMachine { }
            }
        }
        """;

    // Application metadata exposes the ref compiler protocol; the bootstrap
    // library implements the retained-owner operations used by its specialization.
    public static string ForReference(bool libraryBootstrap) => !libraryBootstrap ? Declarations : Declarations
        .Replace("public void Start<TState>(ref TState stateMachine) where TState : IAsyncStateMachine { }",
            "public void Start(IAsyncStateMachine stateMachine) { }")
        .Replace("public void AwaitOnCompleted<TAwaiter, TState>(ref TAwaiter awaiter, ref TState stateMachine) where TAwaiter : ITaskAwaiter where TState : IAsyncStateMachine { }",
            "public void AwaitOnCompleted(ITaskAwaiter awaiter, IAsyncStateMachine stateMachine) { }");

    public static void Project(ModuleDefinition module)
    {
        var current = module.GetType(Queue).Methods.Single(m => m.Name == "get_Current");
        current.Attributes = (current.Attributes & ~MethodAttributes.MemberAccessMask) | MethodAttributes.Assembly;
        foreach (var method in module.GetType(Prefix + "Task`1").Methods.Where(m => m.IsConstructor || m.Name == "Dispatcher")
            .Concat(module.GetType(Prefix + "Promise`1").Methods.Where(m => m.Name is "Completed" or "Cancelled" or "Read" or "Register" or "Dispatcher")))
            method.Attributes = (method.Attributes & ~MethodAttributes.MemberAccessMask) | MethodAttributes.Assembly;
    }

    public static bool SameType(TypeReference left, TypeReference right) =>
        left.FullName == right.FullName && left.FullName is Prefix + "Task`1" or Prefix + "Promise`1" or Queue
        && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);

    public static string? Type(TypeReference type)
    {
        if (type.IsValueType || !RuntimeSignatures.IsCore(type.Scope)) return null;
        if (type.FullName == Queue) { Shapes[Queue] = ("TaskQueue", ""); return Queue; }
        if (type is not GenericInstanceType g || g.GenericArguments.Count != 1) return null;
        var kind = g.ElementType.FullName switch
        {
            Prefix + "Task`1" => "Task",
            Prefix + "Promise`1" => "Promise",
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
        var internalMember = kind == "TaskQueue" && definition.Name == "get_Current"
            || kind == "Task" && (definition.IsConstructor || definition.Name == "Dispatcher")
            || kind == "Promise" && definition.Name is "Completed" or "Cancelled" or "Read" or "Register" or "Dispatcher";
        if (internalMember ? !library || !definition.IsAssembly : !definition.IsPublic)
            throw new InvalidDataException("Invalid Task member visibility.");
        if (kind == "Task" && (definition.DeclaringType.Interfaces.Count != 1
            || definition.DeclaringType.Interfaces[0].InterfaceType.FullName != "System.Runtime.CompilerServices.ITaskAwaiter"
            || !RuntimeSignatures.IsCore(definition.DeclaringType.Interfaces[0].InterfaceType.Scope)))
            throw new InvalidDataException("Invalid Task awaiter interface.");
        var staticMember = kind == "TaskQueue" && definition.Name is "get_Current" or "get_Default";
        if (!definition.DeclaringType.IsSealed || definition.DeclaringType.IsInterface
            || (kind != "Task" && definition.DeclaringType.HasInterfaces)
            || definition.DeclaringType.GenericParameters.Any(p => p.HasConstraints || p.Attributes != GenericParameterAttributes.NonVariant)
            || reference.HasThis == staticMember || definition.IsStatic != staticMember || definition.HasGenericParameters
            || !definition.IsPublic && !(library && definition.IsAssembly)
            || definition.IsConstructor != construct)
            throw new InvalidDataException("Unsupported provisional Task contract.");
        var expected = (kind, definition.Name) switch
        {
            ("TaskQueue", "get_Default") => ("", Queue),
            ("TaskQueue", "get_Current") when library => ("", Queue),
            ("TaskQueue", ".ctor") => ("", "noresult"),
            ("TaskQueue", "Post" or "Run") => ("System.Func<Void>", "noresult"),
            ("TaskQueue", "Drain") => ("", "noresult"),
            ("Task", ".ctor") => (Prefix + "Promise<" + payload + ">", "noresult"),
            ("Task", "get_State") => ("", EnumBindings.TaskState),
            ("Task", "get_Outcome") => ("", "System.Option<System.Tasks.TaskOutcome<" + payload + ">>"),
            ("Task", "get_IsCompleted" or "get_IsCancelled") => ("", "Boolean"),
            ("Task" or "Promise", "Dispatcher") when library => ("", Queue),
            ("Task", "GetAwaiter") => ("", owner),
            ("Task", "GetResult") => ("", payload),
            ("Task", "OnCompleted") => ("System.Func<Void>", "noresult"),
            ("Promise", ".ctor") when definition.Parameters.Count == 0 => ("", "noresult"),
            ("Promise", ".ctor") => (Queue, "noresult"),
            ("Promise", "get_Task") => ("", Prefix + "Task<" + payload + ">"),
            ("Promise", "Cancel") => ("", "Boolean"),
            ("Promise", "Complete") => (payload, "Boolean"),
            ("Promise", "Completed" or "Cancelled") when library => ("", "Boolean"),
            ("Promise", "Read") when library => ("", payload),
            ("Promise", "Register") when library => ("System.Func<Void>", "noresult"),
            _ => throw new InvalidDataException("Unsupported Task member: " + definition.FullName)
        };
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type,
            allowOpenMethodParameters: library);
        if (string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported Task signature.");
        return construct
            ? new(args, owner, $"newobj instance {owner}::.ctor({string.Join(',', args)})")
            : new(staticMember ? args : new[] { owner }.Concat(args).ToArray(), result, $"call {(staticMember ? "" : "instance ")}{owner}::{definition.Name}({string.Join(',', args)})");
    }
}
