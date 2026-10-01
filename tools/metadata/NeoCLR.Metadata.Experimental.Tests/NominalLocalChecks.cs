using System.Reflection;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class NominalLocalChecks
{
    internal static AssemblyBuilder Create()
    {
        var graph = InstanceObjectChecks.Create();
        var entry = graph.EntryPoint!;
        entry.ClearBody();
        var owner = graph.Types[0];
        var original = entry.DeclareLocal(owner);
        var alias = entry.DeclareLocal(owner);
        entry.LoadConstant(41); entry.Emit(OpCode.Ldc_Bool, true); entry.NewObject(owner.Methods[0]); entry.StoreLocal(original);
        entry.LoadLocal(original); entry.StoreLocal(alias);
        entry.LoadLocal(alias); entry.LoadConstant(42); entry.Call(owner.Methods[3]);
        entry.LoadLocal(original); entry.Call(owner.Methods[1]); entry.Return();
        return graph;
    }
    internal static void Run()
    {
        var graph = Create();
        var entry = graph.EntryPoint!;
        var local = entry.Locals[0];
        if (local.Type is not null || local.ClassType != graph.Types[0]) throw new Exception("nominal local identity");
        var assembly = Assembly.Load(graph.Write());
        if (!Equals(assembly.EntryPoint!.Invoke(null, null), 42) || assembly.EntryPoint.GetMethodBody()!.LocalVariables[0].LocalType.Name != "Order")
            throw new Exception("CLI nominal local aliasing");
        _ = NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary);
        void RejectWrite()
        {
            try { graph.Write(); throw new Exception("invalid nominal body accepted"); } catch (InvalidDataException) { }
        }
        entry.ClearBody(); entry.LoadLocal(local); entry.Call(graph.Types[0].Methods[1]); entry.Return(); RejectWrite();
        entry.ClearBody(); entry.LoadConstant(42); entry.StoreLocal(local); entry.LoadConstant(42); entry.Return(); RejectWrite();
        var other = graph.AddClass("Example", "Other"); var constructor = other.AddConstructor([]); constructor.Return();
        entry.ClearBody(); entry.NewObject(constructor); entry.StoreLocal(local); entry.LoadConstant(42); entry.Return(); RejectWrite();
        var count = entry.Locals.Count;
        foreach (var invalid in new[] { graph.AddType("Example", "Static"), InstanceObjectChecks.Create().Types[0] })
        {
            try { entry.DeclareLocal(invalid); throw new Exception("invalid class local accepted"); } catch (ArgumentException) { }
        }
        if (entry.Locals.Count != count) throw new Exception("rejected local mutated declarations");
    }
}
