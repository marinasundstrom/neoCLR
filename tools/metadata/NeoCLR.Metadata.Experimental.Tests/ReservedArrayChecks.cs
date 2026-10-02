using System.Diagnostics;
using System.Security.Cryptography;
using System.Text.Json;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ReservedArrayChecks
{
    static AssemblyBuilder Create(bool unread = false)
    {
        var host = typeof(object).Assembly.GetName();
        var graph = new AssemblyBuilder(new("ReservedArray", new Version(1, 0, 0, 0)), new(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!)));
        var reserve = graph.AddFunction("Reserve", new MethodSignature(SignatureType.ArrayOf(SignatureType.MethodParameter(0)), [PrimitiveType.Int32], ["T"]));
        reserve.LoadArgument(0);
        reserve.Emit(OpCode.ReserveArray, SignatureType.MethodParameter(0));
        reserve.Return();
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        var values = main.DeclareLocal(SignatureType.ArrayOf(PrimitiveType.Int32));
        main.LoadConstant(2); main.Call(reserve.MakeGenericInstance(PrimitiveType.Int32)); main.StoreLocal(values);
        main.LoadLocal(values); main.LoadConstant(0); main.LoadConstant(42); main.StoreArrayElement(PrimitiveType.Int32);
        main.LoadLocal(values); main.LoadConstant(unread ? 1 : 0); main.LoadArrayElement(PrimitiveType.Int32); main.Return();
        return graph;
    }
    internal static void Run()
    {
        var graph = Create();
        _ = RuntimeAssemblyContainer.ReadCliProjection(RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        void Reject(Action action)
        {
            try { action(); } catch (Exception e) when (e is ArgumentException or InvalidDataException) { return; }
            throw new Exception("invalid reservation admitted");
        }
        Reject(() => graph.Write());
        Reject(() => graph.EntryPoint!.ReserveArray(SignatureType.MethodParameter(0)));
        Reject(() => graph.EntryPoint!.ReserveArray(PrimitiveType.Void));
        var bad = new AssemblyBuilder(new("BadReserve", new Version(1, 0, 0, 0)), graph.CoreLibrary);
        var body = bad.AddFunction("Main"); bad.EntryPoint = body;
        body.Emit(OpCode.Ldc_Bool, true); body.ReserveArray(PrimitiveType.Int32); body.LoadArrayLength(); body.Return();
        Reject(() => bad.WriteNativeAssembly());
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        Run(); Directory.CreateDirectory(output);
        var reports = new List<object>();
        foreach (var unread in new[] { false, true })
        {
            var graph = Create(unread); var path = Path.Combine(output, unread ? "Unread.dll" : "Written.dll");
            File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
            foreach (var command in new[] { "verify", "run" })
            {
                var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
                start.ArgumentList.Add(command); start.ArgumentList.Add(path);
                using var process = Process.Start(start)!;
                var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
                await process.WaitForExitAsync(); var text = await stdout + await stderr;
                if (command == "run" && unread ? process.ExitCode == 0 || !text.Contains("uninitialized", StringComparison.OrdinalIgnoreCase) : process.ExitCode != (command == "verify" ? 0 : 42))
                    throw new Exception(command + ": " + text);
                reports.Add(new { unread, command, exitCode = process.ExitCode, text });
            }
        }
        File.WriteAllText(Path.Combine(output, "validation.json"), JsonSerializer.Serialize(new {
            runtimeSha256 = Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(runtime))), cases = reports
        }, new JsonSerializerOptions { WriteIndented = true }) + "\n");
        Console.WriteLine("PASS reserved arrays: written slot 42; unread slot faults; executable CLI rejected");
    }
}
