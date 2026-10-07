using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental.Model;

internal static class ArrayBackingChecks
{
    static AssemblyBuilder Create(bool sourceRoot = false)
    {
        var host = typeof(object).Assembly.GetName();
        var graph = new AssemblyBuilder(new("ArrayBacking", new Version(1, 0, 0, 0)),
            new(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!)));
        var objectRoot = sourceRoot ? graph.AddNativeObjectRoot() : null;
        if (objectRoot is not null)
        {
            var display = objectRoot.AddNativeObjectSlot("ToString", new(PrimitiveType.String, []));
            display.GetILGenerator().Emit(OpCode.Ldstr, "array"); display.GetILGenerator().Return();
            var equals = objectRoot.AddNativeObjectSlot("Equals", new(PrimitiveType.Boolean, [objectRoot]));
            equals.GetILGenerator().Emit(OpCode.Ldc_Bool, false); equals.GetILGenerator().Return();
            var hash = objectRoot.AddNativeObjectSlot("GetHashCode", new(PrimitiveType.Int32, []));
            hash.GetILGenerator().LoadConstant(0); hash.GetILGenerator().Return();
        }
        var shape = objectRoot is null ? graph.AddGenericClass("Example", "Vector", ["T"])
            : graph.AddGenericClass("Example", "Vector", ["T"], objectRoot);
        shape.AddField("storage", SignatureType.ArrayOf(SignatureType.TypeParameter(0)));
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        var il = main.GetILGenerator();
        var vector = il.DeclareLocal(SignatureType.ArrayOf(PrimitiveType.Int32));
        var alias = il.DeclareLocal(SignatureType.ArrayOf(PrimitiveType.Int32));
        il.LoadConstant(1); il.NewArray(PrimitiveType.Int32); il.StoreLocal(vector);
        il.LoadLocal(vector); il.StoreLocal(alias);
        il.LoadLocal(alias); il.LoadConstant(0); il.LoadConstant(42); il.Emit(OpCode.Stelem, PrimitiveType.Int32);
        il.LoadLocal(vector); il.LoadConstant(0); il.Emit(OpCode.Ldelem, PrimitiveType.Int32); il.Return();
        graph.SetArrayBacking(shape);
        return graph;
    }

    internal static void Run()
    {
        _ = NativeAssemblyDefinition.ReadAssembly(Create(true).WriteNativeAssembly());
        var graph = Create();
        var shape = graph.Types[0];
        graph.SetArrayBacking(shape);
        var bytes = graph.WriteNativeAssembly();
        _ = NativeAssemblyDefinition.ReadAssembly(bytes);
        var root = JsonNode.Parse(bytes)!;
        var selection = root["assemblies"]![0]!["array_backing"]!;
        if (selection["index"]!.GetValue<int>() != 0) throw new Exception("wrong backing identity");
        void Reject(Action action)
        {
            try { action(); } catch (Exception e) when (e is ArgumentException or InvalidOperationException or InvalidDataException) { return; }
            throw new Exception("invalid array backing accepted");
        }
        selection["revision"] = "wrong";
        Reject(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(root.ToJsonString())));
        root = JsonNode.Parse(bytes)!;
        root["types"]![0]!["fields"]![0]!["visibility"] = "public";
        Reject(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(root.ToJsonString())));
        var derived = JsonNode.Parse(Create(true).WriteNativeAssembly())!;
        derived["types"]![1]!["base"] = new JsonObject { ["Named"] = "Other" };
        Reject(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(derived.ToJsonString())));
        var foreign = new AssemblyBuilder(new("Foreign", new Version(1, 0, 0, 0)), graph.CoreLibrary);
        Reject(() => foreign.SetArrayBacking(shape));
        var other = graph.AddGenericClass("Example", "Other", ["T"]);
        other.AddField("storage", SignatureType.ArrayOf(SignatureType.TypeParameter(0)));
        Reject(() => graph.SetArrayBacking(other));
        var invalid = graph.AddGenericClass("Example", "Invalid", ["T"]);
        invalid.AddField("storage", PrimitiveType.Int32);
        Reject(() => graph.SetArrayBacking(invalid));
        shape.AddField("extra", PrimitiveType.Int32);
        Reject(() => graph.WriteNativeAssembly());
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        Run(); Directory.CreateDirectory(output);
        foreach (var mode in new[] { "valid", "missing", "storage", "duplicate", "base" })
        {
            var graph = Create();
            var root = JsonNode.Parse(graph.WriteNativeAssembly())!;
            if (mode == "missing") root["assemblies"]![0]!["array_backing"]!["index"] = 999;
            if (mode == "storage") root["types"]![0]!["fields"]![0]!["ty"] = "Int32";
            if (mode == "base") root["types"]![0]!["base"] = new JsonObject { ["Named"] = "System.Object" };
            if (mode == "duplicate") root["assemblies"]!.AsArray().Add(root["assemblies"]![0]!.DeepClone());
            var path = Path.Combine(output, mode + ".json");
            File.WriteAllText(path, root.ToJsonString());
            var start = new System.Diagnostics.ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add("verify"); start.ArgumentList.Add(path);
            using var process = System.Diagnostics.Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (mode == "valid" ? process.ExitCode != 0 : process.ExitCode == 0 || !text.Contains("nominal array backing"))
                throw new Exception(mode + ": " + text);
        }
        var source = Create(true);
        source.EntryPoint = null;
        var libraryBytes = NeoCLR.Metadata.Experimental.RuntimeAssemblyContainer.WriteLibraryBinary(source);
        var library = Path.Combine(output, "SourceArrays.dll"); File.WriteAllBytes(library, libraryBytes);
        var app = new AssemblyBuilder(new("ArrayConsumer", new Version(1, 0, 0, 0)), source.CoreLibrary);
        var reference = app.CreateFunctionReference(source.Identity, source.CoreLibrary,
            Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(libraryBytes)), "", "Main", new(PrimitiveType.Int32, []));
        var entry = app.AddFunction("Main"); app.EntryPoint = entry;
        entry.GetILGenerator().Call(reference); entry.GetILGenerator().Return();
        var consumer = Path.Combine(output, "Consumer.dll");
        File.WriteAllBytes(consumer, NeoCLR.Metadata.Experimental.RuntimeAssemblyContainer.WriteBinary(app));
        var seed = Path.Combine(output, "System.neoil"); File.WriteAllText(seed, ".module System\n.references ()\n");
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new System.Diagnostics.ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var argument in new[] { command, consumer, "--system", seed, "--module", library, "--object-root", library })
                start.ArgumentList.Add(argument);
            using var process = System.Diagnostics.Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync();
            var text = await stdout; var error = await stderr;
            if (process.ExitCode != (command == "run" ? 42 : 0) || error.Length != 0 || command == "run" && text.Length != 0)
                throw new Exception("source Object array " + command + ": " + text + error);
        }
        Console.WriteLine("PASS source Object array backing: PE verify/run, alias mutation returns 42");
        Console.WriteLine("PASS nominal backing validation: valid, missing, storage, duplicate, unselected base");
    }

}
