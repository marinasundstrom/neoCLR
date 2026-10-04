using System.Reflection;
using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;
using MethodDefinition = NeoCLR.Metadata.Experimental.Model.MethodDefinition;

internal static class ValueOverrideAuthoringChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("ValueOverrideAuthoring", new Version(1, 0, 0, 0)), core);
        var signature = new MethodSignature(PrimitiveType.String, []);
        var contract = graph.AddInterface("Example", "Display");
        contract.AddInterfaceMethod("ToString", signature);
        var owner = graph.AddValueType("Example", "Manual");
        var definition = new MethodDefinition("ToString", (ushort)(MethodAttributes.Public | MethodAttributes.Virtual), signature);
        owner.Definition.Methods.Add(definition);
        owner.AddInterfaceImplementation(contract);
        var manual = MethodBuilder.ForDefinition(definition);
        var convenience = graph.AddValueType("Example", "Built").AddOverride("ToString", signature);
        var generic = graph.AddGenericValueType("Example", "Generic", ["T"]).AddOverride("ToString", signature);
        foreach (var method in new[] { manual, convenience, generic })
        {
            var il = method.GetILGenerator();
            il.Emit(OpCode.Ldstr, method.DeclaringType!.Name);
            il.Return();
            Check((method.Definition.Attributes & 0x550) == 0x40, "override must reuse an instance slot without Abstract or NewSlot");
        }
        Check(manual.Definition.Attributes == convenience.Definition.Attributes, "definition/builder flag parity");
        var image = graph.Write();
        var snapshot = AssemblyDefinition.ReadAssembly(image, expectedExtended: false);
        foreach (var type in snapshot.MainModule.Types.Where(t => t.Namespace == "Example" && t.Name != "Display"))
            Check((type.Methods.Single().Attributes & 0x550) == 0x40, "CLI roundtrip preserves override flags");
        var context = new AssemblyLoadContext("value-override-authoring", isCollectible: true);
        try
        {
            var assembly = context.LoadFromStream(new MemoryStream(image));
            foreach (var name in new[] { "Manual", "Built", "Generic`1" })
            {
                var type = assembly.GetType("Example." + name)!;
                if (type.IsGenericTypeDefinition) type = type.MakeGenericType(typeof(int));
                object value = Activator.CreateInstance(type)!;
                Check(value.ToString() == name, "boxed Object.ToString virtual dispatch: " + name);
                if (name == "Manual")
                    Check(Equals(assembly.GetType("Example.Display")!.GetMethod("ToString")!.Invoke(value, null), name), "override also implements the interface slot");
                Check(type.GetMethod("ToString")!.GetBaseDefinition().DeclaringType == typeof(object), "Object slot identity");
            }
        }
        finally { context.Unload(); }
        try { graph.WriteNativeAssembly(); throw new Exception("native override silently encoded as an ordinary method"); }
        catch (InvalidDataException error) when (error.Message.Contains("runtime slot binding")) { }

        foreach (var attributes in new ushort[] { 0x41, 0x43, 0x56, 0x146, 0x446 })
            Reject<ArgumentException>(() => new MethodDefinition("ToString", attributes, signature));
        Reject<ArgumentException>(() => owner.AddOverride("DifferentSlot", signature));
        Reject<ArgumentException>(() => owner.AddOverride("ToString", new(PrimitiveType.Int32, [])));
        Reject<ArgumentException>(() => owner.AddOverride("ToString", new(PrimitiveType.String, [PrimitiveType.Int32])));
        Reject<ArgumentException>(() => owner.AddOverride("ToString", new(PrimitiveType.String, [], ["T"])));
        Reject<ArgumentException>(() => owner.AddOverride("ToString", signature));
        foreach (var invalidOwner in new[] { graph.AddInterface("Example", "Contract"), graph.AddType("Example", "Static") })
        {
            Reject<InvalidOperationException>(() => invalidOwner.AddOverride("ToString", signature));
            Check(invalidOwner.Methods.Count == 0, "rejected builder attachment leaves owner unchanged");
            var detached = new MethodDefinition("ToString", 0x46, signature);
            Reject<InvalidOperationException>(() => invalidOwner.Definition.Methods.Add(detached));
            Check(invalidOwner.Methods.Count == 0, "rejected definition attachment leaves owner unchanged");
        }
    }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); }
        catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
    private static void Check(bool condition, string message)
    {
        if (!condition) throw new Exception(message);
    }
}
