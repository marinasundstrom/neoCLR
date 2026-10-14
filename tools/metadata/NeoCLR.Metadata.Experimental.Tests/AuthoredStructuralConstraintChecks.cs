using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class AuthoredStructuralConstraintChecks
{
    internal static void Run(string? output = null)
    {
        var graph = new AssemblyBuilder(new("AuthoredBounds", new Version(1, 0, 0, 0)), new("Core", new Version(1, 0, 0, 0)));
        var a = graph.AddInterface("Example", "A");
        var b = graph.AddInterface("Example", "B");
        var onlyA = graph.AddClass("Example", "OnlyA");
        onlyA.AddInterfaceImplementation(a);
        var both = graph.AddClass("Example", "Both");
        both.AddInterfaceImplementation(a);
        both.AddInterfaceImplementation(b);
        both.SetSealedClass();
        var neither = graph.AddClass("Example", "Neither");
        var either = graph.AddFunction("Either", new MethodSignature(PrimitiveType.Int32, [], ["T"]));
        either.AddExperimentalStructuralConstraint(0, ReferenceCompositionKind.Union, a, b);
        either.LoadConstant(40); either.Return();
        var all = graph.AddFunction("All", new MethodSignature(PrimitiveType.Int32, [], ["T"]));
        all.AddExperimentalStructuralConstraint(0, ReferenceCompositionKind.Intersection, a, b);
        all.LoadConstant(2); all.Return();
        _ = either.MakeGenericInstance(onlyA);
        _ = all.MakeGenericInstance(both);
        Reject(() => either.MakeGenericInstance(neither));
        Reject(() => all.MakeGenericInstance(onlyA));
        var covered = graph.CreateExperimentalReferenceComposition(ReferenceCompositionKind.Union, onlyA, both);
        var overlap = graph.CreateExperimentalReferenceComposition(ReferenceCompositionKind.Union, onlyA, neither);
        _ = either.MakeGenericInstance(covered);
        Reject(() => either.MakeGenericInstance(overlap));
        Reject(() => all.MakeGenericInstance(covered));
        var generic = graph.AddGenericClass("Example", "Response", ["T"]);
        var closed = generic.MakeGenericInstance(a);
        var genericBound = graph.AddFunction("Closed", new MethodSignature(PrimitiveType.Int32, [], ["T"]));
        genericBound.AddExperimentalStructuralConstraint(0, ReferenceCompositionKind.Union, closed, b);
        genericBound.LoadConstant(42); genericBound.Return();
        _ = genericBound.MakeGenericInstance(closed);
        Reject(() => genericBound.MakeGenericInstance(generic.MakeGenericInstance(b)));
        Reject(() => either.AddExperimentalStructuralConstraint(0, ReferenceCompositionKind.Union, b, a));
        Reject(() => either.AddExperimentalStructuralConstraint(0, ReferenceCompositionKind.Union, neither, neither));
        Reject(() => either.AddExperimentalStructuralConstraint(1, ReferenceCompositionKind.Union, a, b));
        Reject(() => either.AddExperimentalStructuralConstraint(0, ReferenceCompositionKind.Intersection, both, a));
        Reject(() => either.AddExperimentalStructuralConstraint(0, ReferenceCompositionKind.Union, SignatureType.MethodParameter(0), a));
        Reject(() => either.AddExperimentalStructuralConstraint(0, ReferenceCompositionKind.Union, PrimitiveType.Int32, a));
        Reject(() => _ = either.InterfaceConstraints);
        var forwarding = graph.AddFunction("Forward", new MethodSignature(PrimitiveType.Int32, [], ["T"]));
        forwarding.AddExperimentalStructuralConstraint(0, ReferenceCompositionKind.Union, a, b);
        Reject(() => forwarding.Call(either.MakeGenericInstance(SignatureType.MethodParameter(0))));
        forwarding.LoadConstant(42); forwarding.Return();
        var main = graph.AddFunction("Main");
        main.Call(either.MakeGenericInstance(onlyA));
        main.Call(all.MakeGenericInstance(both));
        main.GetILGenerator().Emit(OpCode.Add);
        main.Return(); graph.EntryPoint = main;
        var native = graph.WriteNativeAssembly();
        var image = NativeModuleContainer.WriteBinary(native);
        var snapshot = AssemblyDefinition.ReadNativeAssembly(image, true);
        if (snapshot.MainModule.Functions.Single(m => m.Name == "All").TypeConstraints.Single().Bound.Composition?.Kind != ReferenceCompositionKind.Intersection)
            throw new Exception("authored grouping lost");
        graph.EntryPoint = null;
        var libraryImage = NativeModuleContainer.WriteLibraryBinary(graph.WriteNativeAssembly());
        snapshot = AssemblyDefinition.ReadNativeAssembly(libraryImage, true);
        graph.EntryPoint = main;
        if (output is not null) File.WriteAllBytes(output + ".library.neox", libraryImage);
        var consumer = new AssemblyBuilder(new("ConstraintConsumer", new Version(1, 0, 0, 0)), graph.CoreLibrary);
        var eitherImport = consumer.ImportExperimentalConstrainedReference(snapshot.MainModule.Functions.Single(m => m.Name == "Either"), graph.CoreLibrary);
        var allImport = consumer.ImportExperimentalConstrainedReference(snapshot.MainModule.Functions.Single(m => m.Name == "All"), graph.CoreLibrary);
        ImportedTypeReference ImportType(string name) => consumer.ImportReference(snapshot.MainModule.Types.Single(t => t.Name == name), graph.CoreLibrary);
        var importedA = ImportType("OnlyA");
        var importedBoth = ImportType("Both");
        Reject(() => consumer.ImportReference(snapshot.MainModule.Functions.Single(m => m.Name == "Either"), graph.CoreLibrary));
        var importedCovered = consumer.CreateExperimentalReferenceComposition(ReferenceCompositionKind.Union, importedA, importedBoth);
        _ = eitherImport.MakeGenericInstance(importedCovered);
        var importedOverlap = consumer.CreateExperimentalReferenceComposition(ReferenceCompositionKind.Union, importedA, ImportType("Neither"));
        Reject(() => eitherImport.MakeGenericInstance(importedOverlap));
        Reject(() => eitherImport.MakeGenericInstance(ImportType("Neither")));
        Reject(() => allImport.MakeGenericInstance(importedA));
        var consumerMain = consumer.AddFunction("Main");
        consumerMain.Call(eitherImport.MakeGenericInstance(importedA));
        consumerMain.Call(allImport.MakeGenericInstance(importedBoth));
        consumerMain.GetILGenerator().Emit(OpCode.Add);
        consumerMain.Return(); consumer.EntryPoint = consumerMain;
        var consumerImage = NativeModuleContainer.WriteBinary(consumer.WriteNativeAssembly());
        if (output is not null) File.WriteAllBytes(output + ".consumer.neox", consumerImage);
        Reject(() => consumer.Write());
        Reject(() => graph.Write());
        Reject(() => AssemblyDefinition.ReadNativeAssembly(image));
        if (output is not null) File.WriteAllBytes(output, image);
        // Recheck already-emitted calls after a late declaration change.
        either.AddExperimentalStructuralConstraint(0, ReferenceCompositionKind.Intersection, a, b);
        Reject(() => graph.WriteNativeAssembly());
        Console.WriteLine("PASS authored structural method constraints");
    }
    private static void Reject(Action action)
    {
        try { action(); }
        catch (Exception e) when (e is ArgumentException or InvalidOperationException or InvalidDataException or NotSupportedException) { return; }
        throw new Exception("unsupported structural constraint accepted");
    }
}
