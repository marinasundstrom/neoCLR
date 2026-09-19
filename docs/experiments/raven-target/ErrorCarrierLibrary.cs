using Mono.Cecil;
using Mono.Cecil.Cil;

// Explicitly selected, existing error carriers. Storage is one erased value;
// nested empty cases and all public exports must match the bootstrap contract.
static class ErrorCarrierLibrary
{
    public static bool IsCarrier(TypeDefinition type) => ErrorBindings.Cases.TryGetValue(type.FullName, out var cases) && cases.Length > 0;
    public static bool IsMatched(TypeDefinition type) => IsCarrier(type) && ApplicationTypes.IsLibrary(type);
    public static bool IsByValueReceiver(MethodReference method) => IsMatched(method.DeclaringType.Resolve()) && method.HasThis;
    public static bool IsCase(TypeDefinition type) => type.DeclaringType is { } owner && IsCarrier(owner)
        && ErrorBindings.Cases[owner.FullName].Contains(type.Name);
    public static bool SameCase(TypeReference left, TypeReference right, TypeDefinition source, TypeDefinition core) =>
        right.Resolve() is { } r && left.Resolve() is { } l && r.DeclaringType == source && l.DeclaringType == core
        && IsCase(r) && IsCase(l) && r.Name == l.Name && r.IsValueType && l.IsValueType;
    public static void Project(ModuleDefinition module)
    {
        foreach (var type in module.Types.Where(IsCarrier))
        {
            if (type.HasFields) throw new InvalidDataException("Expected bootstrap carrier without storage.");
            type.Fields.Add(new FieldDefinition("Stored", FieldAttributes.Private, module.GetType("System.Value")));
            type.PackingSize = -1;
            type.ClassSize = -1;
            foreach (var nested in type.NestedTypes)
            {
                nested.PackingSize = -1;
                nested.ClassSize = -1;
            }
        }
    }
    public static MethodDefinition[] Validate(TypeDefinition source, TypeDefinition core)
    {
        foreach (var type in new[] { source, core })
        {
            if (!type.IsValueType || type.HasGenericParameters || type.Fields.Count != 1
                || type.Fields[0].Name != "Stored" || type.Fields[0].FieldType.FullName != "System.Value"
                || !RuntimeSignatures.IsCore(type.Fields[0].FieldType.Scope)
                || type.NestedTypes.Count != ErrorBindings.Cases[type.FullName].Length
                || type.NestedTypes.Any(n => !IsCase(n) || !n.IsNestedPublic || !n.IsValueType
                    || n.HasFields || n.HasGenericParameters || n.HasInterfaces || n.HasNestedTypes
                    || !n.IsSequentialLayout || n.PackingSize != -1 || n.ClassSize != -1
                    || n.HasProperties || n.HasEvents || n.Methods.Count != 1
                    || !n.Methods[0].IsConstructor || !n.Methods[0].IsPublic || n.Methods[0].HasParameters))
                throw new InvalidDataException("Unsupported error carrier storage or case layout.");
        }
        foreach (var nested in source.NestedTypes)
        {
            if (!PrimitiveLibrary.IsDefaultConstructor(nested.Methods[0]))
                throw new InvalidDataException("Error cases require empty constructors.");
            ApplicationTypes.BindLibrary(nested, nested.FullName.Replace('/', '.'));
        }
        return source.NestedTypes.SelectMany(t => t.Methods).ToArray();
    }
    public static string? Constructor(MethodDefinition method, Func<TypeReference, bool, string> map)
    {
        if (!IsMatched(method.DeclaringType) || !method.IsConstructor) return null;
        var body = method.Body.Instructions.Where(i => i.OpCode.Code != Code.Nop).ToArray();
        // Read no uninitialized receiver: lower the checked single field assignment
        // to the existing value-constructor ABI, rather than a managed CLR address.
        if (method.Parameters.Count != 1 || body.Length != 9 || method.Body.Variables.Count != 1
            || method.Body.Variables[0].VariableType.FullName != "System.Value"
            || !RuntimeSignatures.IsCore(method.Body.Variables[0].VariableType.Scope)
            || body[0].OpCode.Code != Code.Ldarg_0 || body[1].OpCode.Code != Code.Call
            || body[1].Operand is not MethodReference parent || parent.Name != ".ctor" || !parent.HasThis
            || parent.HasParameters || parent.HasGenericParameters || parent.ExplicitThis
            || parent.ReturnType.MetadataType != MetadataType.Void
            || parent.DeclaringType.FullName != "System.ValueType" || !RuntimeSignatures.IsCore(parent.DeclaringType.Scope)
            || body[2].OpCode.Code != Code.Ldarg_1 || body[3].OpCode.Code != Code.Call
            || body[3].Operand is not GenericInstanceMethod pack
            || ValueStorageBindings.Bind(pack, pack.Resolve(), t => map(t, false)) is not { Instruction: var operation }
            || operation != "value.pack " + map(method.Parameters[0].ParameterType, false)
            || body[4].OpCode.Code != Code.Stloc_0 || body[5].OpCode.Code != Code.Ldarg_0
            || body[6].OpCode.Code != Code.Ldloc_0
            || body[7].OpCode.Code != Code.Stfld || body[7].Operand is not FieldReference field
            || field.Resolve() != method.DeclaringType.Fields[0] || body[8].OpCode.Code != Code.Ret)
            throw new InvalidDataException("Unsupported error carrier constructor body: " + string.Join(";", body.Select(i => i.ToString())));
        var owner = map(method.DeclaringType, false);
        var arg = map(method.Parameters[0].ParameterType, false);
        return $".method instance .ctor({arg} value) -> Void\nldarg value\nvalue.pack {arg}\nnewobj {owner}\nstarg this\nldvoid\nret\n.end\n";
    }
}
