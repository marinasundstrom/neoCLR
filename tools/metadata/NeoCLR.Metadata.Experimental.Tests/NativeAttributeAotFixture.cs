using System.Text.Json;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;

// Isolate native retention/materialization from the still-incomplete Raven annotation emitter.
internal static class NativeAttributeAotFixture
{
    internal static void Write(string input, string output, string roots)
    {
        var module = JsonNode.Parse(RuntimeAssemblyContainer.Read(File.ReadAllBytes(input)))!;
        var types = module["types"]!.AsArray();
        var subject = types.Single(t => t!["origin"]!["name"]!.GetValue<string>() == "Subject")!;
        var plain = types.Single(t => t!["origin"]!["name"]!.GetValue<string>() == "Plain")!;
        var marker = types.Single(t => t!["origin"]!["name"]!.GetValue<string>() == "NoteAttribute")!;
        var constructor = module["functions"]!.AsArray().Single(f =>
            f!["origin"]!["name"]!.GetValue<string>() == ".ctor" &&
            f["owner"]!["Named"]!.GetValue<string>() == marker["name"]!.GetValue<string>())!;
        var reference = new JsonObject();
        foreach (var key in new[] { "definition", "name", "owner", "instance", "parameters" })
            if (constructor[key] is JsonNode value) reference[key] = value.DeepClone();
        JsonObject Attribute(string? text, int? target = null)
        {
            var result = new JsonObject
            {
                ["constructor"] = reference.DeepClone(),
                ["arguments"] = new JsonArray(new JsonObject { ["String"] = text },
                    new JsonObject { ["Int32"] = 42 }, new JsonObject { ["Boolean"] = true },
                    new JsonObject { ["Int32"] = 7 })
            };
            if (target is int token) result["target_token"] = token;
            return result;
        }
        subject["custom_attributes"] = new JsonArray(Attribute("type ☃"), Attribute(null),
            Attribute("property", subject["origin"]!["property_tokens"]![0]!.GetValue<int>()));
        var setter = module["functions"]!.AsArray().Single(f =>
            f!["origin"]!["name"]!.GetValue<string>() == "set_Name")!;
        setter["custom_attributes"] = new JsonArray(Attribute("method"),
            Attribute("parameter", setter["origin"]!["parameter_tokens"]![0]!.GetValue<int>()));
        File.WriteAllBytes(output, NativeModuleContainer.WriteLibraryBinary(System.Text.Encoding.UTF8.GetBytes(module.ToJsonString())));
        var retained = new JsonArray();
        foreach (var type in new[] { subject, plain })
            retained.Add(new JsonObject
            {
                ["definition"] = new JsonObject { ["module"] = module["name"]!.DeepClone(), ["revision"] = module["revision"]!.DeepClone(), ["index"] = types.IndexOf(type) },
                ["construct"] = false,
                ["properties"] = ReferenceEquals(type, subject),
                ["getters"] = false,
                ["setters"] = false,
                ["customAttributes"] = true
            });
        File.WriteAllText(roots, new JsonObject { ["schemaVersion"] = 3, ["types"] = retained }
            .ToJsonString(new JsonSerializerOptions { WriteIndented = true }));
    }
}
