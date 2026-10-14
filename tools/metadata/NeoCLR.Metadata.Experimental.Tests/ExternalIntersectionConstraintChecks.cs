using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ExternalIntersectionConstraintChecks
{
    private sealed class Resolver(AssemblyDefinition? definition) : IAssemblyResolver
    {
        public AssemblyDefinition? Resolve(AssemblyIdentity identity) => definition;
    }

    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var provider = new AssemblyBuilder(new("IntersectionContracts", new Version(1, 0, 0, 0)), core);
        provider.AddInterface("Example", "A");
        provider.AddGenericInterface("Example", "B", ["T"]);
        provider.AddClass("Example", "Open");
        provider.AddClosedClass("Example", "Hierarchy");
        var closed = provider.AddClass("Example", "Closed"); closed.SetSealedClass();
        var snapshot = AssemblyDefinition.ReadNativeAssembly(NativeModuleContainer.WriteLibraryBinary(provider.WriteNativeAssembly()));
        var graph = new AssemblyBuilder(new("IntersectionConsumer", new Version(1, 0, 0, 0)), core);
        ImportedTypeReference Import(string name) => graph.ImportReference(snapshot.MainModule.Types.Single(t => t.Name == name), core);
        var a = Import("A");
        var b = Import("B`1").MakeGenericInstance(PrimitiveType.Int32);
        var method = graph.AddFunction("Both", new MethodSignature(PrimitiveType.Int32, [], ["T"]));
        method.AddExperimentalStructuralConstraint(0, ReferenceCompositionKind.Intersection, a, b);
        method.LoadConstant(42); method.Return();
        Reject(() => method.AddExperimentalStructuralConstraint(0, ReferenceCompositionKind.Intersection, a, Import("Closed")));
        foreach (var name in new[] { "Open", "Hierarchy" })
        {
            var withClass = graph.AddFunction("With" + name, new MethodSignature(PrimitiveType.Int32, [], ["T"]));
            withClass.AddExperimentalStructuralConstraint(0, ReferenceCompositionKind.Intersection, a, Import(name));
            withClass.LoadConstant(42); withClass.Return();
        }
        var onlyA = graph.AddInterface("Example", "OnlyA");
        // Definition imports and completed interface contracts serve distinct purposes.
        var image = NativeModuleContainer.WriteBinary(graph.WriteNativeAssembly());
        Reject(() => AssemblyDefinition.ReadNativeAssembly(image, true));
        Reject(() => AssemblyDefinition.ReadNativeAssembly(image, true, new Resolver(null)));
        Reject(() => AssemblyDefinition.ReadNativeAssembly(image, false, new Resolver(snapshot)));
        var loaded = AssemblyDefinition.ReadNativeAssembly(image, true, new Resolver(snapshot));
        if (loaded.MainModule.Functions.Single(m => m.Name == "Both").TypeConstraints.Single().Bound.Composition?.Kind != ReferenceCompositionKind.Intersection)
            throw new Exception("external intersection grouping lost");
        var replacement = new AssemblyBuilder(provider.Identity, core);
        replacement.AddClass("Example", "A").SetSealedClass();
        replacement.AddGenericInterface("Example", "B", ["T"]);
        replacement.AddClass("Example", "Open");
        replacement.AddClosedClass("Example", "Hierarchy");
        var finalSnapshot = AssemblyDefinition.ReadNativeAssembly(NativeModuleContainer.WriteLibraryBinary(replacement.WriteNativeAssembly()));
        Reject(() => AssemblyDefinition.ReadNativeAssembly(image, true, new Resolver(finalSnapshot)));
        var valueReplacement = new AssemblyBuilder(provider.Identity, core);
        valueReplacement.AddValueType("Example", "A");
        valueReplacement.AddGenericInterface("Example", "B", ["T"]);
        valueReplacement.AddClass("Example", "Open");
        valueReplacement.AddClosedClass("Example", "Hierarchy");
        var valueSnapshot = AssemblyDefinition.ReadNativeAssembly(NativeModuleContainer.WriteLibraryBinary(valueReplacement.WriteNativeAssembly()));
        Reject(() => AssemblyDefinition.ReadNativeAssembly(image, true, new Resolver(valueSnapshot)));
        var wrong = new AssemblyBuilder(new("WrongIdentity", new Version(1, 0, 0, 0)), core);
        wrong.AddInterface("Example", "A");
        wrong.AddGenericInterface("Example", "B", ["T"]);
        wrong.AddClass("Example", "Open");
        wrong.AddClosedClass("Example", "Hierarchy");
        var wrongSnapshot = AssemblyDefinition.ReadNativeAssembly(NativeModuleContainer.WriteLibraryBinary(wrong.WriteNativeAssembly()));
        Reject(() => AssemblyDefinition.ReadNativeAssembly(image, true, new Resolver(wrongSnapshot)));
        Reject(() => method.MakeGenericInstance(onlyA));
        Reject(() => graph.Write());
        Console.WriteLine("PASS resolved external intersection constraints");
    }

    private static void Reject(Action action)
    {
        try { action(); }
        catch (Exception e) when (e is ArgumentException or InvalidDataException or NotSupportedException) { return; }
        throw new Exception("invalid external intersection accepted");
    }
}
