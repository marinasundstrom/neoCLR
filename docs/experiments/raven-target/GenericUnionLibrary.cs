using Mono.Cecil;
using Mono.Cecil.Cil;

// Explicit implementation families; consumer metadata retains compiler-only union
// recognition members, while generated runtime exports preserve the existing ABI.
static class GenericUnionLibrary
{
    public static bool IsCarrier(TypeDefinition? type) => type?.FullName is "System.Option`1" or "System.Result`2";
    public static bool IsContainer(TypeDefinition? type) => type?.FullName is "System.Option" or "System.Result";
    public static bool IsCase(TypeDefinition? type) => type?.FullName is "System.Option/None" or "System.Option/Some`1" or "System.Result/Ok`1" or "System.Result/Error`1";
    public static bool IsFamily(TypeDefinition type) => IsCarrier(type) || IsCase(type);
    public static bool IsMatched(TypeDefinition type) => IsFamily(type) && ApplicationTypes.IsLibrary(type);
    public static bool IsByValueReceiver(MethodReference method) => IsMatched(method.DeclaringType.Resolve()) && method.HasThis
        && method.Name is not ("TryGetOutput" or "TryGetResidual");
    public static bool IsProtocol(MethodDefinition method) => IsCarrier(method.DeclaringType)
        ? method.Name is "get_Value" or "TryGetValue"
        : IsCase(method.DeclaringType) && method.Name is "set_Value" or "Deconstruct";
    public static bool IsRuntimeProperty(PropertyDefinition property) => !(IsCarrier(property.DeclaringType) && property.Name is "Value" or "IsOkCase" or "IsErrorCase");
    public static bool IsConditionalOutput(MethodDefinition method, ParameterDefinition parameter) => IsCarrier(method.DeclaringType)
        && method.HasThis && method.Name is "TryGet" or "TryGetOutput" or "TryGetResidual"
        && method.Parameters.Count == 1 && parameter == method.Parameters[0]
        && parameter.IsOut && !parameter.IsIn && parameter.ParameterType.IsByReference
        && method.ReturnType.MetadataType == MetadataType.Boolean;
    public static void Project(ModuleDefinition module)
    {
        foreach (var type in module.Types.Where(IsCarrier).Concat(module.Types.Where(IsContainer).SelectMany(t => t.NestedTypes)))
        {
            if (!IsFamily(type)) throw new InvalidDataException("Unexpected union bootstrap type.");
            type.PackingSize = -1;
            type.ClassSize = -1;
            if (IsCase(type) && type.HasGenericParameters)
            {
                if (type.Fields.Count != 1 || type.Fields[0].Name != "<Value>k__BackingField"
                    || !type.Fields[0].IsPrivate || type.Fields[0].FieldType != type.GenericParameters[0])
                    throw new InvalidDataException("Unexpected union case bootstrap storage.");
                type.Fields[0].Name = "Stored";
            }
            else
            {
                if (type.HasFields) throw new InvalidDataException("Unexpected union bootstrap storage.");
                if (IsCarrier(type)) type.Fields.Add(new FieldDefinition("Stored", FieldAttributes.Private, module.GetType("System.Value")));
            }
        }
    }
    public static MethodDefinition[] Roots(ModuleDefinition source, ModuleDefinition core, string owner,
        Func<TypeDefinition, TypeDefinition, string, MethodDefinition[]> instanceRoots)
    {
        var container = source.GetType(owner) ?? throw new InvalidDataException("Missing union case container.");
        var reference = core.GetType(owner);
        if (!container.IsPublic || !container.IsAbstract || !container.IsSealed || container.HasFields
            || container.HasMethods || container.HasGenericParameters || container.HasInterfaces || container.HasProperties
            || container.NestedTypes.Count != reference.NestedTypes.Count
            || container.NestedTypes.Any(t => !IsCase(t)))
            throw new InvalidDataException("Unsupported union case container.");
        ApplicationTypes.BindLibrary(container, owner);
        _ = ApplicationTypes.Type(container);
        var methods = new List<MethodDefinition>();
        foreach (var type in container.NestedTypes)
            methods.AddRange(instanceRoots(type, reference.NestedTypes.Single(t => t.Name == type.Name),
                type.FullName.Split('`')[0].Replace('/', '.')));
        var name = owner + (owner == "System.Option" ? "`1" : "`2");
        methods.AddRange(instanceRoots(source.GetType(name) ?? throw new InvalidDataException("Missing union carrier with expected arity."), core.GetType(name), owner));
        return methods.ToArray();
    }
    public static void Validate(TypeDefinition source, TypeDefinition core)
    {
        if (!IsFamily(source) || !IsFamily(core) || source.FullName != core.FullName
            || source.GenericParameters.Count != core.GenericParameters.Count
            || source.Fields.Count != core.Fields.Count
            || source.Fields.Zip(core.Fields).Any(p => p.First.Name != "Stored" || p.Second.Name != "Stored"
                || !LibraryImplementation.SameType(p.First.FieldType, p.Second.FieldType)))
            throw new InvalidDataException("Unsupported union storage layout.");
        if (source.FullName == "System.Option/None" && (source.Methods.Count != 1 || !PrimitiveLibrary.IsDefaultConstructor(source.Methods[0])))
            throw new InvalidDataException("Option.None requires an empty constructor.");
    }
    public static string? CaseConstructor(MethodDefinition method, Func<TypeReference, bool, string> map)
    {
        if (!IsMatched(method.DeclaringType) || !IsCase(method.DeclaringType) || !method.IsConstructor || !method.HasParameters) return null;
        var body = method.Body.Instructions.Where(i => i.OpCode.Code != Code.Nop).ToArray();
        if (method.Parameters.Count != 1 || method.Body.Variables.Count != 1 || !LibraryImplementation.SameType(method.Body.Variables[0].VariableType, method.Parameters[0].ParameterType) || body.Length != 8
            || body[0].OpCode.Code != Code.Ldarg_0 || body[1].OpCode.Code != Code.Call
            || body[1].Operand is not MethodReference parent || parent.Name != ".ctor" || !parent.HasThis || parent.HasParameters
            || parent.HasGenericParameters || parent.ExplicitThis || parent.CallingConvention != MethodCallingConvention.Default
            || parent.ReturnType.MetadataType != MetadataType.Void || parent.DeclaringType.FullName != "System.ValueType" || !RuntimeSignatures.IsCore(parent.DeclaringType.Scope)
            || body[2].OpCode.Code != Code.Ldarg_1 || body[3].OpCode.Code != Code.Stloc_0
            || body[4].OpCode.Code != Code.Ldarg_0 || body[5].OpCode.Code != Code.Ldloc_0
            || body[6].OpCode.Code != Code.Stfld || body[6].Operand is not FieldReference field
            || field.Resolve() != method.DeclaringType.Fields[0] || body[7].OpCode.Code != Code.Ret)
            throw new InvalidDataException("Unsupported union case constructor: " + string.Join(';', body.Select(i => i.ToString())));
        return $".method instance .ctor({map(method.Parameters[0].ParameterType, false)} value) -> Void\nldarg value\nnewobj {map(method.DeclaringType, false)}\nstarg this\nldvoid\nret\n.end\n";
    }
}
