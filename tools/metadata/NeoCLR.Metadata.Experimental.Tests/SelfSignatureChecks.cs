using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Introspection;
using NeoCLR.Metadata.Experimental.Model;

internal static class SelfSignatureChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("NeoCLR.CoreProbe", new Version(1, 0, 0, 0));
        var builder = new AssemblyBuilder(new("SelfContracts", new Version(1, 0, 0, 0)), core);
        var contract = builder.AddInterface("Example", "Copyable");
        contract.AddInterfaceMethod("Copy", new MethodSignature(SignatureType.Self, [SignatureType.Self]));
        var getter = contract.AddInterfaceMethod("get_Current", new MethodSignature(SignatureType.Self, []));
        contract.AddProperty("Current", SignatureType.Self, getter);
        var manual = new MethodDefinition("Many", 0x546, new MethodSignature(SignatureType.ArrayOf(SignatureType.Self), [SignatureType.Self]));
        contract.Definition.Methods.Add(manual);
        var generic = builder.AddGenericInterface("Example", "Other", ["T"]);
        generic.AddInterfaceMethod("Choose", new MethodSignature(SignatureType.Self, [SignatureType.TypeParameter(0)]));
        var box = builder.AddGenericClass("Example", "Box", ["T"]);
        contract.AddInterfaceMethod("Box", new MethodSignature(box.MakeGenericInstance(SignatureType.Self), []));
        box.AddField("Value", PrimitiveType.Int32);
        var main = builder.AddFunction("Main", new MethodSignature(PrimitiveType.Int32, []));
        main.LoadConstant(42); main.Return(); builder.EntryPoint = main;
        Check(SignatureType.Self.IsSelf && SignatureType.Self.TypeParameterIndex is null && SignatureType.Self.MethodParameterIndex is null && SignatureType.Self.ToString() == "Self", "distinct signature category");
        Reject<ArgumentException>(() => builder.AddFunction("Bad", new MethodSignature(SignatureType.Self, [])));
        Reject<ArgumentException>(() => box.AddField("Bad", SignatureType.Self));
        Reject<ArgumentException>(() => box.AddField("NestedBad", box.MakeGenericInstance(SignatureType.Self)));
        Reject<ArgumentException>(() => box.AddInstanceMethod("Bad", new MethodSignature(SignatureType.Self, [])));
        Reject<InvalidDataException>(() => builder.Write());
        foreach (var image in new[] { RuntimeAssemblyContainer.WriteBinary(builder), RuntimeAssemblyContainer.Write(builder.WriteNativeAssembly(), core) })
        {
            var snapshot = AssemblyDefinition.ReadNativeAssembly(image);
            var context = new MetadataLoadContext([snapshot]);
            var types = context.Resolve(snapshot.Identity).GetTypes();
            var copy = types.Single(t => t.Name == "Copyable");
            var method = copy.GetMethods().Single(m => m.Name == "Copy");
            var self = (SelfTypeInfo)method.ReturnType;
            Check(ReferenceEquals(self.DeclaringType, copy) && !self.IsNominalType && copy.GenericArity == 0, "implicit interface scope without generic arity");
            Check(ReferenceEquals(self, method.GetParameters().Single().ParameterType), "canonical parameter and result");
            Check(ReferenceEquals(self, ((ArrayTypeInfo)copy.GetMethods().Single(m => m.Name == "Many").ReturnType).ElementType), "manual definition/vector parity");
            Check(ReferenceEquals(self, ((ConstructedTypeInfo)copy.GetMethods().Single(m => m.Name == "Box").ReturnType).TypeArguments.Single()), "nested construction");
            var property = copy.GetProperties().Single();
            Check(ReferenceEquals(property.PropertyType, self) && ReferenceEquals(property.GetMethod, copy.GetMethods().Single(m => m.Name == "get_Current")), "canonical Self property accessor");
            var other = types.Single(t => t.Name == "Other`1");
            Check(other.GenericArity == 1 && !ReferenceEquals(other.GetMethods().Single().ReturnType, self), "owner scopes differ");
            var closed = other.MakeGenericType(context.ResolveSignature(PrimitiveType.Int32));
            var closedMethod = closed.GetMethods().Single();
            Check(ReferenceEquals(((SelfTypeInfo)closedMethod.ReturnType).DeclaringType, closed) && closedMethod.GetParameters().Single().ParameterType is PrimitiveTypeInfo { Kind: PrimitiveType.Int32 }, "Self and ordinary generic substitution remain independent");
            Reject<InvalidDataException>(() => context.ResolveSignature(SignatureType.Self));
            Check(RuntimeAssemblyContainer.Read(snapshot.Write()).SequenceEqual(RuntimeAssemblyContainer.Read(image)), "lossless native snapshot");
        }
        var json = JsonNode.Parse(builder.WriteNativeAssembly())!;
        json["functions"]![0]!["returns"] = "SelfType";
        Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(json.ToJsonString())));
        json = JsonNode.Parse(builder.WriteNativeAssembly())!;
        json["functions"]![0]!["locals"] = new JsonArray("SelfType");
        Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(json.ToJsonString())));
        json = JsonNode.Parse(builder.WriteNativeAssembly())!;
        json["types"]!.AsArray().Last()!["fields"]![0]!["ty"] = "SelfType";
        Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(json.ToJsonString())));
        if (Environment.GetEnvironmentVariable("NEOCLR_SELF_ARTIFACT") is { } path)
            File.WriteAllBytes(path, RuntimeAssemblyContainer.WriteBinary(builder));
    }
    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
