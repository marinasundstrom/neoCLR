using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Runtime.Loader;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;

internal static class NativeReaderChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var library = new AssemblyBuilder(new("NativeRead", new Version(2, 3, 4, 5)), core);
        var global = library.AddFunction("Notify", returnsValue: false); global.Return();
        var method = library.AddType("Exämple", "Math").AddMethod("Twice", 1);
        method.LoadArgument(0); method.LoadConstant(2); method.Multiply(); method.Return();
        var image = library.WriteNativeAssembly();
        var read = NativeAssemblyDefinition.ReadAssembly(image);
        Check(read.Identity.Equals(library.Identity), "native identity preserved");
        Array.Clear(image); method.ClearBody(); // Snapshot owns declarations; no producer body is required.
        var reference = read.CreateReferenceAssembly(core);
        var metadata = AssemblyDefinition.ReadAssembly(reference, false);
        Check(metadata.Identity.Equals(library.Identity) && metadata.MainModule.Functions.Single().Name == "Notify", "projected identity and global function");
        var projected = metadata.MainModule.Types.Single(t => t.Namespace == "Exämple").Methods.Single();
        Check(projected.Name == "Twice" && projected.TryGetStaticInt32Signature(out var count, out var result) && count == 1 && result, "projected Unicode owner and signature");
        using (var pe = new PEReader(new MemoryStream(reference)))
        {
            var reader = pe.GetMetadataReader();
            var attribute = reader.GetCustomAttribute(reader.GetAssemblyDefinition().GetCustomAttributes().Single());
            var constructor = reader.GetMemberReference((MemberReferenceHandle)attribute.Constructor);
            var type = reader.GetTypeReference((TypeReferenceHandle)constructor.Parent);
            Check(reader.GetString(type.Name) == "ReferenceAssemblyAttribute", "reference-only marker");
            Check(pe.PEHeaders.CorHeader!.EntryPointTokenOrRelativeVirtualAddress == 0, "no projected entry point");
        }
        var context = new AssemblyLoadContext("native reference projection", isCollectible: true);
        try
        {
            try { context.LoadFromStream(new MemoryStream(reference)); throw new Exception("reference projection executed as assembly"); }
            catch (BadImageFormatException) { }
        }
        finally { context.Unload(); }
        method.LoadArgument(0); method.Return();
        var valid = library.WriteNativeAssembly();
        foreach (Action<JsonNode> mutate in new Action<JsonNode>[] {
            n => n["format"] = 4,
            n => n["types"] = new JsonArray(Enumerable.Range(0, 257).Select(_ => n["types"]![0]!.DeepClone()).ToArray()),
            n => n["functions"] = new JsonArray(Enumerable.Range(0, 4097).Select(_ => n["functions"]![0]!.DeepClone()).ToArray()),
            n => n["functions"]![1]!["parameters"] = new JsonArray(Enumerable.Range(0, 257).Select(_ => JsonValue.Create("Int32") as JsonNode).ToArray()),
            n => n["name"] = "Wrong",
            n => n["revision"] = "9.0.0.0",
            n => n["extra"] = true,
            n => n["types"]![0]!["fields"]!.AsArray().Add("unsupported"),
            n => n["types"]![0]!["is_sealed"] = false,
            n => n["types"]![0]!["origin"]!["token"] = 1,
            n => n["functions"]![1]!["owner"]!["Named"] = "Missing",
            n => n["functions"]![1]!["parameters"]![0] = "String",
            n => n["functions"]![1]!["returns"] = "Void",
            n => n["functions"]![1]!["origin"]!["member_access"] = "Private",
            n => n["functions"]![1]!["origin"]!["parameter_tokens"]![0] = 1,
            n => n["entry"] = "Missing",
            n => n["references"]!.AsArray().Add(new JsonObject { ["name"] = "Extra", ["revision"] = "1.0.0.0" })
        })
        {
            var node = JsonNode.Parse(valid)!; mutate(node);
            Reject(Encoding.UTF8.GetBytes(node.ToJsonString()));
        }
        Reject(Encoding.UTF8.GetBytes(Encoding.UTF8.GetString(valid).Replace("\"format\":5", "\"format\":5,\"format\":5")));
        Reject(valid[..^1]); Reject(new byte[4 * 1024 * 1024 + 1]);
        var opaqueBody = JsonNode.Parse(valid)!;
        opaqueBody["functions"]![1]!["body"] = new JsonArray(new JsonObject { ["op"] = "future.op" });
        Check(NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(opaqueBody.ToJsonString())).Identity.Equals(read.Identity), "body inspection is explicitly outside metadata reader");
    }
    internal static void References()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var first = new AssemblyBuilder(new("Leaf", new Version(1, 0, 0, 0), "fr"), core);
        var second = new AssemblyBuilder(new("Leaf", new Version(2, 0, 0, 0), "fr"), core);
        var one = first.AddFunction("One", returnsValue: false); one.Return();
        var two = second.AddFunction("Two", returnsValue: false); two.Return();
        var outer = new AssemblyBuilder(new("Outer", new Version(1, 0, 0, 0)), core);
        var method = outer.AddFunction("Call", returnsValue: false); method.Call(two); method.Call(one); method.Call(two); method.Return();
        var bytes = outer.WriteNativeAssembly();
        var snapshot = NativeAssemblyDefinition.ReadAssembly(bytes);
        Array.Clear(bytes);
        Check(snapshot.References.Count == 2 && snapshot.References[0].Equals(second.Identity) && snapshot.References[1].Equals(first.Identity), "exact direct identities in manifest order, repeated calls deduplicated");
        Check(!AssemblyDefinition.ReadAssembly(snapshot.CreateReferenceAssembly(core), false).MainModule.AssemblyReferences.Any(r => r.Identity.Name == "Leaf"), "implementation-only references stay outside primitive reference projection");
        try { ((IList<AssemblyIdentity>)snapshot.References)[0] = first.Identity; throw new Exception("mutable native references"); }
        catch (NotSupportedException) { }
        var wrong = JsonNode.Parse(outer.WriteNativeAssembly())!;
        wrong["references"]![0]!["revision"] = "3.0.0.0";
        Reject(Encoding.UTF8.GetBytes(wrong.ToJsonString()));
        var duplicate = JsonNode.Parse(outer.WriteNativeAssembly())!;
        duplicate["references"]!.AsArray().Add(duplicate["references"]![0]!.DeepClone());
        duplicate["assemblies"]![0]!["references"]!.AsArray().Add(duplicate["assemblies"]![0]!["references"]![0]!.DeepClone());
        Reject(Encoding.UTF8.GetBytes(duplicate.ToJsonString()));
    }
    private static void Reject(byte[] bytes)
    {
        try { NativeAssemblyDefinition.ReadAssembly(bytes); } catch (InvalidDataException) { return; }
        throw new Exception("expected native reader rejection");
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
}
