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
        var main = app.AddFunction("Main"); app.EntryPoint = main;
        main.LoadConstant(42); main.Call(reference); main.Return();
        _ = app.WriteNativeAssembly();
        Reject<InvalidDataException>(() => app.CreateFunctionReference(dependency, core, new string('b', 64), "Example", "Identity", signature));
        Reject<InvalidDataException>(() => app.CreateFunctionReference(dependency, core, hash, "Example", "Identity", new MethodSignature(PrimitiveType.Boolean, [PrimitiveType.Int32])));
        Reject<ArgumentException>(() => app.CreateFunctionReference(dependency, core, "bad", "Example", "Identity", signature));
        Reject<InvalidDataException>(() => app.CreateFunctionReference(dependency, new("Wrong", new Version(1, 0, 0, 0)), hash, "Example", "Identity", signature));
        Reject<InvalidDataException>(() => app.CreateFunctionReference(app.Identity, core, hash, "Example", "Identity", signature));
        Reject<ArgumentException>(() => app.CreateFunctionReference(dependency, core, hash, "Example", "Broken", new MethodSignature(SignatureType.MethodParameter(0), [])));
    }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
