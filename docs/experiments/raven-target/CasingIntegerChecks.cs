using Mono.Cecil;
using System.Text.Json;

static class CasingIntegerChecks
{
    public static void Write(string output)
    {
        Directory.CreateDirectory(output);
        var path = Path.Combine(output, "NeoCLR.CoreProbe.dll");
        CoreDeclarations.Write(path, unionProbe: true, collectionProbe: true);
        using var core = AssemblyDefinition.ReadAssembly(path);
        var module = core.MainModule;
        var checks = new List<string>();
        void Check(string name, bool valid) {
            if (!valid) throw new Exception("Casing/integer check failed: " + name);
            checks.Add(name);
        }
        void Reject(string name, Action action) {
            try { action(); } catch (InvalidDataException) { checks.Add(name); return; }
            throw new Exception("Casing/integer check did not reject: " + name);
        }
        MethodReference Reference(MethodDefinition method, TypeReference owner) {
            var reference = new MethodReference(method.Name, method.ReturnType, owner) { HasThis = method.HasThis };
            foreach (var parameter in method.Parameters)
                reference.Parameters.Add(new ParameterDefinition(parameter.Name, parameter.Attributes, parameter.ParameterType));
            return reference;
        }
        var integer = module.GetType("System.Int64");
        foreach (var name in new[] { "Parse", "ToString", "get_MinValue", "get_MaxValue" }) {
            var method = integer.Methods.Single(m => m.Name == name);
            Check(name + " binds", PrimitiveBindings.Bind(Reference(method, integer), method) is not null);
            var wrong = Reference(method, integer);
            wrong.ReturnType = module.TypeSystem.Boolean;
            Reject(name + " rejects wrong result", () => PrimitiveBindings.Bind(wrong, method));
            wrong = Reference(method, integer);
            wrong.HasThis = !wrong.HasThis;
            Reject(name + " rejects wrong receiver", () => PrimitiveBindings.Bind(wrong, method));
        }
        var text = module.GetType("System.String");
        foreach (var name in new[] { "ToUpperInvariant", "ToLowerInvariant" }) {
            var method = text.Methods.Single(m => m.Name == name);
            Check(name + " binds", StringBindings.Bind(Reference(method, text), method, false)?.Result == "String");
            var wrong = Reference(method, text);
            wrong.ReturnType = module.TypeSystem.Char;
            Reject(name + " rejects char result", () => StringBindings.Bind(wrong, method, false));
        }
        File.WriteAllText(Path.Combine(output, "casing-integer-signatures.json"), JsonSerializer.Serialize(checks, new JsonSerializerOptions { WriteIndented = true }));
        Console.WriteLine($"{checks.Count} casing/integer signature checks passed");
    }
}
