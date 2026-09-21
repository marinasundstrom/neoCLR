using Mono.Cecil;

// Bounded provisional compiler protocol; Task payloads remain ordinary types.
static class AsyncBindings
{
    const string Prefix = "System.Runtime.CompilerServices.";
    public static bool IsName(string name) => name is Prefix + "IAsyncStateMachine" or Prefix + "ITaskAwaiter" or Prefix + "AsyncTaskMethodBuilder`1";
    public static bool SameType(TypeReference left, TypeReference right) => left.FullName == right.FullName
        && IsName(left.FullName) && RuntimeSignatures.IsCore(left.Scope) && ApplicationTypes.IsLibrary(right);
    public static string? Type(TypeReference type)
    {
        if (!RuntimeSignatures.IsCore(type.Scope) || type.IsValueType) return null;
        if (type.FullName is Prefix + "IAsyncStateMachine" or Prefix + "ITaskAwaiter") return type.FullName;
        if (type is GenericInstanceType g && g.ElementType.FullName == Prefix + "AsyncTaskMethodBuilder`1"
            && g.GenericArguments.Count == 1 && GenericUnionBindings.Type(g.GenericArguments[0]) is { } payload)
            return Prefix + "AsyncTaskMethodBuilder<" + payload + ">";
        return null;
    }
    public static bool IsType(string name) => name.StartsWith(Prefix + "AsyncTaskMethodBuilder<")
        || name is Prefix + "IAsyncStateMachine" or Prefix + "ITaskAwaiter";
    public static CollectionBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, bool construct)
    {
        var owner = Type(reference.DeclaringType);
        if (owner is null) return null;
        if (!definition.IsPublic || definition.HasGenericParameters || definition.IsConstructor != construct
            || definition.IsStatic != (definition.Name == "Create") || definition.DeclaringType.IsValueType)
            throw new InvalidDataException("Invalid provisional async member.");
        var contract = definition.DeclaringType;
        var interfaceOwner = owner is Prefix + "IAsyncStateMachine" or Prefix + "ITaskAwaiter";
        var allowed = owner switch {
            Prefix + "IAsyncStateMachine" => definition.Name is "MoveNext" or "SetStateMachine",
            Prefix + "ITaskAwaiter" => definition.Name == "OnCompleted",
            _ => definition.Name is ".ctor" or "Create" or "get_Task" or "SetResult" or "Start" or "SetStateMachine" or "AwaitOnCompleted"
        };
        if (!allowed || contract.IsInterface != interfaceOwner || contract.HasInterfaces
            || (!interfaceOwner && !contract.IsSealed)
            || contract.GenericParameters.Any(p => p.HasConstraints || p.Attributes != GenericParameterAttributes.NonVariant)
            || (interfaceOwner && (!definition.IsAbstract || !definition.IsVirtual)))
            throw new InvalidDataException("Invalid provisional async owner.");
        var (args, result) = RuntimeSignatures.Match(reference, definition, GenericUnionBindings.Type);
        var state = Prefix + "IAsyncStateMachine";
        var payload = reference.DeclaringType is GenericInstanceType g ? GenericUnionBindings.Type(g.GenericArguments[0]) : null;
        var expected = definition.Name switch {
            ".ctor" => ("System.Tasks.TaskQueue", "noresult"),
            "Create" => ("", owner),
            "get_Task" => ("", "System.Tasks.Task<" + payload + ">"),
            "SetResult" => (payload!, "noresult"),
            "Start" or "SetStateMachine" => (state, "noresult"),
            "AwaitOnCompleted" => (Prefix + "ITaskAwaiter," + state, "noresult"),
            "MoveNext" => ("", "noresult"),
            "OnCompleted" => ("System.Func<Void>", "noresult"),
            _ => throw new InvalidDataException("Unsupported async member: " + definition.FullName)
        };
        if (string.Join(',', args) != expected.Item1 || result != expected.Item2)
            throw new InvalidDataException("Unsupported async signature.");
        var signature = string.Join(',', args);
        return construct ? new(args, owner, $"newobj instance {owner}::.ctor({signature})")
            : new(definition.IsStatic ? args : new[] { owner }.Concat(args).ToArray(), result,
                $"{(definition.DeclaringType.IsInterface ? "callvirt" : "call")} {(definition.IsStatic ? "" : "instance ")}{owner}::{definition.Name}({signature})");
    }
}
