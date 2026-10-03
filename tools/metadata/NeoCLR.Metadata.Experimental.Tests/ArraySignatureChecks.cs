using System.Reflection;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class ArraySignatureChecks
{
    internal static AssemblyBuilder Create()
    {
        var graph = InstanceObjectChecks.Create();
        var owner = graph.Types[0];
        var items = SignatureType.ArrayOf(owner);
        var identity = graph.AddFunction("IdentityArray", new MethodSignature(items, [items]));
        var local = identity.DeclareLocal(items);
        identity.LoadArgument(0); identity.StoreLocal(local); identity.LoadLocal(local); identity.Return();
        var integers = SignatureType.ArrayOf(PrimitiveType.Int32);
        var other = graph.AddFunction("IdentityArray", new MethodSignature(integers, [integers]));
        other.LoadArgument(0); other.Return();
        var nested = SignatureType.ArrayOf(integers);
        var nestedIdentity = graph.AddFunction("IdentityNestedArray", new MethodSignature(nested, [nested]));
        nestedIdentity.LoadArgument(0); nestedIdentity.Return();
        var field = owner.AddField("Items", items);
        var get = owner.AddInstanceMethod("GetItems", new MethodSignature(items, []));
        get.LoadArgument(0); get.LoadField(field); get.Return();
        var set = owner.AddInstanceMethod("SetItems", new MethodSignature(PrimitiveType.Void, [items]));
        set.LoadArgument(0); set.LoadArgument(1); set.StoreField(field); set.Return();
        owner.AddProperty("Items", items, get, set);
        return graph;
    }

    internal static void Run()
    {
        var graph = Create();
        var loaded = Assembly.Load(graph.Write());
        int[][] nested = [[42], []];
        var nestedIdentity = loaded.ManifestModule.GetMethods().Single(m => m.Name == "IdentityNestedArray");
        if (!ReferenceEquals(nested, nestedIdentity.Invoke(null, [nested]))) throw new Exception("nested vector identity");
        var native = AssemblyDefinition.ReadNativeAssembly(NeoCLR.Metadata.Experimental.RuntimeAssemblyContainer.WriteBinary(graph));
        var context = new NeoCLR.Metadata.Experimental.Introspection.MetadataLoadContext([native]);
        var function = context.Resolve(native.Identity).GetModules().Single().GetFunctions().Single(m => m.Name == "IdentityNestedArray");
        if (function.ReturnType is not NeoCLR.Metadata.Experimental.Introspection.ArrayTypeInfo { ElementType: NeoCLR.Metadata.Experimental.Introspection.ArrayTypeInfo })
            throw new Exception("nested vector introspection roundtrip");
        SignatureType depth = PrimitiveType.Int32;
        for (int i = 0; i < 16; i++) depth = SignatureType.ArrayOf(depth);
        try { SignatureType.ArrayOf(depth); throw new Exception("excessive vector nesting accepted"); } catch (ArgumentException) { }
        var order = loaded.GetType("Example.Order")!;
        var array = Array.CreateInstance(order, 2);
        var identity = loaded.ManifestModule.GetMethods().Single(m => m.Name == "IdentityArray" && m.ReturnType == order.MakeArrayType());
        if (!ReferenceEquals(array, identity.Invoke(null, [array]))) throw new Exception("array local/parameter/result identity");
        var instance = Activator.CreateInstance(order, [42, true]);
        var property = order.GetProperty("Items")!;
        property.SetValue(instance, array);
        if (!ReferenceEquals(array, property.GetValue(instance))) throw new Exception("array property/field identity");
        var projected = AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary), false);
        var direct = AssemblyDefinition.ReadAssembly(graph.Write(), false);
        foreach (var type in direct.MainModule.Types)
        {
            var copy = projected.MainModule.Types.Single(t => t.Name == type.Name);
            if (!type.Methods.Zip(copy.Methods).All(p => p.First.GetSignature().SequenceEqual(p.Second.GetSignature())) ||
                !type.Fields.Zip(copy.Fields).All(p => p.First.GetSignature().SequenceEqual(p.Second.GetSignature())) ||
                !type.Properties.Zip(copy.Properties).All(p => p.First.GetSignature().SequenceEqual(p.Second.GetSignature())))
                throw new Exception("array declaration projection");
        }
        foreach (var element in new SignatureType[] { PrimitiveType.Void, SignatureType.ByReference(PrimitiveType.Int32) })
        {
            try { SignatureType.ArrayOf(element); throw new Exception("invalid array element accepted"); } catch (ArgumentException) { }
        }
        var foreign = SignatureType.ArrayOf(InstanceObjectChecks.Create().Types[0]);
        var count = graph.Types[0].Fields.Count;
        try { graph.Types[0].AddField("Foreign", foreign); throw new Exception("foreign array accepted"); } catch (ArgumentException) { }
        if (graph.Types[0].Fields.Count != count) throw new Exception("failed array declaration mutated graph");
        var malformed = JsonNode.Parse(graph.WriteNativeAssembly())!;
        malformed["types"]![0]!["fields"]![2]!["ty"] = new JsonObject { ["ArrayRef"] = "Void" };
        try { NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(malformed.ToJsonString())); throw new Exception("invalid native array accepted"); } catch (InvalidDataException) { }
        var bad = graph.AddFunction("BadArray", new MethodSignature(SignatureType.ArrayOf(PrimitiveType.Int32), [SignatureType.ArrayOf(PrimitiveType.Boolean)]));
        bad.LoadArgument(0); bad.Call(graph.Functions.Single(m => m.Name == "IdentityArray" && m.Signature.ReturnType.ArrayElement!.Primitive == PrimitiveType.Int32)); bad.Return();
        try { graph.Write(); throw new Exception("mismatched array call accepted"); } catch (InvalidDataException) { }
    }
}
