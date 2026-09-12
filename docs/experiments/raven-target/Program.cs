using System.Reflection;
using System.Text.Json;
using Mono.Cecil;
using Mono.Cecil.Cil;
using Raven.CodeAnalysis;
using Raven.CodeAnalysis.Syntax;
using AssemblyDefinition = Mono.Cecil.AssemblyDefinition;

// Declaration-only fixture. Never execute these generated assemblies.
var output = Path.GetFullPath(args.Length == 1 ? args[0] : throw new ArgumentException("Supply a new output directory."));
if (Directory.Exists(output)) throw new IOException("Output directory must not exist.");
Directory.CreateDirectory(output);
var facadePath = Path.Combine(output, "NeoCLR.Probe.System.dll");
CreateFacade(facadePath);
var framework = TargetFrameworkResolver.ResolveVersion("net11.0");
var referencePaths = TargetFrameworkResolver.GetReferenceAssemblies(framework);
var targetReferences = referencePaths
    .Where(path => !string.Equals(Path.GetFileName(path), "System.Console.dll", StringComparison.OrdinalIgnoreCase))
    .Append(facadePath).Select(MetadataReference.CreateFromFile).ToArray();
const string source = """
import System.Console.*
func Main() {
    WriteLine("Hello from Raven on neoCLR")
}
""";
var control = Compile("Control", source, referencePaths.Select(MetadataReference.CreateFromFile).ToArray(), null);
var target = Compile("Target", source, targetReferences, AssemblyName.GetAssemblyName(facadePath));
var bad = Compilation.Create("MissingMember", [SyntaxTree.ParseText(source.Replace("WriteLine(", "MissingWriteLine("))],
    targetReferences, new CompilationOptions(OutputKind.ConsoleApplication));
var noLibrary = Compilation.Create("MissingLibrary", [SyntaxTree.ParseText(source)],
    referencePaths.Where(path => !string.Equals(Path.GetFileName(path), "System.Console.dll", StringComparison.OrdinalIgnoreCase))
        .Select(MetadataReference.CreateFromFile).ToArray(), new CompilationOptions(OutputKind.ConsoleApplication));
var missingLibraryErrors = noLibrary.GetDiagnostics().Where(d => d.Severity == DiagnosticSeverity.Error).Select(d => d.ToString()).ToArray();
var missingLibraryImage = missingLibraryErrors.Length == 0
    ? Compile("MissingLibrary", source, referencePaths
        .Where(path => !string.Equals(Path.GetFileName(path), "System.Console.dll", StringComparison.OrdinalIgnoreCase))
        .Select(MetadataReference.CreateFromFile).ToArray(), null)
    : null;
var errors = bad.GetDiagnostics().Where(d => d.Severity == DiagnosticSeverity.Error).Select(d => d.ToString()).ToArray();
if (errors.Length == 0) throw new Exception("Missing-member input unexpectedly bound.");
var targetCalls = target.Methods.SelectMany(m => m.Instructions).Where(i => i.Contains("::WriteLine(")).ToArray();
if (targetCalls.Length != 1 || !targetCalls[0].Contains("scope=NeoCLR.Probe.System"))
    throw new Exception("WriteLine did not bind to the target fixture: " + string.Join(";", targetCalls));
if (target.References.Any(r => r.StartsWith("System.Console,") || r.StartsWith("System.Runtime,") || r.StartsWith("System.Private.CoreLib,")))
    throw new Exception("Retargeted output retained a forbidden framework reference.");
if (!control.Methods.SelectMany(m => m.Instructions).Any(i => i.Contains("::WriteLine(") && i.Contains("scope=System.Console")))
    throw new Exception("Control did not call framework Console.");
var report = new
{
    Scope = "emission-only; fixture is not neoCLR System and generated code was not executed",
    FrameworkReferences = referencePaths.Length,
    Control = control,
    Target = target,
    MissingMemberErrors = errors,
    MissingLibraryErrors = missingLibraryErrors,
    MissingLibraryIsolationPassed = missingLibraryErrors.Length != 0,
    MissingLibraryImage = missingLibraryImage
};
File.WriteAllText(Path.Combine(output, "report.json"), JsonSerializer.Serialize(report, new JsonSerializerOptions { WriteIndented = true }));
Console.WriteLine($"PASS: control, target Console binding, core retargeting and missing-member diagnostic. Missing-library isolation passed: {missingLibraryErrors.Length != 0}. Report: {output}/report.json");

ImageReport Compile(string name, string text, MetadataReference[] references, AssemblyName? targetIdentity)
{
    var compilation = Compilation.Create(name, [SyntaxTree.ParseText(text)], references,
        new CompilationOptions(OutputKind.ConsoleApplication));
    var diagnostics = compilation.GetDiagnostics().Where(d => d.Severity == DiagnosticSeverity.Error).ToArray();
    if (diagnostics.Length != 0) throw new Exception(string.Join("\n", diagnostics.Select(d => d.ToString())));
    using var image = new MemoryStream();
    var result = targetIdentity is null ? compilation.Emit(image) : compilation.Emit(image, null, new EmitOptions(targetIdentity));
    if (!result.Success) throw new Exception(string.Join("\n", result.Diagnostics));
    File.WriteAllBytes(Path.Combine(output, name + ".dll"), image.ToArray());
    image.Position = 0;
    using var assembly = AssemblyDefinition.ReadAssembly(image);
    var methods = assembly.MainModule.GetTypes().SelectMany(t => t.Methods).Select(m => new MethodReport(
        m.FullName, m.HasBody ? m.Body.Instructions.Select(Describe).ToArray() : [],
        m.HasBody ? m.Body.ExceptionHandlers.Count : 0)).ToArray();
    // These referenced definitions are absent from our deliberately incomplete facade.
    var unresolvedCoreTypes = assembly.MainModule.GetTypeReferences()
        .Where(t => t.Scope.Name == "NeoCLR.Probe.System" && t.FullName != "System.Console")
        .Select(t => t.FullName).Distinct().Order().ToArray();
    return new ImageReport(assembly.MainModule.AssemblyReferences.Select(r => r.FullName).ToArray(),
        assembly.MainModule.Types.Select(t => t.FullName).ToArray(), methods, unresolvedCoreTypes,
        assembly.MainModule.EntryPoint?.FullName,
        assembly.MainModule.GetTypeReferences().Select(t => $"{t.FullName} scope={t.Scope.Name}").Order().ToArray());
}

static string Describe(Instruction i) => i.Operand is MethodReference m
    ? $"{i.OpCode} {m.FullName} scope={m.DeclaringType.Scope.Name}"
    : $"{i.OpCode} {i.Operand}".TrimEnd();

static void CreateFacade(string path)
{
    using var assembly = AssemblyDefinition.CreateAssembly(new AssemblyNameDefinition("NeoCLR.Probe.System", new Version(0, 0, 0, 1)),
        "NeoCLR.Probe.System", ModuleKind.Dll);
    var module = assembly.MainModule;
    var console = new TypeDefinition("System", "Console", Mono.Cecil.TypeAttributes.Public | Mono.Cecil.TypeAttributes.Abstract |
        Mono.Cecil.TypeAttributes.Sealed, module.TypeSystem.Object);
    module.Types.Add(console);
    var method = new MethodDefinition("WriteLine", Mono.Cecil.MethodAttributes.Public | Mono.Cecil.MethodAttributes.Static,
        module.TypeSystem.Void);
    method.Parameters.Add(new ParameterDefinition("value", Mono.Cecil.ParameterAttributes.None, module.TypeSystem.String));
    console.Methods.Add(method);
    method.Body.Instructions.Add(Instruction.Create(OpCodes.Ret));
    assembly.Write(path);
}
record ImageReport(string[] References, string[] Types, MethodReport[] Methods, string[] MissingFacadeTypes, string? EntryPoint, string[] TypeReferences);
record MethodReport(string Signature, string[] Instructions, int ExceptionRegions);
