using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class MethodClassConstraintChecks
{
    internal static void Run()
    {
        var graph = new AssemblyBuilder(new("MethodClasses", new(1, 0, 0, 0)), new("System.Runtime", new(10, 0, 0, 0)));
        var bound = graph.AddClass("Example", "Base");
        bound.AddConstructor([]).Return();
        var method = graph.AddFunction("Keep", new(SignatureType.MethodParameter(0), [SignatureType.MethodParameter(0)], ["T"]));
        method.AddBaseTypeConstraint(0, bound);
        method.LoadArgument(0);
        method.Return();
        method.MakeGenericInstance(bound);
        var projected = AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary), false);
        var child = graph.AddClass("Example", "Child", bound);
        method.MakeGenericInstance(child);
        var genericChild = graph.AddGenericClass("Example", "GenericChild", ["T"], bound);
        method.MakeGenericInstance(genericChild.MakeGenericInstance(PrimitiveType.Int32));
        var owner = graph.AddGenericClass("Example", "Owner", ["T"]);
        owner.AddBaseTypeConstraint(0, bound);
        var fromOwner = owner.AddMethod("Forward", new(SignatureType.TypeParameter(0), [SignatureType.TypeParameter(0)]));
        fromOwner.LoadArgument(0);
        fromOwner.Call(method.MakeGenericInstance(SignatureType.TypeParameter(0)));
        fromOwner.Return();
        var unrelated = graph.AddClass("Example", "Unrelated");
        Reject(() => method.MakeGenericInstance(unrelated));
        Reject(() => method.AddBaseTypeConstraint(0, graph.AddInterface("Example", "Contract")));
        var forward = graph.AddFunction("Forward", new(SignatureType.MethodParameter(0), [SignatureType.MethodParameter(0)], ["T"]));
        forward.AddBaseTypeConstraint(0, child);
        forward.LoadArgument(0);
        forward.Call(method.MakeGenericInstance(SignatureType.MethodParameter(0)));
        forward.Return();
        var unconstrained = graph.AddFunction("Unconstrained", new(PrimitiveType.Void, [], ["T"]));
        Reject(() => unconstrained.Call(method.MakeGenericInstance(SignatureType.MethodParameter(0))));
        unconstrained.Return();
        var loaded = System.Reflection.Assembly.Load(graph.Write());
        var reflected = loaded.ManifestModule.GetMethod("Keep")!;
        reflected.MakeGenericMethod(loaded.GetType("Example.Child")!);
        Reject(() => reflected.MakeGenericMethod(typeof(int)));
        Reject(() => method.MakeGenericInstance(PrimitiveType.Int32));
        Reject(() => method.AddBaseTypeConstraint(0, bound));
        foreach (var snapshot in new[] {
            AssemblyDefinition.ReadAssembly(graph.Write(), false),
            AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(graph)),
            projected
        })
        {
            var definition = snapshot.MainModule.Functions.Single(m => m.Name == "Keep");
            if (definition.TypeConstraints.Single().Bound.ReferencedType!.Name != "Base") throw new Exception("class bound lost");
            var consumer = new AssemblyBuilder(new("Consumer", new(1, 0, 0, 0)), graph.CoreLibrary);
            var imported = consumer.ImportReference(definition, graph.CoreLibrary);
            Reject(() => imported.MakeGenericInstance(PrimitiveType.Int32));
            var importedBase = consumer.ImportReference(snapshot.MainModule.Types.Single(t => t.Name == "Base"), graph.CoreLibrary);
            imported.MakeGenericInstance(importedBase);
        }
        var malformed = System.Text.Json.Nodes.JsonNode.Parse(graph.WriteNativeAssembly())!;
        var keep = malformed["functions"]!.AsArray().First(f => f!["generic_constraints"]?.AsArray().Any() == true)!;
        keep["generic_constraints"]!.AsArray().Add(new System.Text.Json.Nodes.JsonObject {
            ["parameter"] = 0,
            ["kind"] = new System.Text.Json.Nodes.JsonObject { ["TypeBound"] = new System.Text.Json.Nodes.JsonObject { ["Named"] = "Example.Unrelated" } }
        });
        Reject(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(malformed.ToJsonString())));
        Console.WriteLine("PASS nominal method class constraints and CLI/native/reference/import contracts");
    }
    private static void Reject(Action action)
    {
        try { action(); }
        catch (Exception error) when (error is ArgumentException or InvalidDataException or InvalidOperationException) { return; }
        throw new Exception("invalid class constraint accepted");
    }
}
