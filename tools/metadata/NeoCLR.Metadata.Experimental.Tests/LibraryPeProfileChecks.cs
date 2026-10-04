using System.Diagnostics;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class LibraryPeProfileChecks
{
    private static AssemblyBuilder Create()
    {
        var graph = new AssemblyBuilder(new("LargeLibrary", new Version(1, 0, 0, 0)), new("System.Runtime", new Version(10, 0, 0, 0)));
        for (int i = 0; i < 64; i++)
        {
            var method = graph.AddFunction("Padding" + i, new MethodSignature(PrimitiveType.Void, []));
            var il = method.GetILGenerator();
            for (int j = 0; j < 4; j++)
            { il.Emit(OpCode.Ldstr, new string('x', 16384)); il.Emit(OpCode.Pop); }
            il.Return();
        }
        var answer = graph.AddFunction("Answer");
        answer.GetILGenerator().LoadConstant(42); answer.GetILGenerator().Return();
        return graph;
    }

    internal static void Run()
    {
        var graph = Create();
        bool rejected = false;
        try { _ = RuntimeAssemblyContainer.WriteBinary(graph); } catch (InvalidDataException) { rejected = true; }
        if (!rejected) throw new Exception("legacy application envelope unexpectedly accepted large library");
        var image = RuntimeAssemblyContainer.WriteLibraryBinary(graph);
        if (image.Length <= 4 * 1024 * 1024) throw new Exception("fixture does not exceed the old PE bound");
        var native = AssemblyDefinition.ReadNativeAssembly(image);
        if (native.MainModule.Functions.Count != 65 || !native.Write().SequenceEqual(image))
            throw new Exception("large native snapshot lost declarations or image ownership");
        if (RuntimeAssemblyContainer.ReadCliProjection(image).MainModule.Functions.Count != 65)
            throw new Exception("large reference projection lost declarations");
        rejected = false;
        try { _ = RuntimeAssemblyContainer.Read(new byte[RuntimeAssemblyContainer.MaxLibraryImageSize + 1]); }
        catch (InvalidDataException) { rejected = true; }
        if (!rejected) throw new Exception("library image budget was not enforced");
    }

    internal static async Task RunRuntime(string runtime, string directory)
    {
        Run();
        Directory.CreateDirectory(directory);
        var graph = Create();
        var libraryImage = RuntimeAssemblyContainer.WriteLibraryBinary(graph);
        var library = Path.Combine(directory, "LargeLibrary.dll");
        File.WriteAllBytes(library, libraryImage);
        var native = AssemblyDefinition.ReadNativeAssembly(libraryImage);
        var consumer = new AssemblyBuilder(new("LargeConsumer", new Version(1, 0, 0, 0)), graph.CoreLibrary);
        var method = consumer.ImportReference(native.MainModule.Functions.Single(m => m.Name == "Answer"), graph.CoreLibrary);
        var main = consumer.AddFunction("Main"); consumer.EntryPoint = main;
        main.GetILGenerator().Call(method); main.GetILGenerator().Return();
        var app = Path.Combine(directory, "LargeConsumer.dll");
        File.WriteAllBytes(app, RuntimeAssemblyContainer.WriteBinary(consumer));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var argument in new[] { command, app, "--module", library }) start.ArgumentList.Add(argument);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(60));
            try { await process.WaitForExitAsync(timeout.Token); } catch { process.Kill(true); throw; }
            var output = await stdout; var error = await stderr;
            if (process.ExitCode != (command == "run" ? 42 : 0) || error.Length != 0 || command == "run" && output.Length != 0)
                throw new Exception(output + error);
        }
        Console.WriteLine("PASS schema-3 PE library >4 MiB, native reimport and linked execution");
    }
}
