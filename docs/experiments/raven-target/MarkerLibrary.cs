using Mono.Cecil;
using Mono.Cecil.Cil;

// Explicitly admitted Object members and the empty union marker declaration.
// Equality and hashing remain compiler-facing reference metadata.
static class MarkerLibrary
{
    public static bool IsOwner(string owner) => owner is "System.Object" or "System.Runtime.CompilerServices.UnionAttribute";
    public static bool IsMatched(TypeDefinition type) => IsOwner(type.FullName) && ApplicationTypes.IsLibrary(type);
    public static MethodDefinition[] Roots(ModuleDefinition source, ModuleDefinition core, string owner)
    {
        var type = source.GetType(owner);
        var contract = core.GetType(owner);
        var isObject = owner == "System.Object";
        var baseName = isObject ? "System.Object" : "System.Attribute";
        if (type is null || contract is null || !type.IsPublic || !type.IsClass || type.IsValueType
            || type.IsAbstract != isObject || type.IsSealed == isObject || type.HasGenericParameters
            || type.HasFields || type.HasInterfaces || type.HasProperties || type.HasEvents || type.HasNestedTypes
            || type.IsExplicitLayout || type.BaseType?.FullName != baseName || !RuntimeSignatures.IsCore(type.BaseType.Scope)
            || !contract.IsPublic || !contract.IsClass || contract.HasFields || contract.HasGenericParameters
            || contract.IsAbstract != type.IsAbstract || contract.IsSealed != type.IsSealed || type.Methods.Count != (isObject ? 3 : 1) || !EmptyConstructor(type.Methods.Single(m => m.IsConstructor), baseName))
            throw new InvalidDataException("Unsupported empty marker declaration: " + owner);
        ApplicationTypes.BindLibrary(type, owner);
        _ = ApplicationTypes.Type(type);
        if (!isObject) return [];
        var method = type.Methods.Single(m => m.Name == "GetType");
        var expected = contract.Methods.Single(m => m.Name == "GetType");
        if (method.Name != "GetType" || !method.IsPublic || method.IsStatic || method.IsVirtual
            || method.HasParameters || method.HasGenericParameters || !method.HasBody
            || !LibraryImplementation.SameType(expected.ReturnType, method.ReturnType))
            throw new InvalidDataException("Unsupported Object.GetType contract.");
        var display = type.Methods.Single(m => m.Name == "ToString");
        var expectedDisplay = contract.Methods.Single(m => m.Name == "ToString");
        if (!display.IsPublic || display.IsStatic || !display.IsVirtual || !display.IsNewSlot
            || display.IsFinal || display.HasParameters || display.HasGenericParameters || !display.HasBody
            || !LibraryImplementation.SameType(expectedDisplay.ReturnType, display.ReturnType))
            throw new InvalidDataException("Unsupported Object.ToString contract.");
        return [method, display];
    }
    static bool EmptyConstructor(MethodDefinition method, string baseName)
    {
        if (!method.IsConstructor || !(baseName == "System.Object" ? method.IsFamily : method.IsPublic) || method.IsStatic || method.HasParameters
            || method.HasGenericParameters || method.ExplicitThis || method.IsVirtual
            || method.CallingConvention != MethodCallingConvention.Default || method.ReturnType.MetadataType != MetadataType.Void
            || !method.HasBody || method.Body.HasVariables || method.Body.HasExceptionHandlers) return false;
        var body = method.Body.Instructions.Where(i => i.OpCode.Code != Code.Nop).ToArray();
        return body.Length == 3 && body[0].OpCode.Code == Code.Ldarg_0
            && body[1].OpCode.Code == Code.Call && body[2].OpCode.Code == Code.Ret
            && body[1].Operand is MethodReference call && call.DeclaringType.FullName == baseName
            && RuntimeSignatures.IsCore(call.DeclaringType.Scope) && call.Name == ".ctor" && call.HasThis
            && !call.HasParameters && !call.HasGenericParameters && !call.ExplicitThis
            && call.CallingConvention == MethodCallingConvention.Default && call.ReturnType.MetadataType == MetadataType.Void;
    }
    public static string Declaration(TypeDefinition type, Dictionary<MethodDefinition, string> bodies) => type.FullName == "System.Object"
        ? ".type class abstract System.Object\n.method instance .ctor() -> noresult\nret\n.end\n" + string.Concat(bodies.Where(p => p.Key.DeclaringType == type).Select(p => p.Value)) + ".end\n"
        : ".type System.Runtime.CompilerServices.UnionAttribute\n.method instance .ctor() -> Void\nldvoid\nret\n.end\n.end\n";
}
