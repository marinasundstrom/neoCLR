using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class AuthoredStructuralConstraintChecks
{
    internal static void Run(string? output = null)
    {
        TraitCalls();
        ExternalTraitCalls();
        ExternalConstructedTraitCalls();
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
        forwarding.Call(either.MakeGenericInstance(SignatureType.MethodParameter(0)));
        forwarding.Return();
        var weak = graph.AddFunction("Weak", new MethodSignature(PrimitiveType.Int32, [], ["T"]));
        weak.AddExperimentalStructuralConstraint(0, ReferenceCompositionKind.Union, a, b);
        Reject(() => weak.Call(all.MakeGenericInstance(SignatureType.MethodParameter(0))));
        weak.LoadConstant(0); weak.Return();
        var unbounded = graph.AddFunction("Unbounded", new MethodSignature(PrimitiveType.Int32, [], ["T"]));
        Reject(() => unbounded.Call(either.MakeGenericInstance(SignatureType.MethodParameter(0))));
        unbounded.LoadConstant(0); unbounded.Return();
        var nominalTarget = graph.AddFunction("NominalTarget", new MethodSignature(PrimitiveType.Int32, [], ["T"]));
        nominalTarget.AddInterfaceConstraint(0, a);
        nominalTarget.LoadConstant(0); nominalTarget.Return();
        Reject(() => weak.Call(nominalTarget.MakeGenericInstance(SignatureType.MethodParameter(0))));
        var strong = graph.AddFunction("Strong", new MethodSignature(PrimitiveType.Int32, [], ["T"]));
        strong.AddExperimentalStructuralConstraint(0, ReferenceCompositionKind.Intersection, a, b);
        strong.Call(nominalTarget.MakeGenericInstance(SignatureType.MethodParameter(0))); strong.Return();
        var combined = graph.AddFunction("Combined", new MethodSignature(PrimitiveType.Int32, [], ["T"]));
        combined.AddInterfaceConstraint(0, a); combined.AddInterfaceConstraint(0, b);
        combined.Call(all.MakeGenericInstance(SignatureType.MethodParameter(0)));
        combined.Return();
        var main = graph.AddFunction("Main");
        main.Call(forwarding.MakeGenericInstance(onlyA));
        main.Call(combined.MakeGenericInstance(both));
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
    private static void TraitCalls()
    {
        var graph = new AssemblyBuilder(new("TraitContracts", new Version(1, 0, 0, 0)), new("Core", new Version(1, 0, 0, 0)));
        var a = graph.AddInterface("Example", "A");
        var b = graph.AddInterface("Example", "B");
        var read = a.AddInterfaceMethod("Read", new MethodSignature(PrimitiveType.Int32, []));
        var parameter = SignatureType.MethodParameter(0);
        var strong = graph.AddFunction("Strong", new MethodSignature(PrimitiveType.Int32, [parameter], ["T"]));
        strong.AddExperimentalStructuralConstraint(0, ReferenceCompositionKind.Intersection, a, b);
        var il = strong.GetILGenerator();
        il.LoadArgumentAddress(0);
        il.CallConstrained(parameter, read);
        il.Return();
        var weak = graph.AddFunction("Weak", new MethodSignature(PrimitiveType.Int32, [parameter], ["T"]));
        weak.AddExperimentalStructuralConstraint(0, ReferenceCompositionKind.Union, a, b);
        Reject(() => weak.GetILGenerator().CallConstrained(parameter, read));
        weak.LoadConstant(0); weak.Return();
        _ = graph.WriteNativeAssembly();
        Reject(() => graph.Write());
    }

    private static void ExternalTraitCalls()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var library = new AssemblyBuilder(new("ExternalTraits", new Version(1, 0, 0, 0)), core);
        var trait = library.AddInterface("Example", "Reads");
        trait.AddInterfaceMethod("Read", new MethodSignature(PrimitiveType.Int32, []));
        var image = NativeModuleContainer.WriteLibraryBinary(library.WriteNativeAssembly());
        var graph = new AssemblyBuilder(new("ExternalTraitConsumer", new Version(1, 0, 0, 0)), core);
        var owner = graph.CreateInterfaceReference(library.Identity, core, Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(image)), "Example", "Reads");
        var imported = graph.CreateMethodReference(owner, "Read", new MethodSignature(PrimitiveType.Int32, []));
        graph.CompleteInterfaceReference(owner);
        var a = graph.AddInterface("Example", "A");
        a.AddBaseInterface(owner);
        var b = graph.AddInterface("Example", "B");
        var parameter = SignatureType.MethodParameter(0);
        var strong = graph.AddFunction("Strong", new MethodSignature(PrimitiveType.Int32, [parameter], ["T"]));
        strong.AddExperimentalStructuralConstraint(0, ReferenceCompositionKind.Intersection, a, b);
        var il = strong.GetILGenerator();
        il.LoadArgumentAddress(0);
        il.CallConstrained(parameter, imported);
        il.Return();
        var weak = graph.AddFunction("Weak", new MethodSignature(PrimitiveType.Int32, [parameter], ["T"]));
        weak.AddExperimentalStructuralConstraint(0, ReferenceCompositionKind.Union, a, b);
        Reject(() => weak.GetILGenerator().CallConstrained(parameter, imported));
        Reject(() => il.CallConstrained(SignatureType.MethodParameter(1), imported));
        weak.LoadConstant(0); weak.Return();
        _ = graph.WriteNativeAssembly();
        Reject(() => graph.Write());
    }

    private static void ExternalConstructedTraitCalls()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var graph = new AssemblyBuilder(new("ConstructedTraitConsumer", new Version(1, 0, 0, 0)), core);
        var owner = graph.CreateInterfaceReference(new("TraitLibrary", new Version(1, 0, 0, 0)), core, new string('A', 64), "Example", "Reads`1", 1);
        var read = graph.CreateMethodReference(owner, "Read", new MethodSignature(SignatureType.TypeParameter(0), []));
        graph.CompleteInterfaceReference(owner);
        var a = graph.AddInterface("Example", "A");
        a.AddBaseInterface(owner.MakeGenericInstance(PrimitiveType.Int32));
        var b = graph.AddInterface("Example", "B");
        var parameter = SignatureType.MethodParameter(0);
        var strong = graph.AddFunction("Strong", new MethodSignature(PrimitiveType.Int32, [parameter], ["T"]));
        strong.AddExperimentalStructuralConstraint(0, ReferenceCompositionKind.Intersection, a, b);
        var il = strong.GetILGenerator();
        il.LoadArgumentAddress(0);
        il.CallConstrained(parameter, read.MakeConstructedReference([PrimitiveType.Int32]));
        il.Return();
        Reject(() => il.CallConstrained(parameter, read.MakeConstructedReference([PrimitiveType.String])));
        var weak = graph.AddFunction("Weak", new MethodSignature(PrimitiveType.Int32, [parameter], ["T"]));
        weak.AddExperimentalStructuralConstraint(0, ReferenceCompositionKind.Union, a, b);
        Reject(() => weak.GetILGenerator().CallConstrained(parameter, read.MakeConstructedReference([PrimitiveType.Int32])));
        weak.LoadConstant(0); weak.Return();
        _ = graph.WriteNativeAssembly();
        Reject(() => graph.Write());
    }

    private static void Reject(Action action)
    {
        try { action(); }
        catch (Exception e) when (e is ArgumentException or InvalidOperationException or InvalidDataException or NotSupportedException) { return; }
        throw new Exception("unsupported structural constraint accepted");
    }
}
