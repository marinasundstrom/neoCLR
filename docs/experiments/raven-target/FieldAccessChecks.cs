using Mono.Cecil;

// Exercise importer access control with metadata, including calls a source compiler rejects.
static class FieldAccessChecks
{
    public static void Verify()
    {
        using var module = ModuleDefinition.CreateModule("FieldAccessChecks", ModuleKind.Dll);
        var owner = new TypeDefinition("Checks", "Owner", TypeAttributes.Public | TypeAttributes.Class, module.TypeSystem.Object);
        var other = new TypeDefinition("Checks", "Other", TypeAttributes.Public | TypeAttributes.Class, module.TypeSystem.Object);
        module.Types.Add(owner);
        module.Types.Add(other);
        var nested = new TypeDefinition("", "Nested", TypeAttributes.NestedPrivate | TypeAttributes.Class, module.TypeSystem.Object);
        owner.NestedTypes.Add(nested);
        var deep = new TypeDefinition("", "Deep", TypeAttributes.NestedPrivate | TypeAttributes.Class, module.TypeSystem.Object);
        nested.NestedTypes.Add(deep);
        var field = new FieldDefinition("value", FieldAttributes.Private, module.TypeSystem.Int32);
        owner.Fields.Add(field);
        MethodDefinition Method(TypeDefinition type, string name = "Read")
        {
            var method = new MethodDefinition(name, MethodAttributes.Public, module.TypeSystem.Void);
            if (name == ".ctor") method.Attributes |= MethodAttributes.SpecialName | MethodAttributes.RTSpecialName;
            type.Methods.Add(method);
            return method;
        }
        var ownRead = Method(owner);
        var nestedRead = Method(nested);
        var deepRead = Method(deep);
        var unrelatedRead = Method(other);
        var ownerInit = Method(owner, ".ctor");
        var nestedInit = Method(nested, ".ctor");
        ApplicationTypes.Reset(module);
        foreach (var caller in new[] { ownRead, nestedRead, deepRead })
            if (ApplicationTypes.Field(field, caller, (_, _) => "Int32") is null)
                throw new Exception("Legal containing-type access was not admitted.");
        Reject(() => ApplicationTypes.Field(field, unrelatedRead, (_, _) => "Int32"), "Unsupported application field access.");
        field.IsInitOnly = true;
        ApplicationTypes.CheckFieldWrite(field, ownerInit);
        Reject(() => ApplicationTypes.CheckFieldWrite(field, nestedInit), "Readonly field writes require the declaring constructor or init accessor.");
        Console.WriteLine("Field access: own/nested/deep accepted; unrelated/private and nested/readonly writes rejected");
    }

    static void Reject(Action action, string message)
    {
        try { action(); }
        catch (InvalidDataException error) when (error.Message == message) { return; }
        throw new Exception("Expected rejection: " + message);
    }
}
