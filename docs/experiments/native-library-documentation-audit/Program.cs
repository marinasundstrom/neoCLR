using Raven.CodeAnalysis;
using Raven.CodeAnalysis.NeoClr;
using System.Text.Json;
if (args.Length != 2) throw new ArgumentException("Specify bundle and fresh output directory.");
var bundle = Path.GetFullPath(args[0]);
var output = Path.GetFullPath(args[1]);
if (Directory.Exists(output)) throw new IOException("Output must be fresh.");
Directory.CreateDirectory(output);
var names = new[] { "System.Runtime", "System.Data", "System.Networking", "System.Web" };
var paths = names.Select(n => Path.Combine(bundle, n + ".dll")).ToArray();
var provider = new NeoClrProjectMetadataProvider();
var settings = new Dictionary<string, string>
{
    ["RavenNeoClrCoreReference"] = Path.Combine(bundle, "Core.dll"),
    ["RavenNeoClrRuntimeSeed"] = Path.Combine(bundle, "System.runtime.neox"),
    ["RavenNeoClrBootstrapOwnership"] = Path.Combine(bundle, "ownership.json"),
    ["RavenNeoClrObjectLibrary"] = "System.Runtime",
    ["RavenNeoClrAsyncLibrary"] = "System.Runtime"
};
var config = provider.Load(Path.Combine(bundle, "Docs.rvnproj"), "Documentation", CompilationOptions.NeoCLR.WithOutputKind(OutputKind.DynamicallyLinkedLibrary), settings, paths);
var compilation = Compilation.Create("Documentation", syntaxTrees: [], references: config.References.ToArray(), options: config.Options);
var errors = compilation.GetDiagnostics().Where(d => d.Severity == DiagnosticSeverity.Error).ToArray();
if (errors.Length != 0) throw new Exception(string.Join("\n", errors.Select(d => d.ToString())));
var assemblies = config.References.OfType<NeoClrMetadataReference>().Select(r => (IAssemblySymbol)compilation.GetAssemblyOrModuleSymbol(r)!).ToArray();
var inventory = new List<object>();
void Visit(INamespaceOrTypeSymbol owner)
{
    foreach (var member in owner.GetMembers())
    {
        if (member is INamespaceSymbol ns) Visit(ns);
        else if (member is INamedTypeSymbol type && type.DeclaredAccessibility == Accessibility.Public)
        {
            inventory.Add(new { name = type.ToDisplayString(), assembly = type.ContainingAssembly.Name, kind = type.TypeKind.ToString() }); Visit(type);
        }
    }
}
foreach (var assembly in assemblies) Visit(assembly.GlobalNamespace);
File.WriteAllText(Path.Combine(output, "types.json"), JsonSerializer.Serialize(inventory, new JsonSerializerOptions { WriteIndented = true }));
// Audit harness only: invoke the existing internal combined renderer in the host tool.
// No inspected library is loaded or executed through reflection.
typeof(DocumentationGenerator).GetMethod("ProcessAssemblies", System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Static)!
 .Invoke(null, [compilation, assemblies, Path.Combine(output, "site"), new DocumentationSiteOptions([], ExtensionNamespaces: ["System", "System.Linq", "System.Runtime.Reflection", "System.Tasks"])]);
Console.WriteLine("PASS production native metadata rendering, with explicit temporary CLI bootstrap");
