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
        if (owner.Fields.Count != 0 || owner.StaticFields.Count != 0) throw new Exception("literal allocated storage");
        foreach (var snapshot in new[] { AssemblyDefinition.ReadAssembly(graph.Write(), false), AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteLibraryBinary(graph)), AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary), false) })
        {
            var fields = snapshot.MainModule.Types.Single(t => t.Name == "Constants").Fields;
            if (fields.Count != 2 || fields.Single(f => f.Name == "Answer").Constant != 42 || fields.Any(f => !f.IsLiteral)) throw new Exception("literal metadata lost");
            if (fields.Single(f => f.Name == "Minimum").Constant != int.MinValue) throw new Exception("signed literal lost");
        }
        try { owner.Definition.Fields.Add(new FieldDefinition("Answer", 0x8056, PrimitiveType.Int32, 0)); throw new Exception("duplicate literal accepted"); }
        catch (ArgumentException) { }
        Console.WriteLine("PASS class constant CLI/native roundtrip and storage exclusion");
    }
}
