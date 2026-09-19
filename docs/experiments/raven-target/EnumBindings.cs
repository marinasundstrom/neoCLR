using Mono.Cecil;
using System.Text;

// Preserve CLI enum metadata while adapting the interpreter's nominal value layout.
static class EnumBindings
{
    public const string Flags = "System.Introspection.BindingFlags";
    static readonly (string Name, int Value)[] Literals = [("Default", 0), ("DeclaredOnly", 2), ("Instance", 4), ("Static", 8), ("Public", 16), ("NonPublic", 32)];
    public static void Validate(ModuleDefinition module)
    {
        var type = module.GetType(Flags);
        if (type is null || !type.IsPublic || !type.IsSealed || type.IsNested || type.HasProperties || type.HasEvents || type.HasNestedTypes
            || type.BaseType?.FullName != "System.Enum" || !RuntimeSignatures.IsCore(type.BaseType.Scope) || !type.IsEnum || type.HasGenericParameters || type.Interfaces.Count != 0 || type.Methods.Count != 0
            || type.CustomAttributes.Count(a => a.AttributeType.FullName == "System.FlagsAttribute") != 1
            || type.Fields.Count(f => !f.IsStatic) != 1
            || type.Fields.Single(f => !f.IsStatic) is not { Name: "value__", IsSpecialName: true, IsRuntimeSpecialName: true, FieldType.MetadataType: MetadataType.Int32 }
            || !type.Fields.Where(f => f.IsStatic).Select(f => (f.Name, f.IsLiteral && f.Constant is int n ? n : int.MinValue)).Order().SequenceEqual(Literals.Order()))
            throw new InvalidDataException("Unsupported BindingFlags enum metadata.");
    }
    // CLI enums have literals and an underlying value field, not authored method
    // bodies. Lower their common operations to the existing nominal enum ABI,
    // mirroring src/enums.rs so Raven and archived Neo use the same semantics.
    public static string Declaration(TypeDefinition type)
    {
        var name = type.FullName;
        var result = new StringBuilder($".type {name}\n.enum Int32 flags\n.field private Bits Int32\n");
        foreach (var member in type.Fields.Where(f => f.IsLiteral))
            result.AppendLine($".literal {member.Name} {member.Constant}");
        void Method(string signature, string body) => result.AppendLine($".method {signature}\n{body}\nret\n.end");
        Method($"static FromValue(Int32 value) -> {name}", $"ldarg value\nnewobj {name}");
        Method("instance get_Value() -> Int32", "ldarg this\nldfld 0");
        foreach (var (method, opcode) in new[] { ("Or", "or"), ("And", "and"), ("Xor", "xor") })
            Method($"instance {method}({name} other) -> {name}", $"ldarg this\nldfld 0\nldarg other\nldfld 0\n{opcode}\nnewobj {name}");
        Method($"instance Not() -> {name}", $"ldarg this\nldfld 0\nnot\nnewobj {name}");
        Method($"instance HasFlag({name} other) -> Boolean", "ldarg this\nldfld 0\nldarg other\nldfld 0\nand\nldarg other\nldfld 0\nceq");
        Method($"instance Equals({name} other) -> Boolean", "ldarg this\nldfld 0\nldarg other\nldfld 0\nceq");
        foreach (var member in type.Fields.Where(f => f.IsLiteral))
            Method($"static {member.Name}() -> {name}", $"ldc.i4 {member.Constant}\nnewobj {name}");
        result.AppendLine(".end");
        return result.ToString();
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
