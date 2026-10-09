using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class PrimitiveReferenceChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new(1, 0, 0, 0));
        var graph = new AssemblyBuilder(new("References", new(1, 0, 0, 0)), core);
        var text = graph.AddClass("System", "String");
        text.SetNativePrimitiveReference(PrimitiveType.String);
        var image = RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), core);
        var loaded = AssemblyDefinition.ReadNativeAssembly(image);
        var declaration = loaded.MainModule.Types.Single(t => t.Name == "String");
        if (!declaration.IsNativePrimitiveReference || declaration.NativePrimitive != PrimitiveType.String)
            throw new Exception("primitive reference flag was lost by the definition reader");
        Reject(() => graph.Write());
        var method = text.AddMethod("Forbidden", new(PrimitiveType.Int32, []));
        method.GetILGenerator().LoadConstant(0);
        method.GetILGenerator().Return();
        Reject(() => graph.WriteNativeAssembly());
        Console.WriteLine("PASS primitive reference round-trip and member/CLI rejection");
    }
    private static void Reject(Action action)
    {
        try { action(); }
        catch (Exception error) when (error is ArgumentException or InvalidDataException) { return; }
        throw new Exception("Invalid primitive reference admitted");
    }
}
