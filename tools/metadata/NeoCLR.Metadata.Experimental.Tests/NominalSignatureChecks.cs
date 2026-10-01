using System.Reflection;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class NominalSignatureChecks
{
    internal static AssemblyBuilder Create()
    {
        var graph = InstanceObjectChecks.Create();
        var order = graph.Types[0];
        var identity = graph.AddFunction("Identity", new MethodSignature(order, [order]));
        identity.LoadArgument(0); identity.Return();
        var self = order.AddInstanceMethod("Self", new MethodSignature(order, []));
        self.LoadArgument(0); self.Return();
        var update = graph.AddFunction("Update", new MethodSignature(PrimitiveType.Void, [(SignatureType)order, PrimitiveType.Int32]));
        update.LoadArgument(0); update.LoadArgument(1); update.Call(order.Methods[3]); update.Return();
        var entry = graph.EntryPoint!;
        entry.ClearBody();
        var local = entry.DeclareLocal(order);
        entry.LoadConstant(41); entry.Emit(OpCode.Ldc_Bool, true); entry.NewObject(order.Methods[0]);
        entry.Call(identity); entry.Call(self); entry.StoreLocal(local);
        entry.LoadLocal(local); entry.LoadConstant(42); entry.Call(update);
        entry.LoadLocal(local); entry.Call(order.Methods[1]); entry.Return();
        return graph;
    }
    internal static void Run()
    {
        var graph = Create();
        var loaded = Assembly.Load(graph.Write());
        if (!Equals(loaded.EntryPoint!.Invoke(null, null), 42)) throw new Exception("nominal CLI call result");
        var order = loaded.GetType("Example.Order")!;
        if (order.GetMethod("Self")!.ReturnType != order) throw new Exception("nominal result metadata");
        var projected = NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary);
        var snapshot = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition.ReadAssembly(projected, false);
        var direct = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition.ReadAssembly(graph.Write(), false);
        byte[] SelfSignature(NeoCLR.Metadata.Experimental.Model.AssemblyDefinition definition) => definition.MainModule.Types.Single(t => t.Name == "Order").Methods.Single(m => m.Name == "Self").GetSignature();
        if (!SelfSignature(snapshot).SequenceEqual(SelfSignature(direct))) throw new Exception("nominal projection signature mismatch");
        var malformed = System.Text.Json.Nodes.JsonNode.Parse(graph.WriteNativeAssembly())!;
        var identity = malformed["functions"]!.AsArray().Single(f => f!["origin"]!["name"]!.GetValue<string>() == "Identity")!;
        identity["parameters"]![0] = new System.Text.Json.Nodes.JsonObject { ["Named"] = "Missing" };
        try { NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(malformed.ToJsonString())); throw new Exception("unknown named signature accepted"); } catch (InvalidDataException) { }
        var entry = graph.EntryPoint!;
        var other = graph.AddClass("Example", "Other");
        var ctor = other.AddConstructor([]); ctor.Return();
        entry.ClearBody(); entry.NewObject(ctor); entry.Call(graph.Functions.Single(m => m.Name == "Identity")); entry.Emit(OpCode.Pop); entry.LoadConstant(42); entry.Return();
        try { graph.Write(); throw new Exception("wrong nominal argument accepted"); } catch (InvalidDataException) { }
        var foreign = InstanceObjectChecks.Create().Types[0];
        var count = graph.Functions.Count;
        try { graph.AddFunction("Foreign", new MethodSignature(foreign, [])); throw new Exception("foreign signature accepted"); } catch (ArgumentException) { }
        if (graph.Functions.Count != count) throw new Exception("failed declaration mutated graph");
        var result = graph.Types[0].AddInstanceMethod("BadReturn", new MethodSignature(graph.Types[0], []));
        result.NewObject(ctor); result.Return();
        entry.ClearBody(); entry.LoadConstant(42); entry.Return();
        try { graph.Write(); throw new Exception("wrong nominal result accepted"); } catch (InvalidDataException) { }
    }
}
