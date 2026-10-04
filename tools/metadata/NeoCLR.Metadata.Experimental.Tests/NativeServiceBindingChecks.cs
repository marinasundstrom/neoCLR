using NeoCLR.Metadata.Experimental.Model;

internal static class NativeServiceBindingChecks
{
    internal static void Run(string seedPath, string corePath)
    {
        var core = AssemblyDefinition.ReadAssembly(File.ReadAllBytes(corePath), false);
        var seed = NativeLibraryDefinition.ReadAssembly(File.ReadAllBytes(seedPath));
        var graph = new AssemblyBuilder(new("ServiceContracts", new Version(1, 0, 0, 0)), core.Identity);
        graph.BindNativeLibrary(core, seed, core.Identity);
        var methods = core.MainModule.Types.Single(t => t.Namespace == "System.Runtime.CompilerServices" && t.Name == "RuntimeServices").Methods;
        foreach (var method in methods)
            _ = graph.ImportReference(method, core.Identity);
        Console.WriteLine($"PASS {methods.Count} exact native service declaration/binding contracts");
    }
}
