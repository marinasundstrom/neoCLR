using Raven.CodeAnalysis;
using Raven.CodeAnalysis.NeoClr;
using System.Text.Json;
if (args.Length is not (2 or 3)) throw new ArgumentException("Specify bundle, fresh output directory and optional neoCLR repository root for website styling.");
var bundle = Path.GetFullPath(args[0]);
var output = Path.GetFullPath(args[1]);
if (Directory.Exists(output)) throw new IOException("Output must be fresh.");
Directory.CreateDirectory(output);
// Historical bundle sidecars contain XML blocks mislabeled as Markdown. Normalize
// only those complete XML blocks in a private input copy for this migration audit.
if (args.Length == 3)
{
    var inputCopy = Path.Combine(output, "inputs");
    foreach (var file in Directory.EnumerateFiles(bundle, "*", SearchOption.AllDirectories))
    {
        var target = Path.Combine(inputCopy, Path.GetRelativePath(bundle, file));
        Directory.CreateDirectory(Path.GetDirectoryName(target)!);
        File.Copy(file, target);
    }
    var normalized = new List<string>();
    foreach (var file in Directory.EnumerateFiles(inputCopy, "*.md", SearchOption.AllDirectories))
    {
        var text = File.ReadAllText(file);
        var delimiter = text.IndexOf("\n---", StringComparison.Ordinal);
        if (!text.StartsWith("---\n", StringComparison.Ordinal) || delimiter < 0) continue;
        var start = delimiter + 4;
        var body = text[start..].Trim();
        if (!body.StartsWith("<summary>", StringComparison.Ordinal)) continue;
        var xml = System.Xml.Linq.XElement.Parse("<doc>" + body + "</doc>");
        if (xml.Elements().Any(element => element.HasElements || element.Name.LocalName is not ("summary" or "param" or "returns" or "remarks")))
            throw new InvalidDataException("Unsupported historical XML sidecar: " + file);
        var markdown = string.Join("\n\n", xml.Elements().Select(element => element.Name.LocalName switch
        {
            "param" => "@param " + element.Attribute("name")!.Value + " " + element.Value,
            "returns" => "@returns " + element.Value,
            "remarks" => "@remarks " + element.Value,
            _ => element.Value
        }));
        File.WriteAllText(file, text[..start] + "\n\n" + markdown + "\n");
        normalized.Add(Path.GetRelativePath(inputCopy, file));
    }
    File.WriteAllText(Path.Combine(output, "normalized-sidecars.json"), JsonSerializer.Serialize(normalized));
    bundle = inputCopy;
}
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
if (args.Length == 3)
{
    // Recover reviewed XML help only when its canonical ID matches a real native
    // declaration. NamespaceMembers is the documented temporary CLI carrier.
    var sourceXml = System.Xml.Linq.XDocument.Load(Path.Combine(Path.GetFullPath(args[2]), "api-docs/NeoCLR.CoreProbe.xml"));
    var comments = sourceXml.Descendants("member").ToDictionary(
        element => element.Attribute("name")!.Value.Replace(".NamespaceMembers.", ".", StringComparison.Ordinal));
    var idBuilder = typeof(Compilation).Assembly.GetType("Raven.CodeAnalysis.Documentation.DocumentationCommentIdBuilder")!
        .GetMethod("TryGetMemberId")!;
    var restored = new List<object>();
    foreach (var assembly in assemblies)
    {
        var xmlPath = Path.Combine(bundle, assembly.Name + ".xml");
        var xml = System.Xml.Linq.XDocument.Load(xmlPath);
        var members = xml.Root!.Element("members")!;
        var existing = members.Elements("member").Select(element => element.Attribute("name")!.Value).ToHashSet();
        void Recover(ISymbol symbol)
        {
            object?[] arguments = [symbol, null];
            if ((bool)idBuilder.Invoke(null, arguments)! && arguments[1] is string id &&
                comments.TryGetValue(id, out var comment) && existing.Add(id))
            {
                var copy = new System.Xml.Linq.XElement(comment);
                copy.SetAttributeValue("name", id);
                members.Add(copy);
                restored.Add(new { assembly = assembly.Name, id, sourceId = comment.Attribute("name")!.Value });
            }
            if (symbol is INamespaceOrTypeSymbol owner)
                foreach (var child in owner.GetMembers()) Recover(child);
        }
        Recover(assembly.GlobalNamespace);
        xml.Save(xmlPath);
    }
    File.WriteAllText(Path.Combine(output, "restored-xml-comments.json"), JsonSerializer.Serialize(restored, new JsonSerializerOptions { WriteIndented = true }));
    // Reload immutable documentation snapshots after preparing the private inputs.
    config = provider.Load(Path.Combine(bundle, "Docs.rvnproj"), "Documentation", CompilationOptions.NeoCLR.WithOutputKind(OutputKind.DynamicallyLinkedLibrary), settings, paths);
    compilation = Compilation.Create("Documentation", syntaxTrees: [], references: config.References.ToArray(), options: config.Options);
    errors = compilation.GetDiagnostics().Where(d => d.Severity == DiagnosticSeverity.Error).ToArray();
    if (errors.Length != 0) throw new InvalidDataException(string.Join("\n", errors.Select(d => d.ToString())));
    assemblies = config.References.OfType<NeoClrMetadataReference>().Select(r => (IAssemblySymbol)compilation.GetAssemblyOrModuleSymbol(r)!).ToArray();
}
File.WriteAllText(Path.Combine(output, "constants.json"), JsonSerializer.Serialize(
    config.References.OfType<NeoClrMetadataReference>().Select(reference => new
    {
        assembly = reference.Definition.Identity.Name,
        constants = reference.Definition.MainModule.Constants.Select(constant => new { constant.Namespace, constant.Name, constant.Value, visibility = constant.Visibility.ToString() })
    }), new JsonSerializerOptions { WriteIndented = true }));
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
var site = Path.Combine(output, "site");
var apiOutput = site;
var options = new DocumentationSiteOptions([], ExtensionNamespaces: ["System", "System.Linq", "System.Runtime.Reflection", "System.Tasks"]);
if (args.Length == 3)
{
    var repository = Path.GetFullPath(args[2]);
    using var document = JsonDocument.Parse(File.ReadAllText(Path.Combine(repository, "website/site.json")));
    var theme = document.RootElement;
    string? Setting(string name) => theme.TryGetProperty(name, out var value) ? value.GetString() : null;
    var links = theme.GetProperty("links").EnumerateArray()
        .Select(link => new DocumentationSiteLink(link.GetProperty("label").GetString()!, link.GetProperty("url").GetString()!)).ToArray();
    apiOutput = Path.Combine(site, "docs/api");
    options = options with
    {
        Links = links,
        SiteRootDirectory = site,
        ProjectName = Setting("name"),
        Logo = Setting("logo"),
        Stylesheet = Setting("stylesheet"),
        Footer = Setting("footer"),
        Subtitle = Setting("subtitle"),
        Notice = "Development · Native API migration preview · Coverage validation in progress",
        ReleaseUrl = Setting("releaseUrl"),
        ReleaseLabel = Setting("releaseLabel"),
        Favicon = Setting("favicon"),
        NamespaceNavigation = Setting("namespaceNavigation") ?? "flat",
        SharedApiNavigation = true,
        ApiContent = Path.Combine(repository, "website/api-content")
    };
}
// Audit harness only: invoke the existing internal combined renderer in the host tool.
// No inspected library is loaded or executed through reflection.
typeof(DocumentationGenerator).GetMethod("ProcessAssemblies", System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Static)!
 .Invoke(null, [compilation, assemblies, apiOutput, options]);
Console.WriteLine("PASS production native metadata rendering, with explicit temporary CLI bootstrap");
