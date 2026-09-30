using System.Text.Json;
using NeoCLR.Metadata.Experimental;

// Test-only transport: the production resolver has no JSON or fixture catalog dependency.
internal static class ReferenceChecks
{
    internal static void Run(byte[] input)
    {
        using var document = JsonDocument.Parse(input);
        foreach (var test in document.RootElement.EnumerateArray())
        {
            var name = test.GetProperty("name").GetString();
            bool reject = test.GetProperty("result").GetString() == "reject";
            try
            {
                if (test.TryGetProperty("table", out var table))
                {
                    var bytes = Convert.FromBase64String(table.GetString()!);
                    var bindings = ReferenceTable.Read(bytes);
                    Require(ReferenceTable.Write(bindings).SequenceEqual(bytes), "reference roundtrip");
                }
                else
                {
                    var left = Resolve(test.GetProperty("left"));
                    if (test.TryGetProperty("right", out var right))
                    {
                        var other = Resolve(right);
                        bool equal = test.GetProperty("result").GetString() == "equal";
                        Require(left.Equals(other) == equal && left.Equals((object)other) == equal, "equality");
                        Require(!equal || left.GetHashCode() == other.GetHashCode(), "equal hash");
                        Require(!left.Equals(null) && !left.Equals("unrelated"), "foreign equality");
                    }
                }
                Require(!reject, "invalid input accepted");
            }
            catch (InvalidDataException) when (reject) { }
            catch (Exception error) { throw new Exception($"{name}: {error.Message}", error); }
        }
        var reference = new MetadataReference(Guid.Parse("01234567-89ab-cdef-0123-456789abcdef"),
            Guid.Parse("fedcba98-7654-3210-fedc-ba9876543210"), 0x02000001);
        var source = new[] { reference };
        var owned = new ReferenceBindings(source, typeOwner: 1);
        source[0] = default;
        Require(owned.References[0] == reference, "reference list ownership");
        var golden = Convert.FromHexString("01000100000000000123456789ABCDEF0123456789ABCDEFFEDCBA9876543210FEDCBA987654321001000002");
        Require(ReferenceTable.Write(owned).SequenceEqual(golden), "independent network UUID emission");
        Require(ReferenceTable.Read(golden).References[0] == reference, "network UUID reading");
        foreach (var invalid in new[] { new ReferenceBindings([reference, reference]),
            new ReferenceBindings([default]), new ReferenceBindings([reference], typeOwner: -1),
            new ReferenceBindings([reference], methodOwner: 1) })
        {
            try { ReferenceTable.Write(invalid); }
            catch (InvalidDataException) { continue; }
            throw new Exception("invalid writer input accepted");
        }
        Console.WriteLine($"{document.RootElement.GetArrayLength()} shared reference/identity vectors and writer ownership checks passed");
    }

    internal static ResolvedTypeIdentity Resolve(JsonElement specification)
    {
        var (root, context, bindings, catalog) = Profile(specification);
        return StructuralIdentity.Resolve(root, context, bindings, catalog);
    }

    internal static (TypeExpression, SignatureContext, ReferenceBindings, Dictionary<MetadataReference, MetadataDefinition>) Profile(JsonElement specification)
    {
        var signature = StructuralSignature.Read(Convert.FromBase64String(specification.GetProperty("signature").GetString()!), true);
        var bindings = ReferenceTable.Read(Convert.FromBase64String(specification.GetProperty("bindings").GetString()!));
        var catalog = new Dictionary<MetadataReference, MetadataDefinition>();
        foreach (var entry in specification.GetProperty("catalog").EnumerateArray())
        {
            var owner = entry.GetProperty("owner");
            catalog.Add(Reference(entry.GetProperty("reference")), new MetadataDefinition(
                entry.GetProperty("kind").GetString()!, entry.GetProperty("arity").GetInt32(),
                owner.ValueKind == JsonValueKind.Null ? null : Reference(owner)));
        }
        return (signature.Root, signature.Context, bindings, catalog);
    }

    private static MetadataReference Reference(JsonElement element) => new(
        Guid.Parse(element[0].GetString()!), Guid.Parse(element[1].GetString()!), element[2].GetUInt32());
    private static void Require(bool condition, string message)
    {
        if (!condition) throw new Exception(message);
    }
}
