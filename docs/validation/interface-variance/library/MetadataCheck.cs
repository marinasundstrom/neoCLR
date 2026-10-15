using NeoCLR.Metadata.Experimental.Model;
var assembly = AssemblyDefinition.ReadNativeAssembly(File.ReadAllBytes(args[0]));
var expected = new Dictionary<string, GenericVariance>();
foreach (var name in new[] { "Iterator", "Iterable", "Collection", "Sequence", "AsyncIterator", "AsyncIterable", "ConvertibleInto" }) expected[name] = GenericVariance.Covariant;
foreach (var name in new[] { "Comparer", "EqualityComparer", "ComparableTo", "EquatableTo" }) expected[name] = GenericVariance.Contravariant;
var observed = new Dictionary<string, string[]>();
foreach (var type in assembly.MainModule.Types) {
    var name = type.Name.Split('`')[0];
    if (expected.TryGetValue(name, out var annotation)) {
        if (type.GenericParameterVariance is not { Count: 1 } flags || flags[0] != annotation) throw new Exception("Incorrect variance: " + name);
        observed[name] = flags.Select(x => x.ToString()).ToArray();
    }
    if (new[] { "Sliceable", "Result", "List", "MutableSequence", "ArrayList", "Set", "Map", "Array" }.Contains(name)) {
        if (type.GenericParameterVariance is not { } flags || flags.Any(x => x != GenericVariance.Invariant)) throw new Exception("Expected invariant: " + name);
        if (flags.Count > 0) observed[name] = flags.Select(x => x.ToString()).ToArray();
    }
}
if (expected.Keys.Any(name => !observed.ContainsKey(name))) throw new Exception("Missing annotation");
Console.WriteLine(System.Text.Json.JsonSerializer.Serialize(observed, new System.Text.Json.JsonSerializerOptions { WriteIndented = true }));
