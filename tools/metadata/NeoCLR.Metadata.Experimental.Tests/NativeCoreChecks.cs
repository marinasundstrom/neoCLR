using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;

internal static class NativeCoreChecks
{
    internal static void Run()
    {
        var identity = new AssemblyIdentity("NativeCore", new(1, 0, 0, 0));
        AssemblyBuilder Create(bool includeValueBase = true, bool includeMarker = true)
        {
            var graph = new AssemblyBuilder(identity, identity);
            var root = graph.AddNativeObjectRoot();
            var rootCtor = root.AddConstructor(Array.Empty<PrimitiveType>());
            rootCtor.GetILGenerator().Return();
            if (includeValueBase) graph.AddClass("System", "ValueType", root);
            if (includeMarker)
            {
                var attribute = graph.AddClass("System", "Attribute", root);
                var attributeCtor = attribute.AddConstructor(Array.Empty<PrimitiveType>());
                attributeCtor.GetILGenerator().LoadArgument(0);
                attributeCtor.GetILGenerator().Call(rootCtor);
                attributeCtor.GetILGenerator().Return();
                var marker = graph.AddClass("System.Runtime.CompilerServices", "ReferenceAssemblyAttribute", attribute);
                var markerCtor = marker.AddConstructor(Array.Empty<PrimitiveType>());
                markerCtor.GetILGenerator().LoadArgument(0);
                markerCtor.GetILGenerator().Call(attributeCtor);
                markerCtor.GetILGenerator().Return();
            }
            graph.AddValueType("System", "Int32").SetNativePrimitive(PrimitiveType.Int32);
            return graph;
        }
        var graph = Create();
        var image = RuntimeAssemblyContainer.WriteLibraryBinary(graph);
        if (!image.SequenceEqual(RuntimeAssemblyContainer.WriteLibraryBinary(graph)))
            throw new Exception("self-core output is not deterministic");
        var native = AssemblyDefinition.ReadNativeAssembly(image);
        if (native.MainModule.AssemblyReferences.Count != 0 ||
            native.MainModule.Types.Single(t => t.Namespace == "System" && t.Name == "Int32").NativePrimitive != PrimitiveType.Int32)
            throw new Exception("self-core retained external dependencies or lost primitive ownership");
        using var pe = new PEReader(new MemoryStream(image));
        var reader = pe.GetMetadataReader();
        if (reader.AssemblyReferences.Count != 0 || reader.TypeReferences.Count != 0)
            throw new Exception("physical self-core references must use local definitions");
        foreach (var handle in reader.TypeDefinitions)
        {
            var type = reader.GetTypeDefinition(handle);
            var name = reader.GetString(type.Name);
            if (name == "Object" && !type.BaseType.IsNil) throw new Exception("Object must be baseless");
            if (name == "Int32" && (type.BaseType.Kind != HandleKind.TypeDefinition ||
                reader.GetString(reader.GetTypeDefinition((TypeDefinitionHandle)type.BaseType).Name) != "ValueType"))
                throw new Exception("primitive base did not resolve locally");
        }
        var attributeRow = reader.GetCustomAttribute(reader.GetAssemblyDefinition().GetCustomAttributes().Single());
        var constructor = reader.GetMemberReference((MemberReferenceHandle)attributeRow.Constructor);
        if (constructor.Parent.Kind != HandleKind.TypeDefinition ||
            reader.GetString(reader.GetTypeDefinition((TypeDefinitionHandle)constructor.Parent).Name) != "ReferenceAssemblyAttribute")
            throw new Exception("reference assembly marker must resolve locally");
        Reject(() => RuntimeAssemblyContainer.WriteLibraryBinary(Create(includeValueBase: false)));
        Reject(() => RuntimeAssemblyContainer.WriteLibraryBinary(Create(includeMarker: false)));
        Reject(() => graph.Write());
        var missingRoot = new AssemblyBuilder(identity, identity);
        missingRoot.AddClass("System", "Object");
        Reject(() => RuntimeAssemblyContainer.WriteLibraryBinary(missingRoot));
        Console.WriteLine("PASS native self-core local definitions, primitive ownership, marker, deterministic output and rejection controls");
    }

    private static void Reject(Action action)
    {
        try { action(); }
        catch (InvalidDataException) { return; }
        throw new Exception("Expected self-core rejection");
    }
}
