using Mono.Cecil;

// Preserve CLI enum metadata while adapting the interpreter's nominal value layout.
static class EnumBindings
{
    public const string Flags = "System.Introspection.BindingFlags";
    static readonly (string Name, int Value)[] Literals = [("Default", 0), ("DeclaredOnly", 2), ("Instance", 4), ("Static", 8), ("Public", 16), ("NonPublic", 32)];
    public static void Validate(ModuleDefinition module)
    {
        var type = module.GetType(Flags);
        if (type is null || !type.IsEnum || type.HasGenericParameters || type.Interfaces.Count != 0 || type.Methods.Count != 0
            || type.CustomAttributes.Count(a => a.AttributeType.FullName == "System.FlagsAttribute") != 1
            || type.Fields.Count(f => !f.IsStatic) != 1
            || type.Fields.Single(f => !f.IsStatic) is not { Name: "value__", IsSpecialName: true, IsRuntimeSpecialName: true, FieldType.MetadataType: MetadataType.Int32 }
            || !type.Fields.Where(f => f.IsStatic).Select(f => (f.Name, f.IsLiteral && f.Constant is int n ? n : int.MinValue)).Order().SequenceEqual(Literals.Order()))
            throw new InvalidDataException("Unsupported BindingFlags enum metadata.");
    }
    public static bool Converts(string source, string target) => source == Flags && target == "Int32" || source == "Int32" && target == Flags;
    public static string Convert(string source, string target) => source == Flags && target == "Int32"
        ? $"call instance {Flags}::get_Value()\n" : source == "Int32" && target == Flags ? $"call {Flags}::FromValue(Int32)\n" : "";
    public const string Adapters = """
        .function RuntimeBitsOr(Int32 left,Int32 right) -> Int32
        ldarg left
        ldarg right
        or
        ret
        .end
        .function RuntimeBitsAnd(Int32 left,Int32 right) -> Int32
        ldarg left
        ldarg right
        and
        ret
        .end
        .function RuntimeBitsXor(Int32 left,Int32 right) -> Int32
        ldarg left
        ldarg right
        xor
        ret
        .end
        """;
}
