using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class StringRepresentationChecks
{
    internal static void Run()
    {
        var graph = new AssemblyBuilder(new("StringImplementation", new Version(1, 0, 0, 0)), new("System.Runtime", new Version(10, 0, 0, 0)));
        var text = graph.AddClass("System", "String");
        text.SetNativePrimitive(PrimitiveType.String);
        var identity = text.AddInstanceMethod("Identity", new(PrimitiveType.String, []));
        identity.GetILGenerator().LoadArgument(0);
        identity.GetILGenerator().Return();
        var contract = graph.AddInterface("Example", "TextCount");
        var getter = contract.AddInterfaceMethod("get_Count", new(PrimitiveType.Int32, []));
        text.AddInterfaceImplementation(contract);
        var implementation = text.AddInstanceMethod("TextCount.get_Count", new(PrimitiveType.Int32, []), MethodVisibility.Private);
        implementation.AddExplicitInterfaceImplementation(contract, "get_Count");
        implementation.GetILGenerator().LoadConstant(42); implementation.GetILGenerator().Return();
        var main = graph.AddFunction("Main", new(PrimitiveType.Int32, []));
        graph.EntryPoint = main;
        var il = main.GetILGenerator();
        il.Emit(OpCode.Ldstr, "grapheme"); il.Call(identity); il.IsNull();
        var fail = il.DefineLabel(); il.Emit(OpCode.Brtrue, fail); il.Emit(OpCode.Ldstr, "count"); il.CastReference(contract); il.CallVirtual(getter); il.Return();
        il.MarkLabel(fail); il.LoadConstant(1); il.Return();
        var bytes = RuntimeAssemblyContainer.WriteBinary(graph);
        var loaded = AssemblyDefinition.ReadNativeAssembly(bytes).MainModule.Types.Single(t => t.Name == "String");
        if (loaded.IsValueType || loaded.NativePrimitive != PrimitiveType.String) throw new Exception("String reference representation lost");
        if (Environment.GetEnvironmentVariable("NEOCLR_STRING_ARTIFACT") is { } path)
        {
            File.WriteAllBytes(path, bytes);
            File.WriteAllText(path + ".seed", "{\"format\":5,\"name\":\"System\",\"functions\":[]}");
        }
        graph.EntryPoint = null;
        var library = RuntimeAssemblyContainer.WriteBinary(graph);
        var consumer = new AssemblyBuilder(new("StringConsumer", new Version(1, 0, 0, 0)), graph.CoreLibrary);
        var reference = consumer.CreateTypeReference(graph.Identity, graph.CoreLibrary,
            Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(library)), "System", "String");
        consumer.SetNativePrimitive(reference, PrimitiveType.String);
        var imported = consumer.CreateMethodReference(reference, "Identity", new(PrimitiveType.String, []), nativePrimitive: PrimitiveType.String);
        var entry = consumer.AddFunction("Main", new(PrimitiveType.Int32, [])); consumer.EntryPoint = entry;
        var body = entry.GetILGenerator();
        body.Emit(OpCode.Ldstr, "external"); body.Call(imported); body.IsNull();
        var missing = body.DefineLabel(); body.Emit(OpCode.Brtrue, missing); body.LoadConstant(42); body.Return();
        body.MarkLabel(missing); body.LoadConstant(1); body.Return();
        var application = RuntimeAssemblyContainer.WriteBinary(consumer);
        _ = AssemblyDefinition.ReadNativeAssembly(application);
        if (Environment.GetEnvironmentVariable("NEOCLR_STRING_ARTIFACT") is { } externalPath)
        {
            File.WriteAllBytes(externalPath + ".library", library);
            File.WriteAllBytes(externalPath + ".consumer", application);
        }
        var manual = new TypeDefinition("System", "String", 1,
            consumer.Definition.MainModule.ImportReference(consumer.CoreLibrary, "System", "Object"));
        manual.SetNativePrimitive(PrimitiveType.String);
        if (manual.NativePrimitive != text.NativePrimitive || manual.IsValueType) throw new Exception("definition/builder parity");
        try { graph.AddValueType("System", "String").SetNativePrimitive(PrimitiveType.String); }
        catch (ArgumentException) { return; }
        throw new Exception("String value representation accepted");
    }
}
