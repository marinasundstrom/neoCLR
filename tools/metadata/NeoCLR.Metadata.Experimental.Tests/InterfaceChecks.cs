using System.Diagnostics;
using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class InterfaceChecks
{
    internal static AssemblyBuilder Create()
    {
        var host = typeof(object).Assembly.GetName();
        var graph = new AssemblyBuilder(new("InterfaceContracts", new Version(1, 0, 0, 0)),
            new(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? [])));
        var owner = graph.AddGenericInterface("Example", "Comparer", ["T"]);
        var t = SignatureType.TypeParameter(0);
        owner.AddInterfaceMethod("Compare", new(PrimitiveType.Int32, [t, t]));
        graph.AddInterface("Example", "Marker", TypeVisibility.Internal);
        var disposable = graph.AddInterface("Example", "Disposable");
        disposable.AddInterfaceMethod("Dispose", new(PrimitiveType.Void, []));
        var iterator = graph.AddGenericInterface("Example", "Iterator", ["T"]);
        iterator.AddBaseInterface(disposable);
        var get = iterator.AddInterfaceMethod("get_Current", new(t, []));
        var set = iterator.AddInterfaceMethod("set_Current", new(PrimitiveType.Void, [t]));
        iterator.AddProperty("Current", t, get, set);
        var iterable = graph.AddGenericInterface("Example", "Iterable", ["T"]);
        iterable.AddInterfaceMethod("GetIterator", new(iterator.MakeGenericInstance(t), []));
        var forward = graph.AddFunction("Forward", new MethodSignature(iterator.MakeGenericInstance(PrimitiveType.Int32), [iterator.MakeGenericInstance(PrimitiveType.Int32)]));
        forward.LoadArgument(0); forward.Return();
        var main = graph.AddFunction("Main"); main.LoadDefault(iterator.MakeGenericInstance(PrimitiveType.Int32)); main.Call(forward); main.Emit(OpCode.Pop); main.LoadConstant(42); main.Return(); graph.EntryPoint = main;
        return graph;
    }
    internal static void Run()
    {
        var graph = Create(); var image = graph.Write(); var loaded = Assembly.Load(image);
        var type = loaded.GetType("Example.Comparer`1")!;
        if (!type.IsInterface || type.BaseType is not null || !type.GetMethod("Compare")!.IsAbstract ||
            !type.GetMethod("Compare")!.IsVirtual || type.GetMethod("Compare")!.GetMethodBody() is not null ||
            type.MakeGenericType(typeof(int)).GetMethod("Compare")!.GetParameters()[0].ParameterType != typeof(int))
            throw new Exception("CLI interface contract");
        var iterable = loaded.GetType("Example.Iterable`1")!.MakeGenericType(typeof(int));
        if (iterable.GetMethod("GetIterator")!.ReturnType != loaded.GetType("Example.Iterator`1")!.MakeGenericType(typeof(int)))
            throw new Exception("CLI interface-valued generic signature");
        var iterator = loaded.GetType("Example.Iterator`1")!.MakeGenericType(typeof(int));
        if (iterator.GetInterfaces().Single().Name != "Disposable" || iterator.GetProperty("Current")!.PropertyType != typeof(int) ||
            !iterator.GetProperty("Current")!.GetMethod!.IsSpecialName || !iterator.GetProperty("Current")!.SetMethod!.IsAbstract)
            throw new Exception("CLI inherited interface and abstract property");
        if (!Equals(loaded.EntryPoint!.Invoke(null, null), 42)) throw new Exception("CLI entry");
        var projected = NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary);
        using var pe = new PEReader(new MemoryStream(projected)); var reader = pe.GetMetadataReader();
        var definition = reader.GetTypeDefinition(reader.TypeDefinitions.Single(h => reader.GetString(reader.GetTypeDefinition(h).Name) == "Comparer`1"));
        var method = reader.GetMethodDefinition(definition.GetMethods().Single());
        if ((definition.Attributes & TypeAttributes.Interface) == 0 || !definition.BaseType.IsNil || method.RelativeVirtualAddress != 0 ||
            (method.Attributes & (MethodAttributes.Abstract | MethodAttributes.Virtual | MethodAttributes.NewSlot)) != (MethodAttributes.Abstract | MethodAttributes.Virtual | MethodAttributes.NewSlot))
            throw new Exception("projected interface flags/body");
        var iteratorDefinition = reader.GetTypeDefinition(reader.TypeDefinitions.Single(h => reader.GetString(reader.GetTypeDefinition(h).Name) == "Iterator`1"));
        if (iteratorDefinition.GetInterfaceImplementations().Count != 1 || iteratorDefinition.GetProperties().Count != 1)
            throw new Exception("projected interface bases/property");
        void Reject(Action action) { try { action(); } catch (Exception e) when (e is ArgumentException or InvalidOperationException or InvalidDataException) { return; } throw new Exception("invalid interface accepted"); }
        var a = graph.AddInterface("Example", "A"); var b = graph.AddInterface("Example", "B");
        a.AddBaseInterface(b); Reject(() => b.AddBaseInterface(a)); Reject(() => a.AddBaseInterface(b));
        Reject(() => a.AddBaseInterface(graph.Types[0]));
        var owner = graph.Types[0];
        Reject(() => owner.AddConstructor(Array.Empty<PrimitiveType>()));
        Reject(() => owner.AddMethod("Static", new(PrimitiveType.Void, [])));
        Reject(() => owner.AddField("Value", PrimitiveType.Int32));
        Reject(() => graph.Types[1].AddInterfaceMethod("Generic", new(PrimitiveType.Void, [], ["U"])));
        owner.Methods[0].LoadConstant(0); owner.Methods[0].Return();
        Reject(() => graph.Write()); Reject(() => graph.WriteNativeAssembly());
        foreach (var change in new Action<JsonNode>[] {
            n => n["functions"]![2]!["is_abstract"] = false,
            n => n["functions"]![2]!["body"]!.AsArray().Add(new JsonObject { ["op"] = "ret" }),
            n => n["types"]![0]!["is_reference_type"] = true,
            n => n["types"]![3]!["implements"]![0]!["Named"] = n["types"]![3]!["name"]!.GetValue<string>(),
            n => n["types"]![3]!["properties"]![0]!["instance"] = false
        })
        {
            var malformed = JsonNode.Parse(Create().WriteNativeAssembly())!; change(malformed);
            Reject(() => NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(malformed.ToJsonString())));
        }
    }
    internal static async Task RunRuntime(string runtime, string output)
    {
        Run(); if (Directory.Exists(output)) throw new IOException("output must be fresh"); Directory.CreateDirectory(output);
        var graph = Create(); var path = Path.Combine(output, "Interfaces.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add(command); start.ArgumentList.Add(path);
            using var process = Process.Start(start)!; var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (process.ExitCode != (command == "verify" ? 0 : 42)) throw new Exception(command + ": " + text);
        }
        Console.WriteLine("PASS interface declarations: CLI/native binary load, verify and default-reference forwarding entry result 42; dispatch not exercised");
    }
}
