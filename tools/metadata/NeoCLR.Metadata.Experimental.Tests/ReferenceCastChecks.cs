using System.Diagnostics;
using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ReferenceCastChecks
{
    static AssemblyBuilder Create()
    {
        var graph = InterfaceDispatchChecks.Create(); var main = graph.EntryPoint!; main.ClearBody();
        var contract = graph.Types[0]; var invoke = graph.Functions[0];
        foreach (var type in graph.Types.Where(t => t.Name is "First" or "Second"))
        {
            main.NewObject(type.Methods.Single(m => m.IsConstructor));
            main.CastReference(contract); main.Call(invoke);
        }
        main.Emit(OpCode.Add); main.Return(); return graph;
    }
    internal static void Run()
    {
        var graph = Create();
        if (!Equals(Assembly.Load(graph.Write()).EntryPoint!.Invoke(null, null), 42)) throw new Exception("reference casts lost identity/dispatch");
        try { graph.EntryPoint!.CastReference(PrimitiveType.Int32); throw new Exception("value destination admitted"); } catch (ArgumentException) { }
        var bad = Create(); bad.EntryPoint!.ClearBody(); bad.EntryPoint.LoadConstant(1); bad.EntryPoint.CastReference(bad.Types[0]); bad.EntryPoint.Emit(OpCode.Pop); bad.EntryPoint.LoadConstant(42); bad.EntryPoint.Return();
        try { bad.Write(); throw new Exception("value source admitted"); } catch (InvalidDataException) { }
    }
    internal static async Task RunRuntime(string runtime, string directory)
    {
        Run(); Directory.CreateDirectory(directory); var graph = Create(); var path = Path.Combine(directory, "ReferenceCast.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add(command); start.ArgumentList.Add(path);
            using var process = Process.Start(start)!; var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (process.ExitCode != (command == "verify" ? 0 : 42)) throw new Exception(command + ": " + text);
        }
        Console.WriteLine("Reference cast preserves interface dispatch on CLR and neoCLR: 42");
    }
}
