using NeoCLR.Metadata.Experimental;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;

internal static class AssemblyModelChecks
{
    internal static void Run(string ordinaryPath, string markedPath, string invalidPath, string excessiveNamesPath)
    {
        var ordinaryBytes = File.ReadAllBytes(ordinaryPath);
        var markedBytes = File.ReadAllBytes(markedPath);
        var ordinary = AssemblyDefinition.ReadAssembly(ordinaryBytes, expectedExtended: false);
        var marked = AssemblyDefinition.ReadAssembly(markedBytes);
        Require(ordinary.Name == "NeoMetadataContainerProbe", "assembly name");
        Require(ordinary.Name == marked.Name && ordinary.Version == marked.Version, "assembly identity fields");
        Require(ordinary.MainModule.Name == marked.MainModule.Name && ordinary.MainModule.Mvid == marked.MainModule.Mvid, "module fields");
        Require(ordinary.Profile is null && marked.Profile is not null, "structural profile attachment");
        Require(ReferenceEquals(marked.MainModule.Assembly, marked), "assembly ownership");
        var owner = marked.MainModule.Types.Single(t => t.Namespace == "Δοκιμή" && t.Name == "Container`1");
        var nested = marked.MainModule.Types.Single(t => t.Name == "Nested");
        Require(owner.GenericArity == 1 && ReferenceEquals(nested.DeclaringType, owner), "generic/nesting");
        Require(owner.DeclaringType is null && ReferenceEquals(owner.Module, marked.MainModule), "type ownership");
        Require(marked.MainModule.Types[0].Name == "<Module>", "module pseudo-type");
        var reference = owner.ToReference();
        Require(ReferenceEquals(reference.Resolve(), owner) && ReferenceEquals(reference.Module, owner.Module), "reference resolution");
        var otherOwner = ordinary.MainModule.GetTypeDefinition(owner.MetadataToken);
        Require(otherOwner is not null && !ReferenceEquals(otherOwner, owner), "distinct snapshot scope");
        Require(!ReferenceEquals(otherOwner!.ToReference().Resolve(), reference.Resolve()), "same token does not merge scopes");
        Require(marked.MainModule.GetTypeDefinition(0) is null && marked.MainModule.GetTypeDefinition(0x01000001) is null &&
            marked.MainModule.GetTypeDefinition(0x02ffffff) is null, "missing/wrong-kind token");
        Require(ordinary.MainModule.Types.Select(t => (t.MetadataToken, t.Namespace, t.Name, t.GenericArity)).SequenceEqual(
            marked.MainModule.Types.Select(t => (t.MetadataToken, t.Namespace, t.Name, t.GenericArity))), "declaration preservation");
        Array.Clear(ordinaryBytes);
        Array.Clear(markedBytes);
        Require(owner.Name == "Container`1" && reference.Resolve().GenericArity == 1, "owned data after reader disposal");
        Reject(() => AssemblyDefinition.ReadAssembly(File.ReadAllBytes(ordinaryPath)));
        Reject(() => AssemblyDefinition.ReadAssembly(File.ReadAllBytes(invalidPath)));
        Reject(() => AssemblyDefinition.ReadAssembly(File.ReadAllBytes(excessiveNamesPath), false), "decoded declaration names");
        Console.WriteLine("assembly/module declarations, Unicode, generic arity, nesting, scoped references, ownership and malformed CLI checks passed");
    }
    private static void Reject(Action action, string? expected = null)
    {
        try { action(); }
        catch (InvalidDataException error)
        {
            Require(expected is null || error.Message.Contains(expected, StringComparison.Ordinal), "wrong rejection reason");
            return;
        }
        throw new Exception("invalid assembly input accepted");
    }
    private static void Require(bool condition, string message)
    {
        if (!condition) throw new Exception(message);
    }
}
