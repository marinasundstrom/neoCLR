using Mono.Cecil;

static class WorkerBindings
{
    const string Prefix = "System.Concurrency.";
    public static bool IsName(string name) => name is Prefix + "Thread" or Prefix + "ThreadPool" or Prefix + "WorkerCompletion";
    public static bool SameType(TypeReference left, TypeReference right) => left.FullName == right.FullName
        && IsName(left.FullName) && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);
    public static string? Type(TypeReference type) => RuntimeSignatures.IsCore(type.Scope) && !type.IsValueType && IsName(type.FullName) ? type.FullName : null;
    public const string Declarations = """
        namespace Concurrency {
            public sealed class WorkerCompletion {
                public WorkerCompletion(int handle, Tasks.Promise<string> source) { }
                public void Complete() { }
            }
            public sealed class Thread {
                public Thread(Func<string, string> callback, string input) { }
                public Tasks.Task<string> Task => default;
                public bool IsStarted => default;
                public void Start() { }
                public static Tasks.Task<string> Run(Func<string, string> callback, string input) => default;
            }
            public sealed class ThreadPool {
                public static Tasks.Task<string> Queue(Func<string, string> callback, string input) => default;
            }
        }
        """;
    public static bool IsProvider(TypeDefinition type) => type.FullName == Prefix + "WorkerCompletion";
    public static void Project(ModuleDefinition module)
    {
        var type = module.GetType(Prefix + "WorkerCompletion");
        type.Attributes = (type.Attributes & ~TypeAttributes.VisibilityMask) | TypeAttributes.NotPublic;
        foreach (var method in type.Methods)
            method.Attributes = (method.Attributes & ~MethodAttributes.MemberAccessMask) | MethodAttributes.Assembly;
    }
    public static CollectionBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool construct, bool library)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var expected = (owner, definition.Name) switch {
            (Prefix + "WorkerCompletion", ".ctor") when library => ("Int32,System.Tasks.Promise<String>", "noresult", false),
            (Prefix + "WorkerCompletion", "Complete") when library => ("", "noresult", false),
            (Prefix + "Thread", "Run") or (Prefix + "ThreadPool", "Queue") => ("System.Func<String,String>,String", "System.Tasks.Task<String>", true),
            (Prefix + "Thread", ".ctor") => ("System.Func<String,String>,String", "noresult", false),
            (Prefix + "Thread", "Start") => ("", "noresult", false),
            (Prefix + "Thread", "get_Task") => ("", "System.Tasks.Task<String>", false),
            (Prefix + "Thread", "get_IsStarted") => ("", "Boolean", false),
            _ => throw new InvalidDataException("Unsupported worker member.")
        };
        if (definition.IsConstructor != construct || definition.IsStatic != expected.Item3
            || !(definition.IsPublic || library && definition.IsAssembly)
            || definition.IsVirtual || definition.HasGenericParameters || !definition.DeclaringType.IsSealed
            || definition.DeclaringType.HasGenericParameters || string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported worker signature.");
        var signature = string.Join(',', args);
        return construct ? new(args, owner, $"newobj instance {owner}::.ctor({signature})")
            : new(definition.IsStatic ? args : new[] { owner }.Concat(args).ToArray(), result,
                $"call {(definition.IsStatic ? "" : "instance ")}{owner}::{definition.Name}({signature})");
    }
}
