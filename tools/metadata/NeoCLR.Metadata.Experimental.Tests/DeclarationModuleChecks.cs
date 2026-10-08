using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class DeclarationModuleChecks
{
    internal static void Run(string? output)
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var builder = new AssemblyBuilder(new AssemblyIdentity("Package", new Version(1, 0, 0, 0)), core);
        builder.DefineModule("Example.Empty");
        var math = builder.DefineModule("Example.Math");
        if (!ReferenceEquals(math, builder.DefineModule("Example.Math"))) throw new Exception("unstable module");
        builder.DefineModule("Example.Data").AddClass("Item").AddNestedValueType("Nested");
        math.AddConstant("Pi", Math.PI);
        var f = math.AddFunction("Read", new MethodSignature(PrimitiveType.Int32, []));
        f.LoadConstant(42); f.Return(); builder.EntryPoint = f;
        foreach (var image in new[] { RuntimeAssemblyContainer.WriteLibraryBinary(builder), NativeModuleContainer.WriteLibraryBinary(builder.WriteNativeAssembly()) })
        {
            var read = AssemblyDefinition.ReadNativeAssembly(image);
            var modules = read.GetModules();
            if (!modules.Select(m => m.Name).SequenceEqual(new[] { "Example.Data", "Example.Empty", "Example.Math" }) || modules.Any(m => m.IsProjection)) throw new Exception("module table round trip");
            if (modules[1].GetMembers().Count != 0 || modules[2].GetMembers().Count != 2 || modules[0].GetMembers().Count != 1) throw new Exception("module membership");
            foreach (var m in modules)
                if (m.GetMembers().Any(member => !ReferenceEquals(member.DeclaringModule, m))) throw new Exception("module owner");
            var other = new AssemblyBuilder(new AssemblyIdentity("DifferentPackage", new Version(1, 0, 0, 0)), core);
            other.DefineModule("Example.Math");
            var catalog = new MetadataLoadContext([read, AssemblyDefinition.ReadNativeAssembly(NativeModuleContainer.WriteLibraryBinary(other.WriteNativeAssembly()))]);
            if (catalog.GetDeclarationModules().Count(m => m.Name == "Example.Math") != 2) throw new Exception("same path across assemblies merged");
        }
        var json = JsonNode.Parse(builder.WriteNativeAssembly())!;
        void Bad(Action<JsonNode> mutate)
        {
            var copy = json.DeepClone(); mutate(copy["assemblies"]![0]!["declaration_modules"]!);
            try { NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(copy.ToJsonString())); }
            catch (InvalidDataException) { return; }
            throw new Exception("invalid module table accepted");
        }
        Bad(table => table["version"] = 2);
        Bad(table => table["names"]!.AsArray().Add("Example.Math"));
        Bad(table => table["names"]!.AsArray().Clear());
        Bad(table => table["names"]!.AsArray().Add("Bad..Path"));
        json["assemblies"]![0]!.AsObject().Remove("declaration_modules");
        var legacy = AssemblyDefinition.ReadNativeAssembly(NativeModuleContainer.WriteLibraryBinary(Encoding.UTF8.GetBytes(json.ToJsonString())));
        if (legacy.GetModules().Any(m => !m.IsProjection) || legacy.GetModules().Count != 2) throw new Exception("legacy projection");
        if (output is not null) File.WriteAllBytes(output, builder.WriteNativeAssembly());
        Console.WriteLine("PASS logical module ownership, empty modules, same-path assemblies, legacy projection and invalid manifests");
    }
}
