using System.Reflection;
using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;

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
