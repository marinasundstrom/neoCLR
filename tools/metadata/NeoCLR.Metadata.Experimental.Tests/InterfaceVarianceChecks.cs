using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Introspection;
using NeoCLR.Metadata.Experimental.Model;

internal static class InterfaceVarianceChecks
{
    internal static void Run()
    {
        CheckReferenceConversions();
        var core = new AssemblyIdentity("Core", new(1, 0, 0, 0));
        var graph = new AssemblyBuilder(new("VarianceSnapshot", new(1, 0, 0, 0)), core);
        var contract = graph.AddGenericInterface("Example", "Transform", ["TOut", "TIn", "TFixed"]);
        contract.AddInterfaceMethod("Apply", new(SignatureType.TypeParameter(0), [SignatureType.TypeParameter(1)]));
        var main = graph.AddFunction("Main"); main.LoadConstant(42); main.Return(); graph.EntryPoint = main;
        var json = JsonNode.Parse(graph.WriteNativeAssembly())!;
        byte[] Bytes(JsonNode node) => Encoding.UTF8.GetBytes(node.ToJsonString());
        AssemblyDefinition Read(JsonNode node) => AssemblyDefinition.ReadNativeAssembly(NativeModuleContainer.WriteBinary(Bytes(node)));
        var declaration = json["types"]![0]!;
        foreach (var annotation in new JsonArray?[] { null, new(), new("invariant", "invariant", "invariant") })
        {
            if (annotation is null) declaration.AsObject().Remove("generic_variance");
            else declaration["generic_variance"] = annotation;
            var snapshot = Read(json);
            Check(snapshot.MainModule.Types.Single().GenericParameterVariance!.SequenceEqual(Enumerable.Repeat(GenericVariance.Invariant, 3)), "invariant default");
            _ = NativeAssemblyDefinition.ReadAssembly(Bytes(json)).CreateReferenceAssembly(core);
        }
        declaration["generic_variance"] = new JsonArray("covariant", "contravariant", "invariant");
        var image = NativeModuleContainer.WriteBinary(Bytes(json));
        var loaded = AssemblyDefinition.ReadNativeAssembly(image);
        var type = loaded.MainModule.Types.Single();
        var expected = new[] { GenericVariance.Covariant, GenericVariance.Contravariant, GenericVariance.Invariant };
        Check(type.GenericParameterVariance!.SequenceEqual(expected), "variance annotations");
        Check(loaded.Write().SequenceEqual(image), "opaque image roundtrip preserves annotations");
        Array.Clear(image);
        Check(AssemblyDefinition.ReadNativeAssembly(loaded.Write()).MainModule.Types.Single().GenericParameterVariance!.SequenceEqual(expected), "owned snapshot");
        Reject<NotSupportedException>(() => ((IList<GenericVariance>)type.GenericParameterVariance!)[0] = GenericVariance.Invariant);
        var context = new MetadataLoadContext([loaded]);
        var parameters = context.Resolve(loaded.Identity).GetTypes().Single().GetGenericArguments().Cast<GenericParameterTypeInfo>();
        Check(parameters.Select(p => p.Variance).SequenceEqual(expected), "introspection variance");
        var projection = NativeAssemblyDefinition.ReadAssembly(Bytes(json)).CreateReferenceAssembly(core);
        var projected = AssemblyDefinition.ReadAssembly(projection, expectedExtended: false);
        Check(projected.MainModule.Types.Single(t => t.Name == "Transform`3").GenericParameterVariance!.SequenceEqual(expected), "CLI projection preserves variance");
        var consumer = new AssemblyBuilder(new("Consumer", new(1, 0, 0, 0)), core);
        _ = consumer.ImportReference(type, core);
        foreach (var invalid in new JsonArray[] { new("covariant"), new("out", "invariant", "invariant"), new(1, 0, 0) })
        {
            var changed = json.DeepClone(); changed["types"]![0]!["generic_variance"] = invalid;
            Reject<InvalidDataException>(() => Read(changed));
        }
        var noninterface = json.DeepClone(); noninterface["types"]![0]!.AsObject().Remove("representation");
        Reject<InvalidDataException>(() => Read(noninterface));
        var cli = AssemblyDefinition.ReadAssembly(graph.Write(), expectedExtended: false);
        Check(cli.MainModule.Types.Single(t => t.Name == "Transform`3").GenericParameterVariance!.All(v => v == GenericVariance.Invariant), "CLI invariant metadata");
        contract.SetVariance(0, GenericVariance.Covariant);
        contract.SetVariance(1, GenericVariance.Contravariant);
        var authored = graph.WriteNativeAssembly();
        var authoredSnapshot = AssemblyDefinition.ReadNativeAssembly(NativeModuleContainer.WriteBinary(authored));
        Check(authoredSnapshot.MainModule.Types.Single().GenericParameterVariance!.SequenceEqual(expected), "authored native variance");
        var emitted = System.Reflection.Assembly.Load(graph.Write()).GetType("Example.Transform`3")!;
        Check(emitted.GetGenericArguments().Select(p => (int)(p.GenericParameterAttributes & System.Reflection.GenericParameterAttributes.VarianceMask)).SequenceEqual(new[] { 1, 2, 0 }), "CLR annotation emission");
        Reject<ArgumentException>(() => contract.SetVariance(3, GenericVariance.Covariant));
        Reject<ArgumentException>(() => contract.SetVariance(0, (GenericVariance)99));
        var ordinary = graph.AddGenericClass("Example", "Box", ["T"]);
        Reject<ArgumentException>(() => ordinary.SetVariance(0, GenericVariance.Covariant));
        contract.SetVariance(0, GenericVariance.Contravariant);
        Reject<InvalidDataException>(() => graph.WriteNativeAssembly());
        Reject<InvalidDataException>(() => graph.Write());
        contract.SetVariance(0, GenericVariance.Covariant);
        var nested = graph.AddGenericInterface("Example", "Nested", ["T"]);
        nested.SetVariance(0, GenericVariance.Covariant);
        nested.AddInterfaceMethod("Get", new(contract.MakeGenericInstance(SignatureType.TypeParameter(0), PrimitiveType.Int32, PrimitiveType.Int32), []));
        _ = graph.WriteNativeAssembly();
        nested.AddInterfaceMethod("BadArray", new(SignatureType.ArrayOf(SignatureType.TypeParameter(0)), []));
        Reject<InvalidDataException>(() => graph.WriteNativeAssembly());
        if (Environment.GetEnvironmentVariable("NEOCLR_VARIANCE_ARTIFACT") is { } output)
            File.WriteAllBytes(output, NativeModuleContainer.WriteBinary(authored));
        Console.WriteLine("PASS native interface variance metadata, authoring, projection, immutable snapshots and polarity checks");
    }
    private static void CheckReferenceConversions()
    {
        var graph = new AssemblyBuilder(new("VarianceCalls", new(1, 0, 0, 0)), new("System.Runtime", new(10, 0, 0, 0)));
        var animal = graph.AddInterface("Example", "Animal");
        var dog = graph.AddClass("Example", "Dog"); dog.AddInterfaceImplementation(animal);
        var producer = graph.AddGenericInterface("Example", "Producer", ["T"]);
        producer.SetVariance(0, GenericVariance.Covariant);
        producer.AddInterfaceMethod("Get", new(SignatureType.TypeParameter(0), []));
        var consumer = graph.AddGenericInterface("Example", "Consumer", ["T"]);
        consumer.SetVariance(0, GenericVariance.Contravariant);
        consumer.AddInterfaceMethod("Accept", new(PrimitiveType.Int32, [SignatureType.TypeParameter(0)]));
        var widen = graph.AddFunction("Widen", new(producer.MakeGenericInstance(animal), [producer.MakeGenericInstance(dog)]));
        widen.LoadArgument(0); widen.Return();
        var narrow = graph.AddFunction("Narrow", new(consumer.MakeGenericInstance(dog), [consumer.MakeGenericInstance(animal)]));
        narrow.LoadArgument(0); narrow.Return();
        var main = graph.AddFunction("Main"); main.LoadConstant(42); main.Return(); graph.EntryPoint = main;
        _ = graph.WriteNativeAssembly();
        _ = System.Reflection.Assembly.Load(graph.Write()).GetTypes();
        var external = graph.CreateInterfaceReference(new("External", new(1, 0, 0, 0)), graph.CoreLibrary,
            new string('A', 64), "Example", "Producer`1", 1);
        graph.SetInterfaceVariance(external, [GenericVariance.Covariant]);
        Reject<ArgumentException>(() => graph.SetInterfaceVariance(external, [GenericVariance.Contravariant]));
        Reject<ArgumentException>(() => graph.SetInterfaceVariance(external, []));
        graph.CompleteInterfaceReference(external);
        Reject<InvalidOperationException>(() => graph.SetInterfaceVariance(external, [GenericVariance.Covariant]));
        var externalWiden = graph.AddFunction("ExternalWiden", new(external.MakeGenericInstance(animal), [external.MakeGenericInstance(dog)]));
        externalWiden.LoadArgument(0); externalWiden.Return();
        _ = graph.WriteNativeAssembly();
        var invalid = graph.AddFunction("Reverse", new(producer.MakeGenericInstance(dog), [producer.MakeGenericInstance(animal)]));
        invalid.LoadArgument(0); invalid.Return();
        Reject<InvalidDataException>(() => graph.WriteNativeAssembly());
        invalid.Definition.Body.ClearInstructions();
        invalid.GetILGenerator().LoadDefault(producer.MakeGenericInstance(dog)); invalid.Return();
        var values = graph.AddFunction("NoBoxing", new(producer.MakeGenericInstance(animal), [producer.MakeGenericInstance(PrimitiveType.Int32)]));
        values.LoadArgument(0); values.Return();
        Reject<InvalidDataException>(() => graph.WriteNativeAssembly());
    }
    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("Expected " + typeof(T).Name);
    }
}
