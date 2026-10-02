using NeoCLR.Metadata.Experimental.Model;

internal static class AuthoredFunctionReferenceChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var dependency = new AssemblyIdentity("Library", new Version(1, 0, 0, 0));
        var app = new AssemblyBuilder(new("App", new Version(1, 0, 0, 0)), core);
        var hash = new string('a', 64);
        var signature = new MethodSignature(PrimitiveType.Int32, [PrimitiveType.Int32]);
        var reference = app.CreateFunctionReference(dependency, core, hash, "Example", "Identity", signature);
        if (!ReferenceEquals(reference, app.CreateFunctionReference(dependency, core, hash.ToUpperInvariant(), "Example", "Identity", signature)))
            throw new Exception("reference interning");
        var box = app.CreateTypeReference(dependency, core, hash, "Example", "Box`1", 1);
        if (!ReferenceEquals(box, app.CreateTypeReference(dependency, core, hash.ToUpperInvariant(), "Example", "Box`1", 1)))
            throw new Exception("type reference interning");
        var arguments = new SignatureType[] { PrimitiveType.Int32 };
        var constructed = box.MakeGenericInstance(arguments);
        arguments[0] = PrimitiveType.Boolean;
        if (constructed.TypeArguments[0] != PrimitiveType.Int32) throw new Exception("copied type arguments");
        Reject<InvalidDataException>(() => app.CreateTypeReference(dependency, core, new string('b', 64), "Example", "Box`1", 1));
        Reject<InvalidDataException>(() => app.CreateTypeReference(dependency, core, hash, "Example", "Box", 1));
        Reject<InvalidDataException>(() => app.CreateTypeReference(dependency, new("OtherCore", new Version(1, 0, 0, 0)), hash, "Example", "Item"));
        Reject<InvalidDataException>(() => app.CreateTypeReference(app.Identity, core, hash, "Example", "Item"));
        var getterSignature = new MethodSignature(SignatureType.TypeParameter(0), []);
        var getter = app.CreateMethodReference(box, "Get", getterSignature);
        if (!ReferenceEquals(getter, app.CreateMethodReference(box, "Get", getterSignature))) throw new Exception("method interning");
        Reject<InvalidDataException>(() => app.CreateMethodReference(box, "Get", new MethodSignature(PrimitiveType.Boolean, [])));
        Reject<ArgumentException>(() => app.CreateMethodReference(box, ".ctor", new MethodSignature(PrimitiveType.Int32, [])));
        Reject<ArgumentException>(() => app.CreateMethodReference(constructed, "Get", getterSignature));
        Reject<ArgumentException>(() => app.CreateMethodReference(box, "Bad", new MethodSignature(SignatureType.TypeParameter(1), [])));
        Reject<ArgumentException>(() => app.CreateMethodReference(box, "GenericInstance", new MethodSignature(PrimitiveType.Void, [], ["T"])));
        SignatureType nominal = constructed;
        var nominalCall = app.CreateFunctionReference(dependency, core, hash, "Example", "EchoBox", new MethodSignature(nominal, [nominal]));
        var forwarding = app.AddFunction("ForwardBox", new MethodSignature(nominal, [nominal]));
        forwarding.LoadArgument(0); forwarding.Call(nominalCall); forwarding.Return();
        if (nominalCall.Signature.ReturnType != nominal) throw new Exception("nominal signature identity");
        SignatureType open = box.MakeGenericInstance(SignatureType.MethodParameter(0));
        var vectors = SignatureType.ArrayOf(open);
        SignatureType wrongScope = box.MakeGenericInstance(SignatureType.MethodParameter(1));
        Reject<ArgumentException>(() => app.CreateFunctionReference(dependency, core, hash, "Example", "WrongScope",
            new MethodSignature(SignatureType.ArrayOf(wrongScope), [], ["T"])));
        _ = app.CreateFunctionReference(dependency, core, hash, "Example", "EchoBoxes", new MethodSignature(vectors, [vectors], ["T"]));
        var foreign = new AssemblyBuilder(new("Foreign", new Version(1, 0, 0, 0)), core);
        SignatureType foreignType = foreign.CreateTypeReference(dependency, core, hash, "Example", "Item");
        Reject<ArgumentException>(() => app.CreateFunctionReference(dependency, core, hash, "Example", "Foreign", new MethodSignature(foreignType, [])));
        var item = app.CreateTypeReference(dependency, core, hash, "Example", "Item");
        var valueField = app.CreateFieldReference(item, "Value", PrimitiveType.Int32, 2);
        if (!ReferenceEquals(valueField, app.CreateFieldReference(item, "Value", PrimitiveType.Int32, 2))) throw new Exception("field interning");
        Reject<InvalidDataException>(() => app.CreateFieldReference(item, "Other", PrimitiveType.Int32, 2));
        Reject<InvalidDataException>(() => app.CreateFieldReference(item, "Value", PrimitiveType.Int32, 3));
        Reject<InvalidDataException>(() => app.CreateFieldReference(item, "Value", PrimitiveType.Int32, 2, true));
        Reject<ArgumentException>(() => app.CreateFieldReference(item, "Bad", PrimitiveType.Int32, -1));
        Reject<ArgumentException>(() => app.CreateFieldReference(box, "Bad", PrimitiveType.Int32, 0));
        var nominalField = app.CreateFieldReference(item, "Box", nominal, 4);
        var vectorField = app.CreateFieldReference(item, "Boxes", SignatureType.ArrayOf(nominal), 5);
        if (nominalField.FieldType != nominal || vectorField.FieldType.ArrayElement != nominal) throw new Exception("closed field signature");
        Reject<ArgumentException>(() => app.CreateFieldReference(item, "Open", open, 6));
        Reject<ArgumentException>(() => app.CreateFieldReference(item, "Definition", box, 6));
        Reject<ArgumentException>(() => app.CreateFieldReference(item, "Foreign", foreignType, 6));
        Reject<ArgumentException>(() => app.CreateFieldReference(item, "Parameter", SignatureType.TypeParameter(0), 6));
        var read = app.AddFunction("Read", new MethodSignature(PrimitiveType.Int32, [item]));
        read.LoadArgument(0); read.LoadField(valueField); read.Return();
        var contract = app.CreateInterfaceReference(dependency, core, hash, "Example", "IValue");
        var derived = app.CreateInterfaceReference(dependency, core, hash, "Example", "IDerived");
        app.AddInterfaceConversion(derived, contract);
        app.AddInterfaceConversion(item, derived);
        app.AddInterfaceConversion(item, derived); // Idempotent.
        Reject<ArgumentException>(() => app.AddInterfaceConversion(contract, derived));
        Reject<ArgumentException>(() => app.AddInterfaceConversion(contract, item));
        Reject<InvalidDataException>(() => app.CreateTypeReference(dependency, core, hash, "Example", "IValue"));
        var valueMethod = app.CreateMethodReference(contract, "Get", new MethodSignature(PrimitiveType.Int32, []));
        if (!valueMethod.RequiresVirtualDispatch || !valueMethod.IsInterfaceMethod) throw new Exception("interface dispatch flags");
        var conversion = app.AddFunction("AsValue", new MethodSignature(contract, [item]));
        conversion.LoadArgument(0); conversion.Return();
        var dispatch = app.AddFunction("Dispatch", new MethodSignature(PrimitiveType.Int32, [derived]));
        dispatch.LoadArgument(0); dispatch.CallVirtual(valueMethod); dispatch.Return();
        var main = app.AddFunction("Main"); app.EntryPoint = main;
        main.LoadConstant(42); main.Call(reference); main.Return();
        _ = app.WriteNativeAssembly();
        Reject<InvalidDataException>(() => app.CreateFunctionReference(dependency, core, new string('b', 64), "Example", "Identity", signature));
        Reject<InvalidDataException>(() => app.CreateFunctionReference(dependency, core, hash, "Example", "Identity", new MethodSignature(PrimitiveType.Boolean, [PrimitiveType.Int32])));
        Reject<ArgumentException>(() => app.CreateFunctionReference(dependency, core, "bad", "Example", "Identity", signature));
        Reject<InvalidDataException>(() => app.CreateFunctionReference(dependency, new("Wrong", new Version(1, 0, 0, 0)), hash, "Example", "Identity", signature));
        Reject<InvalidDataException>(() => app.CreateFunctionReference(app.Identity, core, hash, "Example", "Identity", signature));
        Reject<ArgumentException>(() => app.CreateFunctionReference(dependency, core, hash, "Example", "Broken", new MethodSignature(SignatureType.MethodParameter(0), [])));
        var readonlyField = app.CreateFieldReference(item, "ReadOnly", PrimitiveType.Int32, 3, true);
        var write = app.AddFunction("WriteReadOnly", new MethodSignature(PrimitiveType.Void, [item, PrimitiveType.Int32]));
        write.LoadArgument(0); write.LoadArgument(1); write.StoreField(readonlyField); write.Return();
        Reject<InvalidDataException>(() => app.WriteNativeAssembly());
    }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
