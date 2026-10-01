using System.Diagnostics;
using System.Reflection;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;

internal static class ClassFieldChecks
{
    internal static AssemblyBuilder Create()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var graph = new AssemblyBuilder(new("ClassFields", new Version(1, 0, 0, 0)), core);
        graph.AddType("Example", "Before");
        var order = graph.AddClass("Example", "Order");
        order.AddField("Number", PrimitiveType.Int32);
        order.AddField("Pending", PrimitiveType.Boolean, FieldVisibility.Internal);
        var other = graph.AddClass("Example", "Other", TypeVisibility.Internal);
        other.AddField("Text", PrimitiveType.String, FieldVisibility.Public);
        other.AddField("Wide", PrimitiveType.Int64);
        graph.AddType("Example", "After");
        var entry = graph.AddFunction("Main"); entry.LoadConstant(42); entry.Return(); graph.EntryPoint = entry;
        return graph;
    }
    internal static void Run()
    {
        var graph = Create();
        var native = graph.WriteNativeAssembly();
        foreach (var image in new[] { graph.Write(), NativeAssemblyDefinition.ReadAssembly(native).CreateReferenceAssembly(graph.CoreLibrary) })
        {
            var snapshot = AssemblyDefinition.ReadAssembly(image, false);
            var order = snapshot.MainModule.Types.Single(t => t.Name == "Order");
            if (((TypeAttributes)order.Attributes & (TypeAttributes.Abstract | TypeAttributes.Sealed)) != 0 || order.Fields.Count != 2)
                throw new Exception("root class flags/fields");
            if (snapshot.MainModule.Fields.Count != 4 || snapshot.MainModule.Types.Single(t => t.Name == "Before").Fields.Count != 0 ||
                snapshot.MainModule.Types.Single(t => t.Name == "After").Fields.Count != 0) throw new Exception("field ownership ranges");
            foreach (var field in snapshot.MainModule.Fields)
            {
                if (!ReferenceEquals(field, snapshot.MainModule.GetFieldDefinition(field.MetadataToken)) || !field.TryGetPrimitiveType(out _))
                    throw new Exception("field snapshot ownership/signature");
                var copy = field.GetSignature(); copy[0] = 0;
                if (field.GetSignature()[0] != 6) throw new Exception("mutable field signature snapshot");
            }
            if ((FieldAttributes)order.Fields[0].Attributes != FieldAttributes.Private ||
                (FieldAttributes)order.Fields[1].Attributes != FieldAttributes.Assembly) throw new Exception("field access flags");
        }
        var cli = Assembly.Load(graph.Write());
        var type = cli.GetType("Example.Order")!;
        if (type.IsAbstract || type.IsSealed || type.GetFields(BindingFlags.Instance | BindingFlags.NonPublic).Length != 2) throw new Exception("CLI field layout");
        try { graph.Types[0].AddField("Bad", PrimitiveType.Int32); throw new Exception("static instance field accepted"); } catch (InvalidOperationException) { }
        foreach (var primitive in new[] { PrimitiveType.Void, (PrimitiveType)99 })
        { try { graph.Types[1].AddField("Bad", primitive); throw new Exception("invalid field type accepted"); } catch (ArgumentException) { } }
        try { graph.Types[1].AddField("Number", PrimitiveType.Int64); throw new Exception("duplicate field accepted"); } catch (ArgumentException) { }
        if (graph.Types[1].Fields.Count != 2) throw new Exception("failed field declaration mutated graph");
        var changed = JsonNode.Parse(native)!;
        changed["types"]![1]!["origin"]!["field_tokens"]![0] = 0x04000002;
        try { NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(changed.ToJsonString())); throw new Exception("mismatched field origin accepted"); } catch (InvalidDataException) { }
    }
    internal static async Task RunRuntime(string runtime, string directory)
    {
        Run();
        if (Directory.Exists(directory)) throw new IOException("output must be fresh");
        Directory.CreateDirectory(directory);
        var graph = Create();
        var path = Path.Combine(directory, "ClassFields.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(graph.WriteNativeAssembly(), graph.CoreLibrary));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add(command); start.ArgumentList.Add(path);
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (process.ExitCode != (command == "verify" ? 0 : 42)) throw new Exception(text);
        }
        Console.WriteLine("PASS root class/field metadata -> binary runtime load/verify; entry returns 42 (no instance allocation yet)");
    }
}
