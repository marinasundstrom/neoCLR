using System.Reflection;
using System.Text.Json;
using Raven.CodeAnalysis;
using Raven.CodeAnalysis.Syntax;

static class MatchProbe
{
    public static void Write(string output)
    {
        output = Path.GetFullPath(output);
        if (Directory.Exists(output)) throw new IOException("Output directory must not exist.");
        Directory.CreateDirectory(output);
        var core = Path.Combine(output, CoreDeclarations.Identity + ".dll");
        CoreDeclarations.Write(core, unionProbe: true, collectionProbe: true);
        var cases = new Dictionary<string, string> {
            ["Forms"] = File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "samples/library-match.rvn")),
            ["CaseImports"] = "import System.Result.*\nfunc Pick(value: int) -> int { return match Math.Abs(value) { Ok(let amount) => amount; Error(_) => -1 } } func Main() { WriteLine(Pick(-42)); WriteLine(Pick(-2147483648)) }",
            ["OptionPositional"] = "func Pick(value: Option<int>) -> int { return match value { .Some(let amount) => amount; .None => -1 } } func Main() { WriteLine(Pick(Option<int>(Option.Some<int>(42)))); WriteLine(Pick(Option<int>(Option.None()))) }",
            ["PositionalSingleEvaluation"] = "func Observe() -> Result<int, OverflowError> { WriteLine(7); return Math.Abs(-42) } func Main() { WriteLine(match Observe() { .Ok(let amount) => amount; .Error(_) => -1 }) }",
            ["PositionalWrongArity"] = "func Main() { match Math.Abs(-42) { .Ok(let a, let b) => WriteLine(a); .Error(_) => WriteLine(-1) } }",
            ["Positional"] = "func Pick(value: int) -> int { return match Math.Abs(value) { .Ok(let amount) => amount; .Error(_) => -1 } } func Main() { WriteLine(Pick(-42)); WriteLine(Pick(-2147483648)) }",
            ["StatementReturn"] = "func Pick(value: int) -> int { match Math.Abs(value) { Result.Ok<int> ok => { return ok.Value }; Result.Error<OverflowError> error => { return -1 } } } func Main() { WriteLine(Pick(-42)); WriteLine(Pick(-2147483648)) }",
            ["Guard"] = "func Pick(value: int) -> int { return match Math.Abs(value) { Result.Ok<int> ok when ok.Value == 42 => 1; Result.Ok<int> ok => 2; Result.Error<OverflowError> error => 3 } } func Main() { WriteLine(Pick(-42)); WriteLine(Pick(-7)); WriteLine(Pick(-2147483648)) }",
            ["StatementTail"] = "func Pick(value: int) -> int { match Math.Abs(value) { Result.Ok<int> ok => ok.Value; Result.Error<OverflowError> error => -1 } } func Main() { WriteLine(Pick(-42)); WriteLine(Pick(-2147483648)) }",
            ["SingleEvaluation"] = "func Observe() -> Result<int, OverflowError> { WriteLine(7); return Math.Abs(-42) } func Main() { WriteLine(match Observe() { Result.Ok<int> ok => ok.Value; Result.Error<OverflowError> error => -1 }) }",
            ["NominalDeconstruction"] = "func Main() { WriteLine(match Math.Abs(-42) { Result.Ok<int>(let amount) => amount; Result.Error<OverflowError>(_) => -1 }) }",
            ["MissingExpression"] = "func Main() { WriteLine(match Math.Abs(-2147483648) { Result.Ok<int> ok => ok.Value }) }",
            ["MissingStatement"] = "func Main() { match Math.Abs(-2147483648) { Result.Ok<int> ok => WriteLine(ok.Value) }; WriteLine(7) }",
            ["UnreachableArm"] = "func Main() { WriteLine(match Math.Abs(-42) { _ => 1; Result.Ok<int> ok => ok.Value }) }",
            ["WrongCase"] = "func Main() { WriteLine(match Math.Abs(-42) { Option.None absent => 1; _ => 2 }) }",
            ["VoidOutput"] = File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "samples/library-match-void.rvn")),
            ["ExpressionBlockReturn"] = "func Pick() -> int { return match Math.Abs(-42) { Result.Ok<int> ok => { return ok.Value }; _ => -1 } } func Main() { WriteLine(Pick()) }"
        };
        var results = new Dictionary<string, object>();
        foreach (var (name, source) in cases)
        {
            var compilation = Compilation.Create(name, [SyntaxTree.ParseText("import System.*\nimport System.Console.*\n" + source)],
                [MetadataReference.CreateFromFile(core)], new CompilationOptions(OutputKind.ConsoleApplication,
                    metadataImportOptions: new MetadataImportOptions(CoreDeclarations.Identity),
                    runtimePropagationContract: new RuntimePropagationContract(CoreDeclarations.Identity, "System.Propagatable`3"))
                .WithTargetCoreAssemblyName(CoreDeclarations.Identity)
                .WithGraphemeChar(true)
                .WithRuntimeUnitContract(new RuntimeUnitContract(CoreDeclarations.Identity, "System.Void")));
            var diagnostics = compilation.GetDiagnostics().Select(d => new { d.Id, Severity = d.Severity.ToString(), Message = d.ToString() }).ToArray();
            if (compilation.GetDiagnostics().Any(d => d.Severity == DiagnosticSeverity.Error))
            { results[name] = new { Stage = "compile-rejected", Diagnostics = diagnostics }; continue; }
            var raw = Path.Combine(output, name + ".raw.dll");
            try
            {
                using var stream = File.Create(raw);
                var emitted = compilation.Emit(stream, null, new EmitOptions(AssemblyName.GetAssemblyName(core)));
                if (!emitted.Success) throw new InvalidOperationException(string.Join("\n", emitted.Diagnostics));
            }
            catch (Exception error) when (error is InvalidOperationException or NotSupportedException)
            {
                results[name] = new { Stage = "emit-failed", Diagnostics = diagnostics, Error = error.ToString() };
                continue;
            }
            var projected = Path.Combine(output, name + ".dll");
            VoidProjection.Write(raw, core, projected);
            try
            {
                UnionImport.Write(projected, core, Path.Combine(output, name + ".neoil"), collectionProfile: true);
                results[name] = new { Stage = "imported", Diagnostics = diagnostics };
            }
            catch (InvalidDataException error) { results[name] = new { Stage = "import-rejected", Diagnostics = diagnostics, Error = error.Message }; }
        }
        File.WriteAllText(Path.Combine(output, "match-results.json"), JsonSerializer.Serialize(results, new JsonSerializerOptions { WriteIndented = true }));
        Console.WriteLine(JsonSerializer.Serialize(results, new JsonSerializerOptions { WriteIndented = true }));
        CheckUninitializedString(Path.Combine(output, "Forms.dll"), core, output);
    }
    static void CheckUninitializedString(string application, string core, string output)
    {
        using var image = Mono.Cecil.AssemblyDefinition.ReadAssembly(application);
        var references = image.MainModule.AssemblyReferences.ToHashSet();
        var method = image.MainModule.GetTypes().SelectMany(t => t.Methods).Single(m => m.Name == "Describe");
        var local = method.Body.Variables.First(v => v.VariableType.MetadataType == Mono.Cecil.MetadataType.String);
        var il = method.Body.GetILProcessor();
        var first = method.Body.Instructions[0];
        il.InsertBefore(first, il.Create(Mono.Cecil.Cil.OpCodes.Ldloc, local));
        il.InsertBefore(first, il.Create(Mono.Cecil.Cil.OpCodes.Pop));
        foreach (var candidate in image.MainModule.GetTypes().SelectMany(t => t.Methods).Where(m => m.HasBody))
        { _ = candidate.Body.Instructions.Count; _ = candidate.Body.Variables.Count; }
        foreach (var reference in image.MainModule.AssemblyReferences.Where(r => !references.Contains(r)).ToArray())
        {
            if (image.MainModule.GetTypeReferences().Any(t => ReferenceEquals(t.Scope, reference)))
                throw new InvalidDataException("Mutation introduced an external scope.");
            image.MainModule.AssemblyReferences.Remove(reference);
        }
        var invalid = Path.Combine(output, "UninitializedString.dll");
        var destination = Path.Combine(output, "UninitializedString.neoil");
        image.Write(invalid);
        try { UnionImport.Write(invalid, core, destination, collectionProfile: true); }
        catch (InvalidDataException error) when (error.Message.Contains("Read of uninitialized"))
        {
            if (File.Exists(destination)) throw new Exception("Rejected string read produced output.");
            return;
        }
        throw new Exception("Uninitialized string read was admitted.");
    }

}
