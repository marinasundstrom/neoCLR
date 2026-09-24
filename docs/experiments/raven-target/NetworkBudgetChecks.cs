using Mono.Cecil;

static class NetworkBudgetChecks
{
    public static void Run()
    {
        var folder = Path.Combine(Path.GetTempPath(), "neoclr-budget-contract-" + Guid.NewGuid());
        Directory.CreateDirectory(folder);
        try
        {
            foreach (var bootstrap in new[] { false, true })
            {
                var path = Path.Combine(folder, bootstrap ? "Bootstrap.dll" : "Application.dll");
                CoreDeclarations.Write(path, unionProbe: true, collectionProbe: true, libraryBootstrap: bootstrap);
                using var assembly = AssemblyDefinition.ReadAssembly(path);
                var module = assembly.MainModule;
                var methods = module.Types.Where(t => t.FullName is "System.Networking.Dns" or "System.Networking.Sockets.Socket")
                    .SelectMany(t => t.Methods).Where(m => m.Name.EndsWith("Until", StringComparison.Ordinal)).ToArray();
                if (methods.Length != 4 || methods.Any(m => bootstrap ? !m.IsPublic : !m.IsAssembly))
                    throw new InvalidDataException("Network budget visibility differs from the selected reference profile.");
                SocketBindings.Project(module);
                foreach (var method in methods)
                {
                    if (!method.IsAssembly || SocketBindings.Bind(method, method, false, true) is null)
                        throw new InvalidDataException("Internal library budget binding failed.");
                    try
                    {
                        SocketBindings.Bind(method, method, false, false);
                        throw new Exception("Application imported a private budget helper.");
                    }
                    catch (InvalidDataException) { }
                    try
                    {
                        RuntimeSignatures.Match(method, method, GenericUnionBindings.Type);
                        throw new Exception("Generic signature matching admitted an internal method by default.");
                    }
                    catch (InvalidDataException) { }
                }
            }
            Console.WriteLine("Network budget bootstrap/application visibility checks passed");
        }
        finally { Directory.Delete(folder, true); }
    }
}
