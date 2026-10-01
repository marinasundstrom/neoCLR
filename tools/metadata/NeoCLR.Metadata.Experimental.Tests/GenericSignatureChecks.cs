using System.Reflection;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class GenericSignatureChecks
{
    internal static AssemblyBuilder Create()
    {
        var graph = InstanceObjectChecks.Create();
        var t = SignatureType.MethodParameter(0);
        var identity = graph.AddFunction("Identity", new MethodSignature(t, [t], ["T"]));
        var slot = identity.DeclareLocal(t);
        identity.LoadArgument(0); identity.StoreLocal(slot); identity.LoadLocal(slot); identity.Return();
        var helper = graph.AddType("Example", "Generic");
        var first = helper.AddMethod("First", new MethodSignature(t, [SignatureType.ArrayOf(t)], ["Element"]));
        first.LoadArgument(0); first.LoadConstant(0); first.LoadArrayElement(t); first.Return();
        return graph;
    }

    internal static void Run()
    {
        var graph = Create();
        var loaded = Assembly.Load(graph.Write());
        var identity = loaded.ManifestModule.GetMethods().Single(m => m.Name == "Identity");
        if (!identity.IsGenericMethodDefinition || identity.GetGenericArguments()[0].Name != "T" ||
            !Equals(identity.MakeGenericMethod(typeof(int)).Invoke(null, [42]), 42)) throw new Exception("generic global definition/local");
        var first = loaded.GetType("Example.Generic")!.GetMethod("First")!;
        if (!Equals(first.MakeGenericMethod(typeof(long)).Invoke(null, [new long[] { 5000000000 }]), 5000000000L)) throw new Exception("generic array signature/token");
        var projected = AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary), false);
        var direct = AssemblyDefinition.ReadAssembly(graph.Write(), false);
        foreach (var type in direct.MainModule.Types)
        {
            var copy = projected.MainModule.Types.Single(t => t.Name == type.Name);
            if (!type.Methods.Zip(copy.Methods).All(p => p.First.GenericArity == p.Second.GenericArity && p.First.GetSignature().SequenceEqual(p.Second.GetSignature()))) throw new Exception("generic reference signatures");
        }
        var owner = graph.Types[0];
        try { owner.AddField("Invalid", SignatureType.MethodParameter(0)); throw new Exception("method parameter used in field"); } catch (ArgumentException) { }
        try { graph.AddFunction("Bad", new MethodSignature(SignatureType.MethodParameter(1), [], ["T"])); throw new Exception("out of scope MVAR"); } catch (ArgumentException) { }
        try { graph.EntryPoint!.Call(graph.Functions.Single(m => m.Name == "Identity")); throw new Exception("open generic call"); } catch (ArgumentException) { }
        var malformed = JsonNode.Parse(graph.WriteNativeAssembly())!;
        var generic = malformed["functions"]!.AsArray().Single(m => m!["origin"]!["name"]!.GetValue<string>() == "Identity")!;
        generic["returns"] = new JsonObject { ["MethodTypeParameter"] = 1 };
        try { NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(malformed.ToJsonString())); throw new Exception("native out of scope MVAR"); } catch (InvalidDataException) { }
    }
}
