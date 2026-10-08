using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using Raven.CodeAnalysis;
using Raven.CodeAnalysis.NeoClr;
using Raven.CodeAnalysis.Syntax;

if (args.Length != 1) throw new ArgumentException("Specify a fresh output directory.");
var output = Path.GetFullPath(args[0]);
if (Directory.Exists(output)) throw new IOException("Output directory already exists.");
Directory.CreateDirectory(output);
var identity = new AssemblyIdentity("NativeCore", new Version(1, 0, 0, 0));
var core = new AssemblyBuilder(identity, identity);
var root = core.AddNativeObjectRoot();
var rootCtor = root.AddConstructor(Array.Empty<PrimitiveType>());
rootCtor.GetILGenerator().Return();
// Minimal fixture slots satisfy the imported root contract; this is not a production Object library.
var display = root.AddNativeObjectSlot("ToString", new(PrimitiveType.String, []));
display.GetILGenerator().Emit(OpCode.Ldstr, "object");
display.GetILGenerator().Return();
var hash = root.AddNativeObjectSlot("GetHashCode", new(PrimitiveType.Int32, []));
hash.GetILGenerator().LoadConstant(0);
hash.GetILGenerator().Return();
var equals = root.AddNativeObjectSlot("Equals", new(PrimitiveType.Boolean, [root]));
equals.GetILGenerator().Emit(OpCode.Ldc_Bool, false);
equals.GetILGenerator().Return();
core.AddClass("System", "ValueType", root);
var attribute = core.AddClass("System", "Attribute", root);
var attributeCtor = attribute.AddConstructor(Array.Empty<PrimitiveType>());
attributeCtor.GetILGenerator().LoadArgument(0);
attributeCtor.GetILGenerator().Call(rootCtor);
attributeCtor.GetILGenerator().Return();
var markerCtor = core.AddClass("System.Runtime.CompilerServices", "ReferenceAssemblyAttribute", attribute)
    .AddConstructor(Array.Empty<PrimitiveType>());
markerCtor.GetILGenerator().LoadArgument(0);
markerCtor.GetILGenerator().Call(attributeCtor);
markerCtor.GetILGenerator().Return();
foreach (var primitive in new[] { PrimitiveType.Int32, PrimitiveType.Int64, PrimitiveType.Boolean, PrimitiveType.Void })
    core.AddValueType("System", primitive.ToString()).SetNativePrimitive(primitive);
core.AddClass("System", "String", root).SetNativePrimitive(PrimitiveType.String);
var bytes = RuntimeAssemblyContainer.WriteLibraryBinary(core);
File.WriteAllBytes(Path.Combine(output, "NativeCore.dll"), bytes);
var reference = NeoClrMetadataReference.ReadAssembly(bytes);
var options = CompilationOptions.NeoCLR.WithRuntimeTypeOfContract(null)
    .WithTargetCoreAssemblyName(identity.Name)
    .WithRuntimeUnitContract(new(identity.Name, "System.Void"))
    .WithMetadataImportOptions(new MetadataImportOptions(identity.Name).WithObjectAssemblyName(identity.Name).WithNativeMetadata());
var source = SyntaxTree.ParseText(File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "consumer.rvn")));
var compilation = Compilation.Create("Consumer", [source], [reference], options);
var errors = compilation.GetDiagnostics().Where(d => d.Severity == DiagnosticSeverity.Error).ToArray();
if (errors.Length != 0)
{
    Console.Error.WriteLine("Native core artifact written; compiler semantic frontier:");
    foreach (var error in errors) Console.Error.WriteLine(error);
    return 2;
}
foreach (var type in new[] { SpecialType.System_Int32, SpecialType.System_Boolean, SpecialType.System_Object })
{
    var symbol = compilation.GetSpecialType(type);
    if (symbol.TypeKind == TypeKind.Error || symbol.SpecialType != type || symbol.ContainingAssembly.Name != identity.Name)
        throw new Exception("Native primitive resolution failed: " + type);
}
try
{
    _ = compilation.CoreAssembly;
    throw new Exception("Native metadata mode exposed a reflection core.");
}
catch (InvalidOperationException error) when (error.Message.Contains("no reflection core")) { }
// Failures must remain diagnostic and leave output unpublished.
using (var rejected = new MemoryStream())
{
    var cli = compilation.Emit(rejected);
    if (cli.Success || rejected.Length != 0) throw new Exception("Native semantic core leaked into CLI emission.");
    var wrongIdentity = new AssemblyIdentity(identity.Name, new Version(2, 0, 0, 0));
    var mismatch = compilation.Emit(rejected, null, new EmitOptions().WithBackend(new NeoClrEmissionBackend(
        new(new("Consumer", new(1, 0, 0, 0)), wrongIdentity, [new NeoClrMetadataDependency(reference, wrongIdentity)]))));
    if (mismatch.Success || rejected.Length != 0 || !mismatch.Diagnostics.Any(d => d.Id == "NEOMETA002"))
        throw new Exception("Native core identity mismatch was not rejected before output.");
}
var missing = Compilation.Create("MissingCore", [source], [], options);
if (!missing.GetDiagnostics().Any(d => d.Severity == DiagnosticSeverity.Error))
    throw new Exception("Missing native core fell back to host declarations.");
using var image = new MemoryStream();
var emitted = compilation.Emit(image, null, new EmitOptions().WithBackend(new NeoClrEmissionBackend(
    new(new("Consumer", new(1, 0, 0, 0)), identity, [new NeoClrMetadataDependency(reference, identity)]))));
if (!emitted.Success) throw new Exception(string.Join("\n", emitted.Diagnostics));
File.WriteAllBytes(Path.Combine(output, "Consumer.dll"), image.ToArray());
Console.WriteLine("PASS native-only core symbols, consumer emission, missing-core/identity rejection and CLI output guard");

return 0;
