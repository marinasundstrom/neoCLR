using System.Text.Json;
using NeoCLR.Metadata.Experimental;

internal static class ProfileChecks
{
    internal static void Run(byte[] input)
    {
        using var vectors = JsonDocument.Parse(input);
        foreach (var test in vectors.RootElement.EnumerateArray())
        {
            bool reject = test.GetProperty("reject").GetBoolean();
            try
            {
                var bytes = Convert.FromBase64String(test.GetProperty("image").GetString()!);
                var document = MetadataProfile.Read(bytes);
                Require(MetadataProfile.Write(document).SequenceEqual(bytes), "roundtrip");
                Require(document.HasMemberTable == test.GetProperty("has_members").GetBoolean(), "member presence");
                Require(document.UnknownOptionalSections.Count == test.GetProperty("unknown").GetInt32(), "opaque extension count");
                if (test.TryGetProperty("profile", out var profile))
                {
                    var (_, _, _, catalog) = ReferenceChecks.Profile(profile);
                    Require(document.ResolveType(catalog).Equals(ReferenceChecks.Resolve(profile)), "type resolution");
                    Require(document.ResolveMembers(catalog).Count == document.Members.Count, "member resolution");
                }
                // Immutable document must remain independent of caller buffers.
                Array.Clear(bytes);
                Require(MetadataProfile.Read(MetadataProfile.Write(document)).Root.Kind == document.Root.Kind, "ownership");
                Require(!reject, "invalid profile accepted");
            }
            catch (InvalidDataException) when (reject) { }
            catch (Exception error) { throw new Exception($"{test.GetProperty("name").GetString()}: {error.Message}", error); }
        }
        var unresolved = MetadataProfile.Read(Convert.FromBase64String(
            vectors.RootElement.EnumerateArray().Single(v => v.GetProperty("name").GetString() == "references-a").GetProperty("image").GetString()!));
        Reject(() => unresolved.ResolveType(new Dictionary<MetadataReference, MetadataDefinition>()));
        Reject(() => unresolved.ResolveMembers(new Dictionary<MetadataReference, MetadataDefinition>()));
        var root = new TypeExpression("tuple", [new("int32"), new("string")]);
        var members = new[] { new StructuralMemberReference("tuple_element", Element: 1), new StructuralMemberReference("tuple_deconstruct") };
        var documentFromWriter = MetadataProfile.Create(root, new(), new([]), members);
        Require(MetadataProfile.Write(documentFromWriter).SequenceEqual(Convert.FromBase64String(
            vectors.RootElement.EnumerateArray().Single(v => v.GetProperty("name").GetString() == "tuple-fixture").GetProperty("image").GetString()!)), "independent profile emission");
        members[0] = new("array_length");
        Require(documentFromWriter.Members[0].Operation == "tuple_element", "writer ownership");
        var empty = MetadataProfile.Create(new("int32"), new(), new([]), []);
        var absent = MetadataProfile.Create(new("int32"), new(), new([]));
        Require(empty.HasMemberTable && !absent.HasMemberTable && empty.Members.Count == 0, "absent vs empty");
        var extension = new MetadataSection(60000, 99, false, [255, 0, 1]);
        Require(MetadataProfile.Create(new("int32"), new(), new([]), optionalSections: [extension]).UnknownOptionalSections[0].GetPayload().SequenceEqual(extension.GetPayload()), "writer extensions");
        foreach (var extras in new MetadataSection[][] { [new(4, 99, false, [])], [new(60000, 1, true, [])], [extension, extension], [null!] })
            Reject(() => MetadataProfile.Create(new("int32"), new(), new([]), optionalSections: extras));
        Reject(() => MetadataProfile.Create(new("int32"), new(), new([]), [new("array_length")]));
        Reject(() => MetadataProfile.Create(new("nominal", index: 1), new(), new([])));
        Console.WriteLine($"{vectors.RootElement.GetArrayLength()} profile vectors plus independent emission, creation and ownership checks passed");
    }
    private static void Reject(Action action)
    {
        try { action(); }
        catch (InvalidDataException) { return; }
        throw new Exception("invalid create input accepted");
    }
    private static void Require(bool condition, string message)
    {
        if (!condition) throw new Exception(message);
    }
}
