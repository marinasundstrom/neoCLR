using System.Text.Json;
using System.Text.RegularExpressions;
using NeoCLR.Metadata.Experimental.Introspection;
using NeoCLR.Metadata.Experimental.Model;

// Metadata-only discovery: never load the target into the CLR or execute constructors.
// Typed Raven adapters make the discovered call graph explicit to native trimming.
try
{
    if (args.Length < 4)
        throw new ArgumentException("Usage: NeoCLR.TestDiscovery TEST-ASSEMBLY REGISTRY.rvn MANIFEST.json DEPENDENCY...");
    var snapshot = AssemblyDefinition.ReadNativeAssembly(File.ReadAllBytes(args[0]));
    var dependencies = args.Skip(3).Select(path => AssemblyDefinition.ReadNativeAssembly(File.ReadAllBytes(path))).ToArray();
    var context = new MetadataLoadContext(new[] { snapshot }.Concat(dependencies));
    var assembly = context.Resolve(snapshot.Identity);
    var marker = assembly.GetTypes().Single(t => t.FullName == "NeoClr.Testing.TestAttribute");
    var failure = assembly.GetTypes().Single(t => t.FullName == "NeoClr.Testing.TestFailure");
    var runtime = context.Assemblies.Single(a => a.Name == "System.Runtime");
    var attribute = runtime.GetTypes().Single(t => t.FullName == "System.Attribute");
    if (!ReferenceEquals(marker.BaseType, attribute))
        throw new InvalidDataException("TestAttribute must derive from the catalog's System.Attribute");
    var unit = runtime.GetTypes().Single(t => t.FullName == "System.Void");
    var tests = new List<(string Id, string Name, string Description)>();
    foreach (var definition in snapshot.MainModule.Methods)
    {
        // Filter before projecting signatures so unrelated unsupported methods do not
        // become accidental prerequisites of test discovery.
        var matches = definition.CustomAttributes.Where(a => a.AttributeType.Namespace == "NeoClr.Testing" && a.AttributeType.Name == "TestAttribute" && ReferenceEquals(context.Resolve(a.AttributeType), marker)).ToArray();
        if (matches.Length == 0) continue;
        var method = context.Resolve(definition);
        var name = method.DeclaringType is NominalTypeInfo owner ? owner.FullName + "." + method.Name :
            method.Namespace.Length == 0 ? method.Name : method.Namespace + "." + method.Name;
        if (matches.Length != 1 || matches[0].GetArguments().Count > 1 || matches[0].GetNamedArguments().Count > 1)
            throw new InvalidDataException(name + ": TestAttribute must occur once with an optional string description");
        if (method.DeclaringType is not null || !method.IsStatic || method.IsAbstract || method.IsGenericMethodDefinition ||
            method.Accessibility is not (MetadataAccessibility.Public or MetadataAccessibility.Assembly) || method.GetParameters().Count != 0)
            throw new InvalidDataException(name + ": tests require accessible parameterless, nongeneric module functions");
        if (method.ReturnType is not ConstructedTypeInfo result || result.Definition.FullName != "System.Result`2" ||
            !ReferenceEquals(result.Definition.Module.Assembly, runtime) || result.TypeArguments.Count != 2 ||
            !ReferenceEquals(result.TypeArguments[1], failure) || !ReferenceEquals(result.TypeArguments[0], unit))
            throw new InvalidDataException(name + ": tests require Result<unit, TestFailure>");
        var arguments = matches[0].GetArguments();
        var description = name;
        if (arguments.Count == 1)
        {
            if (arguments[0].Value is not string text) throw new InvalidDataException(name + ": description must be a string");
            if (text.Length != 0) description = text;
        }
        foreach (var named in matches[0].GetNamedArguments())
        {
            if (named.IsField || named.MemberName != "Description" || named.TypedValue.Value is not string text)
                throw new InvalidDataException(name + ": only the string Description property is supported");
            description = text.Length == 0 ? name : text;
        }
        if (!name.Split('.').All(part => Regex.IsMatch(part, "^[A-Za-z_][A-Za-z0-9_]*$")))
            throw new InvalidDataException(name + ": this adapter requires plain Raven identifiers");
        // Length-prefixed components avoid punctuation ambiguities. Versions are intentionally
        // excluded: IDs remain stable across rebuilds and assembly version increments.
        static string Part(string value) => value.Length + ":" + value;
        var id = Part(assembly.Name) + Part(method.Module.Name) + Part(name) + "()";
        if (tests.Any(test => test.Id == id)) throw new InvalidDataException("Duplicate test ID: " + id);
        tests.Add((id, name, description));
    }
    if (tests.Count == 0) throw new InvalidDataException("No tests discovered");
    tests.Sort((left, right) => StringComparer.Ordinal.Compare(left.Name, right.Name));
    static string Quote(string value) => JsonSerializer.Serialize(value);
    var source = "// Generated from native metadata. Do not edit.\nmodule NeoClr.Testing\n\nfunc RegisterDiscoveredTests(suite: TestSuite) {\n" +
        string.Concat(tests.Select(test => "    suite.Add(" + Quote(test.Id) + ", " + Quote(test.Description) + ", " + test.Name + ")\n")) + "}\n";
    File.WriteAllText(args[1], source);
    File.WriteAllText(args[2], JsonSerializer.Serialize(new { assembly = assembly.Name, tests = tests.Select(t => new { id = t.Id, name = t.Name, description = t.Description }) }, new JsonSerializerOptions { WriteIndented = true }) + "\n");
    Console.WriteLine("Discovered " + tests.Count + " tests");
    return 0;
}
catch (Exception error) when (error is ArgumentException or InvalidDataException or InvalidOperationException or NotSupportedException or IOException)
{
    Console.Error.WriteLine("Test discovery failed: " + error.Message);
    return 2;
}
