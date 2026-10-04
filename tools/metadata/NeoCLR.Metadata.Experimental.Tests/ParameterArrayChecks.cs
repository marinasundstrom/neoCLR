using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ParameterArrayChecks
{
    internal static void Run()
    {
        var name = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(name.Name!, name.Version!, "", Convert.ToHexString(name.GetPublicKeyToken()!));
        var graph = Create(core);
        var image = graph.Write();
        var loaded = AssemblyDefinition.ReadAssembly(image, expectedExtended: false);
        Check(loaded.MainModule.Types.SelectMany(t => t.Methods).Count(m => m.ParameterArrayIndex == 0) == 2, "CLI parameter metadata lost");
        var context = new AssemblyLoadContext("parameter-array-check", isCollectible: true);
        try
        {
            var assembly = context.LoadFromStream(new MemoryStream(image));
            foreach (var method in assembly.GetType("Example.Operations")!.GetMethods().Where(m => m.DeclaringType?.Namespace == "Example"))
            {
                Check(method.GetParameters()[0].IsDefined(typeof(ParamArrayAttribute), false), "CLR marker lost");
                Check((int)method.Invoke(null, [new[] { 42, 7 }])! == 42, "array argument changed");
            }
        }
        finally { context.Unload(); }
        var bad = graph.AddFunction("Bad", new(PrimitiveType.Int32, [PrimitiveType.Int32]));
        Reject(() => bad.SetParameterArray(0));
        Reject(() => graph.Definition.MainModule.Types.SelectMany(t => t.Methods).First(m => m.ParameterArrayIndex == 0).SetParameterArray(1));
        Reject(() => graph.WriteNativeAssembly()); // No implicit native core binding.
    }

    internal static void RunNative(string corePath, string seedPath)
    {
        var core = AssemblyDefinition.ReadAssembly(File.ReadAllBytes(corePath), expectedExtended: false);
        var graph = Create(core.Identity);
        graph.BindNativeLibrary(core, NativeLibraryDefinition.ReadAssembly(File.ReadAllBytes(seedPath)), core.Identity);
        var json = graph.WriteNativeAssembly();
        var binary = RuntimeAssemblyContainer.WriteBinary(json, core.Identity);
        var snapshot = AssemblyDefinition.ReadNativeAssembly(binary);
        var context = new NeoCLR.Metadata.Experimental.Introspection.MetadataLoadContext([snapshot]);
        var methods = context.Resolve(snapshot.Identity).GetTypes().Single(t => t.Name == "Operations").GetMethods();
        Check(methods.Count() == 2 && methods.All(m => m.GetParameters().Single().IsParameterArray), "native introspection lost parameter arrays");
        var projected = RuntimeAssemblyContainer.ReadCliProjection(binary);
        Check(projected.MainModule.Types.Single(t => t.Name == "Operations").Methods.All(m => m.ParameterArrayIndex == 0), "projection lost marker");
        var root = System.Text.Json.Nodes.JsonNode.Parse(json)!;
        var functions = root["functions"]!.AsArray();
        var marked = functions.First(f => f!["custom_attributes"] is not null)!;
        marked["custom_attributes"]![0]!["target_token"] = 0;
        Reject(() => AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(System.Text.Encoding.UTF8.GetBytes(root.ToJsonString()), core.Identity)));
    }

    private static AssemblyBuilder Create(AssemblyIdentity core)
    {
        var graph = new AssemblyBuilder(new("ParameterArrays", new(1, 0, 0, 0)), core);
        var owner = graph.AddClass("Example", "Operations");
        foreach (bool manual in new[] { false, true })
        {
            var signature = new MethodSignature(PrimitiveType.Int32, [SignatureType.ArrayOf(PrimitiveType.Int32)]);
            MethodBuilder method;
            if (manual)
            {
                var definition = new MethodDefinition("Manual", 0x16, signature);
                owner.Definition.Methods.Add(definition); definition.SetParameterArray(0);
                method = MethodBuilder.ForDefinition(definition);
            }
            else { method = owner.AddMethod("Built", signature); method.SetParameterArray(0); }
            method.SetParameterArray(0);
            var il = method.GetILGenerator();
            il.LoadArgument(0); il.LoadConstant(0); il.Emit(OpCode.Ldelem, PrimitiveType.Int32); il.Return();
        }
        return graph;
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
    private static void Reject(Action action)
    {
        try { action(); } catch (Exception e) when (e is ArgumentException or InvalidOperationException or InvalidDataException) { return; }
        throw new Exception("unsupported parameter metadata accepted");
    }
}
