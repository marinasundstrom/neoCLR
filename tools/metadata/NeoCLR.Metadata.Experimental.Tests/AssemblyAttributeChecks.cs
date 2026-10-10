using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental.Model;

static class AssemblyAttributeChecks
{
    public static void Run(string? output = null)
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("AssemblyAnnotations", new(1, 0, 0, 0)), core);
        var reference = graph.Definition.MainModule.ImportReference(core, "System.Runtime.CompilerServices", "InternalsVisibleToAttribute");
        foreach (var friend in new[] { "FirstFriend", "SecondFriend" })
            graph.AddCustomAttribute(new(reference, [new(PrimitiveType.String, friend)]));
        var image = graph.Write();
        var snapshot = AssemblyDefinition.ReadAssembly(image, false);
        var grants = snapshot.CustomAttributes.Where(a => a.AttributeType.Name == "InternalsVisibleToAttribute").ToArray();
        Check(grants.Select(a => (string)a.GetArguments().Single().Value!).SequenceEqual(new[] { "FirstFriend", "SecondFriend" }), "assembly grants lost or reordered");
        Check(snapshot.Write().SequenceEqual(image), "loaded snapshot changed");
        Reject<NotSupportedException>(() => snapshot.CustomAttributes.Clear());
        var foreign = new AssemblyBuilder(new("Other", new(1, 0, 0, 0)), core);
        Reject<ArgumentException>(() => foreign.AddCustomAttribute(graph.Definition.CustomAttributes[0]));
        var nativeCore = new AssemblyIdentity("Core", new(1, 0, 0, 0));
        var nativeGraph = new AssemblyBuilder(new("NativeAnnotations", new(1, 0, 0, 0)), nativeCore);
        var nativeMarker = nativeGraph.AddClass("System.Runtime.CompilerServices", "InternalsVisibleToAttribute");
        var nativeConstructor = nativeMarker.AddConstructor(new MethodSignature(PrimitiveType.Void, [PrimitiveType.String]));
        nativeConstructor.GetILGenerator().Fail("attribute constructor must not run during loading");
        foreach (var friend in new[] { "FirstFriend", "SecondFriend" })
            nativeGraph.AddCustomAttribute(new(nativeConstructor.Definition, [new(PrimitiveType.String, friend)]));
        var main = nativeGraph.AddFunction("Main", new(PrimitiveType.Int32, []));
        main.LoadConstant(42); main.Return(); nativeGraph.EntryPoint = main;
        var native = nativeGraph.WriteNativeAssembly();
        if (output is not null) File.WriteAllBytes(output, NeoCLR.Metadata.Experimental.NativeModuleContainer.WriteLibraryBinary(native));
        foreach (var bytes in new[] { NeoCLR.Metadata.Experimental.RuntimeAssemblyContainer.WriteBinary(native, nativeCore), NeoCLR.Metadata.Experimental.NativeModuleContainer.WriteLibraryBinary(native) })
        {
            var retained = AssemblyDefinition.ReadNativeAssembly(bytes);
            Check(retained.CustomAttributes.Select(a => (string)a.GetArguments().Single().Value!).SequenceEqual(new[] { "FirstFriend", "SecondFriend" }), "native assembly grants lost");
            var metadata = new NeoCLR.Metadata.Experimental.Introspection.MetadataLoadContext([retained]);
            var view = metadata.Resolve(retained.Identity);
            var annotationsView = view.GetCustomAttributes();
            Check(ReferenceEquals(annotationsView, view.GetCustomAttributes()), "assembly views must be cached");
            Check(annotationsView.Count == 2 && annotationsView.All(a => a.GetAttributeType().Name == "InternalsVisibleToAttribute"), "assembly attribute views lost identity");
            Reject<NotSupportedException>(() => ((IList<NeoCLR.Metadata.Experimental.Introspection.CustomAttributeInfo>)annotationsView).Clear());
            Reject<NotSupportedException>(() => retained.CustomAttributes.Clear());
            Check(retained.Write().SequenceEqual(bytes), "native loaded snapshot changed");
        }
        var projected = AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(native).CreateReferenceAssembly(nativeCore), false);
        Check(projected.CustomAttributes.Where(a => a.AttributeType.Name == "InternalsVisibleToAttribute")
            .Select(a => (string)a.GetArguments().Single().Value!).SequenceEqual(new[] { "FirstFriend", "SecondFriend" }), "native CLI projection lost assembly grants");
        var malformed = System.Text.Json.Nodes.JsonNode.Parse(native)!;
        malformed["assemblies"]![0]!["custom_attributes"]![0]!["target_token"] = 1;
        Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(malformed.ToJsonString())));
        malformed = System.Text.Json.Nodes.JsonNode.Parse(native)!;
        var annotations = malformed["assemblies"]![0]!["custom_attributes"]!.AsArray();
        while (annotations.Count <= 256) annotations.Add(annotations[0]!.DeepClone());
        Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(malformed.ToJsonString())));

        var context = new AssemblyLoadContext("assembly-attribute-check", isCollectible: true);
        try
        {
            var assembly = context.LoadFromStream(new MemoryStream(image));
            var actual = assembly.GetCustomAttributesData().Where(a => a.AttributeType.Name == "InternalsVisibleToAttribute");
            Check(actual.Select(a => (string)a.ConstructorArguments.Single().Value!).SequenceEqual(new[] { "FirstFriend", "SecondFriend" }), "CLR assembly attribute mismatch");
        }
        finally { context.Unload(); }

        var local = graph.AddClass("Tests", "NeverRunAttribute");
        var ctor = local.AddConstructor(new MethodSignature(PrimitiveType.Void, []));
        ctor.GetILGenerator().Fail("attribute constructor must not run during metadata inspection");
        graph.AddCustomAttribute(new(ctor.Definition, []));
        var localSnapshot = AssemblyDefinition.ReadAssembly(graph.Write(), false);
        Check(localSnapshot.CustomAttributes.Last().AttributeType.Name == "NeverRunAttribute", "local attribute missing");
        Check(localSnapshot.CustomAttributes.Last().GetArguments().Count == 0, "local constructor data mismatch");
        while (graph.Definition.CustomAttributes.Count < 256)
            graph.AddCustomAttribute(new(reference, [new(PrimitiveType.String, "RepeatedFriend")]));
        Check(AssemblyDefinition.ReadAssembly(graph.Write(), false).CustomAttributes.Count == 256, "assembly attribute limit boundary failed");
        Reject<InvalidDataException>(() => graph.AddCustomAttribute(new(reference, [new(PrimitiveType.String, "Overflow")])));
        var testAssembly = AssemblyDefinition.ReadAssembly(File.ReadAllBytes(typeof(AssemblyAttributeChecks).Assembly.Location), false);
        var testContext = new NeoCLR.Metadata.Experimental.Introspection.MetadataLoadContext([testAssembly]);
        var property = testContext.Resolve(testAssembly.Identity).GetTypes().Single(t => t.Name == nameof(EnumPropertyFixture)).GetProperties().Single();
        Check(property.PropertyType is NeoCLR.Metadata.Experimental.Introspection.NominalTypeInfo { IsEnum: true, Name: nameof(PropertyTargets) }, "CLI enum property identity lost");
        Console.WriteLine("PASS assembly attribute CLI authoring/readback, repeated grants, CLR comparison, read-only snapshots, ownership, bounds and native retention");
    }

    private static void Check(bool condition, string message)
    {
        if (!condition) throw new Exception(message);
    }

    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); }
        catch (T) { return; }
        throw new Exception("Expected " + typeof(T).Name);
    }
}


enum PropertyTargets { Assembly = 1 }
sealed class EnumPropertyFixture
{
    public PropertyTargets ValidOn => PropertyTargets.Assembly;
}
