using System.Runtime.Loader;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using NeoCLR.Metadata.Experimental.Introspection;

internal static class MemberAttributeChecks
{
    internal static void Run(string? output = null)
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var cli = new AssemblyBuilder(new("MethodAttributesCli", new(1, 0, 0, 0)), core);
        var owner = cli.AddClass("Tests", "Operations");
        var method = owner.AddMethod("Read", new(PrimitiveType.Int32, []));
        method.LoadConstant(42); method.Return();
        var marker = new CustomAttributeDefinition(cli.Definition.MainModule.ImportReference(core, "System", "ObsoleteAttribute"), [new(PrimitiveType.String, "test ☃")]);
        method.AddCustomAttribute(marker);
        var image = cli.Write();
        var loaded = AssemblyDefinition.ReadAssembly(image, false);
        var callable = loaded.MainModule.Types.Single(t => t.Name == "Operations").Methods.Single();
        Check((string)callable.CustomAttributes.Single().GetArguments()[0].Value! == "test ☃", "CLI method attribute missing");
        Reject<NotSupportedException>(() => callable.CustomAttributes.Add(marker));
        var foreign = new AssemblyBuilder(new("Foreign", new(1, 0, 0, 0)), core);
        Reject<ArgumentException>(() => foreign.AddFunction("Wrong", new(PrimitiveType.Void, [])).AddCustomAttribute(marker));
        var context = new AssemblyLoadContext("method-attribute-check", isCollectible: true);
        try
        {
            var reflected = context.LoadFromStream(new MemoryStream(image)).GetType("Tests.Operations")!.GetMethod("Read")!;
            Check((string)reflected.GetCustomAttributesData().Single().ConstructorArguments[0].Value! == "test ☃", "CLR method attribute mismatch");
            Check((int)reflected.Invoke(null, null)! == 42, "attributed method behavior changed");
        }
        finally { context.Unload(); }

        var graph = new AssemblyBuilder(new("TestDiscovery", new(1, 0, 0, 0)), new("Core", new(1, 0, 0, 0)));
        var attributeType = graph.AddClass("NeoClr.Testing", "TestAttribute");
        var ctor = attributeType.AddConstructor(new MethodSignature(PrimitiveType.Void, []));
        ctor.GetILGenerator().Fail("Discovery must never execute attribute constructors");
        var declared = new CustomAttributeDefinition(ctor.Definition, []);
        ctor.AddCustomAttribute(declared);
        var first = graph.DefineModule("Tests.Collections").AddFunction("First", new(PrimitiveType.Int32, []));
        first.AddCustomAttribute(declared);
        first.GetILGenerator().Fail("Discovery must never execute tests");
        var last = graph.DefineModule("Tests.Collections").AddFunction("Last", new(PrimitiveType.Int32, []));
        last.AddCustomAttribute(declared);
        last.LoadConstant(7); last.Return();
        var fixture = graph.AddClass("Tests", "OptionalFixture");
        fixture.AddCustomAttribute(declared);
        var field = fixture.AddField("Number", PrimitiveType.Int32);
        field.Definition.CustomAttributes.Add(declared);
        var getter = fixture.AddMethod("get_Value", new(PrimitiveType.Int32, []));
        getter.LoadConstant(9); getter.Return();
        var property = fixture.AddProperty("Value", PrimitiveType.Int32, getter, null);
        property.Definition.CustomAttributes.Add(declared);
        var member = fixture.AddMethod("Example", new(PrimitiveType.Int32, [PrimitiveType.Int32]));
        member.Definition.GetParameterCustomAttributes(0).Add(declared);
        Reject<ArgumentOutOfRangeException>(() => member.Definition.GetParameterCustomAttributes(1));
        member.AddCustomAttribute(declared); member.LoadConstant(9); member.Return();
        var main = graph.AddFunction("Main", new(PrimitiveType.Int32, []));
        main.LoadConstant(42); main.Return(); graph.EntryPoint = main;
        var json = graph.WriteNativeAssembly();
        var binary = RuntimeAssemblyContainer.WriteBinary(json, graph.CoreLibrary);
        if (output is not null) File.WriteAllBytes(output, binary);
        foreach (var bytes in new[] { binary, NativeModuleContainer.WriteLibraryBinary(json) })
        {
            var snapshot = AssemblyDefinition.ReadNativeAssembly(bytes);
            var catalog = new MetadataLoadContext([snapshot]);
            var assembly = catalog.Resolve(snapshot.Identity);
            var expectedType = assembly.GetTypes().Single(t => t.Name == "TestAttribute");
            Check(expectedType.GetConstructors().Single().GetCustomAttributes().Single().GetAttributeType() == expectedType, "constructor attribute lost");
            var discovered = assembly.GetModules().Single().GetFunctions()
                .Where(f => f.GetCustomAttributes().Any(a => ReferenceEquals(a.GetAttributeType(), expectedType)))
                .Select(f => f.Name).Order(StringComparer.Ordinal).ToArray();
            Check(discovered.SequenceEqual(new[] { "First", "Last" }), "classless discovery lost method attributes");
            Check(assembly.GetTypes().Single(t => t.Name == "OptionalFixture").GetMethods().Single(m => m.Name == "Example").GetCustomAttributes().Single().GetAttributeType() == expectedType,
                "optional fixture method missing attribute");
            var fixtureView = assembly.GetTypes().Single(t => t.Name == "OptionalFixture");
            Check(fixtureView.GetCustomAttributes().Single().GetAttributeType() == expectedType, "type attribute lost");
            Check(fixtureView.GetFields().Single().GetCustomAttributes().Single().GetAttributeType() == expectedType, "field attribute lost");
            Check(fixtureView.GetProperties().Single().GetCustomAttributes().Single().GetAttributeType() == expectedType, "property attribute lost");
            Check(fixtureView.GetMethods().Single(m => m.Name == "Example").GetParameters().Single().GetCustomAttributes().Single().GetAttributeType() == expectedType, "parameter attribute lost");
            var loadedFixture = snapshot.MainModule.Types.Single(t => t.Name == "OptionalFixture");
            Reject<NotSupportedException>(() => loadedFixture.Fields.Single().CustomAttributes.Add(declared));
            Reject<NotSupportedException>(() => loadedFixture.Properties.Single().CustomAttributes.Add(declared));
            Reject<NotSupportedException>(() => loadedFixture.Methods.Single(m => m.Name == "Example").GetParameterCustomAttributes(0).Add(declared));
        }
        var projected = RuntimeAssemblyContainer.ReadCliProjection(binary);
        Check(projected.MainModule.Functions.Count(f => f.CustomAttributes.Count == 1) == 2, "CLI projection lost function attributes");
        Check(projected.MainModule.Types.Single(t => t.Name == "OptionalFixture").Methods.Single(m => m.Name == "Example").CustomAttributes.Count == 1, "CLI projection lost method attribute");
        var projectedFixture = projected.MainModule.Types.Single(t => t.Name == "OptionalFixture");
        Check(projectedFixture.Fields.Single().CustomAttributes.Count == 1, "CLI field projection lost attribute");
        Check(projectedFixture.Properties.Single().CustomAttributes.Count == 1, "CLI property projection lost attribute");
        Check(projectedFixture.Methods.Single(m => m.Name == "Example").GetParameterCustomAttributes(0).Count == 1, "CLI parameter projection lost attribute");
        foreach (var corruption in new[] { "constructor", "arguments", "target", "count" })
        {
            var bad = JsonNode.Parse(json)!;
            var attributes = bad["functions"]!.AsArray().First(f => f!["custom_attributes"] is not null)!["custom_attributes"]!.AsArray();
            var attribute = attributes[0]!;
            if (corruption == "constructor") attribute["constructor"]!["name"] = "Missing";
            if (corruption == "arguments") attribute["arguments"]!.AsArray().Add(new JsonObject { ["Int32"] = 1 });
            if (corruption == "target") attribute["target_token"] = 0;
            if (corruption == "count") for (int i = 0; i < 256; i++) attributes.Add(attribute.DeepClone());
            Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(bad.ToJsonString())));
        }
        var wrongTarget = JsonNode.Parse(json)!;
        var attributedType = wrongTarget["types"]!.AsArray().Single(t => t!["custom_attributes"] is JsonArray a && a.Any(value => value!["target_token"] is not null))!;
        attributedType["custom_attributes"]!.AsArray().First(a => a!["target_token"] is not null)!["target_token"] = 0x0400ffff;
        Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(wrongTarget.ToJsonString())));
        Console.WriteLine("PASS member attribute authoring, CLI/native snapshots, type/field/property/parameter inspection, metadata-only discovery, projection and invalid metadata");
    }

    private static void Check(bool condition, string message)
    {
        if (!condition) throw new Exception(message);
    }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); }
        catch (T) { return; }
        throw new Exception("Expected " + typeof(T).Name);
    }
}
