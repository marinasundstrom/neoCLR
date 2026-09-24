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
            _ => definition.Name is ".ctor" or "Create" or "get_Task" or "SetResult" or "SetCancelled" or "Start" or "SetStateMachine" or "HasStateMachine" or "GetStateMachine" or "AwaitOnCompleted"
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
            "SetCancelled" => ("", "noresult"),
            "SetResult" => (payload!, "noresult"),
            "Start" or "SetStateMachine" => (state, "noresult"),
            "HasStateMachine" => ("", "Boolean"),
            "GetStateMachine" => ("", state),
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

    // Compiler-facing ref protocol is specialized at the importer boundary. No
    // managed reference is stored: only the copied first suspended state escapes.
    public static CollectionBindings.Binding? BindByRef(MethodReference reference, MethodDefinition definition,
        string suffix, Func<TypeReference, string> map)
    {
        if (reference is not GenericInstanceMethod generic || Type(reference.DeclaringType) is not { } owner
            || !owner.StartsWith(Prefix + "AsyncTaskMethodBuilder<")) return null;
        _ = RuntimeSignatures.Match(reference, definition, t => map(t));
        if (!definition.DeclaringType.IsSealed || definition.DeclaringType.IsValueType || definition.IsConstructor)
            throw new InvalidDataException("Invalid by-reference async builder owner.");
        var start = definition.Name == "Start";
        if ((!start && definition.Name != "AwaitOnCompleted") || !definition.IsPublic || definition.IsStatic
            || definition.ReturnType.MetadataType != MetadataType.Void
            || generic.GenericArguments.Count != (start ? 1 : 2)
            || definition.GenericParameters.Count != generic.GenericArguments.Count
            || definition.Parameters.Count != generic.GenericArguments.Count)
            throw new InvalidDataException("Invalid by-reference async contract.");
        for (var i = 0; i < definition.Parameters.Count; i++)
        {
            if (definition.Parameters[i].IsOut || definition.Parameters[i].ParameterType is not ByReferenceType { ElementType: GenericParameter parameter }
                || parameter.Position != i || parameter.Type != GenericParameterType.Method)
                throw new InvalidDataException("Async protocol requires exact ref generic parameters.");
            var expected = Prefix + (start || i == 1 ? "IAsyncStateMachine" : "ITaskAwaiter");
            var constraints = definition.GenericParameters[i].Constraints;
            if (constraints.Count != 1 || constraints[0].ConstraintType.FullName != expected
                || !RuntimeSignatures.IsCore(constraints[0].ConstraintType.Scope))
                throw new InvalidDataException("Invalid async generic constraint.");
        }
        var stateType = generic.GenericArguments.Last();
        var state = map(stateType);
        var stateDefinition = stateType.Resolve();
        if (!ApplicationTypes.IsModule(stateDefinition.Module) || stateDefinition.HasGenericParameters
            || !stateDefinition.Interfaces.Any(i => i.InterfaceType.FullName == Prefix + "IAsyncStateMachine" && RuntimeSignatures.IsCore(i.InterfaceType.Scope)))
            throw new InvalidDataException("Async state must be an admitted non-generic application state machine.");
        var machine = Prefix + "IAsyncStateMachine";
        if (start)
        {
            var load = stateDefinition.IsValueType ? "" : "ldobj " + state + "\n";
            return new([owner, state + "&"], "noresult", load + $"call instance {state}::MoveNext()\npop");
        }
        var awaiter = map(generic.GenericArguments[0]);
        if (!awaiter.StartsWith("System.Tasks.Task<"))
            throw new InvalidDataException("Only target Task awaiters are admitted by the ref async protocol.");
        var id = "async_" + suffix;
        var promote = stateDefinition.IsValueType ? $"box {state}\ncastclass {machine}" : $"castclass {machine}";
        var body = $"""
.local {owner} {id}_builder
.local {state}& {id}_state
.local {awaiter}& {id}_awaiter
stloc {id}_state
stloc {id}_awaiter
stloc {id}_builder
ldloc {id}_builder
call instance {owner}::HasStateMachine()
brtrue {id}_retained
ldloc {id}_builder
ldloc {id}_state
ldobj {state}
{promote}
call instance {owner}::SetStateMachine({machine})
{id}_retained:
ldloc {id}_builder
ldloc {id}_awaiter
ldobj {awaiter}
castclass {Prefix}ITaskAwaiter
ldloc {id}_builder
call instance {owner}::GetStateMachine()
call instance {owner}::AwaitOnCompleted({Prefix}ITaskAwaiter,{machine})
""";
        return new([owner, awaiter + "&", state + "&"], "noresult", body);
    }
}
