using System.Reflection;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class NominalPropertyChecks
{
    internal static AssemblyBuilder Create()
    {
        var graph = NominalFieldChecks.Create();
        var order = graph.Types[0];
        var holder = graph.Types[1];
        var get = holder.AddInstanceMethod("GetValue", new MethodSignature(order, []));
        get.LoadArgument(0); get.LoadField(holder.Fields[2]); get.Return();
        var set = holder.AddInstanceMethod("SetValue", new MethodSignature(PrimitiveType.Void, [order]), MethodVisibility.Private);
        set.LoadArgument(0); set.LoadArgument(1); set.StoreField(holder.Fields[2]); set.Return();
        holder.AddProperty("Value", order, get, set);
        var replace = holder.AddInstanceMethod("Replace", new MethodSignature(PrimitiveType.Void, [order]));
        replace.LoadArgument(0); replace.LoadArgument(1); replace.Call(set); replace.Return();
        var self = holder.AddInstanceMethod("Self", new MethodSignature(holder, []));
        self.LoadArgument(0); self.Return();
        holder.AddProperty("Current", holder, self);
        var utility = graph.AddType("Example", "Utility");
        var create = utility.AddMethod("Create", new MethodSignature(order, []));
        create.LoadConstant(41); create.Emit(OpCode.Ldc_Bool, true); create.NewObject(order.Methods[0]); create.Return();
        utility.AddProperty("Fresh", order, create);
        var sink = utility.AddMethod("Ignore", new MethodSignature(PrimitiveType.Void, [order])); sink.Return();
        utility.AddProperty("Sink", order, setter: sink);
        var entry = graph.EntryPoint!;
        entry.ClearBody();
        entry.Call(create); entry.NewObject(holder.Methods[0]);
        entry.Duplicate(); entry.Call(create); entry.Call(replace); entry.Call(self); entry.Call(get);
        entry.Duplicate(); entry.LoadConstant(42); entry.Call(order.Methods[3]); entry.Call(order.Methods[1]); entry.Return();
        return graph;
    }

    internal static void Run()
    {
        var graph = Create();
        var loaded = Assembly.Load(graph.Write());
        if (!Equals(loaded.EntryPoint!.Invoke(null, null), 42)) throw new Exception("nominal property execution");
        var holder = loaded.GetType("Example.Holder")!;
        var order = loaded.GetType("Example.Order")!;
        var value = holder.GetProperty("Value")!;
        if (value.PropertyType != order || !value.SetMethod!.IsPrivate || !value.GetMethod!.IsSpecialName)
            throw new Exception("nominal property accessor contract");
        var first = Activator.CreateInstance(order, [41, true]);
        var instance = Activator.CreateInstance(holder, [first]);
        var replacement = Activator.CreateInstance(order, [42, false]);
        value.SetValue(instance, replacement);
        if (!ReferenceEquals(value.GetValue(instance), replacement)) throw new Exception("nominal setter identity");
        var direct = AssemblyDefinition.ReadAssembly(graph.Write(), false);
        var native = graph.WriteNativeAssembly();
        var projection = AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(native).CreateReferenceAssembly(graph.CoreLibrary), false);
        foreach (var type in direct.MainModule.Types)
        {
            var projected = projection.MainModule.Types.Single(t => t.Name == type.Name);
            if (type.Properties.Count != projected.Properties.Count) throw new Exception("property projection count");
            foreach (var (original, copy) in type.Properties.Zip(projected.Properties))
                if (!original.GetSignature().SequenceEqual(copy.GetSignature()) || original.GetMethod?.Name != copy.GetMethod?.Name || original.SetMethod?.Name != copy.SetMethod?.Name)
                    throw new Exception("nominal property projection");
        }
        foreach (var mutation in new Action<JsonNode>[] {
            n => n["types"]![1]!["properties"]![0]!["ty"] = new JsonObject { ["Named"] = "Missing" },
            n => n["types"]![1]!["properties"]![0]!["setter"]!["parameters"]![0] = "Int32",
            n => n["types"]![1]!["properties"]![0]!["ty"] = n["types"]![1]!["properties"]![1]!["ty"]!.DeepClone(),
        })
        {
            var malformed = JsonNode.Parse(native)!; mutation(malformed);
            try { NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(malformed.ToJsonString())); throw new Exception("invalid nominal property accepted"); }
            catch (InvalidDataException) { }
        }
        var owner = graph.Types[1];
        var count = owner.Properties.Count;
        var wrong = owner.AddInstanceMethod("Wrong", new MethodSignature(owner, [])); wrong.LoadArgument(0); wrong.Return();
        foreach (var type in new SignatureType[] { graph.Types[0], NominalFieldChecks.Create().Types[0] })
        {
            try { owner.AddProperty("Bad", type, wrong); throw new Exception("incompatible property accepted"); }
            catch (ArgumentException) { }
        }
        if (owner.Properties.Count != count) throw new Exception("failed property declaration mutated graph");
    }
}
