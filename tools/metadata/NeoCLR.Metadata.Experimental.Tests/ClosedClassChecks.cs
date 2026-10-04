using System.Diagnostics;
using System.Reflection.Metadata;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;
using TypeDefinition = NeoCLR.Metadata.Experimental.Model.TypeDefinition;
using MethodDefinition = NeoCLR.Metadata.Experimental.Model.MethodDefinition;
using System.Reflection.PortableExecutable;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class ClosedClassChecks
{
    private static AssemblyBuilder Create(bool manual)
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var graph = new AssemblyBuilder(new("ClosedFamily" + Guid.NewGuid().ToString("N"), new(1, 0, 0, 0)), core);
        TypeBuilder root;
        if (manual)
        {
            var definition = new TypeDefinition("Example", "Root", 0x81,
                graph.Definition.MainModule.ImportReference(core, "System", "Object"), true);
            graph.Definition.MainModule.Types.Add(definition);
            root = graph.Types.Single();
        }
        else root = graph.AddClosedClass("Example", "Root");
        var field = root.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public);
        var initialize = root.AddConstructor([PrimitiveType.Int32], MethodVisibility.Protected);
        var il = initialize.GetILGenerator(); il.LoadArgument(0); il.LoadArgument(1); il.StoreField(field); il.Return();
        var child = graph.AddClass("Example", "Child", root);
        var constructor = child.AddConstructor(Array.Empty<PrimitiveType>());
        il = constructor.GetILGenerator(); il.LoadArgument(0); il.LoadConstant(42); il.Call(initialize); il.Return();
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        il = main.GetILGenerator(); il.NewObject(constructor); il.LoadField(field); il.Return();
        return graph;
    }

    internal static void Run()
    {
        foreach (bool manual in new[] { false, true })
        {
            var graph = Create(manual);
            if (!graph.Types[0].IsClosedHierarchy || !graph.Types[0].IsAbstract || graph.Types[0].IsStatic)
                throw new Exception("authored closed family flags lost");
            var image = RuntimeAssemblyContainer.WriteBinary(graph);
            using var pe = new PEReader(new MemoryStream(image));
            var reader = pe.GetMetadataReader();
            var cliRoot = reader.TypeDefinitions.Select(reader.GetTypeDefinition).Single(t => reader.GetString(t.Name) == "Root");
            if ((cliRoot.Attributes & System.Reflection.TypeAttributes.Abstract) == 0 ||
                (cliRoot.Attributes & System.Reflection.TypeAttributes.Sealed) != 0)
                throw new Exception("CLI reference root classification lost");
            var native = AssemblyDefinition.ReadNativeAssembly(image);
            var context = new MetadataLoadContext([native]);
            var root = context.Assemblies.Single().GetTypes().Single(t => t.Name == "Root");
            var child = context.Assemblies.Single().GetTypes().Single(t => t.Name == "Child");
            if (!root.IsClosedHierarchy || !root.IsAbstract || root.IsSealed || root.IsStatic ||
                child.IsClosedHierarchy || !ReferenceEquals(child.BaseType, root) ||
                !ReferenceEquals(root.GetPermittedDirectSubtypes().Single(), child) || child.GetPermittedDirectSubtypes().Count != 0 ||
                root.GetConstructors().Single().Accessibility != MetadataAccessibility.Family)
                throw new Exception("closed family native reader/facade contract lost");
            Reject(() => graph.Write(), "native emission");
        }
        var ordinary = new AssemblyBuilder(new("Ordinary", new(1, 0, 0, 0)), Create(false).CoreLibrary);
        ordinary.AddClass("Example", "Open");
        var cliView = new MetadataLoadContext([AssemblyDefinition.ReadAssembly(ordinary.Write(), expectedExtended: false)]).Assemblies.Single().GetTypes().Single();
        try { _ = cliView.IsClosedHierarchy; throw new Exception("unmaterialized CLI closure default invented"); }
        catch (NotSupportedException) { }
        var referenceHost = new AssemblyBuilder(new("References", new(1, 0, 0, 0)), ordinary.CoreLibrary);
        var dependency = new AssemblyIdentity("Dependency", new(1, 0, 0, 0));
        var parentRef = referenceHost.CreateTypeReference(dependency, ordinary.CoreLibrary, new string('a', 64), "Example", "Base");
        var childRef = referenceHost.CreateTypeReference(dependency, ordinary.CoreLibrary, new string('a', 64), "Example", "Child");
        var leafRef = referenceHost.CreateTypeReference(dependency, ordinary.CoreLibrary, new string('a', 64), "Example", "Leaf");
        referenceHost.DeclareClassBase(childRef, parentRef); referenceHost.DeclareClassBase(leafRef, childRef);
        referenceHost.DeclareClassBase(childRef, parentRef);
        try { referenceHost.DeclareClassBase(parentRef, leafRef); throw new Exception("cyclic reference base accepted"); }
        catch (ArgumentException) { }
        try { referenceHost.DeclareClassBase(leafRef, parentRef); throw new Exception("conflicting reference base accepted"); }
        catch (ArgumentException) { }
        var foreign = ordinary.CreateTypeReference(dependency, ordinary.CoreLibrary, new string('a', 64), "Example", "Foreign");
        try { referenceHost.DeclareClassBase(childRef, foreign); throw new Exception("foreign-owner base accepted"); }
        catch (ArgumentException) { }
        var contract = referenceHost.CreateInterfaceReference(dependency, ordinary.CoreLibrary, new string('a', 64), "Example", "Contract");
        try { referenceHost.DeclareClassBase(childRef, contract); throw new Exception("interface used as class base"); }
        catch (ArgumentException) { }
        var convert = referenceHost.AddFunction("Upcast", new MethodSignature(parentRef, new SignatureType[] { leafRef }));
        convert.GetILGenerator().LoadArgument(0); convert.GetILGenerator().Return();
        _ = referenceHost.WriteNativeAssembly();
        var invalid = Create(false);
        var rootType = invalid.Types[0];
        var factory = rootType.AddMethod("Bad", new(PrimitiveType.Void, []));
        var body = factory.GetILGenerator(); body.LoadConstant(0); body.NewObject(rootType.Methods.Single(m => m.IsConstructor));
        body.Emit(OpCode.Pop); body.Return();
        Reject(() => invalid.WriteNativeAssembly(), "abstract classes");
        var before = invalid.Types.Count;
        try
        {
            invalid.Definition.MainModule.Types.Add(new TypeDefinition("Example", "Invalid", 1,
                invalid.Definition.MainModule.ImportReference(invalid.CoreLibrary, "System", "Object"), true));
            throw new Exception("nonabstract closed root accepted");
        }
        catch (ArgumentException) { }
        if (invalid.Types.Count != before) throw new Exception("invalid closed root was attached");
    }

    private static void Reject(Action action, string message)
    {
        try { action(); }
        catch (InvalidDataException error) when (error.Message.Contains(message)) { return; }
        throw new Exception("expected rejection: " + message);
    }

    internal static async Task RunRuntime(string runtime, string directory)
    {
        Run(); Directory.CreateDirectory(directory);
        var path = Path.Combine(directory, "ClosedFamily.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(Create(false)));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add(command); start.ArgumentList.Add(path);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(30));
            try { await process.WaitForExitAsync(timeout.Token); } catch { process.Kill(true); throw; }
            var output = await stdout; var error = await stderr;
            if (process.ExitCode != (command == "run" ? 42 : 0) || error.Length != 0) throw new Exception(output + error);
        }
        Console.WriteLine("PASS closed-family native PE with protected base constructor: verify/run return 42");
    }
}
