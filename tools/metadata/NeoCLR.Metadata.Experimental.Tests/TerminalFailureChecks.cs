using System.Diagnostics;
using System.Reflection;
using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class TerminalFailureChecks
{
    const string Message = "Invalid propagation carrier: åäö";
    static AssemblyBuilder Create(bool success)
    {
        var core = new AssemblyIdentity("System.Private.CoreLib", typeof(object).Assembly.GetName().Version!, "", "7cec85d7bea7798e");
        var assembly = new AssemblyBuilder(new("TerminalFailure", new Version(1, 0, 0, 0)), core);
        var guard = assembly.AddFunction("Guard", new MethodSignature(PrimitiveType.Int32, [PrimitiveType.Boolean]));
        var good = guard.DefineLabel();
        guard.LoadArgument(0); guard.Emit(OpCode.Brtrue, good); guard.Fail(Message);
        guard.MarkLabel(good); guard.LoadConstant(42); guard.Return();
        var main = assembly.AddFunction("Main"); assembly.EntryPoint = main;
        main.Emit(OpCode.Ldc_Bool, success); main.Call(guard); main.Return();
        return assembly;
    }
    internal static void Run()
    {
        foreach (var success in new[] { false, true })
        {
            var assembly = Create(success);
            var context = new AssemblyLoadContext("terminal-failure", true);
            try
            {
                var loaded = context.LoadFromStream(new MemoryStream(assembly.Write()));
                try
                {
                    var value = loaded.EntryPoint!.Invoke(null, null);
                    if (!success || !Equals(value, 42)) throw new Exception("failure returned or wrong success value");
                }
                catch (TargetInvocationException e) when (!success && e.InnerException is InvalidOperationException failure && failure.Message == Message) { }
            }
            finally { context.Unload(); }
            RuntimeAssemblyContainer.ReadCliProjection(RuntimeAssemblyContainer.WriteBinary(assembly.WriteNativeAssembly(), assembly.CoreLibrary));
        }
        var invalid = Create(true); var main = invalid.EntryPoint!;
        main.ClearBody(); main.LoadConstant(1); main.Fail("not empty"); Reject(invalid);
        main.ClearBody(); main.Fail("terminal"); main.LoadConstant(42); main.Return(); Reject(invalid);
        main.ClearBody(); main.Emit(OpCode.Fail, ""); invalid.Write(); invalid.WriteNativeAssembly();
        var output = invalid.AddFunction("Output", new MethodSignature(PrimitiveType.Void, [SignatureType.ByReference(PrimitiveType.Int32)], outParameters: [0]));
        output.Fail("no normal return"); invalid.Write(); invalid.WriteNativeAssembly();
        try { main.Emit(OpCode.Fail, "\ud800"); throw new Exception("invalid Unicode accepted"); } catch (ArgumentException) { }
    }
    static void Reject(AssemblyBuilder assembly)
    {
        try { assembly.Write(); throw new Exception("invalid terminal body accepted"); } catch (InvalidDataException) { }
        try { assembly.WriteNativeAssembly(); throw new Exception("invalid native terminal body accepted"); } catch (InvalidDataException) { }
    }
    internal static async Task RunRuntime(string runtime, string directory)
    {
        Run(); Directory.CreateDirectory(directory);
        foreach (var success in new[] { false, true })
        {
            var assembly = Create(success);
            var path = Path.Combine(directory, success ? "Success.dll" : "Failure.dll");
            File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(assembly.WriteNativeAssembly(), assembly.CoreLibrary));
            foreach (var command in new[] { "verify", "run" })
            {
                var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
                start.ArgumentList.Add(command); start.ArgumentList.Add(path);
                using var process = Process.Start(start)!;
                var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
                await process.WaitForExitAsync(); var text = await stdout + await stderr;
                if (command == "verify" ? process.ExitCode != 0 : success ? process.ExitCode != 42 : process.ExitCode == 0 || !text.Contains(Message))
                    throw new Exception(command + ": " + text);
            }
        }
        Console.WriteLine("CLI/native success 42 and terminal Unicode failure passed");
    }
}
