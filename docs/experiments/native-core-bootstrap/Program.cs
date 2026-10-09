using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using Raven.CodeAnalysis;
using Raven.CodeAnalysis.NeoClr;
using Raven.CodeAnalysis.Syntax;

if (args.Length is < 1 or > 2 || args.Length == 2 && args[1] is not ("--text-services" or "--source-string")) throw new ArgumentException("Specify a fresh output directory and optional --text-services or --source-string.");
var sourceString = args.Length == 2 && args[1] == "--source-string";
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
if (sourceString)
{
    core.AddClass("System", "Enum", root);
    core.AddClass("System", "Array", root);
}
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
foreach (var primitive in new[] { PrimitiveType.Byte, PrimitiveType.Int32, PrimitiveType.Int64, PrimitiveType.Double, PrimitiveType.Boolean, PrimitiveType.Void })
    core.AddValueType("System", primitive.ToString()).SetNativePrimitive(primitive);
var stringType = core.AddClass("System", "String", root);
if (sourceString) stringType.SetNativePrimitiveReference(PrimitiveType.String);
else stringType.SetNativePrimitive(PrimitiveType.String);
if (args.Length == 2)
{
    var charType = core.AddValueType("System", "Char");
    charType.SetNativeGrapheme();
    var services = core.AddClass("System.Runtime.CompilerServices", "RuntimeServices", root);
    var nativeText = new Dictionary<string, MethodBuilder>();
    MethodBuilder AddTextService(string name, MethodSignature signature)
    {
        var runtimeCall = core.AddFunction("neoCLR.Runtime", name, signature);
        runtimeCall.SetInternalCall();
        var facade = services.AddMethod(name, signature);
        var il = facade.GetILGenerator();
        for (var i = 0; i < signature.ParameterTypes.Count; i++) il.LoadArgument(i);
        il.Call(runtimeCall);
        il.Return();
        nativeText.Add(name, runtimeCall);
        return runtimeCall;
    }
    foreach (var name in new[] { "StringContainsOrdinal", "StringStartsWithOrdinal", "StringEndsWithOrdinal" })
        AddTextService(name, new(PrimitiveType.Boolean, [PrimitiveType.String, PrimitiveType.String]));
    var identitySignature = new MethodSignature(PrimitiveType.Boolean, [root, root]);
    var referenceEquals = core.AddFunction("neoCLR.Runtime", "ObjectReferenceEquals", identitySignature);
    referenceEquals.SetInternalCall();
    var identityWrapper = root.AddMethod("ReferenceEquals", identitySignature);
    identityWrapper.GetILGenerator().LoadArgument(0);
    identityWrapper.GetILGenerator().LoadArgument(1);
    identityWrapper.GetILGenerator().Call(referenceEquals);
    identityWrapper.GetILGenerator().Return();
    foreach (var (member, service, result) in new[] {
        ("Concat", "StringConcat", PrimitiveType.String),
        ("CompareOrdinal", "StringCompareOrdinal", PrimitiveType.Int32) })
    {
        var signature = new MethodSignature(result, [PrimitiveType.String, PrimitiveType.String]);
        var runtimeCall = AddTextService(service, signature);
        if (sourceString) continue;
        var wrapper = stringType.AddMethod(member, signature);
        var il = wrapper.GetILGenerator();
        il.LoadArgument(0);
        il.LoadArgument(1);
        il.Call(runtimeCall);
        il.Return();
    }
    var replaceService = AddTextService("StringReplaceOrdinal",
        new(PrimitiveType.String, [PrimitiveType.String, PrimitiveType.String, PrimitiveType.String]));
    if (!sourceString)
    {
        var replace = stringType.AddInstanceMethod("Replace",
            new(PrimitiveType.String, [PrimitiveType.String, PrimitiveType.String]));
        var replaceIl = replace.GetILGenerator();
        replaceIl.LoadArgument(0);
        replaceIl.LoadArgument(1);
        replaceIl.LoadArgument(2);
        replaceIl.Call(replaceService);
        replaceIl.Return();
    }
    var byteCount = AddTextService("StringByteCount", new(PrimitiveType.Int32, [PrimitiveType.String]));
    if (!sourceString)
    {
        var count = stringType.AddInstanceMethod("GetByteCount", new(PrimitiveType.Int32, []));
        count.GetILGenerator().LoadArgument(0);
        count.GetILGenerator().Call(byteCount);
        count.GetILGenerator().Return();
    }
    if (sourceString)
    {
        core.AddValueType("System", "UInt32").SetNativePrimitive(PrimitiveType.UInt32);
        var value = core.AddValueType("System", "Value");
        value.SetNativePrimitive(PrimitiveType.Value);
        foreach (var name in new[] { "StringIntern", "StringToUpperInvariant", "StringToLowerInvariant" })
            AddTextService(name, new(PrimitiveType.String, [PrimitiveType.String]));
        AddTextService("StringGraphemeCount", new(PrimitiveType.Int32, [PrimitiveType.String]));
        AddTextService("StringCompareOrdinalIgnoreCase", new(PrimitiveType.Int32, [PrimitiveType.String, PrimitiveType.String]));
        AddTextService("StringGraphemeAt", new(charType, [PrimitiveType.String, PrimitiveType.Int32]));
        AddTextService("CharFromString", new(charType, [PrimitiveType.String]));
        AddTextService("CharText", new(PrimitiveType.String, [charType]));
        AddTextService("StringFromChars", new(PrimitiveType.String, [SignatureType.ArrayOf(charType)]));
        AddTextService("StringGraphemes", new(SignatureType.ArrayOf(charType), [PrimitiveType.String]));
        AddTextService("StringScalars", new(SignatureType.ArrayOf(PrimitiveType.UInt32), [PrimitiveType.String]));
        AddTextService("StringJoinParts", new(PrimitiveType.String, [SignatureType.ArrayOf(PrimitiveType.String), PrimitiveType.Int32, PrimitiveType.String, PrimitiveType.Int32]));
        AddTextService("StringSliceUtf8", new(value, [PrimitiveType.String, PrimitiveType.Int32, PrimitiveType.Int32]));
        var equal = services.AddMethod("StringEquals", new(PrimitiveType.Boolean, [PrimitiveType.String, PrimitiveType.String]));
        equal.LoadArgument(0); equal.LoadArgument(1); equal.Call(nativeText["StringCompareOrdinal"]);
        equal.LoadConstant(0); equal.Emit(OpCode.Ceq); equal.Return();
        foreach (var unpack in new[] { false, true })
        {
            var target = SignatureType.MethodParameter(0);
            var valueMethod = services.AddMethod(unpack ? "UnpackValue" : "IsValue", new(unpack ? target : PrimitiveType.Boolean, [value], ["T"]));
            valueMethod.LoadArgument(0); valueMethod.Emit(unpack ? OpCode.ValueUnpack : OpCode.ValueIs, target); valueMethod.Return();
        }
        var storage = core.AddClass("System.Runtime.CompilerServices", "CheckedStorage", root);
        var reserve = storage.AddMethod("Reserve", new(SignatureType.ArrayOf(SignatureType.MethodParameter(0)), [PrimitiveType.Int32], ["T"]));
        reserve.LoadArgument(0); reserve.Emit(OpCode.ReserveArray, SignatureType.MethodParameter(0)); reserve.Return();
        var fail = core.AddFunction("neoCLR.Runtime", "Fail", new(PrimitiveType.Void, [PrimitiveType.String]));
        fail.SetInternalCall();
        var failure = core.AddFunction("System", "Fail", new(PrimitiveType.Void, [PrimitiveType.String]));
        failure.LoadArgument(0); failure.Call(fail); failure.Return();
    }
}
var bytes = RuntimeAssemblyContainer.WriteLibraryBinary(core);
File.WriteAllBytes(Path.Combine(output, "NativeCore.dll"), bytes);
if (sourceString) return SourceString.Build(core, output);
var input = new AssemblyBuilder(new("Input", new(1, 0, 0, 0)), identity);
var method = input.AddType("Example", "Input").AddMethod("Value", new(PrimitiveType.Int32, []));
method.GetILGenerator().LoadConstant(40);
method.GetILGenerator().Return();
var libraryPath = Path.Combine(output, "Input.dll");
File.WriteAllBytes(libraryPath, RuntimeAssemblyContainer.WriteLibraryBinary(input));
var catalog = NeoClrReferenceCatalog.ReadNative(Path.Combine(output, "NativeCore.dll"), [libraryPath]);
var reference = catalog.NativeCore!;
var options = CompilationOptions.NeoCLR.WithRuntimeTypeOfContract(null)
    .WithTargetCoreAssemblyName(identity.Name)
    .WithRuntimeUnitContract(new(identity.Name, "System.Void"))
    .WithMetadataImportOptions(new MetadataImportOptions(identity.Name).WithObjectAssemblyName(identity.Name).WithNativeMetadata());
var source = SyntaxTree.ParseText(File.ReadAllText(Path.Combine(AppContext.BaseDirectory, "consumer.rvn")));
var compilation = Compilation.Create("Consumer", [source], catalog.References.ToArray(), options);
var errors = compilation.GetDiagnostics().Where(d => d.Severity == DiagnosticSeverity.Error).ToArray();
if (errors.Length != 0)
{
    Console.Error.WriteLine("Native core artifact written; compiler semantic frontier:");
    foreach (var error in errors) Console.Error.WriteLine(error);
    return 2;
}
foreach (var type in new[] { SpecialType.System_Byte, SpecialType.System_Int32, SpecialType.System_Boolean, SpecialType.System_Object, SpecialType.System_ValueType })
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
    new(new("Consumer", new(1, 0, 0, 0)), identity, catalog.Dependencies))));
if (!emitted.Success) throw new Exception(string.Join("\n", emitted.Diagnostics));
File.WriteAllBytes(Path.Combine(output, "Consumer.dll"), image.ToArray());
Console.WriteLine("PASS native-only core symbols, consumer emission, missing-core/identity rejection and CLI output guard");

return 0;
