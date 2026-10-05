using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class PrimitiveRepresentationChecks
{
    internal static void Run()
    {
        var graph = new AssemblyBuilder(new("PrimitiveImplementations", new Version(1, 0, 0, 0)), new("System.Runtime", new Version(10, 0, 0, 0)));
        var boolean = graph.AddValueType("System", "Boolean");
        boolean.SetNativePrimitive(PrimitiveType.Boolean);
        var boolIdentity = boolean.AddInstanceMethod("Identity", new(PrimitiveType.Boolean, []));
        boolIdentity.GetILGenerator().LoadArgument(0); boolIdentity.GetILGenerator().LoadObject(PrimitiveType.Boolean); boolIdentity.GetILGenerator().Return();
        var contract = graph.AddInterface("Example", "Scalar");
        var zeroContract = contract.AddInterfaceMethod("get_Zero", new(SignatureType.Self, []), true);
        contract.AddProperty("Zero", SignatureType.Self, zeroContract, null);
        contract.AddInterfaceMethod("Identity", new(SignatureType.Self, []));
        var primitive = graph.AddValueType("System", "Double");
        primitive.SetNativePrimitive(PrimitiveType.Double);
        primitive.AddInterfaceImplementation(contract);
        var zero = primitive.AddMethod("get_Zero", new(PrimitiveType.Double, []));
        zero.GetILGenerator().LoadConstant(0.0); zero.GetILGenerator().Return();
        primitive.AddProperty("Zero", PrimitiveType.Double, zero, null);
        var identity = primitive.AddInstanceMethod("Identity", new(PrimitiveType.Double, []));
        var body = identity.GetILGenerator();
        body.LoadArgument(0); body.LoadObject(PrimitiveType.Double); body.Return();
        var increment = primitive.AddInstanceMethod("Increment", new(PrimitiveType.Void, []));
        body = increment.GetILGenerator();
        body.LoadArgument(0); body.LoadArgument(0); body.LoadObject(PrimitiveType.Double);
        body.LoadConstant(1.0); body.Add(); body.StoreObject(PrimitiveType.Double); body.Return();
        // Definitions authored without builders use exactly the same designation and validation.
        var single = new TypeDefinition("System", "Single", 0x109,
            graph.Definition.MainModule.ImportReference(graph.CoreLibrary, "System", "ValueType"));
        single.SetNativePrimitive(PrimitiveType.Single);
        graph.Definition.MainModule.Types.Add(single);
        var ordinary = graph.AddValueType("System", "Int32");
        if (ordinary.NativePrimitive is not null || ((SignatureType)ordinary).Primitive is not null)
            throw new Exception("ordinary name silently claimed primitive storage");
        var main = graph.AddFunction("Main", new(PrimitiveType.Int32, []));
        body = main.GetILGenerator();
        var local = body.DeclareLocal(PrimitiveType.Double);
        var fail = body.DefineLabel();
        body.Call(zero); body.StoreLocal(local);
        body.LoadLocalAddress(local); body.Call(increment);
        body.LoadLocalAddress(local); body.Call(identity);
        body.LoadConstant(1.0); body.Emit(OpCode.Ceq); body.Emit(OpCode.Brfalse, fail);
        body.LoadConstant(42); body.Return();
        body.MarkLabel(fail); body.LoadConstant(1); body.Return();
        graph.EntryPoint = main;
        var json = graph.WriteNativeAssembly();
        // Readers retain the previous encoded member spelling, including accessor links.
        var legacy = System.Text.Encoding.UTF8.GetString(json);
        foreach (var method in primitive.Methods)
            legacy = legacy.Replace("System.Double." + method.Name,
                "System.Double.M_" + Convert.ToHexString(System.Text.Encoding.UTF8.GetBytes(method.Name)));
        _ = NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(legacy));
        graph.EntryPoint = null;
        var native = RuntimeAssemblyContainer.WriteBinary(graph);
        var snapshot = AssemblyDefinition.ReadNativeAssembly(native);
        if (snapshot.MainModule.Types.Single(t => t.Name == "Boolean").NativePrimitive != PrimitiveType.Boolean ||
            snapshot.MainModule.Types.Single(t => t.Name == "Double").NativePrimitive != PrimitiveType.Double ||
            snapshot.MainModule.Types.Single(t => t.Name == "Single").NativePrimitive != PrimitiveType.Single)
            throw new Exception("native scalar declaration lost");
        var view = new MetadataLoadContext([snapshot]).Resolve(snapshot.Identity).GetTypes().Single(t => t.Name == "Double");
        if (view.NativePrimitive != PrimitiveType.Double || view.GetProperties().Single().GetMethod is null)
            throw new Exception("primitive introspection facts lost");
        var consumer = new AssemblyBuilder(new("PrimitiveConsumer", new Version(1, 0, 0, 0)), graph.CoreLibrary);
        var entry = consumer.AddFunction("Main", new(PrimitiveType.Int32, []));
        var consumerIL = entry.GetILGenerator();
        var storage = consumerIL.DeclareLocal(PrimitiveType.Double);
        consumerIL.LoadConstant(41.0); consumerIL.StoreLocal(storage);
        var declaration = snapshot.MainModule.Types.Single(t => t.Name == "Double");
        consumerIL.LoadLocalAddress(storage);
        consumerIL.Call(consumer.ImportReference(declaration.Methods.Single(m => m.Name == "Increment"), graph.CoreLibrary));
        consumerIL.LoadLocalAddress(storage);
        consumerIL.Call(consumer.ImportReference(declaration.Methods.Single(m => m.Name == "Identity"), graph.CoreLibrary));
        consumerIL.Emit(OpCode.Conv_I4); consumerIL.Return(); consumer.EntryPoint = entry;
        var authored = new AssemblyBuilder(new("AuthoredPrimitiveConsumer", new Version(1, 0, 0, 0)), graph.CoreLibrary);
        var primitiveReference = authored.CreateValueTypeReference(snapshot.Identity, graph.CoreLibrary,
            Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(native)), "System", "Double");
        var importedIdentity = authored.CreateMethodReference(primitiveReference, "Identity", new(PrimitiveType.Double, []), nativePrimitive: PrimitiveType.Double);
        Reject<ArgumentException>(() => authored.CreateMethodReference(primitiveReference, "Identity", new(PrimitiveType.Double, []), nativePrimitive: PrimitiveType.Single));
        Reject<InvalidDataException>(() => authored.CreateMethodReference(primitiveReference, "Identity", new(PrimitiveType.Double, [])));
        Reject<InvalidDataException>(() => authored.CreateMethodReference(primitiveReference, "Other", new(PrimitiveType.Double, [])));
        Reject<InvalidDataException>(() => authored.Write());
        var authoredEntry = authored.AddFunction("Main", new(PrimitiveType.Int32, []));
        var authoredIL = authoredEntry.GetILGenerator();
        var scalarStorage = authoredIL.DeclareLocal(PrimitiveType.Double);
        authoredIL.LoadConstant(42.0); authoredIL.StoreLocal(scalarStorage); authoredIL.LoadLocalAddress(scalarStorage);
        authoredIL.Call(importedIdentity); authoredIL.Emit(OpCode.Conv_I4); authoredIL.Return(); authored.EntryPoint = authoredEntry;
        var authoredNative = RuntimeAssemblyContainer.WriteBinary(authored);
        if (Environment.GetEnvironmentVariable("NEOCLR_PRIMITIVE_CONSUMER") is { } authoredPath) File.WriteAllBytes(authoredPath + ".authored.neox", authoredNative);
        var consumerNative = RuntimeAssemblyContainer.WriteBinary(consumer);
        Reject<InvalidDataException>(() => consumer.Write());
        if (Environment.GetEnvironmentVariable("NEOCLR_PRIMITIVE_CONSUMER") is { } consumerPath) File.WriteAllBytes(consumerPath, consumerNative);
        Reject<InvalidOperationException>(() => snapshot.MainModule.Types.Single(t => t.Name == "Double").SetNativePrimitive(PrimitiveType.Double));
        Reject<InvalidDataException>(() => graph.Write());
        _ = NativeAssemblyDefinition.ReadAssembly(json).CreateReferenceAssembly(graph.CoreLibrary);
        Reject<ArgumentException>(() => primitive.SetNativePrimitive(PrimitiveType.Single));
        Reject<ArgumentException>(() => ordinary.SetNativePrimitive(PrimitiveType.Void));
        Reject<ArgumentException>(() => graph.AddValueType("Other", "Double").SetNativePrimitive(PrimitiveType.Double));
        foreach (var edit in new Action<JsonNode>[] {
            root => root["types"]!.AsArray().Single(t => t!["name"]!.GetValue<string>() == "System.Double")!["name"] = "Other.Double",
            root => root["types"]!.AsArray().Single(t => t!["name"]!.GetValue<string>() == "System.Double")!["representation"] = "Record",
            root => root["types"]!.AsArray().Single(t => t!["name"]!.GetValue<string>() == "System.Double")!["fields"] = new JsonArray(JsonNode.Parse("{\"name\":\"m_value\",\"ty\":\"Double\",\"visibility\":\"private\"}")),
            root => root["functions"]!.AsArray().Single(f => f!["name"]!.GetValue<string>() == "System.Double.Identity")!["owner"] = "Single"
        })
        {
            var root = JsonNode.Parse(json)!; edit(root);
            Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(root.ToJsonString())));
        }
        primitive.AddField("m_value", PrimitiveType.Double, FieldVisibility.Private);
        Reject<ArgumentException>(() => primitive.SetNativePrimitive(PrimitiveType.Double));
        Reject<InvalidDataException>(() => graph.WriteNativeAssembly());
        if (Environment.GetEnvironmentVariable("NEOCLR_PRIMITIVE_ARTIFACT") is { } path) File.WriteAllBytes(path, native);
    }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
