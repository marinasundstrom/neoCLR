using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class EntryArgumentChecks
{
    internal static AssemblyBuilder Create()
    {
        var core = new AssemblyIdentity("System.Private.CoreLib", typeof(object).Assembly.GetName().Version!, "", "7cec85d7bea7798e");
        var graph = new AssemblyBuilder(new("EntryArguments", new Version(1, 0, 0, 0)), core);
        var main = graph.AddFunction("Main", new MethodSignature(PrimitiveType.Int32, [SignatureType.ArrayOf(PrimitiveType.String)]));
        graph.EntryPoint = main;
        var il = main.GetILGenerator();
        il.LoadArgument(0); il.LoadArrayLength(); il.Return();
        return graph;
    }

    internal static void Run()
    {
        var graph = Create();
        if (!Equals(2, Assembly.Load(graph.Write()).EntryPoint!.Invoke(null, [new[] { "Café", "two" }])))
            throw new Exception("CLI entry argument vector lost");
        _ = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(graph));
        var invalid = new AssemblyBuilder(new("InvalidEntry", new Version(1, 0, 0, 0)), graph.CoreLibrary);
        var main = invalid.AddFunction("Main", new MethodSignature(PrimitiveType.Int32, [PrimitiveType.Int32]));
        invalid.EntryPoint = main;
        main.GetILGenerator().LoadArgument(0); main.GetILGenerator().Return();
        try { invalid.Write(); }
        catch (InvalidDataException) { return; }
        throw new Exception("unsupported entry parameter accepted");
    }
}
