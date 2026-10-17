using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class CoreBaseTypeChecks
{
    internal static void Run()
    {
        var identity = new AssemblyIdentity("SourceCore", new Version(1, 0, 0, 0));
        var graph = new AssemblyBuilder(identity, identity);
        var root = graph.AddNativeObjectRoot();
        var valueType = graph.AddClass("System", "ValueType", root);
        valueType.SetAbstractClass();
        var enumBase = graph.AddClass("System", "Enum", valueType);
        if (enumBase.IsValueType || enumBase.IsEnum)
            throw new Exception("System.Enum must remain a reference class.");
        enumBase.SetAbstractClass();
        var number = graph.AddValueType("Example", "Number");
        var choice = graph.AddEnum("Example", "Choice");
        choice.AddEnumMember("First", 1);
        if (!number.IsValueType || number.IsEnum || !choice.IsValueType || !choice.IsEnum)
            throw new Exception("Ordinary value or enum classification changed.");
        graph.AddClass("System.Runtime.CompilerServices", "ReferenceAssemblyAttribute", root);
        var image = RuntimeAssemblyContainer.WriteLibraryBinary(graph);
        var projected = RuntimeAssemblyContainer.ReadCliProjection(image);
        var projectedEnum = projected.MainModule.Types.Single(t => t.Namespace == "System" && t.Name == "Enum");
        if (projectedEnum.IsValueType || projectedEnum.IsEnum)
            throw new Exception("CLI Enum base classification changed on roundtrip.");
        var loaded = AssemblyDefinition.ReadNativeAssembly(image);
        var loadedEnum = loaded.MainModule.Types.Single(t => t.Namespace == "System" && t.Name == "Enum");
        if (loadedEnum.IsValueType || loadedEnum.IsEnum)
            throw new Exception("Native Enum base classification changed on roundtrip.");
        foreach (var snapshot in new[] { projected, loaded })
        {
            var ordinary = snapshot.MainModule.Types.Single(t => t.Name == "Number");
            var enumeration = snapshot.MainModule.Types.Single(t => t.Name == "Choice");
            if (!ordinary.IsValueType || ordinary.IsEnum || !enumeration.IsValueType || !enumeration.IsEnum)
                throw new Exception("Value/enum identity changed on roundtrip.");
        }
        var other = new AssemblyBuilder(new("Consumer", new Version(1, 0, 0, 0)), identity);
        var otherRoot = other.AddNativeObjectRoot();
        var namedBase = other.AddClass("System", "ValueType", otherRoot);
        var namedEnum = other.AddClass("System", "Enum", namedBase);
        if (!namedEnum.IsValueType)
            throw new Exception("A non-core declaration acquired the core Enum exception.");
        Console.WriteLine("PASS source-core base classes and ordinary value/enum identities");
    }
}
