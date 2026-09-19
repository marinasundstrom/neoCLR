using Mono.Cecil;

// Members declared on the configured generic managed-array shape.
static class ArrayCallbackBindings
{
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, Func<TypeReference, string>? libraryMap = null)
    {
        if (reference.DeclaringType.GetElementType().FullName != "System.Array`1") return null;
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope)
            || reference.DeclaringType is not GenericInstanceType owner || owner.GenericArguments.Count != 1
            || reference is GenericInstanceMethod || definition.DeclaringType.IsValueType || definition.DeclaringType.IsInterface
            || definition.DeclaringType.GenericParameters.Count != 1
            || definition.DeclaringType.GenericParameters.Any(p => p.HasConstraints || p.Attributes != GenericParameterAttributes.NonVariant))
            throw new InvalidDataException("Unsupported Array<T> member signature.");
        var element = libraryMap is null ? GenericUnionBindings.Type(owner.GenericArguments[0]) : libraryMap(owner.GenericArguments[0]);
        if (element is null || !ManagedArrayBindings.Defaultable(element) && !(libraryMap is not null && owner.GenericArguments[0] is GenericParameter))
            throw new InvalidDataException("Unsupported Array<T> element.");
        var (args, result) = RuntimeSignatures.Match(reference, definition,
            t => libraryMap is null ? ManagedArrayBindings.Type(t) ?? DelegateBindings.Type(t) ?? GenericUnionBindings.Type(t) : libraryMap(t));
        var array = $"arrayref<{element}>";
        var type = $"System.Array<{element}>";
        if (libraryMap is not null && reference.Name == "get_Length" && reference.HasThis && args.Length == 0 && result == "Int32")
            return new("", [array], result, Instruction: "ldlen\nconv.i4");
        if (reference.Name == "get_Empty" && !reference.HasThis && args.Length == 0 && result == array)
            return new(type + "::get_Empty", args, result,
                Instruction: $"call {type}::get_Empty()");
        if (reference.Name == "ForEach" && reference.HasThis && result == "noresult"
            && args.SequenceEqual(new[] { $"System.Func<{element},Void>" }))
            return new(type + "::ForEach", new[] { array }.Concat(args).ToArray(), result,
                Instruction: $"callvirt instance {type}::ForEach({string.Join(',', args)})");
        throw new InvalidDataException("Unsupported Array<T> member contract.");
    }
}
