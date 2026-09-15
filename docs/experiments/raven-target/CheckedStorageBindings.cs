using Mono.Cecil;

// Bootstrap authoring intrinsic, absent from the consumer reference assembly.
// Unlike newarr this reserves tracked, uninitialized elements; reads remain checked.
static class CheckedStorageBindings
{
    const string Owner = "System.Runtime.CompilerServices.CheckedStorage";
    public const string Declarations = """
        namespace Runtime.CompilerServices {
            public static class CheckedStorage {
                public static T[] Reserve<T>(int length) => default;
            }
        }
        """;
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, Func<TypeReference, string> map)
    {
        if (reference.DeclaringType.FullName != Owner) return null;
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || reference.Name != "Reserve"
            || reference.HasThis || definition.IsVirtual || reference is not GenericInstanceMethod instance
            || instance.GenericArguments.Count != 1 || definition.GenericParameters.Count != 1
            || definition.GenericParameters.Any(p => p.HasConstraints || p.Attributes != GenericParameterAttributes.NonVariant)
            || definition.ReturnType is not ArrayType { IsVector: true, ElementType: GenericParameter parameter }
            || parameter.Owner != definition || parameter.Position != 0)
            throw new InvalidDataException("Unsupported checked-storage intrinsic.");
        var signature = RuntimeSignatures.Match(reference, definition, t => map(t), allowOpenMethodParameters: true);
        var element = map(instance.GenericArguments[0]);
        if (!signature.Args.SequenceEqual(new[] { "Int32" }) || signature.Result != "arrayref<" + element + ">")
            throw new InvalidDataException("Invalid checked-storage signature.");
        return new("", signature.Args, signature.Result, Instruction: "array.reserve " + element);
    }
}
