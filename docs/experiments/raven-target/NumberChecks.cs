using Mono.Cecil;
using System.Text.Json;

static class NumberChecks
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
            if (!valid) throw new Exception("Number check failed: " + name);
            checks.Add(name);
        }
        void Reject(string name, Action action) {
            try { action(); } catch (InvalidDataException) { checks.Add(name); return; }
            throw new Exception("Number check did not reject: " + name);
        }
        MethodReference Reference(MethodDefinition method, TypeReference owner) {
            var reference = new MethodReference(method.Name, method.ReturnType, owner) { HasThis = method.HasThis };
            foreach (var parameter in method.Parameters)
                reference.Parameters.Add(new ParameterDefinition(parameter.Name, parameter.Attributes, parameter.ParameterType));
            return reference;
        }
        NumberBindings.Validate(module);
        Check("Number contract validates", true);
        var contract = module.GetType(NumberBindings.Contract);
        var zero = contract.Methods.Single(m => m.Name == "get_Zero");
        zero.IsStatic = false;
        Reject("Number rejects instance identity", () => NumberBindings.Validate(module));
        zero.IsStatic = true;
        zero.ReturnType = module.TypeSystem.Int32;
        Reject("Number rejects non-generic identity result", () => NumberBindings.Validate(module));
        zero.ReturnType = contract.GenericParameters[0];
        foreach (var name in NumberBindings.Types.Append("Boolean"))
        {
            var type = module.GetType("System." + name);
            foreach (var method in type.Methods.Where(m => NumberBindings.Operators.Contains(m.Name)
                || m.Name is "get_Zero" or "get_One" || NumberBindings.HasNewParser(name) && m.Name == "Parse"))
            {
                Check(name + "." + method.Name + " binds", NumberBindings.Bind(Reference(method, type), method) is not null);
                var wrong = Reference(method, type);
                wrong.HasThis = true;
                Reject(name + "." + method.Name + " rejects receiver", () => NumberBindings.Bind(wrong, method));
                wrong = Reference(method, type);
                wrong.ReturnType = module.TypeSystem.Char;
                Reject(name + "." + method.Name + " rejects wrong result", () => NumberBindings.Bind(wrong, method));
            }
        }
        File.WriteAllText(Path.Combine(output, "number-signatures.json"), JsonSerializer.Serialize(checks, new JsonSerializerOptions { WriteIndented = true }));
        Console.WriteLine($"{checks.Count} number signature checks passed");
    }
}
