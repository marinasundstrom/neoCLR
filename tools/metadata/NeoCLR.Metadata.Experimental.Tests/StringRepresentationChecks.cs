using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class StringRepresentationChecks
{
    private static void CheckSourceRoot(bool faultInBase = false)
    {
        var graph = new AssemblyBuilder(new("SourceString", new(1, 0, 0, 0)), new("System.Runtime", new(10, 0, 0, 0)));
        var root = graph.AddNativeObjectRoot();
        var rootConstructor = root.AddConstructor([], MethodVisibility.Protected);
        if (faultInBase)
        {
            var baseBody = rootConstructor.GetILGenerator();
            baseBody.LoadConstant(1); baseBody.LoadConstant(0); baseBody.Emit(OpCode.Div); baseBody.Emit(OpCode.Pop);
        }
        rootConstructor.GetILGenerator().Return();
        var display = root.AddNativeObjectSlot("ToString", new(PrimitiveType.String, []));
        display.GetILGenerator().Emit(OpCode.Ldstr, "object"); display.GetILGenerator().Return();
        var equals = root.AddNativeObjectSlot("Equals", new(PrimitiveType.Boolean, [root]));
        equals.GetILGenerator().Emit(OpCode.Ldc_Bool, false); equals.GetILGenerator().Return();
        var hash = root.AddNativeObjectSlot("GetHashCode", new(PrimitiveType.Int32, []));
        hash.GetILGenerator().LoadConstant(0); hash.GetILGenerator().Return();
        var text = graph.AddClass("System", "String", root);
        text.SetNativePrimitive(PrimitiveType.String);
        var constructor = text.AddConstructor(new MethodSignature(PrimitiveType.Void, [PrimitiveType.String]));
        var il = constructor.GetILGenerator();
        il.LoadArgument(0); il.Call(rootConstructor);
        il.LoadArgument(1); il.Emit(OpCode.Starg, 0); il.Return();
        var main = graph.AddFunction("Main");
        il = main.GetILGenerator(); il.Emit(OpCode.Ldstr, "source text"); il.NewObject(constructor); il.IsNull();
        var fail = il.DefineLabel(); il.Emit(OpCode.Brtrue, fail); il.LoadConstant(42); il.Return();
        il.MarkLabel(fail); il.LoadConstant(1); il.Return();
        var bytes = RuntimeAssemblyContainer.WriteLibraryBinary(graph);
        var loaded = AssemblyDefinition.ReadNativeAssembly(bytes);
        var value = loaded.MainModule.Types.Single(t => t.Name == "String");
        if (value.NativePrimitive != PrimitiveType.String || value.BaseType?.Resolve() != loaded.MainModule.Types.Single(t => t.Name == "Object"))
            throw new Exception("String source Object base lost");
        if (Environment.GetEnvironmentVariable("NEOCLR_SOURCE_STRING_ARTIFACT") is { } path)
        {
            File.WriteAllBytes(path, bytes);
            var app = new AssemblyBuilder(new("SourceStringConsumer", new(1, 0, 0, 0)), graph.CoreLibrary);
            var call = app.CreateFunctionReference(graph.Identity, graph.CoreLibrary,
                Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(bytes)), "", "Main", new(PrimitiveType.Int32, []));
            var entry = app.AddFunction("Main"); app.EntryPoint = entry;
            entry.GetILGenerator().Call(call); entry.GetILGenerator().Return();
            File.WriteAllBytes(path + ".app", RuntimeAssemblyContainer.WriteBinary(app));
            File.WriteAllText(path + ".neoil", ".module System\n.references ()\n");
        }
    }

    internal static async Task RunRuntime(string runtime, string output)
    {
        Directory.CreateDirectory(output);
        var library = Path.Combine(Path.GetFullPath(output), "SourceString.dll");
        var previous = Environment.GetEnvironmentVariable("NEOCLR_SOURCE_STRING_ARTIFACT");
        try
        {
            Environment.SetEnvironmentVariable("NEOCLR_SOURCE_STRING_ARTIFACT", library);
            CheckSourceRoot();
        }
        finally { Environment.SetEnvironmentVariable("NEOCLR_SOURCE_STRING_ARTIFACT", previous); }
        foreach (var mode in new[] { "verify", "run", "unselected", "base-body" })
        {
            if (mode == "base-body")
            {
                Environment.SetEnvironmentVariable("NEOCLR_SOURCE_STRING_ARTIFACT", library + ".fault");
                try { CheckSourceRoot(faultInBase: true); }
                finally { Environment.SetEnvironmentVariable("NEOCLR_SOURCE_STRING_ARTIFACT", previous); }
            }
            var selectedLibrary = mode == "base-body" ? library + ".fault" : library;
            var start = new System.Diagnostics.ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var argument in new[] { mode == "unselected" ? "verify" : mode == "base-body" ? "run" : mode, selectedLibrary + ".app", "--system", selectedLibrary + ".neoil", "--module", selectedLibrary })
                start.ArgumentList.Add(argument);
            if (mode != "unselected") { start.ArgumentList.Add("--object-root"); start.ArgumentList.Add(selectedLibrary); }
            using var process = System.Diagnostics.Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync();
            var text = await stdout; var error = await stderr;
            File.WriteAllText(Path.Combine(output, mode + ".log"), text + error);
            if (mode == "base-body" ? process.ExitCode == 0 || !error.Contains("division by zero", StringComparison.OrdinalIgnoreCase) :
                mode == "unselected" ? process.ExitCode == 0 :
                process.ExitCode != (mode == "run" ? 42 : 0) || error.Length != 0 || mode == "run" && text.Length != 0)
                throw new Exception("source String " + mode + ": " + text + error);
        }
        Console.WriteLine("PASS source String/Object: native PE verify/run returns 42; unselected root rejects; base constructor body executes");
    }

    internal static void Run()
    {
        CheckSourceRoot();

        var graph = new AssemblyBuilder(new("StringImplementation", new Version(1, 0, 0, 0)), new("System.Runtime", new Version(10, 0, 0, 0)));
        var text = graph.AddClass("System", "String");
        text.SetNativePrimitive(PrimitiveType.String);
        var constructor = text.AddConstructor(new MethodSignature(PrimitiveType.Void, [PrimitiveType.String]));
        constructor.GetILGenerator().LoadArgument(1);
        constructor.GetILGenerator().Emit(OpCode.Starg, 0);
        constructor.GetILGenerator().Return();
        var identity = text.AddInstanceMethod("Identity", new(PrimitiveType.String, []));
        identity.GetILGenerator().LoadArgument(0);
        identity.GetILGenerator().Return();
        var contract = graph.AddInterface("Example", "TextCount");
        var getter = contract.AddInterfaceMethod("get_Count", new(PrimitiveType.Int32, []));
        text.AddInterfaceImplementation(contract);
        var implementation = text.AddInstanceMethod("TextCount.get_Count", new(PrimitiveType.Int32, []), MethodVisibility.Private);
        implementation.AddExplicitInterfaceImplementation(contract, "get_Count");
        implementation.GetILGenerator().LoadConstant(42); implementation.GetILGenerator().Return();
        var main = graph.AddFunction("Main", new(PrimitiveType.Int32, []));
        graph.EntryPoint = main;
        var il = main.GetILGenerator();
        il.Emit(OpCode.Ldstr, "grapheme"); il.NewObject(constructor); il.Call(identity); il.IsNull();
        var fail = il.DefineLabel(); il.Emit(OpCode.Brtrue, fail); il.Emit(OpCode.Ldstr, "count"); il.CastReference(contract); il.CallVirtual(getter); il.Return();
        il.MarkLabel(fail); il.LoadConstant(1); il.Return();
        var bytes = RuntimeAssemblyContainer.WriteBinary(graph);
        var loaded = AssemblyDefinition.ReadNativeAssembly(bytes).MainModule.Types.Single(t => t.Name == "String");
        if (loaded.IsValueType || loaded.NativePrimitive != PrimitiveType.String) throw new Exception("String reference representation lost");
        if (Environment.GetEnvironmentVariable("NEOCLR_STRING_ARTIFACT") is { } path)
        {
            File.WriteAllBytes(path, bytes);
            File.WriteAllText(path + ".seed", "{\"format\":5,\"name\":\"System\",\"functions\":[]}");
        }
        graph.EntryPoint = null;
        var library = RuntimeAssemblyContainer.WriteBinary(graph);
        var consumer = new AssemblyBuilder(new("StringConsumer", new Version(1, 0, 0, 0)), graph.CoreLibrary);
        var reference = consumer.CreateTypeReference(graph.Identity, graph.CoreLibrary,
            Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(library)), "System", "String");
        consumer.SetNativePrimitive(reference, PrimitiveType.String);
        var imported = consumer.CreateMethodReference(reference, "Identity", new(PrimitiveType.String, []), nativePrimitive: PrimitiveType.String);
        var importedConstructor = consumer.CreateMethodReference(reference, ".ctor", new(PrimitiveType.Void, [PrimitiveType.String]), nativePrimitive: PrimitiveType.String);
        if (!loaded.Methods.Any(m => m.Name == ".ctor" && !m.IsStatic)) throw new Exception("String constructor lost");
        var entry = consumer.AddFunction("Main", new(PrimitiveType.Int32, [])); consumer.EntryPoint = entry;
        var body = entry.GetILGenerator();
        body.Emit(OpCode.Ldstr, "external"); body.NewObject(importedConstructor); body.Call(imported); body.IsNull();
        var missing = body.DefineLabel(); body.Emit(OpCode.Brtrue, missing); body.LoadConstant(42); body.Return();
        body.MarkLabel(missing); body.LoadConstant(1); body.Return();
        var application = RuntimeAssemblyContainer.WriteBinary(consumer);
        _ = AssemblyDefinition.ReadNativeAssembly(application);
        if (Environment.GetEnvironmentVariable("NEOCLR_STRING_ARTIFACT") is { } externalPath)
        {
            File.WriteAllBytes(externalPath + ".library", library);
            File.WriteAllBytes(externalPath + ".consumer", application);
        }
        var manual = new TypeDefinition("System", "String", 1,
            consumer.Definition.MainModule.ImportReference(consumer.CoreLibrary, "System", "Object"));
        manual.SetNativePrimitive(PrimitiveType.String);
        if (manual.NativePrimitive != text.NativePrimitive || manual.IsValueType) throw new Exception("definition/builder parity");
        try { graph.AddValueType("System", "String").SetNativePrimitive(PrimitiveType.String); }
        catch (ArgumentException) { return; }
        throw new Exception("String value representation accepted");
    }
}
