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
        InterfaceBindings.Validate(module);
        foreach (var contractName in new[]{"Equatable", "Comparable", "Clonable", "Closable"}) {
            var contract = module.GetType("System." + contractName + "`1");
            var closed = new GenericInstanceType(contract); closed.GenericArguments.Add(module.TypeSystem.Int32);
            var member = contract.Methods.Single();
            Check(contractName + " closed interface signature", InterfaceBindings.Bind(Reference(member, closed), member) is not null);
            var mismatch = Reference(member, closed); mismatch.ReturnType = module.TypeSystem.String;
            Reject(contractName + " mismatched result", () => InterfaceBindings.Bind(mismatch, member));
        }
        var comparableParameter = module.GetType("System.Comparable`1").GenericParameters[0];
        comparableParameter.Attributes = GenericParameterAttributes.Covariant;
        Reject("Comparable invariance", () => InterfaceBindings.Validate(module));
        comparableParameter.Attributes = GenericParameterAttributes.NonVariant;
        NativeArrayBindings.Validate(module);
        var native = module.GetType("System.Array`1");
        var nativeOwner = new GenericInstanceType(native); nativeOwner.GenericArguments.Add(module.TypeSystem.Int32);
        var allocate = native.Methods.Single(m => m.Name == "Allocate");
        Check("Native buffer allocation signature", NativeArrayBindings.Bind(Reference(allocate, nativeOwner), allocate)?.Result == "System.Array<Int32>");
        var dataField = new FieldReference("Data", new PointerType(module.TypeSystem.Double), nativeOwner);
        Reject("Native pointer field mismatch", () => NativeArrayBindings.Field(dataField));
        EnumBindings.Validate(module);
        var flagsField = module.GetType(EnumBindings.Flags).Fields.Single(f => f.Name == "value__");
        flagsField.FieldType = module.TypeSystem.Int64;
        Reject("BindingFlags underlying type mismatch", () => EnumBindings.Validate(module));
        flagsField.FieldType = module.TypeSystem.Int32;
        ReflectionBindings.Validate(module);
        var fieldsMethod = module.GetType("System.Type").Methods.Single(m => m.Name == "GetFields" && m.Parameters.Count == 0);
        Check("Reflection returns managed descriptor vector", ReflectionBindings.Bind(fieldsMethod, fieldsMethod)?.Result == "arrayref<System.Reflection.FieldInfo>");
        var fieldsReference = Reference(fieldsMethod, fieldsMethod.DeclaringType);
        fieldsReference.ReturnType = module.TypeSystem.Int32;
        Reject("Reflection return mismatch", () => ReflectionBindings.Bind(fieldsReference, fieldsMethod));
        var descriptorType = module.GetType("System.Reflection.FieldInfo");
        var descriptorBase = descriptorType.BaseType;
        descriptorType.BaseType = module.TypeSystem.Object;
        Reject("Reflection hierarchy mismatch", () => ReflectionBindings.Validate(module));
        descriptorType.BaseType = descriptorBase;
        var forEach = module.GetType("System.Array").Methods.Single(m => m.Name == "ForEach");
        var forEachCall = new GenericInstanceMethod(forEach);
        forEachCall.GenericArguments.Add(module.TypeSystem.Int32);
        Check("Closed Array.ForEach MethodSpec", ArrayCallbackBindings.Bind(forEachCall, forEach)?.Name == "System.Array::ForEach<Int32>");
        forEachCall.GenericArguments.Add(module.TypeSystem.String);
        Reject("Array.ForEach method arity", () => ArrayCallbackBindings.Bind(forEachCall, forEach));
        forEachCall.GenericArguments.RemoveAt(1);
        forEachCall.GenericArguments[0] = module.TypeSystem.Double;
        Reject("Array.ForEach unsupported element", () => ArrayCallbackBindings.Bind(forEachCall, forEach));
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
        var readError = module.GetType("System.IO.FileReadError");
        var notFound = readError.NestedTypes.Single(t => t.Name == "NotFound");
        var notFoundConstructor = notFound.Methods.Single(m => m.IsConstructor);
        Check("Error case constructor mapping", ErrorBindings.Construct(Reference(notFoundConstructor, notFound), notFoundConstructor)?.Result == "System.IO.FileReadError.NotFound");
        Check("Only empty errors have defaults", ErrorBindings.IsEmpty("System.InvalidDateError") && !ErrorBindings.IsEmpty("System.Int32ParseError"));
        var checkedGetter = readError.Methods.Single(m => m.Name == "GetNotFound");
        var getterReference = Reference(checkedGetter, readError);
        getterReference.ReturnType = module.TypeSystem.Int32;
        Reject("Error getter signature mismatch", () => ErrorBindings.Bind(getterReference, checkedGetter));
        var environment = module.GetType("System.Environment");
        var arguments = environment.Methods.Single(m => m.Name == "GetCommandLineArgs");
        Check("Environment returns a managed string vector", ProcessBindings.Bind(Reference(arguments, environment), arguments)?.Result == "arrayref<String>");
        var variable = environment.Methods.Single(m => m.Name == "GetEnvironmentVariable");
        var variableCall = Reference(variable, environment);
        Check("Environment closes nested Result/Option", ProcessBindings.Bind(variableCall, variable)?.Result == "System.Result<System.Option<String>,System.EnvironmentError>");
        variableCall.Parameters[0].ParameterType = module.TypeSystem.Int32;
        Reject("Environment name signature mismatch", () => ProcessBindings.Bind(variableCall, variable));
        var console = module.GetType("System.Console");
        var readByte = console.Methods.Single(m => m.Name == "ReadByte");
        Check("Console retains Byte payload", ProcessBindings.Bind(Reference(readByte, console), readByte)?.Result == "System.Result<System.Option<Byte>,System.IO.ConsoleReadError>");
        Check("Process vectors exclude multidimensional arrays", ProcessBindings.ArrayType(new ArrayType(module.TypeSystem.String, 2)) is null);
        var arrayListDefinition = module.GetType("System.Collections.ArrayList`1");
        var stringList = new GenericInstanceType(arrayListDefinition);
        stringList.GenericArguments.Add(module.TypeSystem.String);
        var copy = arrayListDefinition.Methods.Single(m => m.Name == "Copy");
        Check("Collection Copy preserves closed owner", CollectionBindings.Bind(Reference(copy, stringList), copy, true)?.Result == "System.Collections.ArrayList<String>");
        var addString = Reference(arrayListDefinition.Methods.Single(m => m.Name == "Add"), stringList);
        Check("Collection Add closes String element", CollectionBindings.Bind(addString, arrayListDefinition.Methods.Single(m => m.Name == "Add"), true)?.Arguments.Last() == "String");
        Check("Collection generic arguments are invariant", !CollectionBindings.Assignable("System.Collections.ArrayList<String>", CollectionBindings.List));
        var boolean = module.GetType("System.Boolean");
        var booleanCompare = boolean.Methods.Single(m => m.Name == "CompareTo");
        var booleanCall = Reference(booleanCompare, boolean);
        Check("Boolean metadata stays exact", BooleanBindings.Bind(booleanCall, booleanCompare)?.Arguments.SequenceEqual(new[] { "Boolean&", "Boolean" }) == true);
        booleanCall.Parameters[0].ParameterType = module.TypeSystem.Int32;
        Reject("Boolean is not Int32 in metadata", () => BooleanBindings.Bind(booleanCall, booleanCompare));
        var func = module.GetType("System.Func`1");
        var voidFunc = new GenericInstanceType(func);
        voidFunc.GenericArguments.Add(new TypeReference("System", "Void", module, module, true));
        var invoke = func.Methods.Single(m => m.Name == "Invoke");
        var voidInvoke = Reference(invoke, voidFunc);
        Check("Generic Void return stays a value signature", RuntimeSignatures.Match(voidInvoke, invoke, GenericUnionBindings.Type).Result == "Void");
        Check("Completion delegate call discards interpreter unit", DelegateBindings.Bind(voidInvoke, invoke, true)?.Result == "noresult");
        Reject("Delegate requires virtual invocation", () => DelegateBindings.Bind(voidInvoke, invoke, false));
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
