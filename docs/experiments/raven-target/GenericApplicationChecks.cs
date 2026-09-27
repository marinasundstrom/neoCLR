using Mono.Cecil;

// Check constructed application metadata independently of Raven's source emitter.
static class GenericApplicationChecks
{
    public static void Verify()
    {
        using var first = ModuleDefinition.CreateModule("First", ModuleKind.Dll);
        using var second = ModuleDefinition.CreateModule("Second", ModuleKind.Dll);
        TypeDefinition Holder(ModuleDefinition module)
        {
            var type = new TypeDefinition("Checks", "Holder`1", TypeAttributes.Public, module.TypeSystem.Object);
            module.Types.Add(type);
            var parameter = new GenericParameter("T", type);
            type.GenericParameters.Add(parameter);
            type.Fields.Add(new FieldDefinition("Value", FieldAttributes.Public, parameter));
            type.Methods.Add(new MethodDefinition("Read", MethodAttributes.Public, parameter));
            return type;
        }
        var holder = Holder(first);
        var other = Holder(second);
        ApplicationTypes.Reset(first, second);
        ApplicationTypes.LibraryMap = type => type is GenericParameter p ? "T" + p.Position
            : type.MetadataType == MetadataType.Int32 ? "Int32"
            : type.MetadataType == MetadataType.String ? "String"
            : throw new InvalidDataException("Unsupported probe type.");
        GenericInstanceType Construct(TypeDefinition type, TypeReference argument)
        {
            var result = new GenericInstanceType(type);
            result.GenericArguments.Add(argument);
            return result;
        }
        var integer = Construct(holder, first.TypeSystem.Int32);
        var text = Construct(holder, first.TypeSystem.String);
        var integerName = ApplicationTypes.Type(integer);
        if (integerName == ApplicationTypes.Type(text) || integerName == ApplicationTypes.Type(Construct(other, second.TypeSystem.Int32))
            || !ApplicationTypes.IsType(integerName!) || !ApplicationTypes.IsReference(integerName!))
            throw new Exception("Constructed application identity changed.");
        var nested = new TypeDefinition("", "State", TypeAttributes.NestedPrivate, first.TypeSystem.Object);
        holder.NestedTypes.Add(nested);
        nested.GenericParameters.Add(new GenericParameter("T", nested));
        if (ApplicationTypes.Type(Construct(nested, first.TypeSystem.Int32)) is null)
            throw new Exception("Nested generic application state was not admitted.");
        var read = holder.Methods[0];
        var reference = new MethodReference("Read", holder.GenericParameters[0], integer) { HasThis = true };
        if (!ApplicationTypes.Matches(reference, read)
            || ApplicationTypes.Close(read.ReturnType, integer).MetadataType != MetadataType.Int32)
            throw new Exception("Constructed application substitution changed.");
        reference.ReturnType = first.TypeSystem.String;
        if (ApplicationTypes.Matches(reference, read)) throw new Exception("Forged constructed result admitted.");
        var wrongArity = new GenericInstanceType(holder);
        Reject(() => ApplicationTypes.Type(wrongArity));
        holder.GenericParameters[0].Attributes = GenericParameterAttributes.ReferenceTypeConstraint;
        Reject(() => ApplicationTypes.Type(holder));
        holder.GenericParameters[0].Attributes = GenericParameterAttributes.NonVariant;
        holder.Fields[0].FieldType = other.GenericParameters[0];
        Reject(() => ApplicationTypes.Type(holder));
        holder.Fields[0].FieldType = holder.GenericParameters[0];
        read.IsStatic = true;
        Reject(() => ApplicationTypes.Type(holder));
        ApplicationTypes.Reset();
        Console.WriteLine("Generic application types: identity, substitution and malformed/unsupported metadata checks passed");
    }

    static void Reject(Action action)
    {
        try { action(); }
        catch (InvalidDataException) { return; }
        throw new Exception("Unsupported generic application type admitted.");
    }
}
