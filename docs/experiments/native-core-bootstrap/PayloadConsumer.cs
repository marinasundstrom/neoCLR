using NeoCLR.Metadata.Experimental.Model;
using Raven.CodeAnalysis;
using Raven.CodeAnalysis.NeoClr;
using Raven.CodeAnalysis.Syntax;

internal static class PayloadConsumer
{
    internal static void Build(string output, NeoClrReferenceCatalog catalog, CompilationOptions options)
    {
        const string text = """
            import System.Runtime.CompilerServices.*
            func Main() -> int {
                let payload = neoCLR.Runtime.ConsoleReadByte()
                if !RuntimeServices.IsValue<byte>(payload) {
                    return 1
                }
                if RuntimeServices.IsValue<int>(payload) {
                    return 2
                }
                let value = RuntimeServices.UnpackValue<byte>(payload)
                let end = neoCLR.Runtime.ConsoleReadByte()
                if !RuntimeServices.IsValue<System.Void>(end) {
                    return 3
                }
                return (int)value
            }
            """;
        var tree = SyntaxTree.ParseText(text);
        var compilation = Compilation.Create("PayloadConsumer", [tree], catalog.References.ToArray(), options);
        var errors = compilation.GetDiagnostics().Where(d => d.Severity == DiagnosticSeverity.Error).ToArray();
        if (errors.Length != 0) throw new Exception(string.Join("\n", errors.Select(d => d.ToString())));
        using var image = new MemoryStream();
        var result = compilation.Emit(image, null, new EmitOptions().WithBackend(new NeoClrEmissionBackend(
            new(new("PayloadConsumer", new(1, 0, 0, 0)), catalog.CoreIdentity, catalog.Dependencies))));
        if (!result.Success) throw new Exception(string.Join("\n", result.Diagnostics));
        File.WriteAllBytes(Path.Combine(output, "PayloadConsumer.dll"), image.ToArray());
        File.WriteAllText(Path.Combine(output, "PayloadConsumer.rvn"), text);
        var invalid = Compilation.Create("InvalidPayload",
            [SyntaxTree.ParseText(text.Replace("UnpackValue<byte>", "UnpackValue<int>"))],
            catalog.References.ToArray(), options);
        using var invalidImage = new MemoryStream();
        var invalidResult = invalid.Emit(invalidImage, null, new EmitOptions().WithBackend(new NeoClrEmissionBackend(
            new(new("InvalidPayload", new(1, 0, 0, 0)), catalog.CoreIdentity, catalog.Dependencies))));
        if (!invalidResult.Success) throw new Exception(string.Join("\n", invalidResult.Diagnostics));
        File.WriteAllBytes(Path.Combine(output, "InvalidPayload.dll"), invalidImage.ToArray());
        var completionText = "func Main() -> int { System.Runtime.CompilerServices.RuntimeServices. }";
        var completionTree = SyntaxTree.ParseText(completionText);
        var editor = Compilation.Create("Completion", [completionTree], catalog.References.ToArray(), options);
        var items = new CompletionService().GetCompletions(editor, completionTree, completionText.LastIndexOf('.') + 1);
        if (!items.Any(i => i.DisplayText.StartsWith("UnpackValue", StringComparison.Ordinal)))
            throw new Exception("Native payload helper absent from completion.");
        Console.WriteLine("PASS native payload helpers compile and complete without BootstrapReference");
    }
}
