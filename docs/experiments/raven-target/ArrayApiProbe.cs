using System.Reflection;
using Raven.CodeAnalysis;
using Raven.CodeAnalysis.Syntax;

static class ArrayApiProbe
{
    public static void Write(string output)
    {
        Directory.CreateDirectory(output);
        var core = Path.Combine(output, CoreDeclarations.Identity + ".dll");
        CoreDeclarations.Write(core, unionProbe: true, collectionProbe: true);
        Compilation Create(string source) => Compilation.Create("ArrayApi", [SyntaxTree.ParseText(source)],
            [MetadataReference.CreateFromFile(core)], new CompilationOptions(OutputKind.ConsoleApplication,
                metadataImportOptions: new MetadataImportOptions(CoreDeclarations.Identity),
                runtimeIterationContract: new RuntimeIterationContract(CoreDeclarations.Identity,
                    "System.Collections.Iterable`1", "System.Collections.Iterator`1", ArrayShapeTypeName: "System.Array`1"))
                .WithTargetCoreAssemblyName(CoreDeclarations.Identity)
                .WithGraphemeChar(true)
                .WithRuntimeUnitContract(new RuntimeUnitContract(CoreDeclarations.Identity, "System.Void"))
                .WithRuntimeTypeOfContract(new RuntimeTypeOfContract(CoreDeclarations.Identity,
                    "System.Introspection.TypeInfo", "System.Runtime.RuntimeContext")));
        foreach (var name in new[] { "library-array-callbacks", "library-array-foreach", "library-array-shapes", "library-managed-array-metadata" })
        {
            var source = File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "samples", name + ".rvn"));
            var path = Path.Combine(output, name + ".dll");
            using (var stream = File.Create(path))
            {
                var result = Create(source).Emit(stream, null, new EmitOptions(AssemblyName.GetAssemblyName(core)));
                if (!result.Success) throw new Exception(string.Join("\n", result.Diagnostics));
            }
            using (var image = Mono.Cecil.AssemblyDefinition.ReadAssembly(path))
            {
                foreach (var constructor in image.MainModule.GetMemberReferences().OfType<Mono.Cecil.MethodReference>()
                    .Where(m => m.Name == ".ctor" && m.DeclaringType is Mono.Cecil.GenericInstanceType g && g.ElementType.Name == "Func`2"))
                {
                    var argument = ((Mono.Cecil.GenericInstanceType)constructor.DeclaringType).GenericArguments[1];
                    if (argument.FullName != "System.Void" || !argument.IsValueType || argument.MetadataToken.TokenType != Mono.Cecil.TokenType.TypeRef || argument.MetadataToken.RID == 0 || !RuntimeSignatures.IsCore(argument.Scope))
                        throw new Exception($"Func<T,Void> requires a nominal generic argument, not a no-result marker: {argument.FullName}, {argument.IsValueType}, {argument.MetadataType}, {argument.Scope}.");
                }
            }
            UnionImport.Write(path, core, Path.Combine(output, name + ".neoil"), collectionProfile: true);
        }
        foreach (var invalid in new[] {
            "import System.*\nfunc Main() { Array<int>.Empty = [1] }",
            "import System.*\nfunc Main() { Array.ForEach([1], x => ()) }",
            "import System.*\nfunc Main() { let values = [1]; values.ForEach((x: string) => ()) }" })
            if (!Create(invalid).GetDiagnostics().Any(d => d.Severity == DiagnosticSeverity.Error))
                throw new Exception("Expected invalid array API rejection: " + invalid);
        Console.WriteLine("Generic array API compile/import checks passed.");
    }
}
