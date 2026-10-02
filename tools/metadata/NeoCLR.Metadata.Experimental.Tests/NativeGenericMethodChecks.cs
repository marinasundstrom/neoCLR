using System.Diagnostics;
using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class NativeGenericMethodChecks
{
    private static (AssemblyBuilder Library, AssemblyBuilder App, byte[] Image) Create(bool binary)
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var library = new AssemblyBuilder(new("NativeGenericLibrary", new Version(1, 0, 0, 0)), core);
        var parameter = SignatureType.MethodParameter(0);
        var identity = library.AddType("Example", "Functions").AddMethod("Identity", new MethodSignature(parameter, [parameter], ["TItem"]));
        identity.LoadArgument(0); identity.Return();
        var vector = SignatureType.ArrayOf(parameter);
        var array = library.AddFunction("Example", "ArrayIdentity", new MethodSignature(vector, [vector], ["TElement"]));
        array.LoadArgument(0); array.Return();
        var constant = identity.DeclaringType!.AddMethod("Constant", new MethodSignature(PrimitiveType.Int32, [], ["TUnused"]));
        constant.LoadConstant(42); constant.Return();
        var native = library.WriteNativeAssembly();
        var image = binary ? RuntimeAssemblyContainer.WriteBinary(native, core) : RuntimeAssemblyContainer.Write(native, core);
        var read = AssemblyDefinition.ReadNativeAssembly(image);
        var method = read.MainModule.Types.Single().Methods.Single(m => m.Name == "Identity");
        Check(method.GenericArity == 1 && method.TryGetSignature(out var signature) &&
            signature!.GenericParameterNames.SequenceEqual(new[] { "TItem" }) &&
            signature.ReturnType.MethodParameterIndex == 0 && signature.ParameterTypes[0] == signature.ReturnType, "generic method signature");
        Check(method.TryGetStaticGenericValueSignature(out var genericSignature) && genericSignature!.GenericParameterNames[0] == "TItem", "generic helper");
        var readConstant = read.MainModule.Types.Single().Methods.Single(m => m.Name == "Constant");
        Check(!readConstant.TryGetStaticPrimitiveSignature(out _) && !readConstant.TryGetStaticValueSignature(out _) &&
            !readConstant.TryGetStaticInt32Signature(out _, out _) && readConstant.TryGetStaticGenericValueSignature(out _), "unused generic parameter retains arity");
        var function = read.MainModule.Functions.Single();
        Check(function.GenericArity == 1 && function.TryGetSignature(out var arraySignature) &&
            arraySignature!.GenericParameterNames[0] == "TElement" && arraySignature.ReturnType.ArrayElement!.MethodParameterIndex == 0, "generic namespace vector signature");
        Check(ReferenceEquals(method, read.MainModule.GetMethodDefinition(method.MetadataToken)), "canonical generic definition");
        Check(read.Write().SequenceEqual(image), "opaque generic image roundtrip");
        var app = new AssemblyBuilder(new("NativeGenericApp", new Version(1, 0, 0, 0)), core);
        var imported = app.ImportReference(method, core);
        Reject<ArgumentException>(() => imported.MakeGenericInstance());
        Reject<ArgumentException>(() => imported.MakeGenericInstance(PrimitiveType.Void));
        Check(imported.Signature.GenericParameterNames[0] == "TItem", "import preserves generic names");
        Check(ReferenceEquals(imported, app.ImportReference(method, core)), "generic reference interning");
        var importedArray = app.ImportReference(function, core).MakeGenericInstance(PrimitiveType.Int32);
        var main = app.AddFunction("Main"); app.EntryPoint = main;
        var values = main.DeclareLocal(SignatureType.ArrayOf(PrimitiveType.Int32));
        main.LoadConstant(1); main.NewArray(PrimitiveType.Int32); main.StoreLocal(values);
        main.LoadLocal(values); main.LoadConstant(0); main.LoadConstant(42); main.StoreArrayElement(PrimitiveType.Int32);
        main.LoadLocal(values); main.Call(importedArray); main.LoadConstant(0); main.LoadArrayElement(PrimitiveType.Int32);
        main.Call(imported.MakeGenericInstance(PrimitiveType.Int32)); main.Return();
        _ = app.WriteNativeAssembly();
        return (library, app, image);
    }
    internal static void Run()
    {
        foreach (var binary in new[] { false, true })
        {
            var (library, _, _) = Create(binary);
            // CLI global call references remain intentionally unsupported. Execute the
            // same imported static generic contract in a separate CLI consumer.
            var read = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), library.CoreLibrary));
            var cli = new AssemblyBuilder(new("GenericCliConsumer", new Version(1, 0, 0, 0)), library.CoreLibrary);
            var imported = cli.ImportReference(read.MainModule.Types.Single().Methods.Single(m => m.Name == "Identity"), library.CoreLibrary);
            var main = cli.AddFunction("Main"); cli.EntryPoint = main;
            main.LoadConstant(42); main.Call(imported.MakeGenericInstance(PrimitiveType.Int32)); main.Return();
            var context = new AssemblyLoadContext("native-generic-" + binary, true);
            try
            {
                context.LoadFromStream(new MemoryStream(library.Write()));
                Check(Equals(42, context.LoadFromStream(new MemoryStream(cli.Write())).EntryPoint!.Invoke(null, null)), "CLR generic imported execution");
            }
            finally { context.Unload(); }
        }
    }
    internal static async Task RunRuntime(string runtime, string directory)
    {
        Run(); Directory.CreateDirectory(directory);
        foreach (var binary in new[] { false, true })
        {
            var (_, app, image) = Create(binary);
            var dependency = Path.Combine(directory, "Library-" + binary + ".dll");
            var path = Path.Combine(directory, "App-" + binary + ".dll");
            File.WriteAllBytes(dependency, image);
            File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(app.WriteNativeAssembly(), app.CoreLibrary));
            foreach (var command in new[] { "verify", "run" })
            {
                var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
                foreach (var arg in new[] { command, path, "--module", dependency }) start.ArgumentList.Add(arg);
                using var process = Process.Start(start)!;
                var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
                using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(30));
                try { await process.WaitForExitAsync(timeout.Token); }
                catch { process.Kill(entireProcessTree: true); throw; }
                var text = await stdout + await stderr;
                Check(process.ExitCode == (command == "verify" ? 0 : 42), command + ": " + text);
            }
        }
        Console.WriteLine("Native generic definition import, emission and execution: 42 (both containers)");
    }
    private static void Reject<T>(Action action) where T : Exception
    { try { action(); } catch (T) { return; } throw new Exception("expected " + typeof(T).Name); }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
}
