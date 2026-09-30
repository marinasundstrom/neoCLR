using System.Text.Json;
using NeoCLR.Metadata.Experimental;

internal static class MemberChecks
{
    internal static void Run(byte[] input)
    {
        using var document = JsonDocument.Parse(input);
        foreach (var test in document.RootElement.EnumerateArray())
        {
            bool reject = test.GetProperty("result").GetString() == "reject";
            try
            {
                var bytes = Convert.FromBase64String(test.GetProperty("table").GetString()!);
                var members = StructuralMembers.Read(bytes);
                Require(StructuralMembers.Write(members).SequenceEqual(bytes), "table roundtrip");
                if (test.TryGetProperty("profile", out var profile))
                {
                    var descriptors = Resolve(members, profile);
                    if (test.TryGetProperty("contracts", out var contracts))
                    {
                        Require(descriptors.Count == contracts.GetArrayLength(), "contract count");
                        for (int i = 0; i < descriptors.Count; i++)
                        {
                            var actual = descriptors[i];
                            var expected = contracts[i];
                            Require(actual.Identity.Owner.Equals(ReferenceChecks.Resolve(profile)), "owner identity");
                            Require(actual.Identity.Operation == members[i].Operation && actual.Identity.Element == members[i].Element, "operation identity");
                            Require(actual.NoResult == expected.GetProperty("no_result").GetBoolean(), "no-result");
                            Require(actual.Modes.SequenceEqual(expected.GetProperty("modes").EnumerateArray().Select(x => x.GetString()!)), "parameter modes");
                            var parameters = expected.GetProperty("parameters").EnumerateArray().Select(ReferenceChecks.Resolve).ToArray();
                            Require(actual.Parameters.SequenceEqual(parameters), "parameter identities");
                            var result = expected.GetProperty("result");
                            var key = result.ValueKind == JsonValueKind.String ? StructuralMembers.NativeUnsignedResult : ReferenceChecks.Resolve(result);
                            Require(actual.Result.Equals(key), "result identity");
                        }
                    }
                    if (test.TryGetProperty("right", out var right))
                    {
                        var other = Resolve(members, right);
                        bool equal = test.GetProperty("result").GetString() == "equal";
                        Require(descriptors.Select(d => d.Identity).SequenceEqual(other.Select(d => d.Identity)) == equal, "member equality");
                        if (equal)
                            for (int i = 0; i < descriptors.Count; i++)
                            {
                                Require(descriptors[i].Identity.GetHashCode() == other[i].Identity.GetHashCode(), "identity hash");
                                Require(descriptors[i].Result.Equals(other[i].Result) && descriptors[i].Parameters.SequenceEqual(other[i].Parameters) &&
                                    descriptors[i].Modes.SequenceEqual(other[i].Modes) && descriptors[i].NoResult == other[i].NoResult, "equal contracts");
                            }
                    }
                }
                Require(!reject, "invalid input accepted");
            }
            catch (InvalidDataException) when (reject) { }
            catch (Exception error) { throw new Exception($"{test.GetProperty("name").GetString()}: {error.Message}", error); }
        }
        var golden = Convert.FromHexString("0200010002000100010003000000");
        Require(StructuralMembers.Write([new("tuple_element", Element: 1), new("tuple_deconstruct")]).SequenceEqual(golden), "independent emission");
        var owned = StructuralMembers.Read(golden);
        golden[6] = 0;
        Require(owned[0].Element == 1, "reader ownership");
        foreach (var invalid in new StructuralMemberReference[][] {
            [null!], [new("unknown")], [new("array_length", Owner: 0)], [new("array_length", Element: 1)],
            [new("tuple_element", Element: -1)], [new("tuple_element", Element: 256)],
            [new("array_length"), new("array_length")], Enumerable.Repeat(new StructuralMemberReference("array_length"), 257).ToArray()
        })
        {
            try { StructuralMembers.Write(invalid); }
            catch (InvalidDataException) { continue; }
            throw new Exception("invalid writer input accepted");
        }
        Console.WriteLine($"{document.RootElement.GetArrayLength()} shared member vectors and writer/ownership checks passed");
    }
    private static IReadOnlyList<StructuralMemberDescriptor> Resolve(IReadOnlyList<StructuralMemberReference> members, JsonElement profile)
    {
        var (root, context, bindings, catalog) = ReferenceChecks.Profile(profile);
        return StructuralMembers.Resolve(members, root, context, bindings, catalog);
    }
    private static void Require(bool condition, string message)
    {
        if (!condition) throw new Exception(message);
    }
}
