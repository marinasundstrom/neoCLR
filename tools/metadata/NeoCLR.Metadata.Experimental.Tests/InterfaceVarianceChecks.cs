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
        CheckExternalBaseConversions();
        CheckStringConversions();
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
    private static void CheckStringConversions()
    {
        var graph = new AssemblyBuilder(new("StringVariance", new(1, 0, 0, 0)), new("System.Runtime", new(10, 0, 0, 0)));
        var root = graph.AddNativeObjectRoot();
        var view = graph.AddInterface("Example", "TextView");
        var text = graph.AddClass("System", "String", root);
        text.SetNativePrimitive(PrimitiveType.String);
        text.AddInterfaceImplementation(view);
        var producer = graph.AddGenericInterface("Example", "Producer", ["T"]);
        producer.SetVariance(0, GenericVariance.Covariant);
        producer.AddInterfaceMethod("Get", new(SignatureType.TypeParameter(0), []));
        var consumer = graph.AddGenericInterface("Example", "Consumer", ["T"]);
        consumer.SetVariance(0, GenericVariance.Contravariant);
        consumer.AddInterfaceMethod("Accept", new(PrimitiveType.Int32, [SignatureType.TypeParameter(0)]));
        void Conversion(string name, SignatureType from, SignatureType to)
        {
            var method = graph.AddFunction(name, new(to, [from]));
            method.LoadArgument(0); method.Return();
        }
        Conversion("ObjectProducer", producer.MakeGenericInstance(PrimitiveType.String), producer.MakeGenericInstance(root));
        Conversion("ObjectConsumer", consumer.MakeGenericInstance(root), consumer.MakeGenericInstance(PrimitiveType.String));
        Conversion("InterfaceProducer", producer.MakeGenericInstance(PrimitiveType.String), producer.MakeGenericInstance(view));
        Conversion("InterfaceConsumer", consumer.MakeGenericInstance(view), consumer.MakeGenericInstance(PrimitiveType.String));
        var snapshot = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(graph));
        var imported = new AssemblyBuilder(new("ImportedStringVariance", new(1, 0, 0, 0)), graph.CoreLibrary);
        ImportedTypeReference Import(string name) => imported.ImportReference(snapshot.MainModule.Types.Single(t => t.Name == name), graph.CoreLibrary);
        _ = Import("String");
        var importedView = Import("TextView");
        var importedProducer = Import("Producer`1");
        var importedConsumer = Import("Consumer`1");
        foreach (var (name, from, to) in new (string, SignatureType, SignatureType)[] {
            ("View", importedProducer.MakeGenericInstance(PrimitiveType.String), importedProducer.MakeGenericInstance(importedView)),
            ("Consumer", importedConsumer.MakeGenericInstance(importedView), importedConsumer.MakeGenericInstance(PrimitiveType.String)),
        })
        {
            var method = imported.AddFunction(name, new(to, [from]));
            method.LoadArgument(0); method.Return();
        }
        _ = imported.WriteNativeAssembly();
        Conversion("Reverse", producer.MakeGenericInstance(root), producer.MakeGenericInstance(PrimitiveType.String));
        Reject<InvalidDataException>(() => graph.WriteNativeAssembly());
    }

    private static void CheckExternalBaseConversions()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var library = new AssemblyBuilder(new("VarianceBaseLibrary", new(1, 0, 0, 0)), core);
        var parent = library.AddClass("Example", "Base");
        parent.SetAbstractClass();
        parent.AddConstructor([], MethodVisibility.Protected).Return();
        var snapshot = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(library));
        var graph = new AssemblyBuilder(new("VarianceBaseConsumer", new(1, 0, 0, 0)), core);
        var external = graph.ImportReference(snapshot.MainModule.Types.Single(), core);
        graph.DeclareFieldlessClassBase(external);
        var derived = graph.AddClass("Example", "Derived", external);
        var leaf = graph.AddClass("Example", "Leaf", derived);
        var unrelated = graph.AddClass("Example", "Unrelated");
        var producer = graph.AddGenericInterface("Example", "Producer", ["T"]);
        producer.SetVariance(0, GenericVariance.Covariant);
        producer.AddInterfaceMethod("Get", new(SignatureType.TypeParameter(0), []));
        var consumer = graph.AddGenericInterface("Example", "Consumer", ["T"]);
        consumer.SetVariance(0, GenericVariance.Contravariant);
        consumer.AddInterfaceMethod("Accept", new(PrimitiveType.Void, [SignatureType.TypeParameter(0)]));
        void Conversion(string name, SignatureType from, SignatureType to)
        {
            var method = graph.AddFunction(name, new(to, [from]));
            method.LoadArgument(0); method.Return();
        }
        Conversion("Direct", producer.MakeGenericInstance(derived), producer.MakeGenericInstance(external));
        Conversion("Transitive", producer.MakeGenericInstance(leaf), producer.MakeGenericInstance(external));
        Conversion("Contravariant", consumer.MakeGenericInstance(external), consumer.MakeGenericInstance(leaf));
        Conversion("Nested", producer.MakeGenericInstance(consumer.MakeGenericInstance(external)),
            producer.MakeGenericInstance(consumer.MakeGenericInstance(leaf)));
        var image = graph.WriteNativeAssembly();
        Check(AssemblyDefinition.ReadNativeAssembly(NativeModuleContainer.WriteBinary(image)).MainModule.Types.Count == 5,
            "external-base variance image retains nominal declarations");
        var context = new System.Runtime.Loader.AssemblyLoadContext("variance-external-base", true);
        try
        {
            context.LoadFromStream(new MemoryStream(library.Write()));
            var loaded = context.LoadFromStream(new MemoryStream(graph.Write()));
            foreach (var name in new[] { "Direct", "Transitive", "Contravariant", "Nested" })
            {
                var method = loaded.ManifestModule.GetMethods().Single(m => m.Name == name);
                Check(method.Invoke(null, [null]) is null, "CLR external-base variance " + name);
            }
        }
        finally { context.Unload(); }
        var invalid = graph.AddFunction("Invalid", new(producer.MakeGenericInstance(external), [producer.MakeGenericInstance(unrelated)]));
        invalid.LoadArgument(0); invalid.Return();
        Reject<InvalidDataException>(() => graph.WriteNativeAssembly());
        invalid.ClearBody(); invalid.GetILGenerator().LoadDefault(producer.MakeGenericInstance(external)); invalid.Return();
        Conversion("Reverse", producer.MakeGenericInstance(external), producer.MakeGenericInstance(derived));
        Reject<InvalidDataException>(() => graph.WriteNativeAssembly());
    }
    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("Expected " + typeof(T).Name);
    }
}
