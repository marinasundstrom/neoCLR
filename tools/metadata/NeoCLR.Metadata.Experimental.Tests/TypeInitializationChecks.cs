using System.Runtime.Loader;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class TypeInitializationChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        AssemblyBuilder Graph(string name) => new(new(name, new Version(1, 0, 0, 0)), core);
        var rootGraph = Graph("RootTiming");
        rootGraph.AddNativeObjectRoot().SetBeforeFieldInit();
        _ = rootGraph.WriteNativeAssembly();
        var graph = Graph("Initializers");
        var owner = graph.AddClass("Example", "State");
        var field = owner.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public, isReadOnly: true, isStatic: true);
        var initializer = owner.AddTypeInitializer();
        initializer.LoadField(field); initializer.LoadConstant(40); initializer.GetILGenerator().Add(); initializer.StoreField(field); initializer.Return();
        var generic = graph.AddGenericClass("Example", "Cache", ["T"]);
        generic.SetBeforeFieldInit();
        var cached = generic.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public, isReadOnly: true, isStatic: true);
        var genericInitializer = generic.AddTypeInitializer();
        genericInitializer.LoadConstant(1);
        genericInitializer.StoreField(cached.MakeConstructedReference(SignatureType.TypeParameter(0)));
        genericInitializer.Return();
        var marker = graph.AddInterface("Example", "Marker");
        marker.AddTypeInitializer().Return();
        owner.AddInterfaceImplementation(marker);
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        main.LoadField(field);
        main.LoadField(cached.MakeConstructedReference(PrimitiveType.Int32)); main.GetILGenerator().Add();
        main.LoadField(cached.MakeConstructedReference(PrimitiveType.String)); main.GetILGenerator().Add(); main.Return();
        var context = new AssemblyLoadContext("type-initializers", true);
        try
        {
            var assembly = context.LoadFromStream(new MemoryStream(graph.Write()));
            if (!Equals(42, assembly.EntryPoint!.Invoke(null, null)) || !Equals(42, assembly.EntryPoint.Invoke(null, null)))
                throw new Exception("CLR readonly initialization failed");
        }
        finally { context.Unload(); }
        var native = graph.WriteNativeAssembly();
        foreach (var snapshot in new[] {
            AssemblyDefinition.ReadAssembly(graph.Write(), false),
            AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(graph)),
            AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(native).CreateReferenceAssembly(core), false) })
        {
            var state = snapshot.MainModule.Types.Single(t => t.Name == "State");
            var cache = snapshot.MainModule.Types.Single(t => t.Name == "Cache`1");
            if (state.IsBeforeFieldInit || !cache.IsBeforeFieldInit) throw new Exception("initialization timing lost");
            foreach (var type in new[] { state, cache })
            {
                var method = type.Methods.Single(m => m.Name == ".cctor");
                if ((method.Attributes & 0x1817) != 0x1811 || (type.Fields.Single().Attributes & 0x30) != 0x30)
                    throw new Exception("initializer/readonly metadata lost");
            }
        }
        Reject<ArgumentException>(() => owner.AddTypeInitializer());
        Reject<ArgumentException>(() => new MethodDefinition(".cctor", 0x1816, new(PrimitiveType.Void, Array.Empty<SignatureType>())));
        Reject<ArgumentException>(() => new MethodDefinition(".cctor", 0x1811, new(PrimitiveType.Void, new[] { PrimitiveType.Int32 })));
        var badStore = Graph("BadStore");
        var badOwner = badStore.AddClass("Example", "State");
        var readOnly = badOwner.AddField("Value", PrimitiveType.Int32, isReadOnly: true, isStatic: true);
        var writer = badOwner.AddMethod("Write", new MethodSignature(PrimitiveType.Void, Array.Empty<SignatureType>()));
        writer.LoadConstant(1); writer.StoreField(readOnly); writer.Return();
        Reject<InvalidDataException>(() => badStore.WriteNativeAssembly());
        var wrongGeneric = Graph("WrongGeneric");
        var wrongOwner = wrongGeneric.AddGenericClass("Example", "Cache", ["T"]);
        var wrongField = wrongOwner.AddField("Value", PrimitiveType.Int32, isReadOnly: true, isStatic: true);
        var wrongInitializer = wrongOwner.AddTypeInitializer();
        wrongInitializer.LoadConstant(1); wrongInitializer.StoreField(wrongField.MakeConstructedReference(PrimitiveType.Int32)); wrongInitializer.Return();
        Reject<InvalidDataException>(() => wrongGeneric.WriteNativeAssembly());
        var calls = Graph("DirectCall");
        var callOwner = calls.AddClass("Example", "State");
        var cctor = callOwner.AddTypeInitializer(); cctor.Return();
        calls.EntryPoint = cctor;
        Reject<InvalidDataException>(() => calls.WriteNativeAssembly());
        calls.EntryPoint = null;
        var caller = callOwner.AddMethod("Call", new MethodSignature(PrimitiveType.Void, Array.Empty<SignatureType>()));
        caller.Call(cctor); caller.Return();
        Reject<InvalidDataException>(() => calls.WriteNativeAssembly());
        var malformed = JsonNode.Parse(native)!.AsObject();
        var functions = malformed["functions"]!.AsArray();
        functions.First(f => f!["origin"]!["name"]!.GetValue<string>() == ".cctor")!["visibility"] = "public";
        Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(malformed.ToJsonString())));
        Console.WriteLine("PASS type initializer execution, CLI/native/reference timing, readonly authority and malformed declarations");
    }

    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); }
        catch (T) { return; }
        throw new Exception($"expected {typeof(T).Name}");
    }
}
