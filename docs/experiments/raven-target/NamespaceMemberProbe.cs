using Mono.Cecil;
using Raven.CodeAnalysis;
using Raven.CodeAnalysis.Syntax;
using System.Text.Json;

// Reduced target-contract probe. Uses Raven's normal compiler and completion APIs.
static class NamespaceMemberProbe
{
    public static void Write(string output)
    {
        if (Directory.Exists(output)) throw new IOException("Output directory must not exist.");
        Directory.CreateDirectory(output);
        var core = Path.Combine(output, CoreDeclarations.Identity + ".dll");
        CoreDeclarations.Write(core, unionProbe: true, collectionProbe: true);
        var options = new CompilationOptions(OutputKind.DynamicallyLinkedLibrary,
            metadataImportOptions: new MetadataImportOptions(CoreDeclarations.Identity))
            .WithTargetCoreAssemblyName(CoreDeclarations.Identity);
        var references = new List<MetadataReference> { MetadataReference.CreateFromFile(core) };
        Compilation Create(string name, string text, CompilationOptions? selected = null) => Compilation.Create(name,
            [SyntaxTree.ParseText(text)], references.ToArray(), selected ?? options);
        var library = Create("NamespaceLibrary", """
            namespace Workflows
            public func Answer() -> int { return 42 }
            public func Answer(value: int) -> int { return value }
            internal func Hidden() -> int { return 0 }
            """);
        var libraryPath = Path.Combine(output, "NamespaceLibrary.dll");
        using (var stream = File.Create(libraryPath))
        {
            var emitted = library.Emit(stream);
            if (!emitted.Success) throw new InvalidDataException(string.Join("\n", emitted.Diagnostics));
        }
        var checks = new List<string>();
        void Check(string name, bool success)
        {
            if (!success) throw new InvalidDataException("Namespace member check failed: " + name);
            checks.Add(name);
        }
        using (var image = AssemblyDefinition.ReadAssembly(libraryPath))
        {
            var container = image.MainModule.Types.Single(t => t.Namespace == "Workflows");
            Check("emitted target marker", container.CustomAttributes.Any(a => a.AttributeType.FullName == "System.Runtime.CompilerServices.TopLevelAttribute"
                && a.AttributeType.Scope.Name == CoreDeclarations.Identity));
        }
        Check("explicit metadata closure", ClosureAudit.Inspect(libraryPath, core).Length == 0);
        references.Add(MetadataReference.CreateFromFile(libraryPath));
        const string consumer = "import Workflows.*\nclass Consumer { public static func Read() -> int { return Answer() + Answer(1) } }";
        var compilation = Create("Consumer", consumer);
        using (var stream = new MemoryStream())
            Check("separate consumer emission and overloads", compilation.Emit(stream).Success);
        var complete = "import Workflows.*\nfunc Main() { Ans }";
        var tree = SyntaxTree.ParseText(complete);
        var completion = Compilation.Create("Completion", [tree], references.ToArray(), options);
        var items = new CompletionService().GetCompletions(completion, tree, complete.IndexOf("Ans }") + 3).ToArray();
        Check("completion from referenced namespace", items.Any(i => i.DisplayText == "Answer"));
        Check("internal function rejected", Create("Private", consumer.Replace("Answer() + Answer(1)", "Hidden()")).GetDiagnostics().Any(d => d.Severity == DiagnosticSeverity.Error));
        Check("disabled namespace imports rejected", Create("Disabled", consumer, options.WithAllowNamespaceMemberImports(false)).GetDiagnostics().Any(d => d.Severity == DiagnosticSeverity.Error));
        var lookalike = Create("Lookalike", "namespace Pretend\npublic class NamespaceMembers { public static func Fake() -> int { return 0 } }");
        var lookalikePath = Path.Combine(output, "Lookalike.dll");
        using (var stream = File.Create(lookalikePath))
            Check("unmarked class emitted", lookalike.Emit(stream).Success);
        references.Add(MetadataReference.CreateFromFile(lookalikePath));
        Check("container name does not imply namespace membership", Create("NoGuessing", consumer.Replace("Workflows", "Pretend").Replace("Answer() + Answer(1)", "Fake()")).GetDiagnostics().Any(d => d.Severity == DiagnosticSeverity.Error));
        File.WriteAllText(Path.Combine(output, "checks.json"), JsonSerializer.Serialize(checks, new JsonSerializerOptions { WriteIndented = true }));
        Console.WriteLine($"{checks.Count} namespace member checks passed.");
    }
}
