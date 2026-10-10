using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class ExternalClassBaseReaderChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var library = new AssemblyBuilder(new("AttributeLibrary", new Version(1, 0, 0, 0)), core);
        library.AddClass("System", "Attribute").SetAbstractClass();
        var parent = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(library.WriteNativeAssembly(), core));
        var consumer = new AssemblyBuilder(new("AttributeConsumer", new Version(1, 0, 0, 0)), core);
        var imported = consumer.ImportReference(parent.MainModule.Types.Single(), core);
        consumer.AddClass("Tests", "TestAttribute");
        var echo = consumer.AddFunction("Echo", new MethodSignature(imported, [imported]));
        echo.LoadArgument(0); echo.Return();
        var root = JsonNode.Parse(consumer.WriteNativeAssembly())!;
        // Author the next reader contract explicitly; output-side inheritance remains gated.
        var binding = new JsonObject
        {
            ["native_name"] = root["functions"]![0]!["returns"]!["Named"]!.GetValue<string>(),
            ["assembly"] = root["assemblies"]![0]!["references"]![0]!.DeepClone(),
            ["namespace"] = "System", ["name"] = "Attribute", ["arity"] = 0,
            ["value_type"] = false, ["declaring"] = null
        };
        root["assemblies"]![0]!["native_type_bindings"] = new JsonArray(binding);
        root["assemblies"]![0]!["native_module_bindings"] = new JsonArray(new JsonObject
        {
            ["assembly"] = binding["assembly"]!.DeepClone(),
            ["module"] = root["references"]![0]!["name"]!.DeepClone(),
            ["revision"] = root["references"]![0]!["revision"]!.DeepClone()
        });
        var child = root["types"]![0]!;
        child["base"] = new JsonObject { ["Named"] = binding["native_name"]!.GetValue<string>() };
        byte[] Bytes() => Encoding.UTF8.GetBytes(root.ToJsonString());
        var model = Bytes();
        foreach (var image in new[] { NativeModuleContainer.WriteLibraryBinary(model), NativeModuleContainer.WriteBinary(model) })
        {
            var snapshot = AssemblyDefinition.ReadNativeAssembly(image);
            var reference = snapshot.MainModule.Types.Single().BaseType!;
            Check(reference.Namespace == "System" && reference.Name == "Attribute", "base name lost");
            Check(reference.ResolutionScopeToken == snapshot.MainModule.AssemblyReferences.Single().MetadataToken, "base scope lost");
            Check(ReferenceEquals(reference.Resolve(new Resolver(parent)), parent.MainModule.Types.Single()), "base identity lost");
            Reject<InvalidDataException>(() => reference.Resolve());
            var other = new AssemblyBuilder(new("AttributeLibrary", new Version(2, 0, 0, 0)), core);
            other.AddClass("System", "Attribute");
            var wrong = AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(other.WriteNativeAssembly(), core));
            Reject<InvalidDataException>(() => reference.Resolve(new Resolver(wrong)));
            var missingContext = new MetadataLoadContext([snapshot]);
            Reject<InvalidDataException>(() => _ = missingContext.Assemblies.Single().GetTypes().Single().BaseType);
            var wrongContext = new MetadataLoadContext([snapshot, wrong]);
            Reject<InvalidDataException>(() => _ = wrongContext.Assemblies.Single(a => a.Name == "AttributeConsumer").GetTypes().Single().BaseType);
            var context = new MetadataLoadContext([snapshot, parent]);
            var view = context.Assemblies.Single(a => a.Name == "AttributeConsumer").GetTypes().Single();
            var parentView = context.Assemblies.Single(a => a.Name == "AttributeLibrary").GetTypes().Single();
            Check(ReferenceEquals(view.BaseType, parentView), "facade did not preserve canonical external base");
            var copied = AssemblyDefinition.ReadNativeAssembly(snapshot.Write());
            Check(ReferenceEquals(copied.MainModule.Types.Single().BaseType!.Resolve(new Resolver(parent)), parent.MainModule.Types.Single()), "copy lost base scope");
            Reject<NotSupportedException>(() => NativeAssemblyDefinition.ReadAssembly(model).CreateReferenceAssembly(core));
        }
        child["base"]!["Named"] = "System.Attribute";
        Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(Bytes()));
        child["base"]!["Named"] = binding["native_name"]!.GetValue<string>();
        binding["value_type"] = true;
        Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(Bytes()));
        binding["value_type"] = false;
        binding["arity"] = 1;
        Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(Bytes()));
        binding["arity"] = 0;
        root["assemblies"]![0]!["native_type_bindings"] = new JsonArray();
        Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(Bytes()));
        Console.WriteLine("PASS scoped external class base snapshots and resolution");
    }
    private sealed class Resolver(AssemblyDefinition candidate) : IAssemblyResolver
    { public AssemblyDefinition? Resolve(AssemblyIdentity identity) => candidate; }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
    private static void Reject<T>(Action action) where T : Exception
    { try { action(); } catch (T) { return; } throw new Exception("expected " + typeof(T).Name); }
}
