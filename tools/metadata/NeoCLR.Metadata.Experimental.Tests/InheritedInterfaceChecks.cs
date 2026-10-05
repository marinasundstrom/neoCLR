using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class InheritedInterfaceChecks
{
    internal static AssemblyBuilder Create()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("InheritedInterfaces", new(1, 0, 0, 0)), core);
        var contract = graph.AddInterface("Example", "IValue");
        var slot = contract.AddInterfaceMethod("Read", new(PrimitiveType.Int32, []));
        var parent = graph.AddClass("Example", "Base");
        var ctor = parent.AddConstructor([]); ctor.GetILGenerator().Return();
        var body = parent.AddInstanceMethod("Read", new(PrimitiveType.Int32, []));
        body.GetILGenerator().LoadConstant(42); body.GetILGenerator().Return();
        var unrelated = parent.AddInstanceMethod("Other", new(PrimitiveType.Int32, []));
        unrelated.GetILGenerator().LoadConstant(7); unrelated.GetILGenerator().Return();
        var child = graph.AddClass("Example", "Derived", parent);
        child.AddInterfaceImplementation(contract);
        var create = child.AddConstructor([]);
        var il = create.GetILGenerator(); il.LoadArgument(0); il.Call(ctor); il.Return();
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        il = main.GetILGenerator(); il.NewObject(create); il.CallVirtual(slot); il.Return();
        return graph;
    }
    internal static void Run()
    {
        var graph = Create();
        var cli = System.Reflection.Assembly.Load(graph.Write());
        if ((int)cli.EntryPoint!.Invoke(null, null)! != 42) throw new Exception("inherited CLI interface dispatch failed");
        var parent = cli.GetType("Example.Base")!;
        if (!parent.GetMethod("Read")!.IsVirtual || parent.GetMethod("Other")!.IsVirtual)
            throw new Exception("incorrect inherited implementation flags");
        var native = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(graph));
        var owner = native.MainModule.Types.Single(t => t.Name == "Derived");
        if (owner.Interfaces.Count != 1 || owner.BaseType!.Resolve().Name != "Base")
            throw new Exception("native inherited relationship lost");
    }
}
