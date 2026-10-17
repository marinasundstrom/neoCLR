using NeoCLR.Metadata.Experimental.Model;
using Raven.CodeAnalysis;
using Raven.CodeAnalysis.NeoClr;
using Raven.CodeAnalysis.Syntax;

internal static class IntrinsicConsumer
{
    internal static void Build(string output, NeoClrReferenceCatalog catalog, CompilationOptions options)
    {
        const string text = """
            import System.Runtime.CompilerServices.*
            func Main() -> int {
                let values = CheckedStorage.Reserve<int>(2)
                values[0] = 40
                values[1] = 2
                let references = CheckedStorage.Reserve<string>(1)
                references[0] = "native"
                RuntimeServices.TypeHandle<int>()
                RuntimeServices.TypeHandle<string>()
                return values[0] + values[1]
            }
            """;
        var tree = SyntaxTree.ParseText(text);
        var compilation = Compilation.Create("IntrinsicConsumer", [tree], catalog.References.ToArray(), options);
        var errors = compilation.GetDiagnostics().Where(d => d.Severity == DiagnosticSeverity.Error).ToArray();
        if (errors.Length != 0) throw new Exception(string.Join("\n", errors.Select(d => d.ToString())));
        using var image = new MemoryStream();
        var result = compilation.Emit(image, null, new EmitOptions().WithBackend(new NeoClrEmissionBackend(
            new(new("IntrinsicConsumer", new(1, 0, 0, 0)), catalog.CoreIdentity, catalog.Dependencies))));
        if (!result.Success) throw new Exception(string.Join("\n", result.Diagnostics));
        File.WriteAllBytes(Path.Combine(output, "IntrinsicConsumer.dll"), image.ToArray());
        File.WriteAllText(Path.Combine(output, "IntrinsicConsumer.rvn"), text);
        var invalid = Compilation.Create("InvalidReservation",
            [SyntaxTree.ParseText(text.Replace("Reserve<int>(2)", "Reserve<int>(-1)"))],
            catalog.References.ToArray(), options);
        using var invalidImage = new MemoryStream();
        var invalidResult = invalid.Emit(invalidImage, null, new EmitOptions().WithBackend(new NeoClrEmissionBackend(
            new(new("InvalidReservation", new(1, 0, 0, 0)), catalog.CoreIdentity, catalog.Dependencies))));
        if (!invalidResult.Success) throw new Exception(string.Join("\n", invalidResult.Diagnostics));
        File.WriteAllBytes(Path.Combine(output, "InvalidReservation.dll"), invalidImage.ToArray());
        var completionText = "func Main() -> int { System.Runtime.CompilerServices.CheckedStorage. }";
        var completionTree = SyntaxTree.ParseText(completionText);
        var editor = Compilation.Create("Completion", [completionTree], catalog.References.ToArray(), options);
        var items = new CompletionService().GetCompletions(editor, completionTree, completionText.LastIndexOf('.') + 1);
        if (!items.Any(i => i.DisplayText.StartsWith("Reserve", StringComparison.Ordinal)))
            throw new Exception("Native reservation contract absent from completion.");
        Console.WriteLine("PASS native intrinsic method import, generic value/reference calls and completion without BootstrapReference");
    }
}
