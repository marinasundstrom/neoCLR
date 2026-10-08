using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class NamespaceConstantChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var builder = new AssemblyBuilder(new AssemblyIdentity("Constants", new Version(1, 0, 0, 0)), core);
        builder.AddNamespaceConstant(new("System.Math", "Pi", Math.PI));
        builder.AddNamespaceConstant(new("System.Math", "E", Math.E));
        builder.AddNamespaceConstant(new("System.Math", "Tau", Math.Tau));
        builder.AddNamespaceConstant(new("", "NegativeZero", -0.0, MethodVisibility.Internal));
        Reject<ArgumentException>(() => builder.AddNamespaceConstant(new("System.Math", "Pi", 1)));
        foreach (var bad in new[] { double.NaN, double.PositiveInfinity, double.NegativeInfinity })
            Reject<ArgumentException>(() => new NamespaceConstantDefinition("N", "Bad", bad));
        foreach (var image in new[] { RuntimeAssemblyContainer.WriteBinary(builder), RuntimeAssemblyContainer.WriteLibraryBinary(builder) })
        {
            var snapshot = AssemblyDefinition.ReadNativeAssembly(image);
            var values = snapshot.MainModule.NamespaceConstants;
            if (values.Count != 4 || values[0].Namespace != "System.Math" || values[0].Name != "Pi" || values[0].Value != Math.PI ||
                values[1].Value != Math.E || values[2].Value != Math.Tau || BitConverter.DoubleToInt64Bits(values[3].Value) != long.MinValue ||
                values[3].Visibility != MethodVisibility.Internal) throw new Exception("namespace constant round trip");
        }
        Reject<InvalidDataException>(() => builder.Write());
        Reject<NotSupportedException>(() => NativeAssemblyDefinition.ReadAssembly(builder.WriteNativeAssembly()).CreateReferenceAssembly(core));
        var source = JsonNode.Parse(builder.WriteNativeAssembly())!.AsObject();
        void Bad(Action<JsonArray> change)
        {
            var copy = source.DeepClone();
            change(copy["assemblies"]![0]!["namespace_constants"]!.AsArray());
            Reject<InvalidDataException>(() => NativeAssemblyDefinition.ReadAssembly(System.Text.Encoding.UTF8.GetBytes(copy.ToJsonString())));
        }
        Bad(rows => rows.Add(rows[0]!.DeepClone()));
        Bad(rows => rows[0]!["type"] = "Single");
        Bad(rows => rows[0]!["bits"] = "7ff0000000000000");
        Bad(rows => rows[0]!["bits"] = "XYZ");
        Bad(rows => rows[0]!["visibility"] = "private");
        Bad(rows => rows[0]!["namespace"] = "A..B");
        Bad(rows => rows[0]!["extra"] = 1);
        Console.WriteLine("PASS namespace Double constants: metadata round trips, exact bits and invalid contracts");
    }
    private static void Reject<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("Expected " + typeof(T).Name);
    }
}
