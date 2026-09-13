using Mono.Cecil;

static class NativeMemoryBindings
{
    public const string Declaration = """
        namespace Runtime.InteropServices {
            public static unsafe class NativeMemory {
                public static void* Alloc(nuint byteCount) => default;
                public static void* Alloc(nuint elementCount, nuint elementSize) => default;
                public static void Free(void* pointer) { }
            }
        }
        """;
    public const string Owner = "System.Runtime.InteropServices.NativeMemory";
    public static bool IsPointer(string type) => type.EndsWith('*') &&
        (type[..^1] is "Void" or "Int32" or "Double" or "Boolean" || PrimitiveBindings.Types.Contains(type[..^1]));
    public static string? Type(TypeReference type)
    {
        if (type is not PointerType pointer) return null;
        // Raven may materialize its empty Unit carrier in a pointer local even when
        // the imported native method correctly retains CLI void*. Project that
        // empty carrier under the same target-specific Unit/Void policy as slots.
        var pointee = pointer.ElementType;
        var unit = pointee.FullName == "System.Unit" && pointee.IsValueType
            && pointee.Resolve() is { } definition && !definition.Fields.Any(f => !f.IsStatic);
        var element = pointee.MetadataType == MetadataType.Void || unit ? "Void" : GenericUnionBindings.Type(pointee);
        return element is not null && IsPointer(element + "*") ? element + "*" : null;
    }
    public static void Validate(ModuleDefinition module)
    {
        var type = module.GetType(Owner);
        if (type is null || !type.IsAbstract || !type.IsSealed || type.HasGenericParameters
            || type.Fields.Count != 0 || type.Interfaces.Count != 0 || type.Methods.Count != 3)
            throw new InvalidDataException("Unsupported NativeMemory metadata.");
        foreach (var method in type.Methods) Bind(method, method);
    }
    public static ResultBindings.Binding? Bind(MethodReference reference, MethodDefinition definition)
    {
        if (reference.DeclaringType.FullName != Owner) return null;
        if (!RuntimeSignatures.IsCore(reference.DeclaringType.Scope) || reference.HasThis || reference.HasGenericParameters)
            throw new InvalidDataException("Unsupported NativeMemory member.");
        var (args, result) = RuntimeSignatures.Match(reference, definition, t => Type(t) ?? GenericUnionBindings.Type(t), pointers: true);
        var valid = reference.Name switch {
            "Alloc" => result == "Void*" && (args.SequenceEqual(new[] { "UIntPtr" }) || args.SequenceEqual(new[] { "UIntPtr", "UIntPtr" })),
            "Free" => result == "noresult" && args.SequenceEqual(new[] { "Void*" }),
            _ => false
        };
        if (!valid) throw new InvalidDataException("Unsupported NativeMemory signature.");
        return new($"{Owner}::{reference.Name}", args, result);
    }
}
