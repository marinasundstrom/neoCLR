using Mono.Cecil;
using Mono.Cecil.Cil;

// Only explicitly matched bootstrap implementations may project opaque storage.
static class OpaqueLibrary
{
    public static bool IsString(TypeDefinition type) => type.FullName == "System.String" && ApplicationTypes.IsLibrary(type);
    public static bool IsByRefString(MethodReference method) => IsString(method.DeclaringType.Resolve())
        && method.HasThis && method.Name is "Equals" or "ContainsOrdinal" or "StartsWithOrdinal" or "EndsWithOrdinal" or "GetIterator";
    public static bool IsOmittedConstructor(MethodDefinition method) =>
        method.DeclaringType.FullName == "System.String" && IsStringConstructor(method);
    static bool IsStringConstructor(MethodDefinition method)
    {
        if (!method.IsConstructor || !method.IsPublic || method.IsStatic || method.HasParameters
            || method.HasGenericParameters || method.ExplicitThis || method.IsVirtual
            || method.CallingConvention != MethodCallingConvention.Default || method.ReturnType.MetadataType != MetadataType.Void
            || !method.HasBody || method.Body.HasVariables || method.Body.HasExceptionHandlers) return false;
        var body = method.Body.Instructions.Where(i => i.OpCode.Code != Code.Nop).ToArray();
        if (RuntimeSignatures.IsCore(method.DeclaringType.Scope))
            return body.Length == 2 && body[0].OpCode.Code == Code.Ldnull && body[1].OpCode.Code == Code.Throw;
        return body.Length == 3 && body[0].OpCode.Code == Code.Ldarg_0
            && body[1].OpCode.Code == Code.Call && body[2].OpCode.Code == Code.Ret
            && body[1].Operand is MethodReference call && call.DeclaringType.FullName == "System.Object"
            && RuntimeSignatures.IsCore(call.DeclaringType.Scope) && call.Name == ".ctor" && call.HasThis
            && !call.HasParameters && !call.HasGenericParameters && !call.ExplicitThis
            && call.CallingConvention == MethodCallingConvention.Default && call.ReturnType.MetadataType == MetadataType.Void;
    }
    public static bool IsStringOperator(MethodDefinition method) =>
        method.DeclaringType.FullName == "System.String" && method.Name is "op_Equality" or "op_Inequality";
    // The Raven reference already exposes value0/value1 named arguments, while
    // the existing runtime introspection surface uses these descriptive names.
    // Preserve both contracts during source migration; API alignment is separate.
    public static string ParameterName(MethodDefinition method, int index) => IsString(method.DeclaringType)
        ? method.Name switch {
            "Concat" or "CompareOrdinal" => index == 0 ? "left" : "right",
            "Equals" => "other",
            "ContainsOrdinal" or "StartsWithOrdinal" or "EndsWithOrdinal" => "value",
            "SliceUtf8" => index == 0 ? "byteStart" : "byteLength",
            _ => method.Parameters[index].Name
        } : GenericUnionLibrary.IsMatched(method.DeclaringType) && GenericUnionLibrary.IsConditionalOutput(method, method.Parameters[index]) ? "destination" : method.Parameters[index].Name;
    public static void Project(ModuleDefinition module)
    {
        var type = module.GetType("System.String");
        if (type.HasFields) throw new InvalidDataException("Expected fieldless bootstrap String.");
        type.Fields.Add(new FieldDefinition("m_value", FieldAttributes.Private, module.TypeSystem.String));
    }
    public static void ValidateString(TypeDefinition type, TypeDefinition contract)
    {
        foreach (var candidate in new[] { type, contract })
            if (candidate.IsValueType || !candidate.IsSealed || candidate.HasGenericParameters
                || candidate.Fields.Count != 1 || candidate.Fields[0].Name != "m_value"
                || candidate.Fields[0].FieldType.MetadataType != MetadataType.String
                || !candidate.Fields[0].IsPrivate || candidate.Fields[0].IsStatic
                || candidate.Methods.Any(m => m.IsConstructor && !IsStringConstructor(m)))
                throw new InvalidDataException("Unsupported intrinsic String storage or constructor: " + candidate.Module.Name + " sealed=" + candidate.IsSealed + " fields=" + string.Join(";", candidate.Fields.Select(f => f.FullName)) + " ctors=" + string.Join(";", candidate.Methods.Where(m => m.IsConstructor).Select(m => m.Attributes + ":" + string.Join(",", m.Body.Instructions))));
        if (type.Methods.Any(IsStringOperator)) throw new InvalidDataException("String operators remain compiler intrinsics.");
    }
}
