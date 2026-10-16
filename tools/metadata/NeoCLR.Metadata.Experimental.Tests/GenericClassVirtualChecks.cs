using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class GenericClassVirtualChecks
{
    internal static void Run() => RunCore(false);
    internal static void RunAbstract() => RunCore(true);

    private static void RunCore(bool abstractBase)
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("GenericClassVirtual", new(1, 0, 0, 0)), core);
        var t = SignatureType.TypeParameter(0);
        var second = SignatureType.TypeParameter(1);
        var parent = graph.AddGenericClass("Example", "Base", ["T"]);
        if (abstractBase) parent.SetAbstractClass();
        var abstractSlot = abstractBase ? parent.AddAbstractMethod("Get", new(t, [])) : null;
        var parentCtor = parent.AddConstructor([]); parentCtor.Return();
        var slot = parent.AddVirtualMethod("Echo", new(t, [t])); slot.LoadArgument(1); slot.Return();
        var derived = graph.AddGenericClass("Example", "Derived", ["Unused", "T"], parent.MakeGenericInstance(second));
        var stored = derived.AddField("stored", second, FieldVisibility.Private);
        var ctor = derived.AddConstructor(new MethodSignature(PrimitiveType.Void, [second]));
        ctor.LoadArgument(0); ctor.Call(parentCtor.MakeConstructedReference([second]));
        ctor.LoadArgument(0); ctor.LoadArgument(1); ctor.StoreField(stored); ctor.Return();
        var implementation = derived.AddOverride("Echo", new(second, [second]));
        implementation.LoadArgument(0); implementation.LoadArgument(0); implementation.LoadField(stored);
        implementation.Call(slot.MakeConstructedReference([second])); implementation.Return();
        if (abstractSlot is not null)
        {
            var get = derived.AddOverride("Get", new(second, []));
            get.LoadArgument(0); get.LoadField(stored); get.Return();
        }
        var box = graph.AddClass("Example", "Box");
        var number = box.AddField("Number", PrimitiveType.Int32, FieldVisibility.Public);
        var boxCtor = box.AddConstructor([PrimitiveType.Int32]);
        boxCtor.LoadArgument(0); boxCtor.LoadArgument(1); boxCtor.StoreField(number); boxCtor.Return();
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        var scalar = main.DeclareLocal(parent.MakeGenericInstance(PrimitiveType.Int32));
        main.LoadConstant(40); main.NewObject(ctor.MakeConstructedReference([PrimitiveType.Boolean, PrimitiveType.Int32])); main.StoreLocal(scalar);
        main.LoadLocal(scalar);
        if (abstractSlot is not null) main.CallVirtual(abstractSlot.MakeConstructedReference([PrimitiveType.Int32]));
        else { main.LoadConstant(99); main.CallVirtual(slot.MakeConstructedReference([PrimitiveType.Int32])); }
        main.LoadConstant(2); main.NewObject(boxCtor); main.NewObject(ctor.MakeConstructedReference([PrimitiveType.Boolean, box]));
        main.LoadConstant(99); main.NewObject(boxCtor); main.CallVirtual(slot.MakeConstructedReference([box])); main.LoadField(number); main.Add();
        main.LoadLocal(scalar); main.LoadConstant(7); main.Call(slot.MakeConstructedReference([PrimitiveType.Int32])); main.Add();
        main.LoadConstant(7); main.Subtract(); main.Return();
        var native = RuntimeAssemblyContainer.WriteBinary(graph);
        var loaded = System.Reflection.Assembly.Load(graph.Write());
        if (!Equals(loaded.EntryPoint!.Invoke(null, null), 42)) throw new Exception("generic virtual dispatch or direct base call failed");
        var snapshot = AssemblyDefinition.ReadNativeAssembly(native);
        if (snapshot.MainModule.Types.Single(t => t.Name == "Derived`2").BaseTypeSignature!.ReferencedGenericInstance!.TypeArguments.Single().TypeParameterIndex != 1)
            throw new Exception("generic base argument lost");
        if (abstractBase)
        {
            var consumer = new AssemblyBuilder(new("AbstractConsumer", new(1, 0, 0, 0)), core);
            var imported = consumer.CreateTypeReference(graph.Identity, core,
                Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(native)), "Example", "Base`1", 1);
            var importedSlot = consumer.CreateMethodReference(imported, "Get", new(t, []), isVirtual: true, isAbstract: true);
            var call = consumer.AddFunction("Read", new(PrimitiveType.Int32, [imported.MakeGenericInstance(PrimitiveType.Int32)]));
            call.LoadArgument(0); call.CallVirtual(importedSlot.MakeConstructedReference([PrimitiveType.Int32])); call.Return();
            consumer.WriteNativeAssembly();
            try { call.Call(importedSlot.MakeConstructedReference([PrimitiveType.Int32])); throw new Exception("direct imported abstract call accepted"); }
            catch (ArgumentException) { }
            if ((snapshot.MainModule.Types.Single(t => t.Name == "Base`1").Attributes & 0x80) == 0)
                throw new Exception("abstract generic flag lost");
            var invalidNew = graph.AddFunction("InvalidNew", new(PrimitiveType.Void, []));
            invalidNew.NewObject(parentCtor.MakeConstructedReference([PrimitiveType.Int32])); invalidNew.GetILGenerator().Emit(OpCode.Pop); invalidNew.Return();
            Reject(() => graph.WriteNativeAssembly());
            invalidNew.ClearBody(); invalidNew.Return();
            var incomplete = graph.AddGenericClass("Example", "Incomplete", ["T"], parent.MakeGenericInstance(t));
            Reject(() => graph.WriteNativeAssembly());
            incomplete.SetAbstractClass();
            graph.WriteNativeAssembly();
            try { main.Call(abstractSlot!.MakeConstructedReference([PrimitiveType.Int32])); throw new Exception("direct abstract call accepted"); }
            catch (ArgumentException) { }
        }
        var invalidMetadata = JsonNode.Parse(graph.WriteNativeAssembly())!;
        invalidMetadata["functions"]!.AsArray().First(f => f!["is_override"]?.GetValue<bool>() == true)!["returns"] = new JsonObject { ["TypeParameter"] = 0 };
        Reject(() => NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(invalidMetadata.ToJsonString())));
        if (Environment.GetEnvironmentVariable("NEOCLR_GENERIC_VIRTUAL_ARTIFACT") is { } path) File.WriteAllBytes(path, native);
        var invalid = graph.AddClass("Example", "Invalid", parent.MakeGenericInstance(PrimitiveType.Int32));
        var badOverride = invalid.AddOverride("Echo", new(PrimitiveType.Boolean, [PrimitiveType.Int32]));
        badOverride.GetILGenerator().LoadDefault(PrimitiveType.Boolean); badOverride.Return();
        Reject(() => graph.WriteNativeAssembly());
        Console.WriteLine("PASS generic class virtual slots, reordered overrides, direct base calls and reference/value arguments");
    }
    private static void Reject(Action action)
    {
        try { action(); } catch (InvalidDataException) { return; }
        throw new Exception("invalid substituted override was accepted");
    }
}
