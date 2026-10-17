using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class AuthoredCompositionMethodChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var dependency = new AssemblyIdentity("Library", new Version(1, 0, 0, 0));
        var app = new AssemblyBuilder(new("Consumer", new Version(1, 0, 0, 0)), core);
        var hash = new string('a', 64);
        var owner = app.CreateTypeReference(dependency, core, hash, "Example", "Services");
        var first = app.CreateValueTypeReference(dependency, core, hash, "Example", "First");
        var second = app.CreateValueTypeReference(dependency, core, hash, "Example", "Second");
        var task = app.CreateTypeReference(dependency, core, hash, "Example", "Task`1", genericArity: 1);
        var result = app.CreateValueTypeReference(dependency, core, hash, "Example", "Result`2", genericArity: 2);
        var union = app.CreateExperimentalReferenceComposition(ReferenceCompositionKind.Union, first, second);
        var nested = task.MakeGenericInstance(result.MakeGenericInstance(SignatureType.MethodParameter(0), union));
        var signature = new MethodSignature(nested, [nested], ["T"]);
        var method = app.CreateMethodReference(owner, "Echo", signature, isStatic: true);
        if (!ReferenceEquals(method, app.CreateMethodReference(owner, "Echo", signature, isStatic: true)))
            throw new Exception("composition method interning");
        var closed = method.MakeGenericInstance(PrimitiveType.Int32);
        var forward = app.AddFunction("Forward", closed.Signature);
        forward.LoadArgument(0);
        forward.Call(closed);
        forward.Return();
        var main = app.AddFunction("Main");
        main.LoadConstant(42);
        main.Return();
        app.EntryPoint = main;
        var image = NativeModuleContainer.WriteBinary(app.WriteNativeAssembly());
        var reader = AssemblyDefinition.ReadNativeAssembly(image);
        var imported = reader.MainModule.Functions.Single(f => f.Name == "Forward");
        if (!imported.TryGetSignature(out var roundtrip)) throw new Exception("missing roundtrip signature");
        var resultArguments = roundtrip!.ReturnType.ReferencedGenericInstance?.TypeArguments.Single().ReferencedGenericInstance?.TypeArguments;
        if (resultArguments is null || resultArguments.Count != 2 || resultArguments[0].Primitive != PrimitiveType.Int32 ||
            resultArguments[1].Composition is not { Kind: ReferenceCompositionKind.Union } readUnion ||
            !readUnion.Constituents.Select(t => t.ReferencedType?.Name).Order().SequenceEqual(new[] { "First", "Second" }))
            throw new Exception("nested union metadata roundtrip");
        Reject(() => app.CreateMethodReference(owner, "WrongScope", new(nested, [nested]), isStatic: true));
        var foreign = new AssemblyBuilder(new("Foreign", new Version(1, 0, 0, 0)), core);
        Reject(() => foreign.CreateMethodReference(
            foreign.CreateTypeReference(dependency, core, hash, "Example", "Services"),
            "Foreign", signature, isStatic: true));
        Console.WriteLine("PASS authored composition methods: nested generic union, interning, substitution, native roundtrip, scope and ownership rejection");
    }

    private static void Reject(Action action)
    {
        try { action(); }
        catch (ArgumentException) { return; }
        catch (InvalidDataException) { return; }
        throw new Exception("expected invalid contract rejection");
    }
}
