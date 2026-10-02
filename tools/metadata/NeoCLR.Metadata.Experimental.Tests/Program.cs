using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;

if (args.Length == 3 && args[0] == "--constructed-inheritance-runtime")
{
    await ConstructedInheritanceChecks.RunRuntime(args[1], args[2]); return 0;
}
if (args.Length == 3 && args[0] == "--native-binding-integration")
{
    await NativeBindingChecks.RunRuntime(args[1], args[2]); return 0;
}
if (args.Length == 3 && args[0] == "--reference-cast-integration")
{
    await ReferenceCastChecks.RunRuntime(args[1], args[2]); return 0;
}
if (args.Length == 3 && args[0] == "--value-override-integration")
{
    await ValueOverrideChecks.RunRuntime(args[1], args[2]); return 0;
}
if (args.Length == 3 && args[0] == "--function-signature-integration")
{
    await FunctionSignatureChecks.RunRuntime(args[1], args[2]); return 0;
}
if (args.Length == 3 && args[0] == "--nested-import-integration")
{
    await ValueConstructorChecks.RunRuntime(args[1], args[2], nested: true); return 0;
}
if (args.Length == 3 && args[0] == "--nested-type-integration")
{
    await NestedTypeChecks.RunRuntime(args[1], args[2]); return 0;
}
if (args.Length == 3 && args[0] == "--value-constructor-integration")
{
    await ValueConstructorChecks.RunRuntime(args[1], args[2]); return 0;
}
if (args.Length == 3 && args[0] == "--terminal-failure-integration")
{
    await TerminalFailureChecks.RunRuntime(args[1], args[2]); return 0;
}
if (args.Length == 3 && args[0] == "--value-receiver-integration")
{
    await ValueReceiverChecks.RunRuntime(args[1], args[2]); return 0;
}
if (args.Length == 3 && args[0] == "--out-integration")
{
    await OutParameterChecks.RunRuntime(args[1], args[2]); return 0;
}
if (args.Length == 3 && args[0] == "--byref-integration")
{
    await ByReferenceChecks.RunRuntime(args[1], args[2]); return 0;
}
if (args.Length == 3 && args[0] == "--local-object-integration")
{
    await LocalObjectChecks.RunRuntime(args[1], args[2]); return 0;
}
if (args.Length == 3 && args[0] == "--imported-interface-integration")
{
    await ImportedInterfaceChecks.RunRuntime(args[1], args[2]); return 0;
}
if (args.Length == 3 && args[0] == "--imported-value-integration")
{
    await ImportedValueChecks.RunRuntime(args[1], args[2]); return 0;
}
if (args.Length == 3 && args[0] == "--authored-definition-integration")
{
    await AuthoredDefinitionChecks.RunRuntime(args[1], args[2]); return 0;
}
if (args.Length == 3 && args[0] == "--value-type-integration")
{
    await ValueTypeChecks.RunRuntime(args[1], args[2]); return 0;
}
if (args.Length == 3 && args[0] == "--imported-type-integration")
{
    await ImportedTypeChecks.RunRuntime(args[1], args[2]); return 0;
}
if (args.Length == 3 && args[0] == "--interface-dispatch-integration")
{
    await InterfaceDispatchChecks.RunRuntime(args[1], args[2]); return 0;
}
if (args.Length == 3 && args[0] == "--interface-integration")
{
    await InterfaceChecks.RunRuntime(args[1], args[2]);
    return 0;
}
if (args.Length == 3 && args[0] == "--type-constraint-integration")
{
    await TypeConstraintChecks.RunRuntime(args[1], args[2]);
    return 0;
}
if (args.Length == 3 && args[0] == "--generic-class-integration")
{
    await GenericClassChecks.RunRuntime(args[1], args[2]);
    return 0;
}
if (args.Length == 3 && args[0] == "--generic-owner-integration")
{
    await GenericOwnerChecks.RunRuntime(args[1], args[2]);
    return 0;
}

if (args.Length == 3 && args[0] == "--default-integration")
{
    await DefaultValueChecks.RunRuntime(args[1], args[2]);
    return 0;
}

if (args.Length == 3 && args[0] == "--generic-instance-integration")
{
    await GenericInstanceChecks.RunRuntime(args[1], args[2]);
    return 0;
}

if (args.Length == 3 && args[0] == "--generic-integration")
{
    await GenericCallChecks.RunRuntime(args[1], args[2]);
    return 0;
}

if (args.Length == 3 && args[0] == "--indexer-integration")
{
    await IndexedPropertyChecks.RunRuntime(args[1], args[2]);
    return 0;
}

if (args.Length == 3 && args[0] == "--array-integration")
{
    await ArrayInstructionChecks.RunRuntime(args[1], args[2]);
    return 0;
}

if (args.Length == 3 && args[0] == "--readonly-field-integration")
{
    await ReadOnlyFieldChecks.RunRuntime(args[1], args[2]);
    return 0;
}

if (args.Length == 3 && args[0] == "--nominal-property-integration")
{
    await InstanceObjectChecks.RunRuntime(args[1], args[2], nominalProperties: true);
    return 0;
}

if (args.Length == 3 && args[0] == "--nominal-field-integration")
{
    await InstanceObjectChecks.RunRuntime(args[1], args[2], nominalFields: true);
    return 0;
}

if (args.Length == 3 && args[0] == "--nominal-signature-integration")
{
    await InstanceObjectChecks.RunRuntime(args[1], args[2], nominalSignatures: true);
    return 0;
}

if (args.Length == 3 && args[0] == "--nominal-local-integration")
{
    await InstanceObjectChecks.RunRuntime(args[1], args[2], nominalLocals: true);
    return 0;
}

if (args.Length == 3 && args[0] == "--instance-object-integration")
{
    await InstanceObjectChecks.RunRuntime(args[1], args[2]);
    return 0;
}

if (args.Length == 3 && args[0] == "--class-field-integration")
{
    await ClassFieldChecks.RunRuntime(args[1], args[2]);
    return 0;
}

if (args.Length == 3 && args[0] == "--function-namespace-integration")
{
    await FunctionNamespaceChecks.RunRuntime(args[1], args[2]);
    return 0;
}

if (args.Length == 3 && args[0] == "--function-visibility-integration")
{
    await FunctionVisibilityChecks.RunRuntime(args[1], args[2]);
    return 0;
}

if (args.Length == 3 && args[0] == "--method-visibility-integration")
{
    await MethodVisibilityChecks.RunRuntime(args[1], args[2]);
    return 0;
}

if (args.Length == 3 && args[0] == "--argument-store-integration")
{
    await ArgumentStoreChecks.RunRuntime(args[1], args[2]);
    return 0;
}

if (args.Length == 3 && args[0] == "--compare-native")
{
    var decoded = NeoCLR.Metadata.Experimental.NativeModuleContainer.Read(File.ReadAllBytes(args[1]));
    if (!System.Text.Json.Nodes.JsonNode.DeepEquals(System.Text.Json.Nodes.JsonNode.Parse(decoded),
        System.Text.Json.Nodes.JsonNode.Parse(File.ReadAllBytes(args[2]))))
        throw new Exception("native assembly values differ from the JSON baseline");
    Console.WriteLine("PASS independent .NET reader: complete native values match JSON baseline");
    return 0;
}

if (args.Length == 2 && args[0] == "--library-json-check")
{
    LibraryBinaryChecks.RoundTripFile(args[1]);
    return 0;
}

if (args.Length == 2 && args[0] == "--emit-library-fixture")
{
    LibraryBinaryChecks.EmitFixture(args[1]);
    return 0;
}

if (args.Length == 3 && args[0] == "--module-integration")
{
    await NativeModuleChecks.RunRuntime(args[1], args[2]);
    return 0;
}

if (args.Length == 3 && args[0] == "--container-integration")
{
    await RuntimeContainerChecks.RunRuntime(args[1], args[2]);
    return 0;
}

if (args.Length == 3 && args[0] == "--native-integration")
{
    await NativeWriterChecks.RunRuntime(args[1], args[2]);
    return 0;
}

if (args.Length == 6 && args[0] == "--runtime-integration")
{
    await RuntimeIntegration.Run(args[1], args[2], args[3], args[4], args[5]);
    return 0;
}

if (args.Length == 3 && args[0] == "--emit-runtime-probe")
{
    WriterChecks.Emit(args[1], AssemblyDefinition.ReadAssembly(File.ReadAllBytes(args[2]), false).Identity);
    return 0;
}

// Self-contained executable C# contract tests. No Python, runtime assembly load or external test package.
var tests = new (string Name, Action Body)[]
{
    ("Method visibility and reference projection", MethodVisibilityChecks.Run),
    ("Integer shifts and count width validation", ShiftChecks.Run),
    ("Integer bitwise operations and operand validation", BitOperationChecks.Run),
    ("Internal assembly function access", FunctionVisibilityChecks.Run),
    ("Assembly function namespaces", FunctionNamespaceChecks.Run),
    ("Root class and instance field metadata", ClassFieldChecks.Run),
    ("Nominal local identity and aliasing", NominalLocalChecks.Run),
    ("Array signatures, slots and projection", ArraySignatureChecks.Run),
    ("Indexed property signatures and overloads", IndexedPropertyChecks.Run),
    ("Generic method declarations and projection", GenericSignatureChecks.Run),
    ("Generic instantiated and forwarded calls", GenericCallChecks.Run),
    ("Generic instance calls and projection", GenericInstanceChecks.Run),
    ("Typed and generic default initialization", DefaultValueChecks.Run),
    ("Typed local object reads stores and definite assignment", LocalObjectChecks.Run),
    ("Managed-reference signatures calls and projection", ByReferenceChecks.Run),
    ("Output parameters assignment imports and projection", OutParameterChecks.Run),
    ("Value receivers and imported generic output calls", ValueReceiverChecks.Run),
    ("Terminal failure flow and diagnostics", TerminalFailureChecks.Run),
    ("Imported value constructor initialization", () => ValueConstructorChecks.Run()),
    ("Nested definition ownership and execution", NestedTypeChecks.Run),
    ("Imported nested constructors", () => ValueConstructorChecks.Run(nested: true)),
    ("Structural Function signatures and binding", FunctionSignatureChecks.Run),
    ("Concrete value overrides retain direct dispatch", ValueOverrideChecks.Run),
    ("Reference casts preserve object identity and dispatch", ReferenceCastChecks.Run),
    ("Constructed interface inheritance", ConstructedInheritanceChecks.Run),
    ("Local core nominal and Function signatures", LocalCoreSignatureChecks.Run),
    ("Explicit native library bindings preserve CLI reference scope", NativeBindingChecks.Run),
    ("Constructed generic static owners", GenericOwnerChecks.Run),
    ("Constructed generic instance classes", GenericClassChecks.Run),
    ("Nominal type constraints", TypeConstraintChecks.Run),
    ("Special type constraints", SpecialConstraintChecks.Run),
    ("Interface declarations", InterfaceChecks.Run),
    ("Interface dispatch", InterfaceDispatchChecks.Run),
    ("Array instructions and execution", ArrayInstructionChecks.Run),
    ("Readonly field construction and projection", ReadOnlyFieldChecks.Run),
    ("Nominal property identity and accessors", NominalPropertyChecks.Run),
    ("Nominal field identity and alias storage", NominalFieldChecks.Run),
    ("Nominal parameter and result identity", NominalSignatureChecks.Run),
    ("Owned property snapshots and accessor identity", PropertySnapshotChecks.Run),
    ("Property accessor metadata and projection", PropertyChecks.Run),
    ("Root class construction and instance field bodies", InstanceObjectChecks.Run),
    ("Signed integer remainder results and faults", IntegerArithmeticChecks.Remainder),
    ("Signed integer division results and faults", IntegerArithmeticChecks.Division),
    ("Type visibility and projection", TypeVisibilityChecks.Run),
    ("Typed argument stores bounds and caller isolation", ArgumentStoreChecks.Run),
    ("String signatures literals locals and rejected operands", StringChecks.Run),
    ("Signed unary integer operations and boundaries", UnaryIntegerChecks.Run),
    ("Int64 signatures constants conversions and locals", Int64Checks.Run),
    ("Primitive signatures projections overloads and imports", PrimitiveSignatureChecks.Run),
    ("Branch labels typed stack joins and loop execution", FlowChecks.Run),
    ("Int32 local ownership initialization and execution", LocalChecks.Run),
    ("Opcode emission typed operands and helper equivalence", EmitChecks.Run),
    ("No-result entry points across CLI and native containers", EntryPointChecks.Run),
    ("Native library inventory and explicit static callable projections", NativeLibrarySymbolChecks.Run),
    ("Library binary profile and UInt64 bounds", LibraryBinaryChecks.Run),
    ("Existing native module transport", NativeModuleChecks.Run),
    ("Binary CBOR profile and container roundtrips", BinaryEncodingChecks.Run),
    ("Native console literal emission and bounds", ConsoleWriterChecks.Run),
    ("Runtime PE container and required execution schema", RuntimeContainerChecks.Run),
    ("Native dependency identity and snapshot ownership", NativeReaderChecks.References),
    ("Native declaration reader and reference-only projection", NativeReaderChecks.Run),
    ("Read-only callable imports and emission", ImportedReferenceChecks.Run),
    ("Primitive vector imported signatures", VectorImportChecks.Run),
    ("Imported nominal and constructed type signatures", ImportedTypeChecks.Run),
    ("Generic imported signatures and MethodSpec execution", GenericImportChecks.Run),
    ("Imported generic interface dispatch", ImportedInterfaceChecks.Run),
    ("Imported value signatures and CLR execution", ImportedValueChecks.Run),
    ("Nominal imported method signatures and CLR execution", NominalMethodImportChecks.Run),
    ("Value-type categories defaults and CLI/native projection", ValueTypeChecks.Run),
    ("Manual definitions share builder identity and writers", AuthoredDefinitionChecks.Run),
    ("Generic signature recognition and constraint rejection", CallableChecks.GenericSignatureRecognition),
    ("Producer MemberRef dependency and overload resolution", MemberReferenceChecks.ProducerReferences),
    ("Local MemberRef resolution and unsupported contracts", MemberReferenceChecks.LocalAndUnsupported),
    ("MemberRef reader bounds and parent validation", MemberReferenceChecks.Bounds),
    ("Callable declaration roundtrip and ownership", CallableChecks.RoundTrip),
    ("Callable signature recognition and opaque preservation", CallableChecks.SignatureRecognition),
    ("Callable reader resource limits", CallableChecks.Limits),
    ("Native emission and top-level functions", NativeWriterChecks.Run),
    ("Writer construction editing imports and validation", WriterChecks.Run),
    ("Resolves physical top-level and nested TypeRefs", () =>
    {
        var dependency = Read(Image("Dependency", culture: "fr"));
        var consumer = Read(Image("Consumer", references: 1, typeReferences: true));
        var resolver = new Resolver(dependency);
        var outer = consumer.MainModule.TypeReferences[0].Resolve(resolver);
        var nested = consumer.MainModule.TypeReferences[1].Resolve(resolver);
        Check(outer.Name == "Widget" && ReferenceEquals(nested.DeclaringType, outer), "physical reference resolution");
        Check(ReferenceEquals(consumer.MainModule.TypeReferences[0].Module, consumer.MainModule), "consuming scope");
        Throws<InvalidDataException>(() => consumer.MainModule.TypeReferences[0].Resolve(), "resolver required");
        Throws<InvalidDataException>(() => consumer.MainModule.TypeReferences[0].Resolve(new Resolver(null)), "not found");
    }),
    ("Rejects missing nominal type and cyclic TypeRef", () =>
    {
        var consumer = Read(Image("Consumer", references: 1, typeReferences: true, missingType: true));
        Throws<InvalidDataException>(() => consumer.MainModule.TypeReferences[0].Resolve(new Resolver(Read(Image("Dependency", culture: "fr")))), "missing or ambiguous");
        Throws<InvalidDataException>(() => Read(Image("Consumer", references: 1, typeReferences: true, cyclic: true)), "cyclic");
    }),
    ("Reads identity and AssemblyRef ownership", () =>
    {
        var consumer = Read(Image("Consumer", references: 1));
        var reference = consumer.MainModule.AssemblyReferences.Single();
        Check(reference.MetadataToken == 0x23000001 && ReferenceEquals(reference.Module, consumer.MainModule), "reference owner/token");
        Check(reference.Identity.Equals(new AssemblyIdentity("Dependency", V(), "fr")), "reference identity");
        Check(consumer.Identity.Name == consumer.Name && consumer.Identity.Version == consumer.Version, "convenience properties");
    }),
    ("Resolves only through explicit host policy", () =>
    {
        var dependency = Read(Image("Dependency", culture: "fr"));
        var reference = Read(Image("Consumer", references: 1)).MainModule.AssemblyReferences[0];
        var resolver = new Resolver(dependency);
        Check(ReferenceEquals(reference.Resolve(resolver), dependency) && resolver.Calls == 1, "resolved snapshot");
        Check(reference.Identity.Equals(resolver.Last), "requested identity");
        reference.Resolve(resolver);
        Check(resolver.Calls == 2, "no hidden resolution cache");
    }),
    ("Rejects missing and wrong-name dependencies", () =>
    {
        var reference = Read(Image("Consumer", references: 1)).MainModule.AssemblyReferences[0];
        Throws<InvalidDataException>(() => reference.Resolve(new Resolver(null)), "not found");
        Throws<InvalidDataException>(() => reference.Resolve(new Resolver(Read(Image("Wrong", culture: "fr")))), "mismatch");
        Throws<ArgumentNullException>(() => reference.Resolve(null!));
    }),
    ("Rejects version culture key and flag mismatches", () =>
    {
        var reference = Read(Image("Consumer", references: 1)).MainModule.AssemblyReferences[0];
        foreach (var bytes in new[] {
            Image("Dependency", version: new(2, 0, 0, 0), culture: "fr"),
            Image("Dependency", culture: ""),
            Image("Dependency", culture: "fr", key: EcmaKey(), flags: AssemblyFlags.PublicKey),
            Image("Dependency", culture: "fr", flags: AssemblyFlags.Retargetable)
        }) Throws<InvalidDataException>(() => reference.Resolve(new Resolver(Read(bytes))), "mismatch");
    }),
    ("Normalizes full public keys and tokens", () =>
    {
        // ECMA key golden token. This checks identity conversion, not signature authenticity.
        var dependency = Read(Image("Dependency", culture: "fr", key: EcmaKey(), flags: AssemblyFlags.PublicKey));
        Check(dependency.Identity.PublicKeyToken == "b77a5c561934e089" && dependency.Identity.Flags == 0, "ECMA golden token");
        var tokenRef = Read(Image("Consumer", references: 1, referenceKey: Convert.FromHexString("B77A5C561934E089"))).MainModule.AssemblyReferences[0];
        var fullRef = Read(Image("Consumer", references: 1, referenceKey: EcmaKey(), referenceFlags: AssemblyFlags.PublicKey)).MainModule.AssemblyReferences[0];
        Check(tokenRef.Identity.Equals(fullRef.Identity), "full-key versus token identity");
        Check(ReferenceEquals(tokenRef.Resolve(new Resolver(dependency)), dependency), "signed identity match");
    }),
    ("Identity equality is exact and hash-consistent", () =>
    {
        var first = new AssemblyIdentity("Dependency", V(), "fr", "B77A5C561934E089");
        var second = new AssemblyIdentity("Dependency", V(), "fr", "b77a5c561934e089");
        Check(first.Equals(second) && first.Equals((object)second) && first.GetHashCode() == second.GetHashCode(), "equal identities");
        Check(!first.Equals(new AssemblyIdentity("dependency", V(), "fr", first.PublicKeyToken)), "ordinal name");
        Check(!first.Equals(new AssemblyIdentity("Dependency", V(), "FR", first.PublicKeyToken)), "ordinal culture");
        Check(!first.Equals(null) && !first.Equals("other"), "foreign equality");
    }),
    ("Rejects invalid constructed identities", () =>
    {
        Throws<ArgumentException>(() => new AssemblyIdentity("", V()));
        Throws<ArgumentException>(() => new AssemblyIdentity("A", new Version(1, 0)));
        Throws<ArgumentException>(() => new AssemblyIdentity("A", V(), publicKeyToken: "zzzzzzzzzzzzzzzz"));
        Throws<ArgumentException>(() => new AssemblyIdentity("A", V(), flags: 1));
        Throws<ArgumentNullException>(() => new AssemblyIdentity(null!, V()));
    }),
    ("Rejects malformed key representations", () =>
    {
        Throws<InvalidDataException>(() => Read(Image("Consumer", references: 1, referenceKey: [1, 2, 3])), "key/token");
        Throws<InvalidDataException>(() => Read(Image("Dependency", flags: AssemblyFlags.PublicKey)), "key/token");
        Throws<InvalidDataException>(() => Read(Image("Dependency", key: [1, 2, 3])), "key/token");
    }),
    ("Enforces reference count boundary", () =>
    {
        Check(Read(Image("Consumer", references: 256)).MainModule.AssemblyReferences.Count == 256, "256 accepted");
        Throws<InvalidDataException>(() => Read(Image("Consumer", references: 257)), "too many assembly references");
    }),
    ("Identity survives input mutation and reader disposal", () =>
    {
        var bytes = Image("Consumer", references: 1);
        var assembly = Read(bytes);
        Array.Clear(bytes);
        Check(assembly.Identity.Name == "Consumer" && assembly.MainModule.AssemblyReferences[0].Identity.Culture == "fr", "owned values");
    }),
    ("Resolver exceptions retain host meaning", () =>
    {
        var reference = Read(Image("Consumer", references: 1)).MainModule.AssemblyReferences[0];
        Throws<IOException>(() => reference.Resolve(new ThrowingResolver()), "host failure");
    })
};
int failures = 0;
foreach (var (name, test) in tests)
{
    try { test(); Console.WriteLine("PASS " + name); }
    catch (Exception error) { failures++; Console.Error.WriteLine($"FAIL {name}: {error}"); }
}
Console.WriteLine($"{tests.Length - failures}/{tests.Length} C# metadata contract tests passed");
return failures == 0 ? 0 : 1;

static Version V() => new(1, 2, 3, 4);
static byte[] EcmaKey() => Convert.FromHexString("00000000000000000400000000000000");
static AssemblyDefinition Read(byte[] bytes) => AssemblyDefinition.ReadAssembly(bytes, expectedExtended: false);
static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
static void Throws<T>(Action action, string? message = null) where T : Exception
{
    try { action(); }
    catch (T error)
    {
        Check(message is null || error.Message.Contains(message, StringComparison.Ordinal), "unexpected exception reason");
        return;
    }
    throw new Exception("expected " + typeof(T).Name);
}
static byte[] Image(string name, Version? version = null, string culture = "", byte[]? key = null,
    AssemblyFlags flags = 0, int references = 0, byte[]? referenceKey = null, AssemblyFlags referenceFlags = 0, bool typeReferences = false, bool missingType = false, bool cyclic = false)
{
    var metadata = new System.Reflection.Metadata.Ecma335.MetadataBuilder();
    metadata.AddModule(0, metadata.GetOrAddString(name + ".dll"), metadata.GetOrAddGuid(Guid.NewGuid()), default, default);
    metadata.AddAssembly(metadata.GetOrAddString(name), version ?? V(), metadata.GetOrAddString(culture),
        metadata.GetOrAddBlob(key ?? []), flags, AssemblyHashAlgorithm.None);
    metadata.AddTypeDefinition(TypeAttributes.NotPublic, default, metadata.GetOrAddString("<Module>"), default,
        System.Reflection.Metadata.Ecma335.MetadataTokens.FieldDefinitionHandle(1),
        System.Reflection.Metadata.Ecma335.MetadataTokens.MethodDefinitionHandle(1));
    var widget = metadata.AddTypeDefinition(TypeAttributes.Public, metadata.GetOrAddString("Example"), metadata.GetOrAddString("Widget"), default,
        System.Reflection.Metadata.Ecma335.MetadataTokens.FieldDefinitionHandle(1), System.Reflection.Metadata.Ecma335.MetadataTokens.MethodDefinitionHandle(1));
    var nested = metadata.AddTypeDefinition(TypeAttributes.NestedPublic, default, metadata.GetOrAddString("Inner"), default,
        System.Reflection.Metadata.Ecma335.MetadataTokens.FieldDefinitionHandle(1), System.Reflection.Metadata.Ecma335.MetadataTokens.MethodDefinitionHandle(1));
    metadata.AddNestedType(nested, widget);
    if (typeReferences)
    {
        EntityHandle scope = cyclic ? System.Reflection.Metadata.Ecma335.MetadataTokens.TypeReferenceHandle(2) :
            System.Reflection.Metadata.Ecma335.MetadataTokens.AssemblyReferenceHandle(1);
        var outer = metadata.AddTypeReference(scope, metadata.GetOrAddString("Example"), metadata.GetOrAddString(missingType ? "Missing" : "Widget"));
        metadata.AddTypeReference(outer, default, metadata.GetOrAddString("Inner"));
    }
    for (int index = 0; index < references; index++)
        metadata.AddAssemblyReference(metadata.GetOrAddString(index == 0 ? "Dependency" : "Dependency" + index), V(),
            metadata.GetOrAddString("fr"), metadata.GetOrAddBlob(referenceKey ?? []), referenceFlags, default);
    var builder = new ManagedPEBuilder(new PEHeaderBuilder(fileAlignment: 4096, sectionAlignment: 4096),
        new System.Reflection.Metadata.Ecma335.MetadataRootBuilder(metadata), new BlobBuilder(), strongNameSignatureSize: 0);
    var image = new BlobBuilder();
    builder.Serialize(image);
    return image.ToArray();
}

sealed class Resolver(AssemblyDefinition? candidate) : IAssemblyResolver
{
    public int Calls { get; private set; }
    public AssemblyIdentity? Last { get; private set; }
    public AssemblyDefinition? Resolve(AssemblyIdentity identity) { Calls++; Last = identity; return candidate; }
}
sealed class ThrowingResolver : IAssemblyResolver
{
    public AssemblyDefinition? Resolve(AssemblyIdentity identity) => throw new IOException("host failure");
}
