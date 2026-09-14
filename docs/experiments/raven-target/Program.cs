using System.Reflection;
using System.Text.Json;
using Mono.Cecil;
using Mono.Cecil.Cil;
using Raven.CodeAnalysis;
using Raven.CodeAnalysis.Syntax;
using AssemblyDefinition = Mono.Cecil.AssemblyDefinition;

if (args.Length == 2 && args[0] == "--namespace-members")
{
    NamespaceMemberProbe.Write(args[1]);
    return;
}

if (args.Length == 1 && args[0] == "--identity-checks")
{
    MetadataIdentityChecks.Run();
    return;
}

if (args.Length == 2 && args[0] == "--parsing")
{
    ParseProbe.Write(args[1]);
    return;
}

if (args.Length == 2 && args[0] == "--slices")
{
    SliceProbe.Write(args[1]);
    return;
}

if (args.Length == 2 && args[0] == "--signatures")
{
    SignatureProbe.Write(args[1]);
    return;
}

if (args.Length == 2 && args[0] == "--files")
{
    FileProbe.Write(args[1]);
    return;
}

if (args.Length == 2 && args[0] == "--matches")
{
    MatchProbe.Write(args[1]);
    return;
}

if (args.Length == 2 && args[0] == "--array-api")
{
    ArrayApiProbe.Write(args[1]);
    return;
}

if (args.Length == 2 && args[0] == "--arrays")
{
    ArrayProbe.Write(args[1]);
    return;
}

if (args.Length == 2 && args[0] == "--interfaces")
{
    InterfaceProbe.Write(args[1]);
    return;
}

if (args.Length == 2 && args[0] == "--unit-contract-check")
{
    try { UnitContractChecks.Verify(args[1]); }
    catch (Exception error) { Console.Error.WriteLine(error.Message); Environment.ExitCode = 1; }
    return;
}

if (args.Length == 2 && args[0] == "--generic-library-core")
{
    GenericLibraryChecks.WriteCore(args[1]);
    return;
}

if (args.Length == 2 && args[0] == "--reference-core")
{
    CoreDeclarations.Write(args[1], unionProbe: true, collectionProbe: true);
    return;
}

if (args.Length == 5 && args[0] == "--library-implementation")
{
    try { LibraryImplementation.Write(args[1], args[2], args[3], args[4]); }
    catch (Exception error) { Console.Error.WriteLine(error.Message); Environment.ExitCode = 1; }
    return;
}

if (args.Length >= 4 && args[0] == "--import")
{
    try { ApplicationImport.Write(args[1], args[2], args[3], args.Skip(4).ToArray()); }
    catch (Exception error) { Console.Error.WriteLine(error.Message); Environment.ExitCode = 1; }
    return;
}

if (args.Length == 3 && args[0] == "--project")
{
    try { ProjectBuild.Write(args[1], args[2]); }
    catch (Exception error) { Console.Error.WriteLine(Environment.GetEnvironmentVariable("NEOCLR_IMPORT_TRACE") == "1" ? error.ToString() : error.Message); Environment.ExitCode = 1; }
    return;
}

// Core declaration bodies never execute. Application IL is imported separately for neoCLR.
var output = Path.GetFullPath(args.Length == 1 ? args[0] : throw new ArgumentException("Supply a new output directory."));
if (Directory.Exists(output)) throw new IOException("Output directory must not exist.");
Directory.CreateDirectory(output);
var facadePath = Path.Combine(output, "NeoCLR.Probe.System.dll");
CreateFacade(facadePath);
var corePath = Path.Combine(output, CoreDeclarations.Identity + ".dll");
CoreDeclarations.Write(corePath);
var coreTypes = CoreDeclarations.ReadDeclaredTypes(corePath);
using (var core = AssemblyDefinition.ReadAssembly(corePath))
    if (!core.CustomAttributes.Any(a => a.AttributeType.FullName == "System.Runtime.CompilerServices.ReferenceAssemblyAttribute"))
        throw new Exception("Core declarations are not marked as a reference assembly.");
var coreErrors = ClosureAudit.Inspect(corePath);
if (coreErrors.Length != 0) throw new Exception(string.Join("\n", coreErrors));
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
var coreOnly = Compile("CoreOnly", source, [MetadataReference.CreateFromFile(corePath)], AssemblyName.GetAssemblyName(corePath), true, CoreDeclarations.Identity);
if (coreOnly.References.Length != 1 || !coreOnly.References[0].StartsWith(CoreDeclarations.Identity + ","))
    throw new Exception("Core-only output leaked an assembly reference.");
var coreOnlyErrors = ClosureAudit.Inspect(Path.Combine(output, "CoreOnly.dll"), corePath);
if (coreOnlyErrors.Length != 0) throw new Exception(string.Join("\n", coreOnlyErrors));
if (!coreOnlyErrors.SequenceEqual(ClosureAudit.Inspect(corePath, Path.Combine(output, "CoreOnly.dll"))))
    throw new Exception("Core closure audit depends on input ordering.");
var coreOnlyCases = new Dictionary<string, ImageReport>();
foreach (var (name, program) in new[] {
    ("CoreLibrary", File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "samples", "library-basics.rvn"))),
    ("CoreEmpty", "func Main() {}"),
    ("CoreNested", "func Empty() {} func Main() { Empty() }"),
    ("CoreInt32", "func Answer() -> int { return 42 } func Main() { let answer = Answer() }") })
{
    coreOnlyCases[name] = Compile(name, program, [MetadataReference.CreateFromFile(corePath)], AssemblyName.GetAssemblyName(corePath), true, CoreDeclarations.Identity);
    var closure = ClosureAudit.Inspect(Path.Combine(output, name + ".dll"), corePath);
    if (closure.Length != 0) throw new Exception(string.Join("\n", closure));
}
foreach (var name in new[] { "CoreOnly", "CoreEmpty", "CoreNested", "CoreInt32", "CoreLibrary" })
    StaticImport.Write(Path.Combine(output, name + ".dll"), corePath, Path.Combine(output, name + ".neoil"));
var staticImportRejections = new Dictionary<string, string>();
foreach (var (name, opcode, diagnostic) in new[] {
    ("ImportUnderflow", OpCodes.Pop, "underflow"),
    ("ImportBadReturn", OpCodes.Ldc_I4_1, "ret stack"),
    ("ImportUnsupported", OpCodes.Ldnull, "Unsupported reachable") })
{
    var path = Path.Combine(output, name + ".dll");
    var original = Path.Combine(output, "CoreEmpty.dll");
    var bytes = File.ReadAllBytes(original);
    using (var image = AssemblyDefinition.ReadAssembly(original))
    using (var pe = new System.Reflection.PortableExecutable.PEReader(new MemoryStream(bytes)))
    {
        var rva = image.EntryPoint.RVA;
        var section = pe.PEHeaders.SectionHeaders.Single(s => rva >= s.VirtualAddress && rva < s.VirtualAddress + s.VirtualSize);
        var body = section.PointerToRawData + rva - section.VirtualAddress;
        var header = (bytes[body] & 3) == 2 ? 1 : (BitConverter.ToUInt16(bytes, body) >> 12) * 4;
        if (bytes[body + header] != 0 || opcode.Size != 1) throw new Exception("Expected leading nop and one-byte mutation.");
        bytes[body + header] = (byte)opcode.Value;
    }
    File.WriteAllBytes(path, bytes);
    try { StaticImport.Write(path, corePath, Path.Combine(output, name + ".neoil")); }
    catch (InvalidDataException error) when (error.Message.Contains(diagnostic))
    { staticImportRejections[name] = error.Message; continue; }
    throw new Exception("Static importer did not reject " + name);
}
var targetCompletions = new Dictionary<string, string[]>();
foreach (var owner in new[] { "Math", "Console" })
{
    var text = $"import System\nfunc Main() {{\n    System.{owner}.\n}}";
    var tree = SyntaxTree.ParseText(text);
    var compilation = Compilation.Create("Completion" + owner, [tree], [MetadataReference.CreateFromFile(corePath)],
        new CompilationOptions(OutputKind.ConsoleApplication, metadataImportOptions: new MetadataImportOptions(CoreDeclarations.Identity)));
    var names = compilation.GetCompletions(tree, text.LastIndexOf('.') + 1)
        .Select(item => item.Symbol?.Name ?? item.DisplayText).Distinct().Order().ToArray();
    targetCompletions[owner] = names;
    var expected = TargetSurface.Methods.Where(m => m.Owner == owner).Select(m => m.Name).Distinct();
    if (expected.Any(name => !names.Contains(name)) || names.Contains("Abs") || names.Contains("Clamp") || names.Contains("ReadLine")
        || compilation.GetTypeByMetadataName("System.IO.File") is not null)
        throw new Exception("Completion did not respect target surface for " + owner + ": " + string.Join(",", names));
}
var coreMissingConsoleErrors = CheckCoreRejection("CoreMissingConsole", false, true);
var coreWrongSignatureErrors = CheckCoreRejection("CoreWrongSignature", true, false);
string[] CheckCoreRejection(string name, bool includeConsole, bool stringParameter)
{
    var path = Path.Combine(output, name + ".dll");
    CoreDeclarations.Write(path, includeConsole, stringParameter);
    var compilation = Compilation.Create(name, [SyntaxTree.ParseText(source)], [MetadataReference.CreateFromFile(path)],
        new CompilationOptions(OutputKind.ConsoleApplication, metadataImportOptions: new MetadataImportOptions(CoreDeclarations.Identity)));
    var diagnostics = compilation.GetDiagnostics().Where(d => d.Severity == DiagnosticSeverity.Error).Select(d => d.ToString()).ToArray();
    var expected = includeConsole ? "RAV1503" : "RAV0103";
    if (!diagnostics.Any(d => d.Contains(expected))) throw new Exception(name + " did not produce its expected binding error.");
    return diagnostics;
}
var control = Compile("Control", source, referencePaths.Select(MetadataReference.CreateFromFile).ToArray(), null);
var target = Compile("Target", source, targetReferences, AssemblyName.GetAssemblyName(facadePath));
var isolatedTarget = Compile("IsolatedTarget", source, targetReferences, AssemblyName.GetAssemblyName(facadePath), true);
var isolatedMissing = Compilation.Create("IsolatedMissing", [SyntaxTree.ParseText(source)],
    referencePaths.Where(path => Path.GetFileName(path) != "System.Console.dll")
        .Select(MetadataReference.CreateFromFile).ToArray(),
    new CompilationOptions(OutputKind.ConsoleApplication, metadataImportOptions: new MetadataImportOptions("System.Runtime")));
var isolatedMissingErrors = isolatedMissing.GetDiagnostics().Where(d => d.Severity == DiagnosticSeverity.Error).Select(d => d.ToString()).ToArray();
if (isolatedMissingErrors.Length == 0 || isolatedMissing.GetTypeByMetadataName("System.Console") is not null)
    throw new Exception("Explicit-only metadata import leaked Console.");
if (!isolatedTarget.Methods.SelectMany(m => m.Instructions).Any(i => i.Contains("::WriteLine(") && i.Contains("scope=NeoCLR.Probe.System")))
    throw new Exception("Isolated target did not bind the fixture Console.");
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
var targetClosureErrors = ClosureAudit.Inspect(Path.Combine(output, "Target.dll"), facadePath);
if (targetClosureErrors.Length == 0) throw new Exception("Incomplete target unexpectedly passed closure audit.");
var hostClosureErrors = missingLibraryImage is null ? [] : ClosureAudit.Inspect(Path.Combine(output, "MissingLibrary.dll"), facadePath);
if (missingLibraryImage is not null && hostClosureErrors.Length == 0) throw new Exception("Host fallback unexpectedly passed closure audit.");
var closedPath = Path.Combine(output, "Closed.dll");
CreateClosedFixture(closedPath);
var closedErrors = ClosureAudit.Inspect(closedPath);
if (closedErrors.Length != 0) throw new Exception(string.Join("\n", closedErrors));
using (var wrong = AssemblyDefinition.ReadAssembly(closedPath))
{
    wrong.MainModule.AssemblyReferences.Add(new AssemblyNameReference("Closed", new Version(9, 0, 0, 0)));
    wrong.Write(Path.Combine(output, "WrongIdentity.dll"));
}
var wrongIdentityErrors = ClosureAudit.Inspect(Path.Combine(output, "WrongIdentity.dll"));
if (!wrongIdentityErrors.Any(e => e.Contains("Version=9.0.0.0"))) throw new Exception("Wrong identity unexpectedly passed closure audit.");
var missingTypeErrors = MutateAndAudit("MissingType", "unresolved type Probe.Absent", image =>
    image.MainModule.GetTypeReferences().Single().Name = "Absent");
var missingMethodErrors = MutateAndAudit("MissingMethod", "::Absent(", image =>
    image.MainModule.GetMemberReferences().OfType<MethodReference>().Single().Name = "Absent");
var wrongSignatureErrors = MutateAndAudit("WrongSignature", "::Identity()", image =>
    image.MainModule.GetMemberReferences().OfType<MethodReference>().Single().Parameters.Clear());
var consumerPath = Path.Combine(output, "Consumer.dll");
CreateConsumerFixture(consumerPath);
var consumerErrors = ClosureAudit.Inspect(consumerPath, closedPath);
if (consumerErrors.Length != 0) throw new Exception(string.Join("\n", consumerErrors));
var missingDependencyErrors = ClosureAudit.Inspect(consumerPath);
if (!missingDependencyErrors.Any(e => e.Contains("unresolved assembly Closed,")))
    throw new Exception("Omitted explicit dependency was accepted.");
var duplicateRejected = false;
try { ClosureAudit.Inspect(closedPath, closedPath); }
catch (InvalidDataException) { duplicateRejected = true; }
if (!duplicateRejected) throw new Exception("Duplicate input identity was accepted.");

string[] MutateAndAudit(string name, string expected, Action<AssemblyDefinition> change)
{
    var path = Path.Combine(output, name + ".dll");
    using (var image = AssemblyDefinition.ReadAssembly(closedPath))
    {
        change(image);
        image.Write(path);
    }
    var errors = ClosureAudit.Inspect(path);
    if (!errors.Any(e => e.Contains(expected))) throw new Exception(name + " did not report its expected resolution failure.");
    return errors;
}
var unionDirectory = Path.Combine(output, "union-probe");
Directory.CreateDirectory(unionDirectory);
var unionCore = Path.Combine(unionDirectory, CoreDeclarations.Identity + ".dll");
CoreDeclarations.Write(unionCore, unionProbe: true);
CoreDeclarations.ReadDeclaredTypes(unionCore);
var unionSource = File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "samples", "library-result.rvn"));
var unionCompilation = Compilation.Create("CoreUnion", [SyntaxTree.ParseText(unionSource)],
    [MetadataReference.CreateFromFile(unionCore)],
    new CompilationOptions(OutputKind.ConsoleApplication, metadataImportOptions: new MetadataImportOptions(CoreDeclarations.Identity)));
var unionDiagnostics = unionCompilation.GetDiagnostics().Where(d => d.Severity == DiagnosticSeverity.Error).ToArray();
if (unionDiagnostics.Length != 0) throw new Exception(string.Join("\n", unionDiagnostics.Select(d => d.ToString())));
var badUnionCompilation = Compilation.Create("BadUnion", [SyntaxTree.ParseText(unionSource.Replace("Math.Abs(value)", "Math.Abs(\"wrong\")"))],
    [MetadataReference.CreateFromFile(unionCore)],
    new CompilationOptions(OutputKind.ConsoleApplication, metadataImportOptions: new MetadataImportOptions(CoreDeclarations.Identity)));
var badUnionDiagnostics = badUnionCompilation.GetDiagnostics().Where(d => d.Severity == DiagnosticSeverity.Error).Select(d => d.ToString()).ToArray();
if (!badUnionDiagnostics.Any(d => d.Contains("RAV1503")))
    throw new Exception("Union API accepted an incorrect argument type.");
var unionErrors = ClosureAudit.Inspect(unionCore);
if (unionErrors.Length != 0) throw new Exception(string.Join("\n", unionErrors));
var unionImage = Compile("CoreUnion", unionSource, [MetadataReference.CreateFromFile(unionCore)],
    AssemblyName.GetAssemblyName(unionCore), true, CoreDeclarations.Identity);
using (var unionAssembly = AssemblyDefinition.ReadAssembly(Path.Combine(output, "CoreUnion.dll")))
{
    var extraction = unionAssembly.MainModule.GetMemberReferences().OfType<MethodReference>()
        .Where(m => m.Name == "TryGetValue").ToArray();
    if (extraction.Length != 2 || extraction.Any(m => m.DeclaringType is not GenericInstanceType
        || m.Parameters.Count != 1 || m.Parameters[0].ParameterType is not ByReferenceType))
        throw new Exception("Union patterns did not reference both generic out-case extractors.");
    var show = unionAssembly.MainModule.GetTypes().SelectMany(t => t.Methods).Single(m => m.Name == "Show");
    if (show.Body.Instructions.Any(i => i.OpCode.Code is Code.Box or Code.Isinst or Code.Unbox_Any))
        throw new Exception("Union patterns fell back to ordinary object type tests.");
}
var unionClosureErrors = ClosureAudit.Inspect(Path.Combine(output, "CoreUnion.dll"), unionCore);
if (unionClosureErrors.Length != 0) throw new Exception(string.Join("\n", unionClosureErrors));
UnionImport.Write(Path.Combine(output, "CoreUnion.dll"), unionCore, Path.Combine(output, "CoreUnion.neoil"));
var optionImage = Compile("CoreOption", File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "samples", "library-option.rvn")),
    [MetadataReference.CreateFromFile(unionCore)], AssemblyName.GetAssemblyName(unionCore), true, CoreDeclarations.Identity);
var optionClosureErrors = ClosureAudit.Inspect(Path.Combine(output, "CoreOption.dll"), unionCore);
if (optionClosureErrors.Length != 0) throw new Exception(string.Join("\n", optionClosureErrors));
UnionImport.Write(Path.Combine(output, "CoreOption.dll"), unionCore, Path.Combine(output, "CoreOption.neoil"));
var voidImage = Compile("CoreVoid", File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "samples", "library-void.rvn")),
    [MetadataReference.CreateFromFile(unionCore)], AssemblyName.GetAssemblyName(unionCore), true, CoreDeclarations.Identity, voidCore: unionCore);
using (var voidAssembly = AssemblyDefinition.ReadAssembly(Path.Combine(output, "CoreVoid.dll")))
{
    var marker = voidAssembly.MainModule.GetTypes().SelectMany(t => t.Methods).Single(m => m.Name == "Marker");
    var argument = ((GenericInstanceType)marker.ReturnType).GenericArguments.Single();
    if (argument.FullName != "System.Void" || !argument.IsValueType
        || argument.Scope.Name != CoreDeclarations.Identity)
        throw new Exception($"Generic Void must be an explicit target value-type reference: {argument.MetadataType}, {argument.IsValueType}, {argument.Scope}.");
    if (voidAssembly.EntryPoint.ReturnType.MetadataType != MetadataType.Void)
        throw new Exception("Ordinary no-result return signature changed.");
}
var voidClosureErrors = ClosureAudit.Inspect(Path.Combine(output, "CoreVoid.dll"), unionCore);
if (voidClosureErrors.Length != 0) throw new Exception(string.Join("\n", voidClosureErrors));
UnionImport.Write(Path.Combine(output, "CoreVoid.dll"), unionCore, Path.Combine(output, "CoreVoid.neoil"));
var voidImportRejections = VoidChecks.Run(Path.Combine(output, "CoreVoid.dll"), Path.Combine(output, "CoreVoid.raw.dll"), unionCore, output);
var optionImportRejections = OptionImportChecks.Run(Path.Combine(output, "CoreOption.dll"), unionCore, output);
var resultImportRejections = ResultImportChecks.Run(Path.Combine(output, "CoreUnion.dll"), unionCore, output);
var report = new
{
    Closure = new
    {
        TargetErrors = targetClosureErrors,
        HostFallbackErrors = hostClosureErrors,
        ClosedFixtureErrors = closedErrors,
        WrongIdentityErrors = wrongIdentityErrors,
        MissingTypeErrors = missingTypeErrors,
        MissingMethodErrors = missingMethodErrors,
        WrongSignatureErrors = wrongSignatureErrors,
        DuplicateIdentityRejected = duplicateRejected,
        ConsumerErrors = consumerErrors,
        MissingDependencyErrors = missingDependencyErrors
    },
    Scope = "emission plus bounded static and Result imports; execute generated neoIL separately against neoCLR System",
    UnionProbe = new { Scope = "metadata binding, emission and bounded Result import; execute generated neoIL separately",
        BindingPassed = true, InvalidArgumentDiagnostics = badUnionDiagnostics, DeclarationClosureErrors = unionErrors, Emission = unionImage, ApplicationClosureErrors = unionClosureErrors },
    VoidImportRejections = voidImportRejections,
    VoidProbe = new { Emission = voidImage, ApplicationClosureErrors = voidClosureErrors },
    OptionImportRejections = optionImportRejections,
    OptionProbe = new { Emission = optionImage, ApplicationClosureErrors = optionClosureErrors },
    ResultImportRejections = resultImportRejections,
    StaticImportRejections = staticImportRejections,
    TargetCompletions = targetCompletions,
    CoreDeclarationTypes = coreTypes,
    CoreOnly = coreOnly,
    CoreOnlyCases = coreOnlyCases,
    CoreMissingConsoleErrors = coreMissingConsoleErrors,
    CoreWrongSignatureErrors = coreWrongSignatureErrors,
    CoreOnlyClosureErrors = coreOnlyErrors,
    FrameworkReferences = referencePaths.Length,
    IsolatedTarget = isolatedTarget,
    ExplicitOnlyIsolationPassed = true,
    IsolatedMissingLibraryErrors = isolatedMissingErrors,
    Control = control,
    Target = target,
    MissingMemberErrors = errors,
    MissingLibraryErrors = missingLibraryErrors,
    MissingLibraryIsolationPassed = missingLibraryErrors.Length != 0,
    MissingLibraryImage = missingLibraryImage
};
File.WriteAllText(Path.Combine(output, "report.json"), JsonSerializer.Serialize(report, new JsonSerializerOptions { WriteIndented = true }));
Console.WriteLine($"PASS: control, target Console binding, core retargeting, missing-member diagnostic and metadata closure checks. Legacy missing-library isolation passed: {missingLibraryErrors.Length != 0}. Explicit-only isolation passed: True. Report: {output}/report.json");

ImageReport Compile(string name, string text, MetadataReference[] references, AssemblyName? targetIdentity, bool isolated = false, string coreName = "System.Runtime", string? voidCore = null)
{
    var compilation = Compilation.Create(name, [SyntaxTree.ParseText(text)], references,
        new CompilationOptions(OutputKind.ConsoleApplication,
            metadataImportOptions: isolated ? new MetadataImportOptions(coreName) : null));
    var diagnostics = compilation.GetDiagnostics().Where(d => d.Severity == DiagnosticSeverity.Error).ToArray();
    if (diagnostics.Length != 0) throw new Exception(string.Join("\n", diagnostics.Select(d => d.ToString())));
    using var image = new MemoryStream();
    var result = targetIdentity is null ? compilation.Emit(image) : compilation.Emit(image, null, new EmitOptions(targetIdentity));
    if (!result.Success) throw new Exception(string.Join("\n", result.Diagnostics));
    File.WriteAllBytes(Path.Combine(output, name + ".dll"), image.ToArray());
    if (voidCore is not null)
    {
        var rawPath = Path.Combine(output, name + ".raw.dll");
        File.WriteAllBytes(rawPath, image.ToArray());
        VoidProjection.Write(rawPath, voidCore, Path.Combine(output, name + ".dll"));
    }
    using var assembly = AssemblyDefinition.ReadAssembly(Path.Combine(output, name + ".dll"));
    var declaredReferences = assembly.MainModule.AssemblyReferences.Select(r => r.FullName).ToArray();
    var methods = assembly.MainModule.GetTypes().SelectMany(t => t.Methods).Select(m => new MethodReport(
        m.FullName, m.HasBody ? m.Body.Instructions.Select(Describe).ToArray() : [],
        m.HasBody ? m.Body.ExceptionHandlers.Count : 0)).ToArray();
    // These referenced definitions are absent from our deliberately incomplete facade.
    var unresolvedCoreTypes = assembly.MainModule.GetTypeReferences()
        .Where(t => t.Scope.Name == "NeoCLR.Probe.System" && t.FullName != "System.Console")
        .Select(t => t.FullName).Distinct().Order().ToArray();
    return new ImageReport(declaredReferences,
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
// Self-contained metadata fixture to test the resolver, not a usable runtime library.
static void CreateClosedFixture(string path)
{
    using var assembly = AssemblyDefinition.CreateAssembly(new AssemblyNameDefinition("Closed", new Version(1, 0, 0, 0)), "Closed", ModuleKind.Dll);
    var module = assembly.MainModule;
    var root = new TypeDefinition("Probe", "Root", Mono.Cecil.TypeAttributes.Public, null);
    module.Types.Add(root);
    var child = new TypeDefinition("Probe", "Child", Mono.Cecil.TypeAttributes.Public,
        new TypeReference("Probe", "Root", module, module));
    module.Types.Add(child);
    var method = new MethodDefinition("Identity", Mono.Cecil.MethodAttributes.Public | Mono.Cecil.MethodAttributes.Static, root);
    method.Parameters.Add(new ParameterDefinition(root));
    method.Body.Instructions.Add(Instruction.Create(OpCodes.Ldarg_0));
    method.Body.Instructions.Add(Instruction.Create(OpCodes.Ret));
    root.Methods.Add(method);
    var caller = new MethodDefinition("Call", Mono.Cecil.MethodAttributes.Public | Mono.Cecil.MethodAttributes.Static, root);
    caller.Parameters.Add(new ParameterDefinition(root));
    var reference = new MethodReference("Identity", root, child.BaseType) { HasThis = false };
    reference.Parameters.Add(new ParameterDefinition(root));
    caller.Body.Instructions.Add(Instruction.Create(OpCodes.Ldarg_0));
    caller.Body.Instructions.Add(Instruction.Create(OpCodes.Call, reference));
    caller.Body.Instructions.Add(Instruction.Create(OpCodes.Ret));
    child.Methods.Add(caller);
    assembly.Write(path);
}
static void CreateConsumerFixture(string path)
{
    using var assembly = AssemblyDefinition.CreateAssembly(new AssemblyNameDefinition("Consumer", new Version(1, 0, 0, 0)), "Consumer", ModuleKind.Dll);
    var module = assembly.MainModule;
    var dependency = new AssemblyNameReference("Closed", new Version(1, 0, 0, 0));
    module.AssemblyReferences.Add(dependency);
    var root = new TypeReference("Probe", "Root", module, dependency);
    var child = new TypeDefinition("Probe", "Consumer", Mono.Cecil.TypeAttributes.Public, root);
    module.Types.Add(child);
    var caller = new MethodDefinition("Call", Mono.Cecil.MethodAttributes.Public | Mono.Cecil.MethodAttributes.Static, root);
    caller.Parameters.Add(new ParameterDefinition(root));
    var reference = new MethodReference("Identity", root, root) { HasThis = false };
    reference.Parameters.Add(new ParameterDefinition(root));
    caller.Body.Instructions.Add(Instruction.Create(OpCodes.Ldarg_0));
    caller.Body.Instructions.Add(Instruction.Create(OpCodes.Call, reference));
    caller.Body.Instructions.Add(Instruction.Create(OpCodes.Ret));
    child.Methods.Add(caller);
    assembly.Write(path);
}
record ImageReport(string[] References, string[] Types, MethodReport[] Methods, string[] MissingFacadeTypes, string? EntryPoint, string[] TypeReferences);
record MethodReport(string Signature, string[] Instructions, int ExceptionRegions);
