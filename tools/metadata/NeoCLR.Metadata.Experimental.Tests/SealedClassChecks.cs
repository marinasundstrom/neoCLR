using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class SealedClassChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("SealedClasses", new(1, 0, 0, 0)), core);
        var built = graph.AddClass("Example", "Built"); built.SetSealedClass(); built.SetSealedClass();
        var manual = new TypeDefinition("Example", "Manual", 0x101, graph.Definition.MainModule.ImportReference(core, "System", "Object"));
        graph.Definition.MainModule.Types.Add(manual);
        var cli = System.Reflection.Assembly.Load(graph.Write());
        if (!cli.GetType("Example.Built")!.IsSealed || !cli.GetType("Example.Manual")!.IsSealed) throw new Exception("CLI final flag lost");
        var snapshot = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(graph));
        var context = new NeoCLR.Metadata.Experimental.Introspection.MetadataLoadContext([snapshot]);
        if (context.Resolve(snapshot.Identity).GetTypes().Where(t => t.Namespace == "Example").Any(t => !t.IsSealed || t.IsAbstract))
            throw new Exception("native final flag lost");
        Reject(() => snapshot.MainModule.Types.Single(t => t.Name == "Built").SetSealedClass());
        Reject(() => graph.AddValueType("Example", "Value").SetSealedClass());
        Reject(() => graph.AddClosedClass("Example", "Family").SetSealedClass());
        graph.AddClass("Example", "InvalidChild", built);
        Reject(() => graph.Write()); Reject(() => graph.WriteNativeAssembly());
    }
    private static void Reject(Action action)
    {
        try { action(); } catch (Exception e) when (e is InvalidOperationException or InvalidDataException or ArgumentException) { return; }
        throw new Exception("invalid final declaration accepted");
    }
}
