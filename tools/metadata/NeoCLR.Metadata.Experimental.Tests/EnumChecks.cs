using System.Diagnostics;
using System.Reflection;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;

internal static class EnumChecks
{
    private static AssemblyBuilder Create(bool manual, bool flags = false)
    {
        var graph = new AssemblyBuilder(new("Enums" + manual + flags, new Version(1, 0, 0, 0)), new("System.Runtime", new Version(10, 0, 0, 0)));
        TypeBuilder type;
        if (manual)
        {
            var definition = new TypeDefinition("Example", "State", (uint)(TypeAttributes.Public | TypeAttributes.Sealed),
                graph.Definition.MainModule.ImportReference(graph.CoreLibrary, "System", "Enum"));
            definition.Fields.Add(new FieldDefinition("value__", 0x606, PrimitiveType.Int32));
            graph.Definition.MainModule.Types.Add(definition);
            type = graph.Types.Single();
            definition.Fields.Add(new FieldDefinition("Ready", 0x8056, (SignatureType)type, 42));
        }
        else
        {
            type = graph.AddEnum("Example", "State");
            type.AddEnumMember("Ready", 42);
        }
        if (flags)
        {
            if (manual) type.Definition.SetEnumFlags();
            else type.SetEnumFlags();
            type.SetEnumFlags(); // Idempotent convenience call.
        }
        type.AddEnumMember("Negative", -1);
        type.AddEnumMember("Alias", 42);
        var echo = graph.AddFunction("Echo", new MethodSignature(type, [type]));
        echo.GetILGenerator().LoadArgument(0); echo.GetILGenerator().Return();
        var run = graph.AddType("Example", "Program").AddMethod("Run");
        run.GetILGenerator().LoadConstant(42); run.GetILGenerator().ConvertToEnum(type);
        run.GetILGenerator().Call(echo); run.GetILGenerator().ConvertFromEnum(type); run.GetILGenerator().Return();
        return graph;
    }

    internal static void Run()
    {
        ValidateRejections();
        foreach (bool manual in new[] { false, true })
        foreach (bool flags in new[] { false, true })
        {
            var graph = Create(manual, flags);
            var cli = graph.Write();
            var loadedAssembly = Assembly.Load(cli);
            var loaded = loadedAssembly.GetType("Example.State")!;
            if (loaded.IsDefined(typeof(FlagsAttribute), false) != flags || !loaded.IsEnum || Enum.GetUnderlyingType(loaded) != typeof(int) || Convert.ToInt32(Enum.Parse(loaded, "Negative")) != -1 ||
                (int)loadedAssembly.GetType("Example.Program")!.GetMethod("Run")!.Invoke(null, null)! != 42)
                throw new Exception("CLI enum behavior was not preserved");
            foreach (var snapshot in new[] { AssemblyDefinition.ReadAssembly(cli, false), AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(graph)) })
            {
                var declaration = snapshot.MainModule.Types.Single(t => t.Name == "State");
                if (declaration.IsFlagsEnum != flags || !declaration.IsEnum || !declaration.IsValueType || declaration.Fields.Count(f => f.IsLiteral) != 3 ||
                    declaration.Fields.Single(f => f.Name == "Ready").Constant != 42 || declaration.Fields.Single(f => f.Name == "Negative").Constant != -1)
                    throw new Exception("enum facts were lost on read");
                if (snapshot.IsNative)
                {
                    var context = new NeoCLR.Metadata.Experimental.Introspection.MetadataLoadContext([snapshot]);
                    var view = context.Resolve(snapshot.Identity).GetModules()[0].GetTypes().Single(t => t.Name == "State");
                    if (view.IsFlagsEnum != flags || !view.IsEnum || view.GetFields().Single(f => f.Name == "Ready").Constant != 42)
                        throw new Exception("enum introspection facts were lost");
                }
            }
        }
    }

    private static void Reject(Action action)
    {
        try { action(); }
        catch (Exception error) when (error is ArgumentException or InvalidOperationException or InvalidDataException) { return; }
        throw new Exception("unsupported enum contract accepted");
    }

    private static void ValidateRejections()
    {
        var graph = Create(false);
        var type = graph.Types.Single(t => t.IsEnum);
        Reject(() => type.AddEnumMember("Ready", 3));
        Reject(() => graph.AddType("Example", "Ordinary").AddEnumMember("Bad", 0));
        Reject(() => new FieldDefinition("Bad", 0x8056, PrimitiveType.Int32));
        Reject(() => new FieldDefinition("Bad", 6, PrimitiveType.Int32, 1));
        Reject(() => graph.AddFunction("BadConversion").GetILGenerator().ConvertToEnum(PrimitiveType.Int32));
        foreach (Action<JsonNode> mutation in new Action<JsonNode>[] {
            row => row["enum_info"]!["flags"] = "invalid",
            row => row["enum_info"]!["underlying"] = "Int64",
            row => row["enum_info"]!["members"]![1]!["name"] = "Ready",
            row => row["fields"]![0]!["ty"] = "Int64",
            row => row["origin"]!["field_readonly"]![0] = true,
        })
        {
            var valid = Create(false);
            var document = JsonNode.Parse(valid.WriteNativeAssembly())!;
            mutation(document["types"]![0]!);
            Reject(() => NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(document.ToJsonString())));
        }
        Reject(() => graph.AddClass("Example", "NotEnum").SetEnumFlags());
        var explicitMarker = Create(false);
        var marked = explicitMarker.Types.Single(t => t.IsEnum);
        marked.AddCustomAttribute(new CustomAttributeDefinition(explicitMarker.Definition.MainModule.ImportReference(explicitMarker.CoreLibrary, "System", "FlagsAttribute"), []));
        if (!AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(explicitMarker)).MainModule.Types.Single(t => t.IsEnum).IsFlagsEnum)
            throw new Exception("manually authored FlagsAttribute lost native classification");
        marked.AddCustomAttribute(new CustomAttributeDefinition(explicitMarker.Definition.MainModule.ImportReference(explicitMarker.CoreLibrary, "System", "FlagsAttribute"), []));
        Reject(() => explicitMarker.WriteNativeAssembly());
        var invalid = Create(false);
        invalid.Types.Single(t => t.IsEnum).AddMethod("Unsupported").GetILGenerator().LoadConstant(0);
        Reject(() => invalid.Write());
    }

    internal static async Task RunRuntime(string runtime, string directory)
    {
        Run(); Directory.CreateDirectory(directory);
        var graph = Create(false);
        var image = RuntimeAssemblyContainer.WriteLibraryBinary(graph);
        var library = Path.Combine(directory, "Enums.dll"); File.WriteAllBytes(library, image);
        var snapshot = AssemblyDefinition.ReadNativeAssembly(image);
        var consumer = new AssemblyBuilder(new("EnumConsumer", new Version(1, 0, 0, 0)), graph.CoreLibrary);
        var type = consumer.ImportReference(snapshot.MainModule.Types.Single(t => t.Name == "State"), graph.CoreLibrary);
        var echo = consumer.ImportReference(snapshot.MainModule.Functions.Single(f => f.Name == "Echo"), graph.CoreLibrary);
        var main = consumer.AddFunction("Main"); consumer.EntryPoint = main;
        var il = main.GetILGenerator(); il.LoadConstant(42); il.ConvertToEnum(type); il.Call(echo); il.ConvertFromEnum(type); il.Return();
        var app = Path.Combine(directory, "Consumer.dll"); File.WriteAllBytes(app, RuntimeAssemblyContainer.WriteBinary(consumer));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var argument in new[] { command, app, "--module", library }) start.ArgumentList.Add(argument);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(60));
            try { await process.WaitForExitAsync(timeout.Token); } catch { process.Kill(true); throw; }
            if (process.ExitCode != (command == "run" ? 42 : 0)) throw new Exception(await stdout + await stderr);
        }
        Console.WriteLine("PASS separately encoded enum library and consumer execute with exit 42");
    }
}
