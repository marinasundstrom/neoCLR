using System.Diagnostics;
using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;

internal static class InstanceObjectChecks
{
    internal static AssemblyBuilder Create()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var graph = new AssemblyBuilder(new("InstanceOrder", new Version(1, 0, 0, 0)), core);
        var order = graph.AddClass("Example", "Order");
        var number = order.AddField("Number", PrimitiveType.Int32);
        var pending = order.AddField("Pending", PrimitiveType.Boolean);
        var ctor = order.AddConstructor([PrimitiveType.Int32, PrimitiveType.Boolean]);
        var done = ctor.DefineLabel();
        ctor.LoadArgument(2); ctor.Emit(OpCode.Brfalse, done);
        ctor.LoadArgument(0); ctor.LoadArgument(1); ctor.StoreField(number);
        ctor.MarkLabel(done);
        ctor.LoadArgument(0); ctor.LoadArgument(2); ctor.Emit(OpCode.Stfld, pending); ctor.Return();
        var getNumber = order.AddInstanceMethod("GetNumber", new(PrimitiveType.Int32, []));
        getNumber.LoadArgument(0); getNumber.LoadField(number); getNumber.Return();
        var getPending = order.AddInstanceMethod("GetPending", new(PrimitiveType.Boolean, []));
        getPending.LoadArgument(0); getPending.Emit(OpCode.Ldfld, pending); getPending.Return();
        var setNumber = order.AddInstanceMethod("SetNumber", new(PrimitiveType.Void, [PrimitiveType.Int32]));
        setNumber.LoadArgument(0); setNumber.LoadArgument(1); setNumber.StoreField(number); setNumber.Return();
        var main = graph.AddFunction("Main");
        main.LoadConstant(41); main.Emit(OpCode.Ldc_Bool, true); main.NewObject(ctor);
        main.Duplicate(); main.LoadConstant(42); main.Call(setNumber);
        main.Emit(OpCode.Dup); main.Call(getPending);
        var failed = main.DefineLabel(); main.Emit(OpCode.Brfalse, failed);
        main.Call(getNumber); main.Return();
        main.MarkLabel(failed); main.Emit(OpCode.Pop); main.LoadConstant(1); main.Return();
        graph.EntryPoint = main;
        return graph;
    }
    internal static void Run()
    {
        var graph = Create();
        var loaded = Assembly.Load(graph.Write());
        if (!Equals(loaded.EntryPoint!.Invoke(null, null), 42)) throw new Exception("CLI allocation/mutation/alias execution");
        var order = loaded.GetType("Example.Order")!;
        var zero = Activator.CreateInstance(order, [99, false]);
        if (!Equals(order.GetMethod("GetNumber")!.Invoke(zero, null), 0)) throw new Exception("constructor branch/default field");
        var projection = AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary), false);
        var type = projection.MainModule.Types.Single(t => t.Name == "Order");
        if (type.Methods.Any(m => m.IsStatic) || type.Methods.Single(m => m.Name == ".ctor").GetSignature()[0] != 0x20)
            throw new Exception("instance/constructor signature projection");
        var entry = graph.EntryPoint!;
        try { entry.Emit(OpCode.Newobj, graph.Types[0].Methods[1]); throw new Exception("ordinary method accepted as constructor"); } catch (ArgumentException) { }
        try { entry.Call(graph.Types[0].Methods[0]); throw new Exception("direct constructor call accepted"); } catch (ArgumentException) { }
        graph.EntryPoint = graph.Types[0].Methods[1];
        Reject(graph, "instance entry"); graph.EntryPoint = entry;
        var other = graph.AddClass("Example", "Other"); var otherCtor = other.AddConstructor([]); otherCtor.Return();
        entry.ClearBody(); entry.NewObject(otherCtor); entry.Call(graph.Types[0].Methods[1]); entry.Return();
        Reject(graph, "wrong receiver class");
        entry.ClearBody(); entry.LoadConstant(1); entry.LoadField(graph.Types[0].Fields[0]); entry.Return();
        Reject(graph, "primitive receiver");
        entry.ClearBody(); entry.LoadConstant(42); entry.Return();
        var getter = graph.Types[0].Methods[1];
        getter.ClearBody(); getter.LoadArgument(0); getter.Emit(OpCode.Starg, 0); getter.LoadConstant(0); getter.Return();
        Reject(graph, "receiver store");
    }
    private static void Reject(AssemblyBuilder graph, string detail)
    {
        try { graph.Write(); throw new Exception("accepted " + detail); } catch (InvalidDataException) { }
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        Run();
        if (Directory.Exists(output)) throw new IOException("output must be fresh");
        Directory.CreateDirectory(output);
        var graph = Create();
        var path = Path.Combine(output, "Order.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add(command); start.ArgumentList.Add(path);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (process.ExitCode != (command == "verify" ? 0 : 42)) throw new Exception(text);
        }
        Console.WriteLine("PASS constructor/instance calls/field mutation/alias -> CLI and native binary execution 42");
    }
}
