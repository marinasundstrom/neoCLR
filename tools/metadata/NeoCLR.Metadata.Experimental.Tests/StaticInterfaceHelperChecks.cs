using NeoCLR.Metadata.Experimental.Model;

internal static class StaticInterfaceHelperChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("System.Private.CoreLib", typeof(object).Assembly.GetName().Version!, "", "7cec85d7bea7798e");
        var graph = new AssemblyBuilder(new("InterfaceHelpers", new(1, 0, 0, 0)), core);
        var contract = graph.AddInterface("Example", "Reader");
        var read = contract.AddInterfaceMethod("Read", new(PrimitiveType.Int32, []));
        var helper = contract.AddMethod("Empty", new(PrimitiveType.Int32, []));
        helper.GetILGenerator().LoadConstant(0);
        helper.GetILGenerator().Return();
        if (helper.IsAbstract || !read.IsAbstract) throw new Exception("Static helpers must be distinct from abstract contracts");
        var implementation = graph.AddClass("Example", "ReaderImplementation");
        implementation.AddInterfaceImplementation(contract);
        var readBody = implementation.AddInstanceMethod("Read", new(PrimitiveType.Int32, []));
        readBody.GetILGenerator().LoadConstant(42);
        readBody.GetILGenerator().Return();
        var image = NeoCLR.Metadata.Experimental.RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), core);
        var loaded = AssemblyDefinition.ReadNativeAssembly(image);
        var methods = loaded.MainModule.Types.Single(t => t.Name == "Reader").Methods;
        if ((methods.Single(m => m.Name == "Empty").Attributes & 0x400) != 0 ||
            (methods.Single(m => m.Name == "Read").Attributes & 0x400) == 0)
            throw new Exception("Native roundtrip lost helper versus contract distinction");
        try
        {
            contract.AddInstanceMethod("Invalid", new(PrimitiveType.Int32, []));
            throw new Exception("Instance interface bodies remain unsupported");
        }
        catch (InvalidOperationException) { }
        Console.WriteLine("PASS static interface helper metadata, implementation obligations and instance-body rejection");
    }
}
