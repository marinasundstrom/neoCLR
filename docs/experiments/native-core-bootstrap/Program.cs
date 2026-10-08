using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using Raven.CodeAnalysis;
using Raven.CodeAnalysis.NeoClr;
using Raven.CodeAnalysis.Syntax;

if (args.Length != 1) throw new ArgumentException("Specify a fresh output directory.");
var output = Path.GetFullPath(args[0]);
if (Directory.Exists(output)) throw new IOException("Output directory already exists.");
Directory.CreateDirectory(output);
var identity = new AssemblyIdentity("NeoCLR.CoreProbe", new Version(1, 0, 0, 0));
var core = new AssemblyBuilder(identity, identity);
var root = core.AddNativeObjectRoot();
var rootCtor = root.AddConstructor(Array.Empty<PrimitiveType>());
rootCtor.GetILGenerator().Return();
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
foreach (var primitive in new[] { PrimitiveType.Int32, PrimitiveType.Boolean, PrimitiveType.Void })
    core.AddValueType("System", primitive.ToString()).SetNativePrimitive(primitive);
var bytes = RuntimeAssemblyContainer.WriteLibraryBinary(core);
File.WriteAllBytes(Path.Combine(output, "NativeCore.dll"), bytes);
var reference = NeoClrMetadataReference.ReadAssembly(bytes);
var options = CompilationOptions.NeoCLR.WithRuntimeTypeOfContract(null)
    .WithTargetCoreAssemblyName(identity.Name)
    .WithMetadataImportOptions(new MetadataImportOptions(identity.Name).WithObjectAssemblyName(identity.Name));
var compilation = Compilation.Create("Consumer",
    [SyntaxTree.ParseText("func Main() -> int { let value = 40; return value + 2 }")], [reference], options);
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
using var image = new MemoryStream();
var emitted = compilation.Emit(image, null, new EmitOptions().WithBackend(new NeoClrEmissionBackend(
    new(new("Consumer", new(1, 0, 0, 0)), identity, [new NeoClrMetadataDependency(reference, identity)]))));
if (!emitted.Success) throw new Exception(string.Join("\n", emitted.Diagnostics));
File.WriteAllBytes(Path.Combine(output, "Consumer.dll"), image.ToArray());
Console.WriteLine("PASS native-only core symbol resolution and consumer emission");

return 0;
