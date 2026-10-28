using System.Reflection;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class GenericInterfaceChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var graph = new AssemblyBuilder(new("GenericInterfaces", new Version(1, 0, 0, 0)),
            new(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? [])));
        var contract = graph.AddInterface("Example", "Echo");
        var state = SignatureType.MethodParameter(0);
        var signature = new MethodSignature(state, [state], ["T"]);
        var apply = contract.AddInterfaceMethod("Apply", signature);
        var owner = graph.AddClass("Example", "Identity");
        owner.AddInterfaceImplementation(contract);
        var constructor = owner.AddConstructor([]);
        constructor.Return();
        var body = owner.AddInstanceMethod("Apply", signature);
        body.LoadArgument(1);
        body.Return();
        var main = graph.AddFunction("Main");
        main.NewObject(constructor);
        main.LoadConstant(42);
        main.Emit(OpCode.Callvirt, apply.MakeGenericInstance(PrimitiveType.Int32));
        main.Return();
        graph.EntryPoint = main;
        var loaded = Assembly.Load(graph.Write());
        if (!Equals(loaded.EntryPoint!.Invoke(null, null), 42)) throw new Exception("generic interface CLI dispatch");
        var method = loaded.GetType("Example.Echo")!.GetMethod("Apply")!;
        if (!method.IsGenericMethodDefinition || !method.IsAbstract || method.GetGenericArguments().Length != 1)
            throw new Exception("generic interface metadata");
        Console.WriteLine("PASS generic interface metadata and constructed virtual call");
    }
}
