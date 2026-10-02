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
