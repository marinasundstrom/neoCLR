using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

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
        var context = new MetadataLoadContext([definition]);
        if (!ReferenceEquals(context.Resolve(method).NativeImport, method.NativeImport))
            throw new Exception("introspection lost immutable native descriptor");
        var booleans = Graph();
        var overrides = new Dictionary<int, string> { [0] = "Bool8" };
        booleans.AddFunction("Test", new(PrimitiveType.Boolean, [PrimitiveType.Boolean]))
            .SetNativeImport(new("missing-library", "test", callingConvention: "System",
                parameterMarshalling: overrides, returnMarshalling: "Bool32"));
        overrides[0] = "Bool32";
        var ordinary = booleans.AddFunction("Ordinary", new(PrimitiveType.Int32, []));
        ordinary.LoadConstant(42); ordinary.Return();
        var booleanSnapshot = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(booleans));
        var booleanContext = new MetadataLoadContext([booleanSnapshot]);
        var views = booleanSnapshot.MainModule.Methods.Select(booleanContext.Resolve).ToArray();
        var descriptor = views.Single(m => m.Name == "Test").NativeImport;
        if (descriptor is not { CallingConvention: "System", ReturnMarshalling: "Bool32" } ||
            descriptor.ParameterMarshalling[0] != "Bool8" || views.Single(m => m.Name == "Ordinary").NativeImport is not null)
            throw new Exception("introspection changed native ABI or ordinary method classification");
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
