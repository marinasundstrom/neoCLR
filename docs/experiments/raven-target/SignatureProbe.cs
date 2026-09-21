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
        foreach (var name in new[] { "op_Equality", "op_Inequality" })
        {
            var method = module.GetType("System.String").Methods.Single(m => m.Name == name);
            var binding = StringBindings.Bind(method, method, false);
            Check("String operator " + name, method.IsPublic && method.IsStatic && method.IsSpecialName
                && binding is { Result: "Boolean" } && binding.Arguments.SequenceEqual(new[] { "String", "String" }));
        }
        Check("Opaque Value metadata identity", GenericUnionBindings.Type(module.GetType("System.Value")) == "Value");
        Check("Unsigned byte array load", ManagedArrayBindings.Type(new ArrayType(module.TypeSystem.Byte)) == "arrayref<Byte>");
        Reject("Array signedness mismatch", () => ManagedArrayBindings.CheckElement(Mono.Cecil.Cil.Code.Ldelem_I1, "Byte", null));
        Reject("Array token mismatch", () => ManagedArrayBindings.CheckElement(Mono.Cecil.Cil.Code.Stelem_Any, "Int32", "Double"));
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
        NativeMemoryBindings.Validate(module);
        var native = module.GetType(NativeMemoryBindings.Owner);
        var allocate = native.Methods.Single(m => m.Name == "Alloc" && m.Parameters.Count == 1);
        Check("Native allocation signature", NativeMemoryBindings.Bind(allocate, allocate)?.Result == "Void*");
        var invalidAllocation = new MethodReference("Alloc", module.TypeSystem.Int32, native);
        invalidAllocation.Parameters.Add(new ParameterDefinition(module.TypeSystem.UIntPtr));
        Reject("Native allocation return mismatch", () => NativeMemoryBindings.Bind(invalidAllocation, allocate));
        EnumBindings.Validate(module);
        var flagsField = module.GetType(EnumBindings.Flags).Fields.Single(f => f.Name == "value__");
        flagsField.FieldType = module.TypeSystem.Int64;
        Reject("BindingFlags underlying type mismatch", () => EnumBindings.Validate(module));
        flagsField.FieldType = module.TypeSystem.Int32;
        ReflectionBindings.Validate(module);
        Check("Old reflection descriptor namespace is absent", module.GetType("System.Reflection.TypeInfo") is null);
        var fieldsMethod = module.GetType("System.Introspection.TypeInfo").Methods.Single(m => m.Name == "GetFields" && m.Parameters.Count == 0);
        Check("Reflection returns descriptor sequence", ReflectionBindings.Bind(fieldsMethod, fieldsMethod)?.Result == "System.Collections.Sequence<System.Introspection.FieldInfo>");
        foreach (var info in ReflectionBindings.ReferenceTypes.Where(n => n.StartsWith("System.Introspection.")))
            Check(info + " has no array-returning methods", !module.GetType(info).Methods.Any(m => m.ReturnType is ArrayType));
        var fieldsReference = Reference(fieldsMethod, fieldsMethod.DeclaringType);
        fieldsReference.ReturnType = module.TypeSystem.Int32;
        Reject("Reflection return mismatch", () => ReflectionBindings.Bind(fieldsReference, fieldsMethod));
        var descriptorType = module.GetType("System.Introspection.FieldInfo");
        var descriptorBase = descriptorType.BaseType;
        descriptorType.BaseType = module.TypeSystem.Object;
        Reject("Reflection hierarchy mismatch", () => ReflectionBindings.Validate(module));
        descriptorType.BaseType = descriptorBase;
        var arrayDefinition = module.GetType("System.Array`1");
        var arrayOwner = new GenericInstanceType(arrayDefinition);
        arrayOwner.GenericArguments.Add(module.TypeSystem.Int32);
        arrayDefinition.GenericParameters[0].Attributes = GenericParameterAttributes.Covariant;
        var forEach = arrayDefinition.Methods.Single(m => m.Name == "ForEach");
        var forEachCall = Reference(forEach, arrayOwner);
        Reject("Mutable Array<T> remains invariant", () => ArrayCallbackBindings.Bind(forEachCall, forEach));
        arrayDefinition.GenericParameters[0].Attributes = GenericParameterAttributes.NonVariant;
        Check("Closed Array<T>.ForEach instance", ArrayCallbackBindings.Bind(forEachCall, forEach)?.Name == "System.Array<Int32>::ForEach");
        forEachCall.HasThis = false;
        Reject("Array<T>.ForEach requires receiver", () => ArrayCallbackBindings.Bind(forEachCall, forEach));
        forEachCall.HasThis = true;
        var empty = arrayDefinition.Methods.Single(m => m.Name == "get_Empty");
        var emptyCall = Reference(empty, arrayOwner);
        Check("Array<T>.Empty result", ArrayCallbackBindings.Bind(emptyCall, empty)?.Result == "arrayref<Int32>");
        emptyCall.ReturnType = module.TypeSystem.Int32;
        Reject("Array<T>.Empty return mismatch", () => ArrayCallbackBindings.Bind(emptyCall, empty));
        arrayOwner.GenericArguments[0] = new PointerType(module.TypeSystem.Int32);
        Reject("Array<T> unsupported element", () => ArrayCallbackBindings.Bind(forEachCall, forEach));
        var ok = module.GetType("System.Result").NestedTypes.Single(t => t.Name == "Ok`1");
        var writableCase = new GenericInstanceType(ok); writableCase.GenericArguments.Add(module.TypeSystem.Int32);
        var setter = ok.Methods.Single(m => m.Name == "set_Value");
        Check("Case payload setter preserves value receiver", GenericUnionBindings.Bind(Reference(setter, writableCase), setter)?.Result == "noresult");
        var nested = new GenericInstanceType(ok);
        nested.GenericArguments.Add(new ArrayType(result.GenericParameters[0]));
        var shape = new ByReferenceType(nested);
        var before = shape.FullName;
        Check("Recursive type argument under vector and byref", RuntimeSignatures.Close(shape, owner).FullName
            == "System.Result/Ok`1<System.String[]>&");
        Check("Input signature remains unchanged", shape.FullName == before);
        var caller = new TypeDefinition("Test", "Caller`2", TypeAttributes.Class);
        caller.GenericParameters.Add(new GenericParameter("A", caller));
        caller.GenericParameters.Add(new GenericParameter("B", caller));
        var nestedCaller = new GenericInstanceType(module.GetType("System.Collections.Iterable`1"));
        nestedCaller.GenericArguments.Add(caller.GenericParameters[1]);
        var nestedOwner = new GenericInstanceType(result);
        nestedOwner.GenericArguments.Add(caller.GenericParameters[0]);
        nestedOwner.GenericArguments.Add(nestedCaller);
        Check("Nested caller parameters are substituted simultaneously",
            ReferenceEquals(RuntimeSignatures.Close(result.GenericParameters[1], nestedOwner,
                allowOpenMethodParameters: true), nestedCaller));
        Reject("Open nested caller still requires library admission", () =>
            RuntimeSignatures.Close(result.GenericParameters[1], nestedOwner));

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
        Check("String IsEmpty property", stringType.Properties.Any(p => p.Name == "IsEmpty" && p.GetMethod.Name == "get_IsEmpty"));
        var utf8Type = module.GetType(Utf8Bindings.Owner);
        foreach (var method in utf8Type.Methods.Where(m => m.Name is "Encode" or "Decode"))
        {
            Check("Utf8 " + method.Name, Utf8Bindings.Bind(Reference(method, utf8Type), method)?.Name == Utf8Bindings.Owner + "::" + method.Name);
            var wrong = Reference(method, utf8Type);
            wrong.Parameters[0].ParameterType = module.TypeSystem.Int32;
            Reject("Utf8 invalid " + method.Name + " argument", () => Utf8Bindings.Bind(wrong, method));
            wrong = Reference(method, utf8Type);
            wrong.HasThis = true;
            Reject("Utf8 instance " + method.Name, () => Utf8Bindings.Bind(wrong, method));
        }
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
        var math = module.Types.Single(t => t.Namespace == "System.Math" && NamespaceFunctions.IsContainer(t));
        var sqrt = math.Methods.Single(m => m.Name == "Sqrt");
        var sqrtCall = Reference(sqrt, math);
        Check("Double Math mapping", DoubleBindings.Bind(sqrtCall, sqrt)?.Result == "Double");
        sqrtCall.Parameters[0].ParameterType = module.TypeSystem.Int32;
        Reject("Double Math argument mismatch", () => DoubleBindings.Bind(sqrtCall, sqrt));
        var doubleType = module.GetType("System.Double");
        var doubleCompare = doubleType.Methods.Single(m => m.Name == "CompareTo");
        Check("Double receiver mapping", DoubleBindings.Bind(Reference(doubleCompare, doubleType), doubleCompare)?.Arguments.SequenceEqual(new[] { "Double&", "Double" }) == true);
        Check("Char retains its own stack category", PrimitiveBindings.Stack("Byte") == "Int32" && PrimitiveBindings.Stack("Char") == "Char");
        Check("Unsigned and Single stack normalization", PrimitiveBindings.Stack("UInt64") == "Int64" && PrimitiveBindings.Stack("Single") == "Double");
        var charType = module.GetType("System.Char");
        var factory = charType.Methods.Single(m => m.Name == "FromString");
        var factoryCall = Reference(factory, charType);
        Check("Char factory returns grapheme storage", PrimitiveBindings.Bind(factoryCall, factory)?.Result == "Char");
        factoryCall.Parameters[0].ParameterType = module.TypeSystem.Int32;
        Reject("Char factory does not accept integers", () => PrimitiveBindings.Bind(factoryCall, factory));
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
        foreach (var name in new[] { "Find", "FindLast", "FindIndex", "FindLastIndex", "FindAll", "Exists", "TrueForAll" }) {
            var search = arrayListDefinition.Methods.Single(m => m.Name == name);
            var searchReference = Reference(search, stringList);
            var expected = name switch {
                "Find" or "FindLast" => "System.Option<String>",
                "FindIndex" or "FindLastIndex" => "System.Option<Int32>",
                "FindAll" => "System.Collections.ArrayList<String>", _ => "Boolean"
            };
            Check(name + " preserves filter outcome", CollectionBindings.Bind(searchReference, search, true)?.Result == expected);
            searchReference.ReturnType = module.TypeSystem.Int32;
            Reject(name + " rejects an old or forged result", () => CollectionBindings.Bind(searchReference, search, true));
        }
        var enumerable = module.GetType("System.Linq.Operators");
        Check("Query metadata exposes only conventional filter/projection names",
            enumerable.Methods.Any(m => m.Name == "Filter") && enumerable.Methods.Any(m => m.Name == "Map")
            && !enumerable.Methods.Any(m => m.Name is "Where" or "Select"));
        foreach (var (name, retired) in new[] { ("Filter", "Where"), ("Map", "Select") }) {
            var operation = enumerable.Methods.Single(m => m.Name == name);
            var operationReference = Reference(operation, enumerable);
            operationReference.CallingConvention = MethodCallingConvention.Generic;
            var call = new GenericInstanceMethod(operationReference);
            call.GenericArguments.Add(module.TypeSystem.String);
            if (name == "Map") call.GenericArguments.Add(module.TypeSystem.Int32);
            var binding = QueryBindings.Bind(call, operation, false);
            Check(name + " binds its renamed generic library method", binding is not null
                && binding.Name.StartsWith("System.Linq.Operators::" + name + "<", StringComparison.Ordinal)
                && binding.Result == (name == "Map" ? "System.Collections.Iterable<Int32>" : "System.Collections.Iterable<String>"));
            operationReference.Name = retired;
            Reject(retired + " rejects a stale method reference", () => QueryBindings.Bind(call, operation, false));
        }
        foreach (var name in new[] { "First", "Last", "Single" })
        foreach (var count in new[] { 1, 2 }) {
            var terminal = enumerable.Methods.Single(m => m.Name == name && m.Parameters.Count == count);
            var terminalReference = Reference(terminal, enumerable);
            terminalReference.CallingConvention = MethodCallingConvention.Generic;
            var call = new GenericInstanceMethod(terminalReference);
            call.GenericArguments.Add(module.TypeSystem.String);
            var expected = name == "Single" ? "System.Result<String,System.Linq.SingleError>" : "System.Option<String>";
            Check(name + " arity " + count + " closes terminal outcome", QueryBindings.Bind(call, terminal, false)?.Result == expected);
            Reject(name + " arity " + count + " rejects virtual call", () => QueryBindings.Bind(call, terminal, true));
            if (count == 2) {
                var predicateType = call.Parameters[1].ParameterType;
                call.Parameters[1].ParameterType = module.TypeSystem.Int32;
                Reject(name + " rejects non-predicate argument", () => QueryBindings.Bind(call, terminal, false));
                call.Parameters[1].ParameterType = predicateType;
            }
            call.ReturnType = module.TypeSystem.String;
            Reject(name + " arity " + count + " rejects payload-only result", () => QueryBindings.Bind(call, terminal, false));
        }
        foreach (var operation in enumerable.Methods.Where(m => m.Name is "Any" or "All" or "Count" or "Fold" or "Take" or "Skip" or "Concat" or "FlatMap")) {
            var basicReference = Reference(operation, enumerable);
            basicReference.CallingConvention = MethodCallingConvention.Generic;
            var call = new GenericInstanceMethod(basicReference);
            call.GenericArguments.Add(module.TypeSystem.Int32);
            if (operation.GenericParameters.Count == 2) call.GenericArguments.Add(module.TypeSystem.String);
            var expected = operation.Name switch {
                "Any" or "All" => "Boolean", "Count" => "Int32", "Fold" => "String",
                "FlatMap" => "System.Collections.Iterable<String>", _ => "System.Collections.Iterable<Int32>"
            };
            var label = operation.Name + " with " + operation.Parameters.Count + " arguments";
            Check(label + " binds closed result", QueryBindings.Bind(call, operation, false)?.Result == expected);
            Reject(label + " rejects virtual call", () => QueryBindings.Bind(call, operation, true));
            call.Parameters[0].ParameterType = module.TypeSystem.Int32;
            Reject(label + " rejects forged receiver", () => QueryBindings.Bind(call, operation, false));
        }
        using (var extensionImage = AssemblyDefinition.ReadAssembly(corePath)) {
            var original = enumerable.Methods.First(m => m.Name == "Filter");
            var implementation = extensionImage.MainModule.GetType("System.Linq.Operators").Methods.First(m => m.Name == "Filter");
            LibraryImplementation.CheckExtensionContract(implementation, original);
            Check("Library extension metadata agrees", true);
            implementation.CustomAttributes.Clear();
            Reject("Unmarked static implementation cannot satisfy an extension contract", () =>
                LibraryImplementation.CheckExtensionContract(implementation, original));
        }
        foreach (var ownerName in new[] { "OptionOperators", "OptionNestedOperators", "ResultOperators" }) {
            var operatorType = module.GetType("System." + ownerName);
            var index = 0;
            foreach (var operation in operatorType.Methods) {
                var operatorReference = Reference(operation, operatorType);
                operatorReference.CallingConvention = MethodCallingConvention.Generic;
                var call = new GenericInstanceMethod(operatorReference);
                var operatorTypeArguments = new[] { module.TypeSystem.Int32, module.TypeSystem.String, module.TypeSystem.Boolean };
                for (var i = 0; i < operation.GenericParameters.Count; i++) call.GenericArguments.Add(operatorTypeArguments[i]);
                var expected = (ownerName, operation.Name) switch {
                    ("OptionOperators", "Map" or "Then") => "System.Option<String>",
                    ("OptionOperators", "Match") => "String",
                    ("OptionOperators", "ThenResult" or "MapResult") => "System.Result<String,Boolean>",
                    ("OptionOperators", "OkOr") => "System.Result<Int32,String>",
                    (_, "UnwrapOr" or "UnwrapOrElse") => "Int32",
                    (_, "ToIterable") => "System.Collections.Iterable<Int32>",
                    ("ResultOperators", "Map" or "Then") => "System.Result<Boolean,String>",
                    ("ResultOperators", "MapError") => "System.Result<Int32,Boolean>",
                    ("ResultOperators", "Match") => "Boolean",
                    ("ResultOperators", _) => "System.Result<Int32,String>",
                    _ => "System.Option<Int32>"
                };
                var label = ownerName + "." + operation.Name + " overload " + index++;
                Check(label + " binds closed outcome", OutcomeOperatorBindings.Bind(call, operation, false)?.Result == expected);
                Reject(label + " rejects virtual call", () => OutcomeOperatorBindings.Bind(call, operation, true));
                call.Parameters[0].ParameterType = module.TypeSystem.Int32;
                Reject(label + " rejects wrong receiver", () => OutcomeOperatorBindings.Bind(call, operation, false));
            }
        }
        MapBindings.Validate(module);
        var mapDefinition = module.GetType("System.Collections.Map`2");
        var closedMap = new GenericInstanceType(mapDefinition);
        closedMap.GenericArguments.Add(module.TypeSystem.Int32);
        closedMap.GenericArguments.Add(module.TypeSystem.String);
        var mapFind = mapDefinition.Methods.Single(m => m.Name == "Find");
        Check("Map Find closes Option payload", MapBindings.Bind(Reference(mapFind, closedMap), mapFind, true)?.Result == "System.Option<String>");
        var wrongMapFind = Reference(mapFind, closedMap);
        wrongMapFind.ReturnType = module.TypeSystem.String;
        Reject("Map Find result mismatch", () => MapBindings.Bind(wrongMapFind, mapFind, true));
        Reject("Map instance requires virtual call", () => MapBindings.Bind(Reference(mapFind, closedMap), mapFind, false));
        mapDefinition.GenericParameters[1].Attributes = GenericParameterAttributes.Covariant;
        Reject("Map variance remains provisional", () => MapBindings.Validate(module));
        mapDefinition.GenericParameters[1].Attributes = GenericParameterAttributes.NonVariant;
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
        var taskDefinition = module.GetType("System.Tasks.Task`1");
        var taskInt = new GenericInstanceType(taskDefinition);
        taskInt.GenericArguments.Add(module.TypeSystem.Int32);
        var getResult = taskDefinition.Methods.Single(m => m.Name == "GetResult");
        Check("Task closes ordinary payload", TaskBindings.Bind(Reference(getResult, taskInt), getResult, false, false)?.Result == "Int32");
        var taskUnit = new GenericInstanceType(taskDefinition);
        taskUnit.GenericArguments.Add(new TypeReference("System", "Void", module, module, true));
        Check("Task unit stays a value", TaskBindings.Bind(Reference(getResult, taskUnit), getResult, false, false)?.Result == "Void");
        var wrongResult = Reference(getResult, taskInt);
        wrongResult.ReturnType = module.TypeSystem.String;
        Reject("Task rejects forged payload", () => TaskBindings.Bind(wrongResult, getResult, false, false));
        var taskConstructor = taskDefinition.Methods.Single(m => m.IsConstructor);
        Reject("Task constructor is not a consumer capability", () => TaskBindings.Bind(Reference(taskConstructor, taskInt), taskConstructor, true, false));
        var constructorAttributes = taskConstructor.Attributes;
        taskConstructor.IsPublic = true;
        Reject("Task rejects forged public constructor", () => TaskBindings.Bind(Reference(taskConstructor, taskInt), taskConstructor, true, true));
        taskConstructor.Attributes = constructorAttributes;
        var taskAttributes = taskDefinition.Attributes;
        taskDefinition.IsSealed = false;
        Reject("Task reference contract remains sealed", () => TaskBindings.Bind(Reference(getResult, taskInt), getResult, false, false));
        taskDefinition.Attributes = taskAttributes;
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
