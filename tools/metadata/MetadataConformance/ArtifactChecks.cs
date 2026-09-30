using System.Text.Json;
using NeoCLR.Metadata.Experimental;

internal static class ArtifactChecks
{
    internal static void Run(byte[] input)
    {
        using var vectors = JsonDocument.Parse(input);
        foreach (var test in vectors.RootElement.EnumerateArray())
        {
            bool reject = test.GetProperty("result").GetString() == "reject";
            try
            {
                var bytes = File.ReadAllBytes(test.GetProperty("path").GetString()!);
                bool expected = test.GetProperty("expected_extended").GetBoolean();
                var artifact = expected ? MetadataArtifactReader.Read(bytes) : MetadataArtifactReader.Read(bytes, false);
                bool extended = test.GetProperty("result").GetString() == "extended";
                Require(artifact.IsExtended == extended && (artifact.Profile is not null) == extended, "classification");
                Array.Clear(bytes);
                if (extended)
                {
                    var payload = Convert.FromBase64String(test.GetProperty("profile").GetString()!);
                    Require(MetadataProfile.Write(artifact.Profile!).SequenceEqual(payload), "profile extraction and ownership");
                }
                Require(!reject, "invalid artifact accepted");
            }
            catch (InvalidDataException) when (reject) { }
            catch (Exception error) { throw new Exception($"{test.GetProperty("name").GetString()}: {error.Message}", error); }
        }
        try { MetadataArtifactReader.Read(new byte[MetadataArtifactReader.MaxImageSize + 1]); }
        catch (InvalidDataException)
        {
            Console.WriteLine($"{vectors.RootElement.GetArrayLength()} shared artifact cases and image-size/ownership checks passed");
            return;
        }
        throw new Exception("oversized artifact accepted");
    }
    private static void Require(bool condition, string message)
    {
        if (!condition) throw new Exception(message);
    }
}
