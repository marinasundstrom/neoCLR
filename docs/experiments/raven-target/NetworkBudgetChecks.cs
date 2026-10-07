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
                    .SelectMany(t => t.Methods).Where(m => m.Name.EndsWith("Until", StringComparison.Ordinal) && m.Parameters.Any(p => p.ParameterType.MetadataType == MetadataType.Int64)).ToArray();
                if (methods.Length != 10 || methods.Any(m => bootstrap ? !m.IsPublic : !m.IsAssembly))
                    throw new InvalidDataException("Network budget visibility differs from the selected reference profile.");
                var deadline = module.GetType("System.Networking.NetworkDeadline");
                if (!deadline.IsValueType || !deadline.IsPublic || deadline.Fields.Any(f => f.IsPublic))
                    throw new InvalidDataException("Deadline must be opaque public value metadata.");
                var typed = module.Types.SelectMany(t => t.Methods).Where(m => m.Name.EndsWith("Until", StringComparison.Ordinal)
                    && m.Parameters.Any(p => p.ParameterType.FullName == deadline.FullName)).ToArray();
                if (typed.Length != 5 || typed.Any(m => !m.IsPublic || m.Parameters[^1].ParameterType.FullName != CancellationBindings.Token))
                    throw new InvalidDataException("Missing typed public deadline/cancellation contracts.");
                var tokenOverloads = module.Types.Where(t => t.FullName is "System.Networking.Dns" or "System.Networking.Sockets.Socket")
                    .SelectMany(t => t.Methods).Where(m => !m.Name.EndsWith("Until", StringComparison.Ordinal)
                        && m.Parameters.LastOrDefault()?.ParameterType.FullName == CancellationBindings.Token).ToArray();
                if (tokenOverloads.Length != 8)
                    throw new InvalidDataException("Missing public network cancellation overloads.");
                foreach (var method in tokenOverloads) {
                    if (!method.IsPublic || SocketBindings.Bind(method, method, false, false) is null)
                        throw new InvalidDataException("Public network cancellation binding failed.");
                    var parameter = method.Parameters[^1];
                    var token = parameter.ParameterType;
                    parameter.ParameterType = module.TypeSystem.Int32;
                    try {
                        SocketBindings.Bind(method, method, false, false);
                        throw new Exception("Network cancellation accepted a non-token parameter.");
                    } catch (InvalidDataException) { }
                    parameter.ParameterType = token;
                }
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
