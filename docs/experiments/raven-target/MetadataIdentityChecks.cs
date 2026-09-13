using Mono.Cecil;

static class MetadataIdentityChecks
{
    public static void Run()
    {
        var checks = 0;
        void Check(bool condition, string message)
        {
            if (!condition) throw new InvalidDataException(message);
            checks++;
        }
        using var assembly = AssemblyDefinition.CreateAssembly(new("IdentityProbe", new Version(1, 0, 0, 0)), "Probe", ModuleKind.Dll);
        var module = assembly.MainModule;
        var type = new TypeDefinition("Example", "Item", TypeAttributes.Public | TypeAttributes.Class, module.TypeSystem.Object);
        module.Types.Add(type);
        var method = new MethodDefinition("Read", MethodAttributes.Public | MethodAttributes.Static, module.TypeSystem.Int32);
        type.Methods.Add(method);
        method.Body.Instructions.Add(Mono.Cecil.Cil.Instruction.Create(Mono.Cecil.Cil.OpCodes.Ldc_I4_0));
        method.Body.Instructions.Add(Mono.Cecil.Cil.Instruction.Create(Mono.Cecil.Cil.OpCodes.Ret));
        type.Fields.Add(new("value", FieldAttributes.Public, module.TypeSystem.Int32));
        var before = Snapshot(assembly);
        module.Types.Insert(1, new TypeDefinition("Unrelated", "Earlier", TypeAttributes.Public, module.TypeSystem.Object));
        type.Methods.Insert(0, new MethodDefinition("Earlier", MethodAttributes.Public | MethodAttributes.Abstract | MethodAttributes.Virtual, module.TypeSystem.Void));
        type.Fields.Insert(0, new FieldDefinition("earlier", FieldAttributes.Public, module.TypeSystem.Int32));
        var after = Snapshot(assembly);
        Check(before.TypeToken != after.TypeToken && before.MethodToken != after.MethodToken && before.FieldToken != after.FieldToken, "Fixture must shift all metadata tokens.");
        Check(before.Type == after.Type && before.Method == after.Method && before.Field == after.Field, "Declaration insertion changed symbol identity.");
        var reference = new TypeReference(type.Namespace, type.Name, module, AssemblyNameReference.Parse(assembly.Name.FullName));
        Check(MetadataIdentity.TypeName(reference) == MetadataIdentity.TypeName(type), "TypeRef and TypeDef identities differ.");
        var original = MetadataIdentity.TypeName(type);
        assembly.Name.Version = new Version(2, 0, 0, 0);
        Check(original != MetadataIdentity.TypeName(type), "Assembly versions collided.");
        var other = new TypeReference(type.Namespace, type.Name, module, new AssemblyNameReference("Other", new Version(2, 0, 0, 0)));
        Check(MetadataIdentity.TypeName(other) != MetadataIdentity.TypeName(type), "Assembly names collided.");
        var initialMethod = MetadataIdentity.FunctionName(method);
        method.Parameters.Add(new(module.TypeSystem.Int32));
        Check(initialMethod != MetadataIdentity.FunctionName(method), "Overloads collided.");
        var valueMethod = MetadataIdentity.FunctionName(method);
        method.Parameters[0].ParameterType = new ByReferenceType(module.TypeSystem.Int32);
        Check(valueMethod != MetadataIdentity.FunctionName(method), "Value and byref signatures collided.");
        method.Parameters[0].ParameterType = module.TypeSystem.Int32;
        method.ReturnType = module.TypeSystem.String;
        Check(valueMethod != MetadataIdentity.FunctionName(method), "Return signatures collided.");
        var nested = new TypeDefinition("", "Child", TypeAttributes.NestedPublic, module.TypeSystem.Object);
        type.NestedTypes.Add(nested);
        var literal = new TypeReference(type.Namespace, type.Name + "/Child", module, module);
        Check(MetadataIdentity.TypeName(nested) != MetadataIdentity.TypeName(literal), "Nested and literal names collided.");
        var generic = new GenericInstanceType(new TypeReference("Example", "Option`1", module, module));
        generic.GenericArguments.Add(module.TypeSystem.Int32);
        var closed = MetadataIdentity.TypeKey(generic);
        generic.GenericArguments[0] = module.TypeSystem.String;
        Check(closed != MetadataIdentity.TypeKey(generic), "Closed generic arguments collided.");
        var escaped = MetadataIdentity.MemberName("<generated>");
        Check(escaped != MetadataIdentity.MemberName(escaped), "Escaped and literal member names collided.");
        Check(MetadataIdentity.MemberName("Read") == "Read", "Ordinary virtual slot name changed.");
        Console.WriteLine($"{checks} metadata identity checks passed.");
    }
    static (string Type, string Method, string Field, uint TypeToken, uint MethodToken, uint FieldToken) Snapshot(AssemblyDefinition assembly)
    {
        using var bytes = new MemoryStream();
        assembly.Write(bytes); bytes.Position = 0;
        using var copy = AssemblyDefinition.ReadAssembly(bytes);
        var type = copy.MainModule.Types.Single(t => t.FullName == "Example.Item");
        var method = type.Methods.Single(m => m.Name == "Read");
        var field = type.Fields.Single(f => f.Name == "value");
        return (MetadataIdentity.TypeName(type), MetadataIdentity.FunctionName(method), MetadataIdentity.MemberName(field.Name),
            type.MetadataToken.ToUInt32(), method.MetadataToken.ToUInt32(), field.MetadataToken.ToUInt32());
    }
}
