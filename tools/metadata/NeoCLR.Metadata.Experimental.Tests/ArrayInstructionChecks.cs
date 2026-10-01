using System.Diagnostics;
using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class ArrayInstructionChecks
{
    internal static AssemblyBuilder Create(string scenario)
    {
        var graph = ArraySignatureChecks.Create();
        var main = graph.EntryPoint!;
        main.ClearBody();
        if (scenario is "negative" or "bounds" or "empty")
        {
            main.LoadConstant(scenario == "negative" ? -1 : 0); main.NewArray(PrimitiveType.Int32);
            if (scenario == "bounds") { main.LoadConstant(0); main.LoadArrayElement(PrimitiveType.Int32); }
            else main.LoadArrayLength();
            main.Return(); return graph;
        }
        // Exercise primitive tokens and default initialization without relying on CLI stack coalescing.
        foreach (var element in new[] { PrimitiveType.Int32, PrimitiveType.Int64, PrimitiveType.Boolean, PrimitiveType.String })
        {
            var slot = main.DeclareLocal(SignatureType.ArrayOf(element));
            main.LoadConstant(1); main.Emit(OpCode.Newarr, (SignatureType)element); main.StoreLocal(slot);
            main.LoadLocal(slot); main.LoadConstant(0);
            if (element == PrimitiveType.Int64) main.Emit(OpCode.Ldc_I8, 9223372036854775806L);
            else if (element == PrimitiveType.Boolean) main.Emit(OpCode.Ldc_Bool, true);
            else if (element == PrimitiveType.String) main.Emit(OpCode.Ldstr, "array λ");
            else main.LoadConstant(42);
            main.Emit(OpCode.Stelem, (SignatureType)element);
            main.LoadLocal(slot); main.LoadConstant(0); main.Emit(OpCode.Ldelem, (SignatureType)element); main.Emit(OpCode.Pop);
        }
        var order = graph.Types[0];
        var array = main.DeclareLocal(SignatureType.ArrayOf(order));
        main.LoadConstant(2); main.NewArray(order); main.StoreLocal(array);
        main.LoadLocal(array); main.LoadConstant(0); main.LoadConstant(1); main.Emit(OpCode.Ldc_Bool, true); main.NewObject(order.Methods[0]); main.StoreArrayElement(order);
        var alias = main.DeclareLocal(SignatureType.ArrayOf(order));
        main.LoadLocal(array); main.Call(graph.Functions.Single(m => m.Name == "IdentityArray" && m.Signature.ReturnType.ArrayElement!.ClassType == order)); main.StoreLocal(alias);
        main.LoadLocal(alias); main.LoadConstant(0); main.LoadArrayElement(order); main.LoadConstant(40); main.Call(order.Methods[3]);
        main.LoadLocal(array); main.LoadConstant(0); main.LoadArrayElement(order); main.Call(order.Methods[1]);
        main.LoadLocal(alias); main.Emit(OpCode.Ldlen); main.Emit(OpCode.Conv_I4); main.Emit(OpCode.Add); main.Return();
        return graph;
    }

    internal static void Run()
    {
        foreach (var scenario in new[] { "valid", "empty", "negative", "bounds" })
        {
            var loaded = Assembly.Load(Create(scenario).Write());
            try
            {
                var result = loaded.EntryPoint!.Invoke(null, null);
                if (!Equals(result, scenario == "valid" ? 42 : 0) || scenario is "negative" or "bounds") throw new Exception("array execution " + scenario);
            }
            catch (TargetInvocationException e) when (scenario == "negative" && e.InnerException is OverflowException || scenario == "bounds" && e.InnerException is IndexOutOfRangeException) { }
        }
        var bad = Create("empty"); var entry = bad.EntryPoint!; entry.ClearBody();
        entry.LoadConstant(1); entry.NewArray(PrimitiveType.Int32); entry.LoadConstant(0); entry.LoadArrayElement(PrimitiveType.Boolean); entry.Emit(OpCode.Pop); entry.LoadConstant(0); entry.Return();
        try { bad.Write(); throw new Exception("wrong array element accepted"); } catch (InvalidDataException) { }
        entry.ClearBody(); entry.LoadConstant(1); entry.NewArray(PrimitiveType.Int32); entry.Emit(OpCode.Ldlen); entry.Return();
        try { bad.Write(); throw new Exception("native-size array length accepted as Int32"); } catch (InvalidDataException) { }
    }

    internal static async Task RunRuntime(string runtime, string output)
    {
        Run();
        if (Directory.Exists(output)) throw new IOException("output must be fresh");
        Directory.CreateDirectory(output);
        foreach (var scenario in new[] { "valid", "empty", "negative", "bounds" })
        {
            var graph = Create(scenario);
            var path = Path.Combine(output, scenario + ".dll");
            File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
            foreach (var command in new[] { "verify", "run" })
            {
                var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
                start.ArgumentList.Add(command); start.ArgumentList.Add(path);
                using var process = Process.Start(start)!;
                var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
                await process.WaitForExitAsync(); var text = await stdout + await stderr;
                var expected = command == "verify" || scenario == "empty" ? 0 : scenario == "valid" ? 42 : -1;
                if (expected >= 0 ? process.ExitCode != expected : process.ExitCode == 0 || !text.Contains(scenario == "negative" ? "non-negative" : "IndexOutOfRange", StringComparison.OrdinalIgnoreCase))
                    throw new Exception(scenario + " " + command + ": " + text);
            }
        }
        Console.WriteLine("PASS binary arrays: primitive/nominal storage and aliasing 42, empty 0, negative length and bounds fault");
    }
}
