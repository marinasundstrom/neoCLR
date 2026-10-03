using System.Runtime.Loader;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Introspection;
using NeoCLR.Metadata.Experimental.Model;

internal static class TaggedValueLayoutChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("System.Private.CoreLib", typeof(object).Assembly.GetName().Version!, "", "7cec85d7bea7798e");
        var graph = new AssemblyBuilder(new("TaggedPayload", new Version(1, 0, 0, 0)), core);
        var payload = graph.AddGenericValueType("Example", "Payload", ["T"]);
        var value = payload.AddField("Value", SignatureType.TypeParameter(0), FieldVisibility.Internal);
        var carrier = graph.AddGenericValueType("Example", "Carrier", ["T"]);
        var tag = carrier.AddField("Tag", PrimitiveType.Int32, FieldVisibility.Internal);
        var stored = carrier.AddField("Payload", payload.MakeGenericInstance(SignatureType.TypeParameter(0)), FieldVisibility.Internal);
        var entry = graph.AddFunction("Main"); graph.EntryPoint = entry;
        var main = entry.GetILGenerator();
        var original = main.DeclareLocal(carrier.MakeGenericInstance(PrimitiveType.Int32));
        var copy = main.DeclareLocal(original.SignatureType);
        var item = main.DeclareLocal(payload.MakeGenericInstance(PrimitiveType.Int32));
        var tagRef = tag.MakeConstructedReference(PrimitiveType.Int32);
        var payloadRef = stored.MakeConstructedReference(PrimitiveType.Int32);
        var valueRef = value.MakeConstructedReference(PrimitiveType.Int32);
        main.LoadDefault(original.SignatureType); main.StoreLocal(original);
        main.LoadDefault(item.SignatureType); main.StoreLocal(item);
        main.LoadLocal(original); main.LoadField(tagRef); Verify(0);
        main.LoadLocal(original); main.LoadField(payloadRef); main.LoadField(valueRef); Verify(0);
        main.LoadLocalAddress(item); main.LoadConstant(40); main.StoreField(valueRef);
        main.LoadLocalAddress(original); main.LoadConstant(1); main.StoreField(tagRef);
        main.LoadLocalAddress(original); main.LoadLocal(item); main.StoreField(payloadRef);
        main.LoadLocal(original); main.StoreLocal(copy);
        main.LoadLocalAddress(item); main.LoadConstant(2); main.StoreField(valueRef);
        main.LoadLocalAddress(copy); main.LoadLocal(item); main.StoreField(payloadRef);
        main.LoadLocal(copy); main.LoadField(tagRef); Verify(1);
        main.LoadLocal(original); main.LoadField(payloadRef); main.LoadField(valueRef);
        main.LoadLocal(copy); main.LoadField(payloadRef); main.LoadField(valueRef); main.Add(); main.Return();
        void Verify(int expected)
        {
            var ok = main.DefineLabel();
            main.LoadConstant(expected); main.Emit(OpCode.Ceq); main.Emit(OpCode.Brtrue, ok);
            main.LoadConstant(99); main.Return(); main.MarkLabel(ok);
        }
        var context = new AssemblyLoadContext("tagged-value-layout", true);
        try
        {
            var assembly = context.LoadFromStream(new MemoryStream(graph.Write()));
            Check(Equals(42, assembly.EntryPoint!.Invoke(null, null)), "CLR payload copy and tag");
        }
        finally { context.Unload(); }
        var image = RuntimeAssemblyContainer.WriteBinary(graph);
        var snapshot = AssemblyDefinition.ReadNativeAssembly(image);
        var metadata = new MetadataLoadContext([snapshot]);
        var definition = metadata.Resolve(snapshot.Identity).GetTypes().Single(t => t.Name == "Carrier`1");
        Check(definition.IsValueType && definition.IsSealed, "native value declaration flags");
        var constructed = definition.MakeGenericType(metadata.ResolveSignature(PrimitiveType.Int32));
        var field = (ConstructedTypeInfo)constructed.GetFields().Single(f => f.Name == "Payload").FieldType;
        Check(field.Definition.IsValueType && field.GetFields().Single().FieldType is PrimitiveTypeInfo { Kind: PrimitiveType.Int32 }, "constructed inline payload projection");
        Check(snapshot.Write().SequenceEqual(image), "immutable native snapshot copy");
        if (Environment.GetEnvironmentVariable("NEOCLR_TAGGED_ARTIFACT") is { } path) File.WriteAllBytes(path, image);

        var malformed = new AssemblyBuilder(new("Malformed", new Version(1, 0, 0, 0)), core);
        malformed.AddValueType("", "Left").AddField("Value", PrimitiveType.Int32);
        malformed.AddValueType("", "Right").AddField("Value", PrimitiveType.Int32);
        var json = JsonNode.Parse(malformed.WriteNativeAssembly())!;
        json["types"]![0]!["fields"]![0]!["ty"] = new JsonObject { ["Named"] = "Right" };
        json["types"]![1]!["fields"]![0]!["ty"] = new JsonObject { ["Named"] = "Left" };
        var invalid = Encoding.UTF8.GetBytes(json.ToJsonString());
        Reject(() => NativeAssemblyDefinition.ReadAssembly(invalid));
        Reject(() => AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(invalid, core)));

        var cycle = new AssemblyBuilder(new("Recursive", new Version(1, 0, 0, 0)), core);
        var left = cycle.AddValueType("", "Left"); var right = cycle.AddValueType("", "Right");
        left.AddField("Right", right); right.AddField("Left", left);
        Reject(() => cycle.Write()); Reject(() => cycle.WriteNativeAssembly());
        var growing = new AssemblyBuilder(new("Growing", new Version(1, 0, 0, 0)), core);
        var first = growing.AddGenericValueType("", "First", ["T"]);
        var second = growing.AddGenericValueType("", "Second", ["T"]);
        first.AddField("Next", second.MakeGenericInstance(first.MakeGenericInstance(SignatureType.TypeParameter(0))));
        second.AddField("Next", SignatureType.TypeParameter(0));
        Reject(() => growing.WriteNativeAssembly());
        var finite = new AssemblyBuilder(new("Finite", new Version(1, 0, 0, 0)), core);
        var box = finite.AddGenericValueType("", "Box", ["T"]); box.AddField("Value", SignatureType.TypeParameter(0));
        finite.AddValueType("", "Holder").Definition.Fields.Add(new FieldDefinition("Value", 6, box.MakeGenericInstance(box.MakeGenericInstance(PrimitiveType.Int32))));
        _ = finite.WriteNativeAssembly();
    }
    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
    private static void Reject(Action action)
    {
        try { action(); } catch (InvalidDataException) { return; }
        throw new Exception("recursive inline layout was accepted");
    }
}
