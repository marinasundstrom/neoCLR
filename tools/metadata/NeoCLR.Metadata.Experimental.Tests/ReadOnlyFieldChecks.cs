using System.Diagnostics;
using System.Reflection;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class ReadOnlyFieldChecks
{
    internal static AssemblyBuilder Create()
    {
        var graph = InstanceObjectChecks.Create();
        var holder = graph.AddClass("Example", "ReadOnlyHolder");
        var value = holder.AddField("Value", graph.Types[0], FieldVisibility.Public, isReadOnly: true);
        var number = holder.AddField("Number", PrimitiveType.Int32, FieldVisibility.Public, isReadOnly: true);
        var ctor = holder.AddConstructor(new MethodSignature(PrimitiveType.Void, [graph.Types[0]]));
        ctor.LoadArgument(0); ctor.LoadArgument(1); ctor.StoreField(value);
        ctor.LoadArgument(0); ctor.LoadConstant(42); ctor.StoreField(number); ctor.Return();
        var get = holder.AddInstanceMethod("Read", new(PrimitiveType.Int32, []));
        get.LoadArgument(0); get.LoadField(value); get.LoadConstant(42); get.Call(graph.Types[0].Methods[3]);
        get.LoadArgument(0); get.LoadField(number); get.Return();
        var entry = graph.EntryPoint!;
        entry.ClearBody(); entry.LoadConstant(1); entry.Emit(OpCode.Ldc_Bool, true); entry.NewObject(graph.Types[0].Methods[0]);
        entry.NewObject(ctor); entry.Call(get); entry.Return();
        return graph;
    }

    internal static void Run()
    {
        var graph = Create();
        var assembly = Assembly.Load(graph.Write());
        if (!Equals(assembly.EntryPoint!.Invoke(null, null), 42)) throw new Exception("readonly CLI construction");
        if (assembly.GetType("Example.ReadOnlyHolder")!.GetFields().Any(f => !f.IsInitOnly)) throw new Exception("missing InitOnly flags");
        var projection = AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary), false);
        if (projection.MainModule.Types.Single(t => t.Name == "ReadOnlyHolder").Fields.Any(f => (f.Attributes & (ushort)FieldAttributes.InitOnly) == 0))
            throw new Exception("readonly projection flags");
        var bad = graph.Types[1].Methods[1];
        bad.ClearBody(); bad.LoadArgument(0); bad.LoadConstant(1); bad.StoreField(graph.Types[1].Fields[1]); bad.LoadConstant(42); bad.Return();
        try { graph.Write(); throw new Exception("readonly ordinary store accepted"); } catch (InvalidDataException) { }
        bad.ClearBody(); bad.LoadConstant(42); bad.Return();
        var foreign = graph.AddClass("Example", "Other").AddConstructor(new MethodSignature(PrimitiveType.Void, [graph.Types[1]]));
        foreign.LoadArgument(1); foreign.LoadConstant(1); foreign.StoreField(graph.Types[1].Fields[1]); foreign.Return();
        try { graph.Write(); throw new Exception("other constructor store accepted"); } catch (InvalidDataException) { }
    }

    internal static async Task RunRuntime(string runtime, string output)
    {
        Run();
        if (Directory.Exists(output)) throw new IOException("output must be fresh");
        Directory.CreateDirectory(output);
        var graph = Create();
        foreach (var mode in new[] { "valid", "store", "address" })
        {
            var payload = graph.WriteNativeAssembly();
            if (mode != "valid")
            {
                var root = JsonNode.Parse(payload)!;
                var method = root["functions"]!.AsArray().Single(f => f!["origin"]!["name"]!.GetValue<string>() == "Read")!;
                method["body"] = JsonNode.Parse(mode == "store"
                    ? """[{"op":"ldarg","arg":0},{"op":"ldc.i4","arg":1},{"op":"stfld","arg":1},{"op":"ldc.i4","arg":42},{"op":"ret"}]"""
                    : """[{"op":"ldarg","arg":0},{"op":"ldflda","arg":1},{"op":"ldc.i4","arg":1},{"op":"stobj","arg":"Int32"},{"op":"ldc.i4","arg":42},{"op":"ret"}]""");
                payload = Encoding.UTF8.GetBytes(root.ToJsonString());
            }
            var path = Path.Combine(output, mode + ".dll");
            File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(payload, graph.CoreLibrary));
            foreach (var command in new[] { "verify", "run" })
            {
                var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
                start.ArgumentList.Add(command); start.ArgumentList.Add(path);
                using var process = Process.Start(start)!;
                var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
                await process.WaitForExitAsync(); var text = await stdout + await stderr;
                if (mode == "valid" ? process.ExitCode != (command == "verify" ? 0 : 42) : process.ExitCode == 0 || !text.Contains("readonly", StringComparison.OrdinalIgnoreCase))
                    throw new Exception(mode + ": " + text);
            }
        }
        Console.WriteLine("PASS readonly binary construction 42; direct and indirect writes rejected by verification and execution");
    }
}
