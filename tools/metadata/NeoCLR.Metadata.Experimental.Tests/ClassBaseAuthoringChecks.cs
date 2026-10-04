using System.Diagnostics;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class ClassBaseAuthoringChecks
{
    private static AssemblyBuilder Create(bool manual = false, bool missingChain = false, bool duplicateChain = false, bool protectedConstructor = false)
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var graph = new AssemblyBuilder(new("AuthoredBases" + Guid.NewGuid().ToString("N"), new(1, 0, 0, 0)), core);
        var parent = graph.AddClass("Example", "Base");
        var number = parent.AddField("Number", PrimitiveType.Int32, FieldVisibility.Public);
        MethodBuilder initialize;
        if (manual && protectedConstructor)
        {
            var definition = new MethodDefinition(".ctor", 4, new MethodSignature(PrimitiveType.Void, [PrimitiveType.Int32]));
            parent.Definition.Methods.Add(definition);
            initialize = MethodBuilder.ForDefinition(definition);
        }
        else initialize = parent.AddConstructor([PrimitiveType.Int32], protectedConstructor ? MethodVisibility.Protected : MethodVisibility.Public);
        var il = initialize.GetILGenerator(); il.LoadArgument(0); il.LoadArgument(1); il.StoreField(number); il.Return();
        TypeBuilder child;
        if (manual)
        {
            var definition = new TypeDefinition("Example", "Derived", 1, parent.Definition.ToReference());
            graph.Definition.MainModule.Types.Add(definition);
            child = graph.Types.Single(t => t.Name == "Derived");
        }
        else child = graph.AddClass("Example", "Derived", parent);
        var extra = child.AddField("Extra", PrimitiveType.Int32, FieldVisibility.Public);
        var construct = child.AddConstructor([PrimitiveType.Int32]);
        il = construct.GetILGenerator();
        if (!missingChain) { il.LoadArgument(0); il.LoadArgument(1); il.Call(initialize); }
        if (duplicateChain) { il.LoadArgument(0); il.LoadArgument(1); il.Call(initialize); }
        if (!missingChain && !duplicateChain) { il.LoadArgument(0); il.LoadConstant(2); il.StoreField(extra); }
        il.Return();
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        il = main.GetILGenerator(); il.LoadConstant(40); il.NewObject(construct); il.Duplicate(); il.LoadField(extra);
        var saved = il.DeclareLocal(PrimitiveType.Int32); il.StoreLocal(saved); il.LoadField(number); il.LoadLocal(saved); il.Add(); il.Return();
        return graph;
    }

    internal static void Run()
    {
        foreach (var manual in new[] { false, true })
        foreach (var family in new[] { false, true })
        {
            var graph = Create(manual, protectedConstructor: family);
            var assembly = System.Reflection.Assembly.Load(graph.Write());
            var parent = assembly.GetType("Example.Base")!;
            var child = assembly.GetType("Example.Derived")!;
            var value = Activator.CreateInstance(child, 42)!;
            if (child.BaseType != parent || (int)parent.GetField("Number")!.GetValue(value)! != 42)
                throw new Exception("CLI base constructor/field initialization failed");
            var constructor = parent.GetConstructors(System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Instance).Single();
            if (constructor.IsFamily != family) throw new Exception("CLI constructor visibility lost");
            var native = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(graph));
            if (native.MainModule.Types.Single(t => t.Name == "Derived").BaseType!.Resolve().Name != "Base")
                throw new Exception("native authored base round trip failed");
            var context = new MetadataLoadContext([native]);
            var callable = context.Assemblies.Single().GetTypes().Single(t => t.Name == "Base").GetConstructors().Single();
            if (callable.Accessibility != (family ? MetadataAccessibility.Family : MetadataAccessibility.Public))
                throw new Exception("native facade constructor visibility lost");
        }
        foreach (var invalid in new[] { Create(missingChain: true), Create(duplicateChain: true) })
        {
            Reject(() => invalid.Write());
            Reject(() => invalid.WriteNativeAssembly());
        }
        var denied = Create(protectedConstructor: true);
        var forbidden = denied.AddFunction("Bad").GetILGenerator();
        forbidden.LoadConstant(1); forbidden.NewObject(denied.Types[0].Methods.Single(m => m.IsConstructor));
        forbidden.Emit(OpCode.Pop); forbidden.LoadConstant(0); forbidden.Return();
        Reject(() => denied.Write()); Reject(() => denied.WriteNativeAssembly());
        var beforeMethods = denied.Types[0].Methods.Count;
        try { denied.Types[0].AddInstanceMethod("Bad", new(PrimitiveType.Void, []), MethodVisibility.Protected); throw new Exception("protected ordinary method accepted"); }
        catch (ArgumentException) { }
        if (denied.Types[0].Methods.Count != beforeMethods) throw new Exception("invalid protected declaration mutated owner");
        try { _ = new MethodDefinition("Bad", 4, new(PrimitiveType.Void, [])); throw new Exception("manual protected method accepted"); }
        catch (ArgumentException) { }
        var graph2 = Create();
        var before = graph2.Types.Count;
        try { graph2.AddClass("Example", "Foreign", Create().Types[0]); throw new Exception("foreign base accepted"); }
        catch (ArgumentException) { }
        if (graph2.Types.Count != before) throw new Exception("failed declaration mutated ownership");
    }

    private static void Reject(Action action)
    {
        try { action(); } catch (InvalidDataException) { return; }
        throw new Exception("invalid constructor initialization accepted");
    }

    internal static async Task RunRuntime(string runtime, string directory)
    {
        Run();
        Directory.CreateDirectory(directory);
        foreach (var family in new[] { false, true })
        {
        var path = Path.Combine(directory, family ? "ProtectedBases.dll" : "AuthoredBases.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(Create(protectedConstructor: family)));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add(command); start.ArgumentList.Add(path);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(30));
            try { await process.WaitForExitAsync(timeout.Token); } catch { process.Kill(true); throw; }
            var output = await stdout; var error = await stderr;
            if (process.ExitCode != (command == "run" ? 42 : 0) || error.Length != 0)
                throw new Exception(output + error);
        }
        }
        Console.WriteLine("PASS public/protected authored derived constructor and inherited field: CLI execution and native PE verify/run return 42");
    }
}
