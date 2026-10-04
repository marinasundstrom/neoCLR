using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class ClassBaseReaderChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var builder = new AssemblyBuilder(new AssemblyIdentity("ClassBases", new Version(1, 0, 0, 0)), core);
        builder.AddClass("Example", "Base");
        builder.AddClass("Example", "Derived");
        var root = JsonNode.Parse(builder.WriteNativeAssembly())!;
        var rows = root["types"]!.AsArray();
        rows[1]!["base"] = new JsonObject { ["Named"] = rows[0]!["name"]!.GetValue<string>() };
        byte[] Bytes() => Encoding.UTF8.GetBytes(root.ToJsonString());
        var snapshot = AssemblyDefinition.ReadNativeAssembly(NativeModuleContainer.WriteLibraryBinary(Bytes()));
        var types = snapshot.MainModule.Types;
        var parent = types.Single(t => t.Name == "Base");
        var child = types.Single(t => t.Name == "Derived");
        Check(parent.BaseType is null && ReferenceEquals(child.BaseType!.Resolve(), parent), "definition base identity");
        var copy = AssemblyDefinition.ReadNativeAssembly(snapshot.Write());
        Check(copy.MainModule.Types.Single(t => t.Name == "Derived").BaseType!.Resolve().Name == "Base", "standalone snapshot copy preserves base");
        var cli = AssemblyDefinition.ReadAssembly(builder.WriteReferenceImage(), false);
        var cliView = new MetadataLoadContext([cli]).Assemblies.Single().GetTypes().First();
        try { _ = cliView.BaseType; throw new Exception("unmaterialized CLI base invented"); }
        catch (NotSupportedException) { }
        var context = new MetadataLoadContext([snapshot]);
        var views = context.Assemblies.Single().GetTypes();
        Check(ReferenceEquals(views.Single(t => t.Name == "Derived").BaseType, views.Single(t => t.Name == "Base")), "canonical facade base identity");
        try { NativeAssemblyDefinition.ReadAssembly(Bytes()).CreateReferenceAssembly(core); throw new Exception("lossy projection accepted"); }
        catch (NotSupportedException) { }
        foreach (var bad in new[] { "missing", rows[1]!["name"]!.GetValue<string>() })
        {
            rows[1]!["base"]!["Named"] = bad;
            Reject(() => NativeAssemblyDefinition.ReadAssembly(Bytes()));
        }
        rows[1]!["base"]!["Named"] = rows[0]!["name"]!.GetValue<string>();
        rows[0]!["base"] = new JsonObject { ["Named"] = rows[1]!["name"]!.GetValue<string>() };
        Reject(() => NativeAssemblyDefinition.ReadAssembly(Bytes()));
        rows[0]!.AsObject().Remove("base");
        rows[0]!["is_abstract"] = true;
        rows[0]!["is_sealed"] = true;
        Reject(() => NativeAssemblyDefinition.ReadAssembly(Bytes()));
        rows[0]!["is_abstract"] = false;
        rows[0]!["is_sealed"] = false;
        rows[1]!["base"] = new JsonObject { ["GenericInstance"] = new JsonObject() };
        Reject(() => NativeAssemblyDefinition.ReadAssembly(Bytes()));
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
    private static void Reject(Action action)
    {
        try { action(); }
        catch (InvalidDataException) { return; }
        throw new Exception("invalid class base accepted");
    }
}
