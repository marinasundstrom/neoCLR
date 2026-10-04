using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class ExplicitInterfaceChecks
{
    internal static void Run()
    {
        foreach (var manual in new[] { false, true })
        {
            var host = typeof(object).Assembly.GetName();
            var graph = new AssemblyBuilder(new("ExplicitAccessors" + manual, new Version(1, 0, 0, 0)),
                new(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!)));
            var first = graph.AddInterface("Example", "First");
            var second = graph.AddInterface("Example", "Second");
            var getFirst = first.AddInterfaceMethod("get_Count", new(PrimitiveType.Int32, []));
            var getSecond = second.AddInterfaceMethod("get_Count", new(PrimitiveType.Int32, []));
            var type = graph.AddClass("Example", "Counter");
            type.AddInterfaceImplementation(first); type.AddInterfaceImplementation(second);
            var ctor = type.AddConstructor(Array.Empty<PrimitiveType>()); ctor.GetILGenerator().Return();
            foreach (var (contract, value) in new[] { (first, 19), (second, 23) })
            {
                var method = type.AddInstanceMethod(contract.Name + ".get_Count", new(PrimitiveType.Int32, []), MethodVisibility.Private);
                if (manual) method.Definition.AddExplicitInterfaceImplementation(new InterfaceImplementation(contract.Definition.ToReference()), "get_Count");
                else method.AddExplicitInterfaceImplementation(contract, "get_Count");
                method.GetILGenerator().LoadConstant(value); method.GetILGenerator().Return();
                type.AddProperty(contract.Name + ".Count", PrimitiveType.Int32, method, null);
                Reject(() => method.AddExplicitInterfaceImplementation(contract, "get_Count"));
            }
            var main = graph.AddFunction("Main"); var il = main.GetILGenerator(); graph.EntryPoint = main;
            il.NewObject(ctor); il.CallVirtual(getFirst); il.NewObject(ctor); il.CallVirtual(getSecond); il.Emit(OpCode.Add); il.Return();
            var loaded = Assembly.Load(graph.Write());
            if (!Equals(loaded.EntryPoint!.Invoke(null, null), 42)) throw new Exception("explicit CLI dispatch");
            var map = loaded.GetType("Example.Counter")!.GetInterfaceMap(loaded.GetType("Example.First")!);
            if (!map.TargetMethods[0].IsPrivate || map.TargetMethods[0].Name != "First.get_Count") throw new Exception("explicit CLI flags");
            var native = RuntimeAssemblyContainer.WriteBinary(graph);
            var snapshot = AssemblyDefinition.ReadNativeAssembly(native);
            var counter = snapshot.MainModule.Types.Single(t => t.Name == "Counter");
            if (counter.Methods.Count(m => m.ExplicitInterfaceImplementations.Count == 1) != 2) throw new Exception("native mapping round trip");
            if (Environment.GetEnvironmentVariable("NEOCLR_EXPLICIT_ARTIFACT") is { } output && !manual) File.WriteAllBytes(output, native);
            var incompatible = type.AddInstanceMethod("Bad", new(PrimitiveType.Int32, []));
            Reject(() => incompatible.AddExplicitInterfaceImplementation(first, "get_Count"));
        }
    }
    static void Reject(Action action)
    {
        try { action(); } catch (Exception e) when (e is ArgumentException or InvalidOperationException or InvalidDataException) { return; }
        throw new Exception("invalid explicit mapping accepted");
    }
}
