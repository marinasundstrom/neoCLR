using System.Reflection;
using System.Diagnostics;
using NeoCLR.Metadata.Experimental;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class IndexedPropertyChecks
{
    internal static AssemblyBuilder Create()
    {
        var graph = InstanceObjectChecks.Create();
        var owner = graph.Types[0];
        var get = owner.AddInstanceMethod("GetItem", new(PrimitiveType.Int32, [PrimitiveType.Int32]));
        get.LoadArgument(0); get.LoadField(owner.Fields[0]); get.LoadArgument(1); get.Emit(OpCode.Add); get.Return();
        var set = owner.AddInstanceMethod("SetItem", new(PrimitiveType.Void, [PrimitiveType.Int32, PrimitiveType.Int32]));
        set.LoadArgument(0); set.LoadArgument(2); set.LoadArgument(1); set.Emit(OpCode.Sub); set.StoreField(owner.Fields[0]); set.Return();
        owner.AddProperty("Item", PrimitiveType.Int32, get, set);
        var wide = owner.AddInstanceMethod("GetWideItem", new(PrimitiveType.Int32, [PrimitiveType.Int64]));
        wide.LoadArgument(0); wide.LoadField(owner.Fields[0]); wide.Return();
        owner.AddProperty("Item", PrimitiveType.Int32, wide);
        var matrixSet = owner.AddInstanceMethod("WriteCell", new(PrimitiveType.Void, [PrimitiveType.Int32, PrimitiveType.Int32, PrimitiveType.Int32]));
        matrixSet.LoadArgument(0); matrixSet.LoadArgument(3); matrixSet.LoadArgument(1); matrixSet.Emit(OpCode.Sub);
        matrixSet.LoadArgument(2); matrixSet.Emit(OpCode.Sub); matrixSet.StoreField(owner.Fields[0]); matrixSet.Return();
        owner.AddProperty("Cell", PrimitiveType.Int32, setter: matrixSet);
        var entry = graph.EntryPoint!; entry.ClearBody();
        var local = entry.DeclareLocal(owner);
        entry.LoadConstant(1); entry.Emit(OpCode.Ldc_Bool, true); entry.NewObject(owner.Methods[0]); entry.StoreLocal(local);
        entry.LoadLocal(local); entry.LoadConstant(2); entry.LoadConstant(3); entry.LoadConstant(42); entry.Call(matrixSet);
        entry.LoadLocal(local); entry.Emit(OpCode.Ldc_I8, 0L); entry.Call(wide); entry.Emit(OpCode.Pop);
        entry.LoadLocal(local); entry.LoadConstant(5); entry.Call(get); entry.Return();
        return graph;
    }

    internal static void Run()
    {
        var graph = Create();
        var assembly = Assembly.Load(graph.Write());
        if (!Equals(assembly.EntryPoint!.Invoke(null, null), 42)) throw new Exception("indexed accessor execution");
        var type = assembly.GetType("Example.Order")!;
        var instance = Activator.CreateInstance(type, [1, true]);
        var property = type.GetProperty("Item", [typeof(int)])!;
        property.SetValue(instance, 44, [4]);
        if (!Equals(property.GetValue(instance, [2]), 42) || !Equals(type.GetProperty("Item", [typeof(long)])!.GetValue(instance, [99L]), 40))
            throw new Exception("indexed property association or overload");
        var projection = AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary), false);
        var projected = projection.MainModule.Types.Single(t => t.Name == "Order").Properties.Where(p => p.Name == "Item").ToArray();
        if (projected.Length != 2 || !projected[0].GetSignature().SequenceEqual(new byte[] { 0x28, 1, 8, 8 }) ||
            !projected[1].GetSignature().SequenceEqual(new byte[] { 0x28, 1, 8, 0x0a }) || projected[1].SetMethod is not null)
            throw new Exception("index parameter projection");
        var cell = type.GetProperty("Cell")!;
        if (cell.GetMethod is not null || cell.GetIndexParameters().Length != 2) throw new Exception("setter-only indices");
        cell.SetValue(instance, 45, [2, 3]);
        if (!Equals(property.GetValue(instance, [2]), 42)) throw new Exception("multiple-index setter association");
        var owner = graph.Types[0];
        var mismatch = owner.AddInstanceMethod("Mismatch", new(PrimitiveType.Void, [PrimitiveType.Boolean, PrimitiveType.Int32]));
        mismatch.Return();
        var before = owner.Properties.Count;
        var fresh = owner.AddInstanceMethod("Fresh", new(PrimitiveType.Int32, [PrimitiveType.Int64]));
        fresh.LoadConstant(0); fresh.Return();
        try { owner.AddProperty("Bad", PrimitiveType.Int32, fresh, mismatch); throw new Exception("mismatched/reused accessor accepted"); } catch (ArgumentException) { }
        var duplicate = owner.AddInstanceMethod("Duplicate", new(PrimitiveType.Int32, [PrimitiveType.Int32]));
        duplicate.LoadConstant(0); duplicate.Return();
        try { owner.AddProperty("Item", PrimitiveType.Int32, duplicate); throw new Exception("duplicate index signature accepted"); } catch (ArgumentException) { }
        if (owner.Properties.Count != before) throw new Exception("rejected property mutated collection");
        var malformed = JsonNode.Parse(graph.WriteNativeAssembly())!;
        malformed["types"]![0]!["properties"]![0]!["parameters"]![0] = "Boolean";
        try { NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(malformed.ToJsonString())); throw new Exception("mismatched index metadata accepted"); } catch (InvalidDataException) { }
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        Run();
        if (Directory.Exists(output)) throw new IOException("output must be fresh");
        Directory.CreateDirectory(output);
        var graph = Create();
        var path = Path.Combine(output, "IndexedProperties.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add(command); start.ArgumentList.Add(path);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (process.ExitCode != (command == "verify" ? 0 : 42)) throw new Exception(command + ": " + text);
        }
        Console.WriteLine("PASS indexed property binary: overloaded getter and multiple-index setter-only association, verify/run 42");
    }

}
