using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class AssemblyConstantChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var builder = new AssemblyBuilder(new AssemblyIdentity("Constants", new Version(1, 0, 0, 0)), core);
        builder.AddConstant(new("System.Math", "Pi", Math.PI));
        builder.AddConstant(new("System.Math", "E", Math.E));
        builder.AddConstant(new("System.Math", "Tau", Math.Tau));
        builder.AddConstant(new("", "NegativeZero", -0.0, MethodVisibility.Internal));
        var owner = builder.AddClass("Example", "Container");
        owner.AddNestedValueType("Nested");
        owner.AddField("Value", PrimitiveType.Int32);
        var owned = owner.AddMethod("Owned", new MethodSignature(PrimitiveType.Int32, []));
        owned.LoadConstant(7); owned.Return();
        var first = builder.AddFunction("Example.Math", "Read", new MethodSignature(PrimitiveType.Int32, []));
        first.LoadConstant(1); first.Return();
        var second = builder.AddFunction("Example.Math", "Read", new MethodSignature(PrimitiveType.Int32, [PrimitiveType.Int32]));
        second.GetILGenerator().LoadArgument(0); second.Return();
        CheckMembers(builder.Definition);
        var other = new AssemblyBuilder(new AssemblyIdentity("Other", new Version(1, 0, 0, 0)), core);
        Reject<InvalidOperationException>(() => other.AddConstant(builder.Definition.MainModule.Constants[0]));
        if (other.Definition.GetMembers().Count != 0) throw new Exception("failed reattachment changed owner");
        other.AddConstant(new("System.Math", "Pi", Math.PI));
        var sameName = other.Definition.GetMembers().Single();
        if (sameName.FullName != "System.Math.Pi" || ReferenceEquals(sameName.Assembly, builder.Definition))
            throw new Exception("qualified names must retain distinct assembly owners");
        Reject<ArgumentException>(() => builder.AddConstant(new("System.Math", "Pi", 1)));
        foreach (var bad in new[] { double.NaN, double.PositiveInfinity, double.NegativeInfinity })
            Reject<ArgumentException>(() => new AssemblyConstantDefinition("N", "Bad", bad));
        foreach (var image in new[] { RuntimeAssemblyContainer.WriteBinary(builder), RuntimeAssemblyContainer.WriteLibraryBinary(builder) })
        {
            var snapshot = AssemblyDefinition.ReadNativeAssembly(image);
            CheckMembers(snapshot);
            var facade = new NeoCLR.Metadata.Experimental.Introspection.MetadataLoadContext([snapshot]).Assemblies.Single();
            var views = facade.GetMembers();
            if (views.Count != 7 || views.Any(m => !ReferenceEquals(m.Assembly, facade) || !ReferenceEquals(m.Module.Assembly, facade)))
                throw new Exception("assembly member facade ownership");
            if (views.Single(m => m.Kind == AssemblyMemberKind.Type).Type?.Name != "Container" ||
                views.Count(m => m.Kind == AssemblyMemberKind.Function && m.FullName == "Example.Math.Read" && m.Function?.DeclaringType is null) != 2 ||
                views.Single(m => m.FullName == "System.Math.Pi").ConstantValue != Math.PI)
                throw new Exception("assembly member facade kinds/names");
            var values = snapshot.MainModule.Constants;
            if (values.Count != 4 || values[0].Namespace != "System.Math" || values[0].Name != "Pi" || values[0].Value != Math.PI ||
                values[1].Value != Math.E || values[2].Value != Math.Tau || BitConverter.DoubleToInt64Bits(values[3].Value) != long.MinValue ||
                values[3].Visibility != MethodVisibility.Internal) throw new Exception("assembly-level constant round trip");
        }
        Reject<InvalidDataException>(() => builder.Write());
        Reject<NotSupportedException>(() => NativeAssemblyDefinition.ReadAssembly(builder.WriteNativeAssembly()).CreateReferenceAssembly(core));
        var source = JsonNode.Parse(builder.WriteNativeAssembly())!.AsObject();
        void Bad(Action<JsonArray> change)
        {
            var copy = source.DeepClone();
            change(copy["assemblies"]![0]!["constants"]!.AsArray());
            Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(copy.ToJsonString())));
        }
        Bad(rows => rows.Add(rows[0]!.DeepClone()));
        Bad(rows => rows[0]!["type"] = "Single");
        Bad(rows => rows[0]!["bits"] = "7ff0000000000000");
        Bad(rows => rows[0]!["bits"] = "XYZ");
        Bad(rows => rows[0]!["visibility"] = "private");
        Bad(rows => rows[0]!["namespace"] = "A..B");
        Bad(rows => rows[0]!["extra"] = 1);
        Console.WriteLine("PASS assembly-level members: writer/reader ownership, qualified names, kinds, overloads, constant bits and invalid contracts");
    }
    private static void CheckMembers(AssemblyDefinition assembly)
    {
        var members = assembly.GetMembers();
        if (members.Count != 7 || members.Any(m => !ReferenceEquals(m.Assembly, assembly) || !ReferenceEquals(m.Module, assembly.MainModule)))
            throw new Exception("assembly member ownership/count");
        if (members.Count(m => m.Kind == AssemblyMemberKind.Type) != 1 ||
            members.Count(m => m.Kind == AssemblyMemberKind.Function && m.FullName == "Example.Math.Read") != 2 ||
            members.Count(m => m.Kind == AssemblyMemberKind.Constant) != 4 ||
            members.Any(m => new object?[] { m.Type, m.Function, m.Constant }.Count(v => v is not null) != 1))
            throw new Exception("assembly member discriminated declarations");
        var pi = members.Single(m => m.FullName == "System.Math.Pi").Constant!;
        if (pi.FullName != "System.Math.Pi" || !ReferenceEquals(pi.Assembly, assembly) || !ReferenceEquals(pi.Module, assembly.MainModule))
            throw new Exception("constant declaration ownership");
        if (members.Any(m => m.Name is "Nested" or "Value" or "Owned")) throw new Exception("type-owned declaration leaked into assembly members");
        if (members.Where(m => m.Kind == AssemblyMemberKind.Function).Select(m => { m.Function!.TryGetSignature(out var signature); return signature!.ParameterTypes.Count; }).Distinct().Count() != 2)
            throw new Exception("overload signatures lost");
    }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("Expected " + typeof(T).Name);
    }
}
