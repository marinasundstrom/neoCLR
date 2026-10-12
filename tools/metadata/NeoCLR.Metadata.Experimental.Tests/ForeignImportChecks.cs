using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ForeignImportChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        AssemblyBuilder Graph() => new(new("Foreign", new(1, 0, 0, 0)), core);
        var graph = Graph();
        var imported = graph.AddFunction("Read", new(PrimitiveType.Int32, [PrimitiveType.String]));
        imported.SetNativeImport(new("missing-library", "read", "Utf8"));
        var bytes = RuntimeAssemblyContainer.WriteBinary(graph);
        var definition = AssemblyDefinition.ReadNativeAssembly(bytes);
        var method = definition.MainModule.Methods.Single();
        if (method.NativeImport is not { Library: "missing-library", EntryPoint: "read", StringMarshalling: "Utf8" })
            throw new Exception("foreign import descriptor lost");
        Reject(() => method.SetNativeImport(new("x", "y")));
        Reject(() => imported.SetInternalCall());
        Reject(() => graph.Write());
        graph.EntryPoint = imported;
        Reject(() => graph.WriteNativeAssembly());
        graph.EntryPoint = null;
        imported.LoadConstant(42); imported.Return();
        Reject(() => graph.WriteNativeAssembly());
        foreach (var signature in new[] { new MethodSignature(PrimitiveType.String, []), new(PrimitiveType.Boolean, []), new(PrimitiveType.Void, [PrimitiveType.String]) })
            Reject(() => Graph().AddFunction("F", signature).SetNativeImport(new("x", "y")));
        Reject(() => new NativeImportDefinition("", "f"));
        Reject(() => new NativeImportDefinition("x", "f\0"));
        Reject(() => new NativeImportDefinition("x", "f", "Ansi"));
        Console.WriteLine("PASS native import authoring, reread and rejection checks");
    }
    private static void Reject(Action action)
    {
        try { action(); }
        catch (Exception error) when (error is InvalidDataException or InvalidOperationException or ArgumentException) { return; }
        throw new Exception("invalid native import accepted");
    }
}
