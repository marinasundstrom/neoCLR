using Mono.Cecil;
using Mono.Cecil.Cil;

// Contract checks below the source compiler, including malformed/unsupported metadata.
static class ApplicationSpecializationChecks
{
    public static void Verify()
    {
        using var module = ModuleDefinition.CreateModule("GenericHelpers", ModuleKind.Dll);
        var owner = new TypeDefinition("Checks", "Helpers", TypeAttributes.Public, module.TypeSystem.Object);
        module.Types.Add(owner);
        var source = new MethodDefinition("Identity", MethodAttributes.Private | MethodAttributes.Static, module.TypeSystem.Void);
        owner.Methods.Add(source);
        var parameter = new GenericParameter("T", source);
        source.GenericParameters.Add(parameter);
        source.ReturnType = parameter;
        source.Parameters.Add(new ParameterDefinition("value", ParameterAttributes.None, parameter));
        source.Body.Instructions.Add(Instruction.Create(OpCodes.Ldarg_0));
        source.Body.Instructions.Add(Instruction.Create(OpCodes.Ret));
        var specialization = new ApplicationSpecialization(module, [module]);
        MethodDefinition Close(TypeReference argument, ApplicationSpecialization? selected = null)
        {
            var call = new GenericInstanceMethod(source);
            call.GenericArguments.Add(argument);
            var caller = new MethodDefinition("Caller", MethodAttributes.Static, module.TypeSystem.Void);
            caller.Body.Instructions.Add(Instruction.Create(OpCodes.Call, call));
            (selected ?? specialization).Rewrite(caller);
            return (MethodDefinition)caller.Body.Instructions[0].Operand;
        }
        var integer = Close(module.TypeSystem.Int32);
        var text = Close(module.TypeSystem.String);
        if (integer == text || Close(module.TypeSystem.Int32) != integer
            || integer.ReturnType.MetadataType != MetadataType.Int32
            || text.ReturnType.MetadataType != MetadataType.String
            || !integer.IsPrivate || integer.HasGenericParameters
            || specialization.Origin(integer) != source)
            throw new Exception("Closed signatures, cache identity, visibility or source origin changed.");

        using var first = ModuleDefinition.CreateModule("First", ModuleKind.Dll);
        using var second = ModuleDefinition.CreateModule("Second", ModuleKind.Dll);
        var firstType = new TypeDefinition("Checks", "Value", TypeAttributes.Public, first.TypeSystem.Object);
        var secondType = new TypeDefinition("Checks", "Value", TypeAttributes.Public, second.TypeSystem.Object);
        first.Types.Add(firstType);
        second.Types.Add(secondType);
        if (Close(firstType) == Close(secondType))
            throw new Exception("Type arguments from distinct assemblies must not share a specialization.");

        Reject(() => Close(module.TypeSystem.Void), "Unsupported application specialization argument");
        Reject(() => Close(parameter), "Unsupported metadata identity signature");
        Reject(() => Close(new ByReferenceType(module.TypeSystem.Int32)), "Unsupported application specialization argument");
        parameter.Attributes = GenericParameterAttributes.ReferenceTypeConstraint;
        Reject(() => Close(module.TypeSystem.String, new ApplicationSpecialization(module, [module])), "Numeric specialization requires");
        parameter.Attributes = GenericParameterAttributes.NonVariant;
        source.IsStatic = false;
        Reject(() => Close(module.TypeSystem.Int32, new ApplicationSpecialization(module, [module])), "Unsupported application specialization");
        Console.WriteLine("Generic specialization: signatures, caching, identity, visibility and rejection checks passed");
    }

    static void Reject(Action action, string diagnostic)
    {
        try { action(); }
        catch (InvalidDataException error) when (error.Message.StartsWith(diagnostic, StringComparison.Ordinal)) { return; }
        throw new Exception("Expected rejection: " + diagnostic);
    }
}
