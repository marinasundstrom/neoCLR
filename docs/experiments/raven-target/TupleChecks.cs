using Mono.Cecil;

static class TupleChecks
{
    public static void VerifyImage(string path)
    {
        using var image = AssemblyDefinition.ReadAssembly(path);
        var count = 0;
        foreach (var method in image.MainModule.Types.SelectMany(t => t.Methods))
        {
            Check(method.ReturnType);
            foreach (var p in method.Parameters) Check(p.ParameterType);
            if (method.HasBody) foreach (var local in method.Body.Variables) Check(local.VariableType);
        }
        foreach (var member in image.MainModule.GetMemberReferences()) Check(member.DeclaringType);
        if (count == 0) throw new Exception("No tuple metadata found.");
        Console.WriteLine($"Tuple metadata identities passed: {count}");
        void Check(TypeReference type)
        {
            if (type is GenericInstanceType generic)
            {
                if (generic.ElementType.FullName.StartsWith("System.Tuple`", StringComparison.Ordinal))
                {
                    if (!generic.IsValueType || !RuntimeSignatures.IsCore(generic.Scope))
                        throw new Exception("Invalid tuple metadata identity: " + generic.FullName);
                    count++;
                }
                foreach (var argument in generic.GenericArguments) Check(argument);
            }
            else if (type is TypeSpecification specification) Check(specification.ElementType);
            else if (type.Name.StartsWith("Tuple`", StringComparison.Ordinal)
                && (type.Namespace != "System" || !RuntimeSignatures.IsCore(type.Scope)))
                throw new Exception("Unresolved tuple projection: " + type.FullName);
        }
    }

    public static void Verify(string path)
    {
        CoreDeclarations.Write(path, unionProbe: true, collectionProbe: true);
        using var image = AssemblyDefinition.ReadAssembly(path);
        var count = 0;
        foreach (var n in Enumerable.Range(1, 7))
        {
            var type = image.MainModule.GetType("System.Tuple`" + n);
            TupleBindings.Validate(type);
            count++;
            var owner = new GenericInstanceType(type);
            foreach (var _ in type.GenericParameters) owner.GenericArguments.Add(image.MainModule.TypeSystem.Int32);
            var constructor = type.Methods.Single(m => m.IsConstructor);
            var reference = new MethodReference(".ctor", constructor.ReturnType, owner) { HasThis = true };
            foreach (var parameter in constructor.Parameters)
                reference.Parameters.Add(new ParameterDefinition(parameter.ParameterType));
            if (TupleBindings.Construct(reference, constructor) is null) throw new Exception("Tuple constructor missing.");
            count++;
            reference.Parameters[0].ParameterType = image.MainModule.TypeSystem.Boolean;
            Reject(() => TupleBindings.Construct(reference, constructor));
            reference.Parameters[0].ParameterType = constructor.Parameters[0].ParameterType;
            var fieldReference = new FieldReference("Item1", type.GenericParameters[0], owner);
            if (TupleBindings.Field(fieldReference) is null) throw new Exception("Tuple field missing.");
            count++;
            fieldReference.FieldType = image.MainModule.TypeSystem.Boolean;
            Reject(() => TupleBindings.Field(fieldReference));
            var field = type.Fields[0];
            field.IsStatic = true;
            Reject(() => TupleBindings.Validate(type));
            field.IsStatic = false;
            var original = field.FieldType;
            field.FieldType = image.MainModule.TypeSystem.Int32;
            Reject(() => TupleBindings.Validate(type));
            field.FieldType = original;
            var name = field.Name;
            field.Name = "Unexpected";
            Reject(() => TupleBindings.Validate(type));
            field.Name = name;
            var attributes = type.Attributes;
            type.IsExplicitLayout = true;
            Reject(() => TupleBindings.Validate(type));
            type.Attributes = attributes;
            var pack = type.PackingSize;
            type.PackingSize = 1;
            Reject(() => TupleBindings.Validate(type));
            type.PackingSize = pack;
        }
        Console.WriteLine($"Tuple layout contracts passed: {count} checks");
        void Reject(Action action)
        {
            try { action(); }
            catch (InvalidDataException) { count++; return; }
            throw new Exception("Invalid tuple layout was admitted.");
        }
    }
}
