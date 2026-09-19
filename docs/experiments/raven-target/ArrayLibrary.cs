using Mono.Cecil;
using Mono.Cecil.Cil;

// CLI vector storage and the existing generic array iteration ABI. Managed loops,
// element access, iterator state and factories remain authored in Array.rvn.
static class ArrayLibrary
{
    public static bool IsMatched(TypeDefinition type) => type.FullName == "System.Array`1" && ApplicationTypes.IsLibrary(type);
    public static bool IsIterator(MethodDefinition method) => IsMatched(method.DeclaringType) && method.Name == "GetIterator";
    public static bool OmitConstructor(MethodDefinition method) => method.DeclaringType.FullName == "System.Array`1" && method.IsConstructor;
    public static void Project(ModuleDefinition module)
    {
        // Length is intrinsic on CLI vectors; expose it only for source authoring.
        var type = module.GetType("System.Array`1");
        var getter = new MethodDefinition("get_Length", MethodAttributes.Public | MethodAttributes.HideBySig | MethodAttributes.SpecialName, module.TypeSystem.Int32);
        type.Methods.Add(getter);
        type.Properties.Add(new PropertyDefinition("Length", PropertyAttributes.None, module.TypeSystem.Int32) { GetMethod = getter });
    }
    public static void Validate(TypeDefinition type)
    {
        if (type.Fields.Count != 1 || type.Fields[0] is not { Name: "m_value", IsPrivate: true, IsStatic: false, FieldType: ArrayType { IsVector: true, ElementType: GenericParameter parameter } }
            || parameter.Position != 0 || parameter.Owner != type
            || type.Methods.Count(m => m.IsConstructor) != 1 || !EmptyConstructor(type.Methods.Single(m => m.IsConstructor)))
            throw new InvalidDataException("Unsupported intrinsic array storage or constructor.");
    }
    static bool EmptyConstructor(MethodDefinition method)
    {
        if (!method.IsPrivate || method.IsStatic || method.HasParameters || method.HasGenericParameters
            || !method.HasBody || method.Body.HasVariables || method.Body.HasExceptionHandlers) return false;
        var body = method.Body.Instructions.Where(i => i.OpCode.Code != Code.Nop).ToArray();
        return body.Length == 3 && body[0].OpCode.Code == Code.Ldarg_0 && body[2].OpCode.Code == Code.Ret
            && body[1].OpCode.Code == Code.Call && body[1].Operand is MethodReference call
            && call.DeclaringType.FullName == "System.Object" && RuntimeSignatures.IsCore(call.DeclaringType.Scope)
            && call.Name == ".ctor" && call.HasThis && !call.HasParameters && !call.HasGenericParameters
            && call.ReturnType.MetadataType == MetadataType.Void;
    }
    public static string IteratorAdapter(string body)
    {
        const string signature = ".method instance GetIterator() -> System.Collections.Iterator<T0>\n";
        if (!body.StartsWith(signature, StringComparison.Ordinal)) throw new InvalidDataException("Unexpected array iterator signature.");
        // The existing array/interface dispatcher passes its receiver as the single
        // static argument. Argument zero and the checked body are unchanged.
        return ".type internal System.Collections.ArrayEnumerable\n.method internal static GetIterator<T0>(arrayref<T0> source) -> System.Collections.Iterator<T0>\n"
            + body[signature.Length..] + ".end\n";
    }
}
