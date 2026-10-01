using NeoCLR.Metadata.Experimental.Model;

internal static class ArgumentStoreChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var graph = new AssemblyBuilder(new("ArgumentStores", new Version(1, 0, 0, 0)), core);
        var type = graph.AddType("", "Arguments");
        (PrimitiveType Type, object Original, object Replacement)[] cases = [
            (PrimitiveType.Int32, 1, 42), (PrimitiveType.Int64, 1L, 4294967338L),
            (PrimitiveType.Boolean, false, true), (PrimitiveType.String, "before", "after 🌍")
        ];
        for (var i = 0; i < cases.Length; i++)
        {
            var item = cases[i];
            var method = type.AddMethod("Replace" + i, new(item.Type, [item.Type]));
            switch (item.Replacement)
            {
                case int number: method.LoadConstant(number); break;
                case long wide: method.Emit(OpCode.Ldc_I8, wide); break;
                case bool flag: method.Emit(OpCode.Ldc_Bool, flag); break;
                case string text: method.Emit(OpCode.Ldstr, text); break;
            }
            if (i % 2 == 0) method.StoreArgument(0); else method.Emit(OpCode.Starg, 0);
            method.LoadArgument(0); method.Return();
        }
        var last = type.AddMethod("Last", new(PrimitiveType.Int32, Enumerable.Repeat(PrimitiveType.Int32, 256)));
        last.LoadConstant(42); last.StoreArgument(255); last.LoadArgument(255); last.Return();
        var loaded = System.Reflection.Assembly.Load(graph.Write()).GetType("Arguments")!;
        for (var i = 0; i < cases.Length; i++)
        {
            object[] arguments = [cases[i].Original];
            if (!Equals(loaded.GetMethod("Replace" + i)!.Invoke(null, arguments), cases[i].Replacement) ||
                !Equals(arguments[0], cases[i].Original)) throw new Exception("by-value argument store contract");
        }
        if (!Equals(loaded.GetMethod("Last")!.Invoke(null, Enumerable.Repeat<object>(0, 256).ToArray()), 42))
            throw new Exception("last bounded argument slot");
        _ = graph.WriteNativeAssembly();
        var bad = type.AddMethod("Invalid", new(PrimitiveType.Int32, [PrimitiveType.Int32]));
        foreach (var slot in new[] { -1, 1, int.MaxValue })
        {
            bad.ClearBody(); bad.LoadConstant(42); bad.StoreArgument(slot); bad.LoadArgument(0); bad.Return();
            Reject(() => graph.Write()); Reject(() => graph.WriteNativeAssembly());
        }
        bad.ClearBody(); bad.StoreArgument(0); bad.LoadArgument(0); bad.Return(); Reject(() => graph.Write());
        bad.ClearBody(); bad.Emit(OpCode.Ldc_Bool, true); bad.StoreArgument(0); bad.LoadArgument(0); bad.Return();
        Reject(() => graph.WriteNativeAssembly());
        // An untouched parameter starts initialized and retains its declared type.
        bad.ClearBody(); bad.LoadArgument(0); bad.Return(); _ = graph.Write();
    }

    internal static async Task RunRuntime(string runtime, string directory)
    {
        if (Directory.Exists(directory)) throw new IOException("output directory must be fresh");
        Directory.CreateDirectory(directory);
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var graph = new AssemblyBuilder(new("NativeArgumentStores", new Version(1, 0, 0, 0)), core);
        var number = graph.AddFunction("Number", new(PrimitiveType.Int32, [PrimitiveType.Int32]));
        number.LoadConstant(42); number.StoreArgument(0); number.LoadArgument(0); number.Return();
        var wide = graph.AddFunction("Wide", new(PrimitiveType.Int64, [PrimitiveType.Int64]));
        wide.Emit(OpCode.Ldc_I8, 4294967296L); wide.StoreArgument(0); wide.LoadArgument(0); wide.Return();
        var flag = graph.AddFunction("Flag", new(PrimitiveType.Boolean, [PrimitiveType.Boolean]));
        flag.Emit(OpCode.Ldc_Bool, true); flag.StoreArgument(0); flag.LoadArgument(0); flag.Return();
        var text = graph.AddFunction("Text", new(PrimitiveType.String, [PrimitiveType.String]));
        text.Emit(OpCode.Ldstr, "after 🌍"); text.StoreArgument(0); text.LoadArgument(0); text.Return();
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        var original = main.DeclareInt32Local();
        main.LoadConstant(1); main.StoreLocal(original);
        main.Emit(OpCode.Ldstr, "before"); main.Call(text); main.WriteConsoleLine();
        main.Emit(OpCode.Ldc_Bool, false); main.Call(flag);
        var success = main.DefineLabel(); main.Emit(OpCode.Brtrue, success);
        main.LoadConstant(0); main.Return(); main.MarkLabel(success);
        main.LoadLocal(original); main.Call(number);
        main.LoadLocal(original); main.Subtract(); main.LoadConstant(1); main.Add();
        main.Emit(OpCode.Ldc_I8, -1L); main.Call(wide); main.Emit(OpCode.Conv_I4); main.Add(); main.Return();
        var image = NeoCLR.Metadata.Experimental.RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), core);
        var path = Path.Combine(directory, "ArgumentStores.dll"); File.WriteAllBytes(path, image);
        await Command(0, "verify", path);
        var executed = await Command(42, "run", path);
        if (executed.Trim() != "after 🌍") throw new Exception("native String parameter result");
        File.WriteAllText(Path.Combine(directory, "validation.json"), System.Text.Json.JsonSerializer.Serialize(new {
            date = "2026-10-01", producer = "independent .NET metadata API", result = 42,
            int32Int64BooleanStringStores = true, callerLocalPreserved = true,
            nativeBinaryLoadedAndVerified = true, runtimeSchemaChanged = false,
            ravenSourceParameterMutation = false,
            runtimeSha256 = Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(File.ReadAllBytes(runtime))).ToLowerInvariant(),
            assemblySha256 = Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(image)).ToLowerInvariant()
        }, new System.Text.Json.JsonSerializerOptions { WriteIndented = true }) + "\n");
        Console.WriteLine("PASS metadata API typed argument stores -> binary neoCLR load: 42");

        async Task<string> Command(int expected, params string[] arguments)
        {
            var start = new System.Diagnostics.ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var argument in arguments) start.ArgumentList.Add(argument);
            using var process = System.Diagnostics.Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            using var timeout = new CancellationTokenSource(TimeSpan.FromMinutes(1));
            try { await process.WaitForExitAsync(timeout.Token); }
            catch (OperationCanceledException) { process.Kill(true); throw; }
            var result = await stdout + await stderr;
            if (process.ExitCode != expected) throw new Exception($"expected exit {expected}, got {process.ExitCode}: {result}");
            return result;
        }
    }

    private static void Reject(Action action)
    {
        try { action(); } catch (InvalidDataException) { return; }
        throw new Exception("invalid argument store accepted");
    }
}
