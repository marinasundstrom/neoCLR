using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental.Model;

static class AssemblyAttributeChecks
{
    public static void Run()
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
        Reject<NotSupportedException>(() => graph.WriteNativeAssembly());

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
        Console.WriteLine("PASS assembly attribute CLI authoring/readback, repeated grants, CLR comparison, read-only snapshots, ownership, bounds and native rejection");
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
