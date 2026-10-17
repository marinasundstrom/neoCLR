using NeoCLR.Metadata.Experimental.Model;
using Raven.CodeAnalysis;
using Raven.CodeAnalysis.NeoClr;
using Raven.CodeAnalysis.Syntax;

internal static class PayloadSourceConsumer
{
    internal static void Build(string output, NeoClrReferenceCatalog catalog, CompilationOptions options)
    {
        const string source = """
            module neoCLR.Runtime
            internal extern func IsValue<T>(value: System.Value) -> bool;
            internal extern func UnpackValue<T>(value: System.Value) -> T;
            """;
        foreach (var variant in new[] { "valid", "unowned", "wrong-result", "constrained" })
        {
            var helper = variant switch
            {
                "wrong-result" => source.Replace("-> bool", "-> int"),
                "constrained" => source.Replace("-> bool;", "-> bool where T : class;"),
                _ => source
            };
            var compilation = Compilation.Create("PayloadSource",
                [SyntaxTree.ParseText("module System\npublic struct Value {}"), SyntaxTree.ParseText(helper)],
                catalog.References.ToArray(), options.WithOutputKind(OutputKind.DynamicallyLinkedLibrary));
            using var image = new MemoryStream();
            var result = compilation.Emit(image, null, new EmitOptions().WithBackend(new NeoClrEmissionBackend(
                new(new("PayloadSource", new(1, 0, 0, 0)), catalog.CoreIdentity, catalog.Dependencies,
                    primitiveImplementations: variant == "unowned" ? [] : [PrimitiveType.Value]))));
            if (result.Success != (variant == "valid"))
                throw new Exception(variant + ": " + string.Join("\n", result.Diagnostics));
            if (result.Success) File.WriteAllBytes(Path.Combine(output, "PayloadSource.dll"), image.ToArray());
            File.WriteAllText(Path.Combine(output, "payload-source-" + variant + ".log"), string.Join("\n", result.Diagnostics));
        }
        Console.WriteLine("PASS source payload ownership, missing-owner, signature and constraint admission checks");
    }
}
