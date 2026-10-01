using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class TypeVisibilityChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var graph = new AssemblyBuilder(new("Visibility", new Version(1, 0, 0, 0)), core);
        var hidden = graph.AddType("Example", "Hidden", TypeVisibility.Internal);
        var value = hidden.AddMethod("Value"); value.LoadConstant(42); value.Return();
        var facade = graph.AddType("Example", "Facade");
        var call = facade.AddMethod("Value"); call.Call(value); call.Return();
        if (hidden.Visibility != TypeVisibility.Internal || facade.Visibility != TypeVisibility.Public) throw new Exception("builder visibility");
        var bytes = graph.Write();
        Check(bytes);
        var loaded = System.Reflection.Assembly.Load(bytes);
        if (loaded.GetType("Example.Hidden")!.IsPublic || !Equals(loaded.GetType("Example.Facade")!.GetMethod("Value")!.Invoke(null, null), 42)) throw new Exception("CLI internal helper execution");
        var native = graph.WriteNativeAssembly();
        Check(NativeAssemblyDefinition.ReadAssembly(native).CreateReferenceAssembly(core));
        var json = JsonNode.Parse(native)!;
        if (json["types"]![0]!["visibility"]!.GetValue<string>() != "internal" || json["types"]![1]!["visibility"] is not null) throw new Exception("native visibility/default encoding");
        json["types"]![0]!["origin"]!["publicly_visible"] = true;
        Reject(json);
        json["types"]![0]!["origin"]!["publicly_visible"] = false;
        json["types"]![0]!["visibility"] = "private"; Reject(json);
        json["types"]![0]!.AsObject().Remove("visibility"); Reject(json);
        try { graph.AddType("", "Bad", (TypeVisibility)99); throw new Exception("accepted invalid visibility"); }
        catch (ArgumentOutOfRangeException) { }
    }

    private static void Check(byte[] bytes)
    {
        using var pe = new PEReader(new MemoryStream(bytes));
        var reader = pe.GetMetadataReader();
        var types = reader.TypeDefinitions.Select(reader.GetTypeDefinition).Where(t => reader.GetString(t.Namespace) == "Example").ToArray();
        if (types.Length != 2 || (types.Single(t => reader.GetString(t.Name) == "Hidden").Attributes & TypeAttributes.VisibilityMask) != TypeAttributes.NotPublic ||
            (types.Single(t => reader.GetString(t.Name) == "Facade").Attributes & TypeAttributes.VisibilityMask) != TypeAttributes.Public) throw new Exception("visibility projection");
    }

    private static void Reject(JsonNode json)
    {
        try { NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(json.ToJsonString())); throw new Exception("accepted inconsistent visibility"); }
        catch (InvalidDataException) { }
    }
}
