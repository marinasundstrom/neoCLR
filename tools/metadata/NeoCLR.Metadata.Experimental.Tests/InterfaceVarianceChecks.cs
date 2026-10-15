using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Introspection;
using NeoCLR.Metadata.Experimental.Model;

internal static class InterfaceVarianceChecks
{
    internal static void Run()
    {
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
        Reject<NotSupportedException>(() => NativeAssemblyDefinition.ReadAssembly(Bytes(json)).CreateReferenceAssembly(core));
        var consumer = new AssemblyBuilder(new("Consumer", new(1, 0, 0, 0)), core);
        Reject<InvalidDataException>(() => consumer.ImportReference(type, core));
        foreach (var invalid in new JsonArray[] { new("covariant"), new("out", "invariant", "invariant"), new(1, 0, 0) })
        {
            var changed = json.DeepClone(); changed["types"]![0]!["generic_variance"] = invalid;
            Reject<InvalidDataException>(() => Read(changed));
        }
        var noninterface = json.DeepClone(); noninterface["types"]![0]!.AsObject().Remove("representation");
        Reject<InvalidDataException>(() => Read(noninterface));
        var cli = AssemblyDefinition.ReadAssembly(graph.Write(), expectedExtended: false);
        Check(cli.MainModule.Types.Single(t => t.Name == "Transform`3").GenericParameterVariance is null, "CLI support is not inferred");
        Console.WriteLine("PASS native interface variance metadata, immutable snapshots and erasure guards");
    }
    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("Expected " + typeof(T).Name);
    }
}
