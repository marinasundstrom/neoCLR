using Mono.Cecil;
using System.Text.Json;

static class SignatureProbe
{
    public static void Write(string output)
    {
        output = Path.GetFullPath(output);
        if (Directory.Exists(output)) throw new IOException("Output directory must not exist.");
        Directory.CreateDirectory(output);
        var corePath = Path.Combine(output, CoreDeclarations.Identity + ".dll");
        CoreDeclarations.Write(corePath, unionProbe: true, collectionProbe: true);
        using var core = AssemblyDefinition.ReadAssembly(corePath);
        var module = core.MainModule;
        var result = module.GetType("System.Result`2");
        var owner = new GenericInstanceType(result);
        owner.GenericArguments.Add(module.TypeSystem.String);
        owner.GenericArguments.Add(module.GetType(FileBindings.ReadError));
        var checks = new List<string>();
        void Check(string name, bool success)
        {
            if (!success) throw new Exception("Signature check failed: " + name);
            checks.Add(name);
        }
        void Reject(string name, Action action)
        {
            try { action(); }
            catch (InvalidDataException) { checks.Add(name); return; }
            throw new Exception("Malformed signature accepted: " + name);
        }
        var ok = module.GetType("System.Result").NestedTypes.Single(t => t.Name == "Ok`1");
        var nested = new GenericInstanceType(ok);
        nested.GenericArguments.Add(new ArrayType(result.GenericParameters[0]));
        var shape = new ByReferenceType(nested);
        var before = shape.FullName;
        Check("Recursive type argument under vector and byref", RuntimeSignatures.Close(shape, owner).FullName
            == "System.Result/Ok`1<System.String[]>&");
        Check("Input signature remains unchanged", shape.FullName == before);
        var extraction = result.Methods.Single(m => m.Name == "TryGetResidual");
        var reference = Reference(extraction, owner);
        var matched = RuntimeSignatures.Match(reference, extraction, ResultBindings.Type);
        Check("Closed residual parameter", matched.Args.SequenceEqual(new[] { FileBindings.ReadError + "&" }) && matched.Result == "Boolean");
        reference.ReturnType = module.TypeSystem.Int32;
        Reject("Return mismatch", () => RuntimeSignatures.Match(reference, extraction, ResultBindings.Type));
        reference = Reference(extraction, owner);
        reference.Parameters[0].ParameterType = new ByReferenceType(module.TypeSystem.String);
        Reject("Parameter mismatch", () => RuntimeSignatures.Match(reference, extraction, ResultBindings.Type));
        reference = Reference(extraction, owner);
        reference.HasThis = false;
        Reject("Receiver mismatch", () => RuntimeSignatures.Match(reference, extraction, ResultBindings.Type));
        var shortOwner = new GenericInstanceType(result);
        shortOwner.GenericArguments.Add(module.TypeSystem.String);
        Reject("Owner arity mismatch", () => RuntimeSignatures.Match(Reference(extraction, shortOwner), extraction, ResultBindings.Type));
        Reject("Out of range generic index", () => RuntimeSignatures.Close(result.GenericParameters[1], shortOwner));
        var openMethod = new MethodReference("Open", module.TypeSystem.Void, result);
        var methodParameter = new GenericParameter("M", openMethod);
        openMethod.GenericParameters.Add(methodParameter);
        Reject("Open method parameter", () => RuntimeSignatures.Close(methodParameter, owner));
        Reject("Pointer requires separate admission", () => RuntimeSignatures.Close(new PointerType(module.TypeSystem.Int32), owner));
        Reject("Multidimensional array requires separate admission", () => RuntimeSignatures.Close(new ArrayType(module.TypeSystem.Int32, 2), owner));
        Reject("Modifier requires explicit semantics", () => RuntimeSignatures.Close(new RequiredModifierType(module.TypeSystem.Int32, module.TypeSystem.String), owner));
        Reject("Nested managed reference", () => RuntimeSignatures.Close(new ByReferenceType(new ByReferenceType(module.TypeSystem.Int32)), owner));
        TypeReference deep = module.TypeSystem.Int32;
        for (var n = 0; n < 34; n++) deep = new ArrayType(deep);
        Reject("Nesting limit", () => RuntimeSignatures.Close(deep, owner));
        Check("CLI VOID return has no result", RuntimeSignatures.Map(module.TypeSystem.Void, _ => null, returns: true) == "noresult");
        var namedVoid = new TypeReference("System", "Void", module, module, true);
        Check("Named Void storage is a type", RuntimeSignatures.Map(namedVoid, _ => null) == "Void");
        Check("Named Void value return is a type", RuntimeSignatures.Map(namedVoid, _ => null, returns: true) == "Void");
        var voidOwner = new GenericInstanceType(result);
        voidOwner.GenericArguments.Add(namedVoid);
        voidOwner.GenericArguments.Add(module.GetType(FileBindings.WriteError));
        Check("Generic Void result remains a carrier", RuntimeSignatures.Map(voidOwner, ResultBindings.Type, returns: true)
            == "System.Result<Void,System.IO.FileWriteError>");
        var stringType = module.GetType("System.String");
        var concat = stringType.Methods.Single(m => m.Name == "Concat");
        Reject("Static String callvirt", () => StringBindings.Bind(Reference(concat, stringType), concat, true));
        var wrongStringArgument = Reference(concat, stringType);
        wrongStringArgument.Parameters[0].ParameterType = module.TypeSystem.Int32;
        Reject("String argument mismatch", () => StringBindings.Bind(wrongStringArgument, concat, false));
        var pathType = module.GetType("System.IO.Path");
        var combine = pathType.Methods.Single(m => m.Name == "Combine");
        var pathCall = Reference(combine, pathType);
        Check("Path static mapping", PathBindings.Bind(pathCall, combine)?.Name == "System.IO.Path::Combine");
        pathCall.Parameters[0].ParameterType = module.TypeSystem.Int32;
        Reject("Path argument mismatch", () => PathBindings.Bind(pathCall, combine));
        pathCall = Reference(combine, pathType);
        pathCall.HasThis = true;
        Reject("Path receiver mismatch", () => PathBindings.Bind(pathCall, combine));
        var integerType = module.GetType("System.Int32");
        var divide = integerType.Methods.Single(m => m.Name == "Divide");
        var divisionCall = Reference(divide, integerType);
        Check("Division Result mapping", Int32Bindings.Bind(divisionCall, divide)?.Result == "System.Result<Int32,System.IntegerDivisionError>");
        divisionCall.ReturnType = module.TypeSystem.Int32;
        Reject("Division return mismatch", () => Int32Bindings.Bind(divisionCall, divide));
        divisionCall = Reference(divide, integerType);
        divisionCall.Parameters[1].ParameterType = module.TypeSystem.String;
        Reject("Division argument mismatch", () => Int32Bindings.Bind(divisionCall, divide));
        var compare = integerType.Methods.Single(m => m.Name == "CompareTo");
        var compareCall = Reference(compare, integerType);
        Check("Int32 managed receiver mapping", Int32Bindings.Bind(compareCall, compare)?.Arguments.SequenceEqual(new[] { "Int32&", "Int32" }) == true);
        compareCall.HasThis = false;
        Reject("Int32 instance receiver mismatch", () => Int32Bindings.Bind(compareCall, compare));
        var math = module.GetType("System.Math");
        var sqrt = math.Methods.Single(m => m.Name == "Sqrt");
        var sqrtCall = Reference(sqrt, math);
        Check("Double Math mapping", DoubleBindings.Bind(sqrtCall, sqrt)?.Result == "Double");
        sqrtCall.Parameters[0].ParameterType = module.TypeSystem.Int32;
        Reject("Double Math argument mismatch", () => DoubleBindings.Bind(sqrtCall, sqrt));
        var doubleType = module.GetType("System.Double");
        var doubleCompare = doubleType.Methods.Single(m => m.Name == "CompareTo");
        Check("Double receiver mapping", DoubleBindings.Bind(Reference(doubleCompare, doubleType), doubleCompare)?.Arguments.SequenceEqual(new[] { "Double&", "Double" }) == true);
        Check("Narrow storage has Int32 stack category", PrimitiveBindings.Stack("Byte") == "Int32" && PrimitiveBindings.Stack("Char") == "Int32");
        Check("Unsigned and Single stack normalization", PrimitiveBindings.Stack("UInt64") == "Int64" && PrimitiveBindings.Stack("Single") == "Double");
        var charType = module.GetType("System.Char");
        var digit = charType.Methods.Single(m => m.Name == "IsDigit");
        var digitCall = Reference(digit, charType);
        Check("Char signature retains storage type", PrimitiveBindings.Bind(digitCall, digit)?.Arguments.SequenceEqual(new[] { "Char" }) == true);
        digitCall.Parameters[0].ParameterType = module.TypeSystem.Int32;
        Reject("Char signature is not Int32 metadata", () => PrimitiveBindings.Bind(digitCall, digit));
        var dateType = module.GetType("System.Date");
        var createDate = dateType.Methods.Single(m => m.Name == "Create");
        Check("Date factory Result mapping", CalendarBindings.Bind(Reference(createDate, dateType), createDate)?.Result == CalendarBindings.DateResult);
        var timeType = module.GetType("System.Time");
        var fromTicks = timeType.Methods.Single(m => m.Name == "FromTicks");
        var ticksCall = Reference(fromTicks, timeType);
        Check("Time ticks stay Int64", CalendarBindings.Bind(ticksCall, fromTicks)?.Arguments.SequenceEqual(new[] { "Int64" }) == true);
        ticksCall.Parameters[0].ParameterType = module.TypeSystem.Int32;
        Reject("Time ticks reject Int32 signature", () => CalendarBindings.Bind(ticksCall, fromTicks));
        var text = JsonSerializer.Serialize(checks, new JsonSerializerOptions { WriteIndented = true });
        File.WriteAllText(Path.Combine(output, "signature-checks.json"), text);
        Console.WriteLine(text);
    }

    static MethodReference Reference(MethodDefinition definition, TypeReference owner)
    {
        var reference = new MethodReference(definition.Name, definition.ReturnType, owner) { HasThis = definition.HasThis };
        foreach (var parameter in definition.Parameters) reference.Parameters.Add(new ParameterDefinition(parameter.ParameterType));
        return reference;
    }
}
