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
        var service = booleans.AddFunction("RuntimeProvided", new(PrimitiveType.Void, []));
        service.SetInternalCall();
        Reject(() => service.SetNativeImport(new("x", "y")));
        var booleanSnapshot = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(booleans));
        var booleanContext = new MetadataLoadContext([booleanSnapshot]);
        var views = booleanSnapshot.MainModule.Methods.Select(booleanContext.Resolve).ToArray();
        var descriptor = views.Single(m => m.Name == "Test").NativeImport;
        if (descriptor is not { CallingConvention: "System", ReturnMarshalling: "Bool32" } ||
            descriptor.ParameterMarshalling[0] != "Bool8" || views.Single(m => m.Name == "Ordinary").NativeImport is not null)
            throw new Exception("introspection changed native ABI or ordinary method classification");
        var serviceView = views.Single(m => m.Name == "RuntimeProvided");
        if (serviceView.GetMethodImplementationFlags() != System.Reflection.MethodImplAttributes.InternalCall ||
            serviceView.NativeImport is not null ||
            views.Where(m => m.Name != "RuntimeProvided").Any(m => m.GetMethodImplementationFlags() != 0))
            throw new Exception("implementation flags conflated runtime services, foreign imports and managed bodies");
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
        var callbacks = Graph();
        var callbackShape = SignatureType.Function(new(PrimitiveType.Int32, [PrimitiveType.Int32]));
        var callbackDescriptors = new Dictionary<int, NativeCallbackDefinition> { [0] = new("System") };
        callbacks.AddFunction("Apply", new(PrimitiveType.Int32, [callbackShape, PrimitiveType.Int32]))
            .SetNativeImport(new("fixture", "apply", callbackParameters: callbackDescriptors));
        callbackDescriptors.Clear();
        callbacks.AddFunction("Notify", new(PrimitiveType.Void, [SignatureType.Function(new(PrimitiveType.Void, [PrimitiveType.Int32]))]))
            .SetNativeImport(new("fixture", "notify", callbackParameters: new Dictionary<int, NativeCallbackDefinition> { [0] = new() }));
        callbacks.AddFunction("GetCallback", new(callbackShape, []))
            .SetNativeImport(new("fixture", "get_callback", functionReturn: new("System")));
        var voidShape = SignatureType.Function(new(PrimitiveType.Void, [PrimitiveType.Int32]));
        callbacks.AddFunction("GetNotification", new(voidShape, []))
            .SetNativeImport(new("fixture", "get_notification", functionReturn: new()));
        var callbackSnapshot = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(callbacks));
        var apply = callbackSnapshot.MainModule.Methods.Single(m => m.Name == "Apply");
        if (apply.NativeImport?.CallbackParameters[0].CallingConvention != "System")
            throw new Exception("callback ABI lost or not defensively copied");
        var notify = callbackSnapshot.MainModule.Methods.Single(m => m.Name == "Notify");
        if (!notify.TryGetSignature(out var notifySignature) ||
            notifySignature!.ParameterTypes[0].FunctionSignature is not { NoResult: true } ||
            notify.NativeImport?.CallbackParameters[0].CallingConvention != "Cdecl")
            throw new Exception("no-result callback ABI lost");
        Reject(() => Graph().AddFunction("F", new(PrimitiveType.Int32, [callbackShape])).SetNativeImport(new("x", "y")));
        foreach (var badType in new SignatureType[] {
            PrimitiveType.Int32,
            SignatureType.Function(new(PrimitiveType.String, [])),
            SignatureType.Function(new(PrimitiveType.Boolean, [])) })
            Reject(() => Graph().AddFunction("F", new(PrimitiveType.Void, [badType])).SetNativeImport(
                new("x", "y", callbackParameters: new Dictionary<int, NativeCallbackDefinition> { [0] = new() })));
        Reject(() => Graph().AddFunction("F", new(PrimitiveType.Void, [callbackShape])).SetNativeImport(
            new("x", "y", callbackParameters: new Dictionary<int, NativeCallbackDefinition> { [1] = new() })));
        Reject(() => Graph().AddFunction("F", new(PrimitiveType.Void, [callbackShape])).SetNativeImport(
            new("x", "y", parameterMarshalling: new Dictionary<int, string> { [0] = "Bool8" },
                callbackParameters: new Dictionary<int, NativeCallbackDefinition> { [0] = new() })));
        var getCallback = callbackSnapshot.MainModule.Methods.Single(m => m.Name == "GetCallback");
        if (getCallback.NativeImport?.FunctionReturn?.CallingConvention != "System" ||
            !getCallback.TryGetSignature(out var returnedSignature) || returnedSignature!.ReturnType != callbackShape)
            throw new Exception("returned function ABI lost");
        var getNotification = callbackSnapshot.MainModule.Methods.Single(m => m.Name == "GetNotification");
        if (getNotification.NativeImport?.FunctionReturn?.CallingConvention != "Cdecl" ||
            !getNotification.TryGetSignature(out var returnedVoid) || returnedVoid!.ReturnType != voidShape)
            throw new Exception("returned no-result function ABI lost");
        Reject(() => Graph().AddFunction("F", new(callbackShape, [])).SetNativeImport(new("x", "y")));
        Reject(() => Graph().AddFunction("F", new(PrimitiveType.Int32, [])).SetNativeImport(new("x", "y", functionReturn: new())));
        Reject(() => Graph().AddFunction("F", new(callbackShape, [])).SetNativeImport(new("x", "y", returnMarshalling: "Bool8", functionReturn: new())));
        Reject(() => Graph().AddFunction("F", new(SignatureType.Function(new(PrimitiveType.String, [])), [])).SetNativeImport(new("x", "y", functionReturn: new())));
        Reject(() => new NativeCallbackDefinition("Stdcall"));
        Console.WriteLine("PASS native import authoring, reread and rejection checks");
    }
    private static void Reject(Action action)
    {
        try { action(); }
        catch (Exception error) when (error is InvalidDataException or InvalidOperationException or ArgumentException) { return; }
        throw new Exception("invalid native import accepted");
    }
}
