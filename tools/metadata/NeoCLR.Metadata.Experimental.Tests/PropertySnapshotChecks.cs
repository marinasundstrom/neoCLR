using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;

internal static class PropertySnapshotChecks
{
    internal static void Run()
    {
        var graph = InstanceObjectChecks.Create();
        var owner = graph.Types[0];
        owner.AddProperty("Number", PrimitiveType.Int32, owner.Methods[1], owner.Methods[3]);
        owner.AddProperty("Pending", PrimitiveType.Boolean, owner.Methods[2]);
        var empty = graph.AddClass("Example", "Empty");
        var utility = graph.AddType("Example", "Utility");
        var getter = utility.AddMethod("Read", new(PrimitiveType.String, [])); getter.Emit(OpCode.Ldstr, "value"); getter.Return();
        utility.AddProperty("Text", PrimitiveType.String, getter);
        var image = graph.Write();
        var snapshot = AssemblyDefinition.ReadAssembly(image, false);
        System.Array.Clear(image);
        Check(snapshot);
        Check(AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(graph.WriteNativeAssembly()).CreateReferenceAssembly(graph.CoreLibrary), false));
        // Corrupt a getter association to a method of another type. Small fixture uses two-byte table indices.
        image = graph.Write();
        using var pe = new PEReader(new MemoryStream(image));
        var reader = pe.GetMetadataReader();
        int semantics = pe.PEHeaders.MetadataStartOffset + reader.GetTableMetadataOffset(TableIndex.MethodSemantics);
        // MetadataTokens supplies the row; do not depend on handle hash representation.
        var foreign = reader.MethodDefinitions.Single(h => reader.GetString(reader.GetMethodDefinition(h).Name) == "Read");
        System.Buffers.Binary.BinaryPrimitives.WriteUInt16LittleEndian(image.AsSpan(semantics + 2, 2), (ushort)MetadataTokens.GetRowNumber(foreign));
        try { AssemblyDefinition.ReadAssembly(image, false); throw new Exception("foreign accessor accepted"); }
        catch (InvalidDataException) { }
    }
    private static void Check(AssemblyDefinition snapshot)
    {
        var module = snapshot.MainModule;
        if (module.Properties.Count != 3 || module.Types.Single(t => t.Name == "Empty").Properties.Count != 0)
            throw new Exception("property ownership/ranges");
        var order = module.Types.Single(t => t.Name == "Order");
        var number = order.Properties.Single(p => p.Name == "Number");
        if (number.Module != module || number.DeclaringType != order || number.MetadataToken != 0x17000001 || number.Attributes != 0 ||
            !ReferenceEquals(number, module.GetPropertyDefinition(number.MetadataToken)) ||
            !ReferenceEquals(number.GetMethod, order.Methods.Single(m => m.Name == "GetNumber")) ||
            !ReferenceEquals(number.SetMethod, order.Methods.Single(m => m.Name == "SetNumber")) || number.OtherMethods.Count != 0)
            throw new Exception("owned accessor identity");
        var signature = number.GetSignature(); signature[0] = 0;
        if (!number.TryGetPrimitiveSignature(out var type, out var isStatic) || type != PrimitiveType.Int32 || isStatic || number.GetSignature()[0] != 0x28)
            throw new Exception("owned primitive instance signature");
        var text = module.Properties.Single(p => p.Name == "Text");
        if (!text.TryGetPrimitiveSignature(out type, out isStatic) || type != PrimitiveType.String || !isStatic || text.SetMethod is not null)
            throw new Exception("static read-only signature");
        if (module.GetPropertyDefinition(0x06000001) is not null || module.GetPropertyDefinition(0x17000004) is not null)
            throw new Exception("wrong token lookup");
    }
}
