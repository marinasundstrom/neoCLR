using Mono.Cecil;

// Bootstrap-only operations. Raven owns the overload composition; checked native
// arithmetic and allocation/lifetime checks are runtime instructions.
static class NativeAllocationBindings
{
    const string Owner = "System.Runtime.CompilerServices.NativeAllocation";
    public const string Declarations = """
        namespace Runtime.CompilerServices {
            public static unsafe class NativeAllocation {
                public static void* Allocate(nuint byteCount) => default;
                public static nuint MultiplyChecked(nuint left, nuint right) => default;
                public static void Release(void* pointer) { }
            }
        }
        """;
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition, Func<TypeReference, string> map)
    {
        if (reference.DeclaringType.FullName != Owner) return null;
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || reference.HasThis
            || reference.HasGenericParameters || reference.ExplicitThis || definition.IsVirtual)
            throw new InvalidDataException("Unsupported native allocation intrinsic.");
        var signature = RuntimeSignatures.Match(reference, definition, map, pointers: true);
        var expected = reference.Name switch {
            "Allocate" => (new[] { "UIntPtr" }, "Void*", "heap.alloc Byte\nptr.cast Void"),
            "MultiplyChecked" => (new[] { "UIntPtr", "UIntPtr" }, "UIntPtr", "mul.ovf.un"),
            "Release" => (new[] { "Void*" }, "noresult", "heap.free\npop"),
            _ => throw new InvalidDataException("Unsupported native allocation operation.")
        };
        if (!signature.Args.SequenceEqual(expected.Item1) || signature.Result != expected.Item2)
            throw new InvalidDataException("Invalid native allocation signature.");
        return new("", signature.Args, signature.Result, Instruction: expected.Item3);
    }
}
