using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;
using FieldDefinition = NeoCLR.Metadata.Experimental.Model.FieldDefinition;

internal static class ClassConstantChecks
{
    internal static void Run()
    {
        var graph = new AssemblyBuilder(new("Constants", new Version(1, 0, 0, 0)), new("System.Runtime", new Version(10, 0, 0, 0)));
        var owner = graph.AddClass("Example", "Constants");
        owner.Definition.Fields.Add(new FieldDefinition("Answer", 0x8056, PrimitiveType.Int32, 42));
        owner.Definition.Fields.Add(new FieldDefinition("Minimum", 0x8051, PrimitiveType.Int32, int.MinValue));
        owner.Definition.Fields.Add(new FieldDefinition("NaN", 0x8056, PrimitiveType.Double, BitConverter.UInt64BitsToDouble(0x7ff8000000000123)));
        owner.Definition.Fields.Add(new FieldDefinition("Infinity", 0x8056, PrimitiveType.Single, float.PositiveInfinity));
        owner.Definition.Fields.Add(new FieldDefinition("NegativeZero", 0x8056, PrimitiveType.Single, -0.0f));
        if (owner.Fields.Count != 0 || owner.StaticFields.Count != 0) throw new Exception("literal allocated storage");
        foreach (var snapshot in new[] { AssemblyDefinition.ReadAssembly(graph.Write(), false), AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(graph)), AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary), false) })
        {
            var fields = snapshot.MainModule.Types.Single(t => t.Name == "Constants").Fields;
            if (fields.Count != 5 || !Equals(fields.Single(f => f.Name == "Answer").Constant, 42) || fields.Any(f => !f.IsLiteral)) throw new Exception("literal metadata lost");
            if (!Equals(fields.Single(f => f.Name == "Minimum").Constant, int.MinValue)) throw new Exception("signed literal lost");
            if (BitConverter.DoubleToUInt64Bits((double)fields.Single(f => f.Name == "NaN").Constant!) != 0x7ff8000000000123 ||
                !float.IsPositiveInfinity((float)fields.Single(f => f.Name == "Infinity").Constant!) ||
                BitConverter.SingleToUInt32Bits((float)fields.Single(f => f.Name == "NegativeZero").Constant!) != 0x80000000)
                throw new Exception("floating literal bits lost");
        }
        try { owner.Definition.Fields.Add(new FieldDefinition("Answer", 0x8056, PrimitiveType.Int32, 0)); throw new Exception("duplicate literal accepted"); }
        catch (ArgumentException) { }
        try { _ = new FieldDefinition("Wrong", 0x8056, PrimitiveType.Single, 1.0); throw new Exception("wrong constant width accepted"); }
        catch (ArgumentException) { }
        var malformed = System.Text.Encoding.UTF8.GetBytes(System.Text.Encoding.UTF8.GetString(graph.WriteNativeAssembly()).Replace("7ff8000000000123", "not-a-bitstring"));
        try { _ = NativeAssemblyDefinition.ReadAssembly(malformed); throw new Exception("bad literal bits accepted"); }
        catch (InvalidDataException) { }
        Console.WriteLine("PASS class constant CLI/native roundtrip, exact floating bits, malformed rejection and storage exclusion");
    }
}
