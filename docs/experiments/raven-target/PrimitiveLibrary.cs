using Mono.Cecil;
using Mono.Cecil.Cil;

// Matched CoreLib primitive wrappers: the backing field denotes intrinsic storage,
// not another nested value. No ordinary user-defined type gets this projection.
static class PrimitiveLibrary
{
    static readonly Dictionary<string, MetadataType> Kinds = new() {
        ["System.SByte"] = MetadataType.SByte,
        ["System.Byte"] = MetadataType.Byte,
        ["System.Int16"] = MetadataType.Int16,
        ["System.UInt16"] = MetadataType.UInt16,
        ["System.UInt32"] = MetadataType.UInt32,
        ["System.Int64"] = MetadataType.Int64,
        ["System.UInt64"] = MetadataType.UInt64,
        ["System.Single"] = MetadataType.Single,
        ["System.Double"] = MetadataType.Double,
        ["System.Boolean"] = MetadataType.Boolean,
        ["System.Char"] = MetadataType.Char,
    };
    public static bool IsPrimitive(TypeDefinition? type) => type is not null && Kinds.ContainsKey(type.FullName);
    public static bool IsMatched(TypeDefinition? type) => IsPrimitive(type) && ApplicationTypes.IsLibrary(type!);

    public static void Project(ModuleDefinition module)
    {
        var marker = module.GetType("System.Runtime.CompilerServices.IsReadOnlyAttribute").Methods.Single(m => m.IsConstructor);
        // Empty compiler-facing declarations do not specify the runtime's opaque
        // handle/erased-value representation or introduce a one-byte ABI.
        foreach (var name in new[] { "System.Value", "System.RuntimeTypeHandle" })
        {
            var empty = module.GetType(name);
            if (empty.HasFields || empty.HasMethods) throw new InvalidDataException("Expected memberless intrinsic declaration.");
            empty.PackingSize = -1;
            empty.ClassSize = -1;
        }
        foreach (var (name, kind) in Kinds)
        {
            var type = module.GetType(name);
            var scalar = type.Methods.Single(m => m.Name == "CompareTo").Parameters[0].ParameterType;
            if (scalar.MetadataType != kind) throw new InvalidDataException("Primitive reference storage does not match.");
            type.PackingSize = -1;
            type.ClassSize = -1;
            type.Fields.Add(new FieldDefinition("m_value", FieldAttributes.Private, scalar));
            foreach (var method in type.Methods.Where(m => m.HasThis && !m.IsConstructor))
                method.CustomAttributes.Add(new CustomAttribute(marker));
        }
    }

    public static bool IsDefaultConstructor(MethodDefinition method)
    {
        if (!method.IsConstructor || !method.IsPublic || method.IsStatic || method.HasParameters
            || method.HasGenericParameters || method.ExplicitThis || method.IsVirtual
            || method.CallingConvention != MethodCallingConvention.Default || method.ReturnType.MetadataType != MetadataType.Void
            || !method.HasBody || method.Body.HasVariables || method.Body.HasExceptionHandlers) return false;
        var body = method.Body.Instructions.Where(i => i.OpCode.Code != Code.Nop).ToArray();
        return body.Length == 3 && body[0].OpCode.Code == Code.Ldarg_0
            && body[1].OpCode.Code == Code.Call && body[2].OpCode.Code == Code.Ret
            && body[1].Operand is MethodReference call && call.DeclaringType.FullName == "System.ValueType"
            && RuntimeSignatures.IsCore(call.DeclaringType.Scope) && call.Name == ".ctor" && call.HasThis
            && !call.HasParameters && !call.HasGenericParameters && !call.ExplicitThis
            && call.CallingConvention == MethodCallingConvention.Default && call.ReturnType.MetadataType == MetadataType.Void;
    }

    public static void Validate(TypeDefinition type, TypeDefinition contract)
    {
        foreach (var candidate in new[] { type, contract })
            if (!candidate.IsValueType || !candidate.IsSequentialLayout || candidate.HasGenericParameters
                || candidate.PackingSize != -1 || candidate.ClassSize != -1
                || candidate.Fields.Count != 1 || candidate.Fields[0].Name != "m_value"
                || candidate.Fields[0].FieldType.MetadataType != Kinds[type.FullName]
                || !candidate.Fields[0].IsPrivate || candidate.Fields[0].IsStatic
                || candidate.Methods.Any(m => m.IsConstructor && !IsDefaultConstructor(m)))
                throw new InvalidDataException("Unsupported primitive library storage or constructor: " + candidate.FullName);

    }
}
