using Mono.Cecil;

static class RecordMetadataChecks
{
    public static void Verify()
    {
        using var module = ModuleDefinition.CreateModule("RecordChecks", ModuleKind.Dll);
        var owner = new TypeDefinition("", "Key", TypeAttributes.Public | TypeAttributes.Class, module.TypeSystem.Object);
        module.Types.Add(owner);
        var field = new FieldDefinition("number", FieldAttributes.Private | FieldAttributes.InitOnly, module.TypeSystem.Int32);
        owner.Fields.Add(field);
        MethodDefinition Method(string name, TypeReference result, MethodAttributes extra = 0)
        {
            var method = new MethodDefinition(name, MethodAttributes.Public | extra, result) { HasThis = true };
            owner.Methods.Add(method);
            return method;
        }
        var ctor = Method(".ctor", module.TypeSystem.Void, MethodAttributes.SpecialName | MethodAttributes.RTSpecialName);
        var ordinary = Method("Change", module.TypeSystem.Void);
        var core = new AssemblyNameReference(CoreDeclarations.Identity, new Version(0, 0, 0, 0));
        var marker = new TypeReference("System.Runtime.CompilerServices", "IsExternalInit", module, core);
        var setter = Method("set_Number", new RequiredModifierType(marker, module.TypeSystem.Void), MethodAttributes.SpecialName);
        setter.Parameters.Add(new ParameterDefinition("value", ParameterAttributes.None, module.TypeSystem.Int32));
        owner.Properties.Add(new PropertyDefinition("Number", PropertyAttributes.None, module.TypeSystem.Int32) { SetMethod = setter });
        _ = setter.IsSetter; // Materialize Cecil's lazy semantics before marking this synthetic accessor.
        setter.IsSetter = true;
        ApplicationTypes.CheckFieldWrite(field, ctor);
        ApplicationTypes.CheckFieldWrite(field, setter);
        Reject(() => ApplicationTypes.CheckFieldWrite(field, ordinary));
        setter.IsStatic = true;
        Reject(() => ApplicationTypes.CheckFieldWrite(field, setter));
        setter.IsStatic = false;
        marker.Scope = new AssemblyNameReference("Foreign.Core", new Version(0, 0));
        Reject(() => ApplicationTypes.CheckFieldWrite(field, setter));
        marker.Scope = core;
        owner.Properties.Clear();
        Reject(() => ApplicationTypes.CheckFieldWrite(field, setter));
        Console.WriteLine("Readonly writes require declaring constructors or recognized init accessors; ordinary, static, foreign and unassociated setters reject.");
    }
    static void Reject(Action action)
    {
        try { action(); }
        catch (InvalidDataException) { return; }
        throw new InvalidOperationException("Expected invalid readonly write rejection.");
    }
}
