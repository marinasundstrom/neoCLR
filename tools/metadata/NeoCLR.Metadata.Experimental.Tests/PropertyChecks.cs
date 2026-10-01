using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental.Model;

internal static class PropertyChecks
{
    internal static void Run()
    {
        var graph = InstanceObjectChecks.Create();
        var owner = graph.Types[0];
        var number = owner.AddProperty("Number", PrimitiveType.Int32, owner.Methods[1], owner.Methods[3]);
        owner.AddProperty("Pending", PrimitiveType.Boolean, owner.Methods[2]);
        var utility = graph.AddType("Example", "Utility");
        var get = utility.AddMethod("Read", new(PrimitiveType.Int64, [])); get.Emit(OpCode.Ldc_I8, 99L); get.Return();
        utility.AddProperty("Value", PrimitiveType.Int64, get);
        var sink = utility.AddMethod("Write", new(PrimitiveType.Void, [PrimitiveType.String]), MethodVisibility.Private); sink.Return();
        utility.AddProperty("Sink", PrimitiveType.String, setter: sink);
        if (number.DeclaringType != owner || number.IsStatic || number.GetMethod != owner.Methods[1] || number.SetMethod != owner.Methods[3])
            throw new Exception("property handle contract");
        var loaded = Assembly.Load(graph.Write());
        var type = loaded.GetType("Example.Order")!;
        var value = Activator.CreateInstance(type, [41, true]);
        var property = type.GetProperty("Number")!;
        if (!Equals(property.GetValue(value), 41)) throw new Exception("getter association");
        property.SetValue(value, 42);
        if (!Equals(property.GetValue(value), 42) || !property.GetMethod!.IsSpecialName || !property.SetMethod!.IsSpecialName)
            throw new Exception("setter association and CLI accessor flags");
        if (!Equals(loaded.GetType("Example.Utility")!.GetProperty("Value")!.GetValue(null), 99L)) throw new Exception("static property");
        var native = graph.WriteNativeAssembly();
        using var projection = new PEReader(new MemoryStream(NativeAssemblyDefinition.ReadAssembly(native).CreateReferenceAssembly(graph.CoreLibrary)));
        var reader = projection.GetMetadataReader();
        var projected = reader.PropertyDefinitions.Select(reader.GetPropertyDefinition).Single(p => reader.GetString(p.Name) == "Number");
        if (!reader.GetBlobBytes(projected.Signature).SequenceEqual(new byte[] { 0x28, 0, 0x08 }) || projected.GetAccessors().Getter.IsNil || projected.GetAccessors().Setter.IsNil)
            throw new Exception("projected property associations");
        var projectedSink = reader.PropertyDefinitions.Select(reader.GetPropertyDefinition).Single(p => reader.GetString(p.Name) == "Sink");
        if (!projectedSink.GetAccessors().Getter.IsNil || projectedSink.GetAccessors().Setter.IsNil ||
            (reader.GetMethodDefinition(projectedSink.GetAccessors().Setter).Attributes & MethodAttributes.MemberAccessMask) != MethodAttributes.Private)
            throw new Exception("write-only private setter projection");
        void Reject(Action action)
        {
            var count = owner.Properties.Count;
            try { action(); throw new Exception("invalid property accepted"); } catch (ArgumentException) { }
            if (owner.Properties.Count != count) throw new Exception("rejected property mutated collection");
        }
        Reject(() => owner.AddProperty("Bad", PrimitiveType.Int32));
        Reject(() => owner.AddProperty("Bad", PrimitiveType.Int32, get));
        Reject(() => owner.AddProperty("Bad", PrimitiveType.Void, owner.Methods[1]));
        Reject(() => owner.AddProperty("Reuse", PrimitiveType.Int32, owner.Methods[1]));
        var wrong = owner.AddInstanceMethod("Wrong", new(PrimitiveType.Int64, [])); wrong.Emit(OpCode.Ldc_I8, 0L); wrong.Return();
        Reject(() => owner.AddProperty("Bad", PrimitiveType.Int32, wrong));
        Reject(() => owner.AddProperty("Number", PrimitiveType.Int64, wrong));
        foreach (var mutation in new Action<JsonNode>[] {
            n => n["types"]![0]!["origin"]!["property_tokens"]![0] = 0x17000002,
            n => n["types"]![0]!["properties"]![0]!["getter"]!["instance"] = false,
            n => n["types"]![0]!["properties"]![0]!["ty"] = "Int64",
            n => n["types"]![0]!["properties"]![0]!["getter"]!["name"] = "missing",
        })
        {
            var node = JsonNode.Parse(native)!; mutation(node);
            try { NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(node.ToJsonString())); throw new Exception("bad native property accepted"); }
            catch (InvalidDataException) { }
        }
    }
}
