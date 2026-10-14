using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ReferenceCompositionImportChecks
{
    internal static void Run(string? output = null)
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var provider = new AssemblyBuilder(new("CompositionProvider", new Version(1, 0, 0, 0)), core);
        var a = provider.AddInterface("Example", "A");
        var b = provider.AddInterface("Example", "B");
        var both = provider.AddClass("Example", "Both");
        var derivedB = provider.AddInterface("Example", "DerivedB");
        derivedB.AddBaseInterface(b);
        both.AddInterfaceImplementation(a); both.AddInterfaceImplementation(derivedB);
        var ctor = both.AddConstructor(Array.Empty<PrimitiveType>()); ctor.Return();
        var produce = provider.AddFunction("Produce", new MethodSignature(a, Array.Empty<SignatureType>()));
        produce.NewObject(ctor); produce.Return();
        var echo = provider.AddFunction("Echo", new MethodSignature(a, new SignatureType[] { a }));
        echo.LoadArgument(0); echo.Return();
        var otherBoth = provider.AddClass("Example", "OtherBoth");
        otherBoth.AddInterfaceImplementation(a); otherBoth.AddInterfaceImplementation(derivedB);
        var otherCtor = otherBoth.AddConstructor(Array.Empty<PrimitiveType>()); otherCtor.Return();
        var source = JsonNode.Parse(provider.WriteNativeAssembly())!;
        JsonNode Bound(int index) => new JsonObject { ["Named"] = source["types"]![index]!["name"]!.DeepClone() };
        JsonNode Compound(string kind, bool reverse = false, int second = 1) => new JsonObject { [kind] = new JsonArray(Bound(reverse ? second : 0), Bound(reverse ? 0 : second)) };
        byte[] Image(string kind, string? inputKind = null, int second = 1)
        {
            var root = source.DeepClone();
            root["functions"]![0]!["returns"] = Compound(kind, second: second);
            root["functions"]![1]!["parameters"]![0] = Compound(inputKind ?? kind, true, second);
            root["functions"]![1]!["returns"] = Compound(inputKind ?? kind, second: second);
            return NativeModuleContainer.WriteLibraryBinary(Encoding.UTF8.GetBytes(root.ToJsonString()));
        }
        foreach (var scenario in new[] { (Kind: "Union", Label: "Union", Second: 1), (Kind: "Intersection", Label: "Intersection", Second: 1), (Kind: "Intersection", Label: "RedundantIntersection", Second: 2) })
        {
            var kind = scenario.Kind;
            var image = Image(kind, second: scenario.Second);
            var snapshot = AssemblyDefinition.ReadNativeAssembly(image, true);
            var consumer = new AssemblyBuilder(new("CompositionConsumer" + scenario.Label, new Version(1, 0, 0, 0)), core);
            var factory = snapshot.MainModule.Functions.Single(m => m.Name == "Produce");
            var identity = snapshot.MainModule.Functions.Single(m => m.Name == "Echo");
            Reject(() => consumer.ImportReference(identity, core));
            Reject(() => consumer.ImportExperimentalCompositionReference(snapshot.MainModule.Types.Single(t => t.Name == "Both").Methods.Single(m => m.Name == ".ctor"), core));
            var importedFactory = consumer.ImportExperimentalCompositionReference(factory, core);
            var importedEcho = consumer.ImportExperimentalCompositionReference(identity, core);
            Check(ReferenceEquals(importedEcho, consumer.ImportExperimentalCompositionReference(identity, core)), "canonical callable import");
            Reject(() => consumer.ImportReference(identity, core));
            var result = importedFactory.Signature.ReturnType;
            Check(result == importedEcho.Signature.ParameterTypes[0], "reordered bounds retain exact imported identity");
            Check(result.Composition!.Constituents.All(t => ReferenceEquals(t.ImportedType!.Owner, consumer)), "output-owned bounds");
            Reject(() => consumer.AddFunction("Authored", new MethodSignature(result, Array.Empty<SignatureType>())));
            var main = consumer.AddFunction("Main"); consumer.EntryPoint = main;
            Reject(() => main.DeclareLocal(result));
            main.Call(importedFactory); main.Call(importedEcho); main.Emit(OpCode.Pop); main.LoadConstant(42); main.Return();
            var json = consumer.WriteNativeAssembly();
            var written = JsonNode.Parse(json)!;
            var parameter = written["functions"]![0]!["body"]!.AsArray()
                .Single(i => i!["op"]!.GetValue<string>() == "call" && i["arg"]!["parameters"]!.AsArray().Count == 1)!["arg"]!["parameters"]![0]!;
            Check(parameter[kind]!.AsArray().Count == 2, "native call preserves compound parameter descriptor");
            Reject(() => consumer.Write());
            Reject(() => RuntimeAssemblyContainer.Write(NativeModuleContainer.Read(image), core));
            Reject(() => new AssemblyBuilder(new("Other", new Version(1, 0, 0, 0)), core)
                .ImportExperimentalCompositionReference(identity, new("WrongCore", new Version(1, 0, 0, 0))));
            if (output is not null)
            {
                Directory.CreateDirectory(output);
                File.WriteAllBytes(Path.Combine(output, scenario.Label + "Provider.neox"), image);
                File.WriteAllBytes(Path.Combine(output, scenario.Label + "Consumer.neox"), NativeModuleContainer.WriteLibraryBinary(json));
            }
        }
        // Proof uses metadata guarantees, never the factory body's concrete allocation.
        foreach (var scenario in new[] {
            (Name: "Equivalent", SourceKind: "Intersection", TargetKind: "Intersection", Source: new[] { 0, 2 }, Target: new[] { 1, 2 }, Accepted: true),
            (Name: "EquivalentReverse", SourceKind: "Intersection", TargetKind: "Intersection", Source: new[] { 1, 2 }, Target: new[] { 0, 2 }, Accepted: true),
            (Name: "Weaken", SourceKind: "Intersection", TargetKind: "Intersection", Source: new[] { 0, 2 }, Target: new[] { 0, 1 }, Accepted: true),
            (Name: "Strengthen", SourceKind: "Intersection", TargetKind: "Intersection", Source: new[] { 0, 1 }, Target: new[] { 0, 2 }, Accepted: false),
            (Name: "UnionWiden", SourceKind: "Union", TargetKind: "Union", Source: new[] { 0, 2 }, Target: new[] { 0, 1 }, Accepted: true),
            (Name: "UnionNarrow", SourceKind: "Union", TargetKind: "Union", Source: new[] { 0, 1 }, Target: new[] { 0, 2 }, Accepted: false),
            (Name: "IntersectionToUnion", SourceKind: "Intersection", TargetKind: "Union", Source: new[] { 0, 1 }, Target: new[] { 0, 2 }, Accepted: true),
            (Name: "IntersectionToUnionMissing", SourceKind: "Intersection", TargetKind: "Union", Source: new[] { 0, 1 }, Target: new[] { 2, 4 }, Accepted: false),
            (Name: "UnionToIntersection", SourceKind: "Union", TargetKind: "Intersection", Source: new[] { 2, 4 }, Target: new[] { 0, 1 }, Accepted: true),
            (Name: "UnionToIntersectionMissing", SourceKind: "Union", TargetKind: "Intersection", Source: new[] { 0, 2 }, Target: new[] { 0, 1 }, Accepted: false) })
        {
            JsonNode Composition(string kind, int[] indices) => new JsonObject { [kind] = new JsonArray(indices.Select(Bound).ToArray()) };
            var root = source.DeepClone();
            root["functions"]![0]!["returns"] = Composition(scenario.SourceKind, scenario.Source);
            root["functions"]![1]!["parameters"]![0] = Composition(scenario.TargetKind, scenario.Target);
            root["functions"]![1]!["returns"] = Composition(scenario.TargetKind, scenario.Target);
            var image = NativeModuleContainer.WriteLibraryBinary(Encoding.UTF8.GetBytes(root.ToJsonString()));
            var snapshot = AssemblyDefinition.ReadNativeAssembly(image, true);
            var consumer = new AssemblyBuilder(new("Proof" + scenario.Name, new Version(1, 0, 0, 0)), core);
            var main = consumer.AddFunction("Main"); consumer.EntryPoint = main;
            main.Call(consumer.ImportExperimentalCompositionReference(snapshot.MainModule.Functions.Single(m => m.Name == "Produce"), core));
            main.Call(consumer.ImportExperimentalCompositionReference(snapshot.MainModule.Functions.Single(m => m.Name == "Echo"), core));
            main.Emit(OpCode.Pop); main.LoadConstant(42); main.Return();
            if (!scenario.Accepted) { Reject(() => consumer.WriteNativeAssembly()); continue; }
            var json = consumer.WriteNativeAssembly();
            if (output is not null)
            {
                File.WriteAllBytes(Path.Combine(output, scenario.Name + "Provider.neox"), image);
                File.WriteAllBytes(Path.Combine(output, scenario.Name + "Consumer.neox"), NativeModuleContainer.WriteLibraryBinary(json));
            }
        }
        var mismatched = AssemblyDefinition.ReadNativeAssembly(Image("Union", "Intersection"), true);
        var bad = new AssemblyBuilder(new("BadCompositionCall", new Version(1, 0, 0, 0)), core);
        var badMain = bad.AddFunction("Main"); bad.EntryPoint = badMain;
        badMain.Call(bad.ImportExperimentalCompositionReference(mismatched.MainModule.Functions.Single(m => m.Name == "Produce"), core));
        badMain.Call(bad.ImportExperimentalCompositionReference(mismatched.MainModule.Functions.Single(m => m.Name == "Echo"), core));
        badMain.Emit(OpCode.Pop); badMain.LoadConstant(42); badMain.Return();
        Reject(() => bad.WriteNativeAssembly());
        Console.WriteLine("PASS experimental composition callable imports, native serialization, union/intersection call entailment and closed CLI/storage gates");
    }
    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
    private static void Reject(Action action)
    {
        try { action(); }
        catch (Exception error) when (error is InvalidDataException or ArgumentException) { return; }
        throw new Exception("expected explicit rejection");
    }
}
