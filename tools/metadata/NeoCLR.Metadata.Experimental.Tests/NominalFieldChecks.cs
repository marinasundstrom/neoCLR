using System.Reflection;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class NominalFieldChecks
{
    internal static AssemblyBuilder Create()
    {
        var graph = InstanceObjectChecks.Create();
        var order = graph.Types[0];
        var holder = graph.AddClass("Example", "Holder");
        // Forward and self references must not depend on declaration order.
        var later = graph.AddClass("Example", "Later");
        holder.AddField("Later", later);
        holder.AddField("Next", holder);
        var value = holder.AddField("Value", order, FieldVisibility.Public);
        var ctor = holder.AddConstructor(new MethodSignature(PrimitiveType.Void, [order]));
        ctor.LoadArgument(0); ctor.LoadArgument(1); ctor.StoreField(value); ctor.Return();
        var entry = graph.EntryPoint!;
        entry.ClearBody();
        var alias = entry.DeclareLocal(order);
        entry.LoadConstant(41); entry.Emit(OpCode.Ldc_Bool, true); entry.NewObject(order.Methods[0]);
        entry.Duplicate(); entry.StoreLocal(alias); entry.NewObject(ctor);
        entry.LoadField(value); entry.LoadConstant(42); entry.Call(order.Methods[3]);
        entry.LoadLocal(alias); entry.Call(order.Methods[1]); entry.Return();
        return graph;
    }

    internal static void Run()
    {
        var graph = Create();
        var loaded = Assembly.Load(graph.Write());
        if (!Equals(loaded.EntryPoint!.Invoke(null, null), 42)) throw new Exception("nominal field alias execution");
        if (loaded.GetType("Example.Holder")!.GetField("Value")!.FieldType != loaded.GetType("Example.Order"))
            throw new Exception("CLI field class identity");
        var direct = AssemblyDefinition.ReadAssembly(graph.Write(), false);
        var native = NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly());
        var projection = AssemblyDefinition.ReadAssembly(native.CreateReferenceAssembly(graph.CoreLibrary), false);
        var fields = direct.MainModule.Types.Single(t => t.Name == "Holder").Fields;
        var projected = projection.MainModule.Types.Single(t => t.Name == "Holder").Fields;
        if (fields.Count != projected.Count || !fields.Zip(projected).All(pair => pair.First.GetSignature().SequenceEqual(pair.Second.GetSignature())))
            throw new Exception("nominal field projection mismatch");
        var malformed = JsonNode.Parse(graph.WriteNativeAssembly())!;
        malformed["types"]![1]!["fields"]![0]!["ty"] = new JsonObject { ["Named"] = "Missing" };
        try { NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(malformed.ToJsonString())); throw new Exception("unknown nominal field accepted"); }
        catch (InvalidDataException) { }
        var holder = graph.Types[1];
        var count = holder.Fields.Count;
        try { holder.AddField("Foreign", Create().Types[0]); throw new Exception("foreign field accepted"); }
        catch (ArgumentException) { }
        if (holder.Fields.Count != count) throw new Exception("failed field addition mutated graph");
        var entry = graph.EntryPoint!;
        var other = graph.Types[2].AddConstructor([]); other.Return();
        entry.ClearBody(); entry.LoadConstant(0); entry.Emit(OpCode.Ldc_Bool, false); entry.NewObject(graph.Types[0].Methods[0]);
        entry.NewObject(holder.Methods[0]); entry.NewObject(other); entry.StoreField(holder.Fields[2]); entry.LoadConstant(42); entry.Return();
        try { graph.Write(); throw new Exception("wrong nominal field value accepted"); }
        catch (InvalidDataException) { }
    }
}
