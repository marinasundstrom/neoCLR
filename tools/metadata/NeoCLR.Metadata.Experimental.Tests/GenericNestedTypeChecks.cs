using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class GenericNestedTypeChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("System.Private.CoreLib", typeof(object).Assembly.GetName().Version!, "", "7cec85d7bea7798e");
        var graph = new AssemblyBuilder(new("GenericNestedTypes", new Version(1, 0, 0, 0)), core);
        var owner = graph.AddGenericClass("Example", "Owner", ["T"]);
        var t = SignatureType.TypeParameter(0);
        var value = owner.AddField("value", t, FieldVisibility.Private);
        var ctor = owner.AddConstructor(new MethodSignature(PrimitiveType.Void, [t]));
        ctor.LoadArgument(0); ctor.LoadArgument(1); ctor.StoreField(value); ctor.Return();
        var nested = owner.AddNestedGenericClass("Frame", ["T", "U"]);
        var get = nested.AddMethod("Read", new MethodSignature(t, [owner.MakeGenericInstance(t)]));
        get.LoadArgument(0); get.LoadField(value.MakeConstructedReference(t)); get.Return();
        nested.AddField("Other", SignatureType.TypeParameter(1), FieldVisibility.Public);
        nested.AddNestedGenericValueType("Deep", ["T", "U", "V"]);
        var main = graph.AddType("", "Program").AddMethod("Main");
        main.LoadConstant(42); main.NewObject(ctor.MakeConstructedReference([PrimitiveType.Int32]));
        main.Call(get.MakeConstructedReference([PrimitiveType.Int32, PrimitiveType.String])); main.Return();
        graph.EntryPoint = main;
        if (!Equals(Assembly.Load(graph.Write()).EntryPoint!.Invoke(null, null), 42))
            throw new Exception("nested generic private access on CLR");

        var bytes = RuntimeAssemblyContainer.WriteBinary(graph);
        var snapshot = AssemblyDefinition.ReadNativeAssembly(bytes);
        var outer = snapshot.MainModule.Types.Single(t => t.Name == "Owner`1");
        var frame = outer.NestedTypes.Single();
        if (frame.GenericArity != 2 || frame.NestedTypes.Single().GenericArity != 3 ||
            !ReferenceEquals(frame.DeclaringType, outer) || !snapshot.Write().SequenceEqual(bytes))
            throw new Exception("native nested generic roundtrip");
        var projection = AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(core), false);
        if (projection.MainModule.Types.Single(t => t.Name == "Owner`1").NestedTypes.Single().GenericArity != 2)
            throw new Exception("nested generic reference projection");

        var consumer = new AssemblyBuilder(new("NestedGenericConsumer", new Version(1, 0, 0, 0)), core);
        var imported = consumer.ImportReference(frame, core);
        if (imported.DeclaringType?.GenericArity != 1 || imported.GenericArity != 2)
            throw new Exception("generic enclosing import identity");
        consumer.AddClass("", "Holder").AddField("Frame", imported.MakeGenericInstance(PrimitiveType.Int32, PrimitiveType.String), FieldVisibility.Public);
        var consumerBytes = RuntimeAssemblyContainer.WriteBinary(consumer);
        var reread = AssemblyDefinition.ReadNativeAssembly(consumerBytes);
        if (!reread.Write().SequenceEqual(consumerBytes))
            throw new Exception("external nested generic roundtrip");

        var bridge = new AssemblyBuilder(new("NestedGenericBridge", new Version(1, 0, 0, 0)), core);
        var cli = AssemblyDefinition.ReadAssembly(graph.Write(), false);
        var inventory = NativeLibraryDefinition.ReadAssembly(NativeModuleContainer.WriteLibraryBinary(System.Text.Encoding.UTF8.GetBytes("""
            {"format":5,"name":"Translated","entry":"","types":[
              {"name":"Example.Owner","generic_parameters":["T"],"is_reference_type":true},
              {"name":"Example.Owner.Frame","generic_parameters":["T","U"],"is_reference_type":true}],
             "functions":[]}
            """)));
        bridge.BindNativeLibrary(cli, inventory, core);
        var bridged = bridge.ImportReference(cli.MainModule.Types.Single(t => t.Name == "Frame`2"), core);
        bridge.AddClass("", "Holder").AddField("Value", bridged.MakeGenericInstance(PrimitiveType.Int32, PrimitiveType.String), FieldVisibility.Public);
        var bridgeBytes = RuntimeAssemblyContainer.WriteBinary(bridge);
        if (!AssemblyDefinition.ReadNativeAssembly(bridgeBytes).Write().SequenceEqual(bridgeBytes))
            throw new Exception("translated nested generic scope roundtrip");

        var malformed = System.Text.Json.Nodes.JsonNode.Parse(bridge.WriteNativeAssembly())!;
        var aliases = malformed["assemblies"]![0]!["native_type_bindings"]!.AsArray();
        var nestedAlias = aliases.Single(row => row!["declaring"] is not null)!;
        nestedAlias["declaring_arity"] = 33;
        try
        {
            NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(malformed.ToJsonString()));
            throw new Exception("invalid enclosing arity admitted");
        }
        catch (System.IO.InvalidDataException) { }

        // Enclosing parameters never leak implicitly into the nested parameter scope.
        var plain = owner.AddNestedClass("Plain");
        try { plain.AddField("Invalid", t, FieldVisibility.Public); throw new Exception("implicit enclosing generic scope accepted"); }
        catch (ArgumentException) { }
    }
}
