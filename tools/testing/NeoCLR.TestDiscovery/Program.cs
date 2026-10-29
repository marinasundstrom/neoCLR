using System.Text.Json;
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
    var tests = new List<(string Id, string Name, string Description, string Code, bool Dynamic, string[] Categories)>();
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
            method.Accessibility is not (MetadataAccessibility.Public or MetadataAccessibility.Assembly))
            throw new InvalidDataException(name + ": tests require accessible nongeneric module functions");
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
        var reference = RavenNames.Reference(method.Namespace, method.Name);
        var categories = new List<string>();
        foreach (var category in definition.CustomAttributes.Where(a => a.AttributeType.Namespace == "NeoClr.Testing" && a.AttributeType.Name == "CategoryAttribute"))
        {
            var own = assembly.GetTypes().Single(t => t.FullName == "NeoClr.Testing.CategoryAttribute");
            if (!ReferenceEquals(context.Resolve(category.AttributeType), own)) continue;
            var values = category.GetArguments();
            if (values.Count != 1 || values[0].Value is not string label || label.Length == 0 || category.GetNamedArguments().Count != 0)
                throw new InvalidDataException(name + ": Category requires one nonempty string");
            if (!categories.Contains(label, StringComparer.Ordinal)) categories.Add(label);
        }
        categories.Sort(StringComparer.Ordinal);
        // Length-prefixed components avoid punctuation ambiguities. Versions are intentionally
        // excluded: IDs remain stable across rebuilds and assembly version increments.
        static string Part(string value) => value.Length + ":" + value;
        var id = Part(assembly.Name) + Part(method.Module.Name) + Part(name) + "()";
        if (tests.Any(test => test.Id == id)) throw new InvalidDataException("Duplicate test ID: " + id);
        foreach (var row in DataAdapters.Read(definition, method, snapshot, context, "NeoClr.Testing", "MethodDataSourceAttribute"))
        {
            var rowName = name + row.Suffix;
            var rowId = id + row.Suffix;
            var display = description + row.Suffix;
            var callback = row.Suffix.Length == 0 ? reference : "() => " + reference + "(" + row.Callback + ")";
            string Registration(string suffix) => "suite.Add(" + Quote(rowId) + suffix + ", " + Quote(display) + suffix + ", " + callback + ", [" + string.Join(", ", categories.Select(Quote)) + "])";
            var code = row.Provider is null ? "    " + Registration("") + "\n" :
                "    if true {\n        let rows = " + row.ProviderReference + "()\n" +
                "        if rows.Length == 0 { Panic(\"Data source returned no rows\") }\n" +
                "        if rows.Length > 10000 { Panic(\"Data source exceeded 10000 rows\") }\n" +
                "        for index in 0..rows.Length {\n            let value = rows[index]\n            " + Registration(" + \"[\" + index.ToString() + \"]\"") + "\n        }\n    }\n";
            tests.Add((rowId, rowName, display, code, row.Provider is not null, categories.ToArray()));
        }
    }
    if (tests.Count == 0) throw new InvalidDataException("No tests discovered");
    tests.Sort((left, right) => StringComparer.Ordinal.Compare(left.Name, right.Name));
    static string Quote(string value) => RavenNames.StringLiteral(value);
    var source = "// Generated from native metadata. Do not edit.\nmodule NeoClr.Testing\n\nimport System.*\n\nfunc RegisterDiscoveredTests(suite: TestSuite) {\n" +
        string.Concat(tests.Select(item => item.Code)) + "}\n";
    File.WriteAllText(args[1], source);
    File.WriteAllText(args[2], JsonSerializer.Serialize(new { assembly = assembly.Name, tests = tests.Select(t => new { id = t.Id, name = t.Name, description = t.Description, dynamic = t.Dynamic, categories = t.Categories }) }, new JsonSerializerOptions { WriteIndented = true }) + "\n");
    Console.WriteLine("Discovered " + tests.Count + " tests");
    return 0;
}
catch (Exception error) when (error is ArgumentException or InvalidDataException or InvalidOperationException or NotSupportedException or IOException)
{
    if (args.Length >= 3) { File.Delete(args[1]); File.Delete(args[2]); }
    Console.Error.WriteLine("Test discovery failed: " + error.Message);
    return 2;
}
