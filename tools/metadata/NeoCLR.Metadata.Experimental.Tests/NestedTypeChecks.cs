using System.Diagnostics;
using System.Runtime.Loader;
using System.Text.Json;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class NestedTypeChecks
{
    private static AssemblyBuilder Create()
    {
        var core = new AssemblyIdentity("System.Private.CoreLib", typeof(object).Assembly.GetName().Version!, "", "7cec85d7bea7798e");
        var assembly = new AssemblyBuilder(new("NestedTypes", new Version(1, 0, 0, 0)), core);
        var outer = assembly.AddClass("Example", "Outer");
        var nested = outer.AddNestedValueType("Item");
        var field = nested.AddField("Value", PrimitiveType.Int32, FieldVisibility.Public);
        var ctor = nested.AddConstructor(new[] { PrimitiveType.Int32 });
        ctor.LoadArgument(0); ctor.LoadArgument(1); ctor.StoreField(field); ctor.Return();
        var second = assembly.AddClass("Example", "Second").AddNestedValueType("Item");
        var deep = second.AddNestedClass("Deep");
        second.AddNestedClass("Hidden", TypeVisibility.Internal);
        outer.AddField("FirstItem", nested, FieldVisibility.Public);
        outer.AddField("SecondItem", second, FieldVisibility.Public);
        assembly.AddValueType("", "Item");
        var main = outer.AddMethod("Main");
        var local = main.DeclareLocal(nested);
        main.LoadConstant(42); main.NewObject(ctor); main.StoreLocal(local);
        main.LoadLocalAddress(local); main.LoadField(field); main.Return();
        assembly.EntryPoint = main;
        if (!ReferenceEquals(nested.Definition.DeclaringType, outer.Definition) || outer.Definition.NestedTypes.Count != 1 ||
            !ReferenceEquals(deep.Definition.ToReference().Resolve().DeclaringType, second.Definition)) throw new Exception("authored nested navigation");
        return assembly;
    }
    internal static void Run()
    {
        var assembly = Create(); var image = assembly.Write();
        var snapshot = AssemblyDefinition.ReadAssembly(image, expectedExtended: false);
        var outer = snapshot.MainModule.Types.Single(t => t.Name == "Outer");
        if (outer.NestedTypes.Single().Name != "Item" || !ReferenceEquals(outer.NestedTypes.Single().DeclaringType, outer)) throw new Exception("CLI nested roundtrip");
        var context = new AssemblyLoadContext("NestedTypes", true);
        try
        {
            var loaded = context.LoadFromStream(new MemoryStream(image));
            if (loaded.GetType("Example.Outer+Item") is not { IsValueType: true } || !Equals(42, loaded.EntryPoint!.Invoke(null, null))) throw new Exception("CLR nested execution");
        }
        finally { context.Unload(); }
        using var native = JsonDocument.Parse(assembly.WriteNativeAssembly());
        var rows = native.RootElement.GetProperty("types").EnumerateArray().ToArray();
        if (rows.Select(t => t.GetProperty("name").GetString()).Distinct().Count() != rows.Length ||
            rows[1].GetProperty("declaring_type").GetProperty("index").GetInt32() != 0 ||
            rows[1].GetProperty("origin").GetProperty("declaring_type_token").GetInt32() != 0x02000002) throw new Exception("native nested identity");
        var directImage = RuntimeAssemblyContainer.WriteBinary(assembly);
        var direct = AssemblyDefinition.ReadNativeAssembly(directImage);
        var views = new MetadataLoadContext([direct]);
        var declared = views.Resolve(direct.Identity).GetTypes();
        var firstItem = declared.Single(t => t.FullName == "Example.Outer+Item");
        var secondItem = declared.Single(t => t.FullName == "Example.Second+Item");
        if (ReferenceEquals(firstItem, secondItem) || firstItem.DeclaringType!.Name != "Outer" ||
            firstItem.Accessibility != MetadataAccessibility.Public || !firstItem.IsValueType ||
            declared.Single(t => t.FullName == "Example.Second+Item+Deep").DeclaringType != secondItem ||
            !direct.Write().SequenceEqual(directImage)) throw new Exception("native nested declaration scopes");
        var ownFields = declared.Single(t => t.FullName == "Example.Outer").GetFields();
        if (!ReferenceEquals(ownFields[0].FieldType, firstItem) || !ReferenceEquals(ownFields[1].FieldType, secondItem) ||
            declared.Single(t => t.FullName == "Example.Second+Item+Hidden").Accessibility != MetadataAccessibility.Assembly)
            throw new Exception("local nested signatures or access lost scope");
        var consumer = new AssemblyBuilder(new("NestedConsumer", new Version(1, 0, 0, 0)), assembly.CoreLibrary);
        var holder = consumer.AddClass("", "Holder");
        foreach (var parent in new[] { "Outer", "Second" })
            holder.AddField(parent, consumer.ImportReference(direct.MainModule.Types.Single(t => t.Name == parent).NestedTypes.Single(), assembly.CoreLibrary), FieldVisibility.Public);
        var consumerSnapshot = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(consumer));
        var linked = new MetadataLoadContext([direct, consumerSnapshot]);
        var fields = linked.Resolve(consumerSnapshot.Identity).GetTypes().Single().GetFields();
        if (((NominalTypeInfo)fields[0].FieldType).FullName != "Example.Outer+Item" ||
            ((NominalTypeInfo)fields[1].FieldType).FullName != "Example.Second+Item") throw new Exception("nested external field scopes collapsed");
        try
        {
            _ = new MetadataLoadContext([consumerSnapshot]).Resolve(consumerSnapshot.Identity).GetTypes().Single().GetFields();
            throw new Exception("missing nested dependency accepted");
        }
        catch (InvalidDataException) { }
        var projected = AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(assembly.WriteNativeAssembly()).CreateReferenceAssembly(assembly.CoreLibrary), expectedExtended: false);
        if (projected.MainModule.Types.Single(t => t.Name == "Outer").NestedTypes.Single().Name != "Item") throw new Exception("native reference projection flattened owner");
        var corrupted = JsonNode.Parse(assembly.WriteNativeAssembly())!;
        corrupted["types"]![1]!["declaring_type"]!["index"] = 1;
        try { NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(corrupted.ToJsonString())); throw new Exception("cyclic nested owner accepted"); } catch (InvalidDataException) { }
        var owner = assembly.Types[0];
        Reject(() => owner.AddNestedValueType("Item"));
        Reject(() => owner.Definition.NestedTypes.Add(owner.Definition));
        Reject(() => assembly.Definition.MainModule.Types.Add(new TypeDefinition("", "Invalid", 2, owner.Definition.BaseType)));
        var generic = assembly.AddGenericValueType("", "Generic", new[] { "T" });
        Reject(() => generic.AddNestedValueType("Captured"));
        if (generic.Definition.NestedTypes.Count != 0 || owner.Definition.NestedTypes.Count != 1) throw new Exception("failed nesting mutated owner");
    }
    private static void Reject(Action action) { try { action(); throw new Exception("invalid nesting accepted"); } catch (ArgumentException) { } }
    internal static async Task RunRuntime(string runtime, string directory)
    {
        Run(); Directory.CreateDirectory(directory); var assembly = Create();
        var path = Path.Combine(directory, "Nested.dll");
        File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(assembly.WriteNativeAssembly(), assembly.CoreLibrary));
        foreach (var command in new[] { "verify", "run" })
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            start.ArgumentList.Add(command); start.ArgumentList.Add(path);
            using var process = Process.Start(start)!; var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var text = await stdout + await stderr;
            if (process.ExitCode != (command == "verify" ? 0 : 42)) throw new Exception(command + ": " + text);
        }
        Console.WriteLine("Nested value constructor executes on CLR and neoCLR: 42");
    }
}
