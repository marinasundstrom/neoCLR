using Mono.Cecil;
using System.Text.Json;

static class ComparerChecks
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
            if (!valid) throw new Exception("Comparer check failed: " + name);
            checks.Add(name);
        }
        void Reject(string name, Action action) {
            try { action(); } catch (InvalidDataException) { checks.Add(name); return; }
            throw new Exception("Comparer check did not reject: " + name);
        }
        MethodReference Reference(MethodDefinition method, TypeReference owner) {
            var reference = new MethodReference(method.Name, method.ReturnType, owner) { HasThis = method.HasThis };
            foreach (var parameter in method.Parameters)
                reference.Parameters.Add(new ParameterDefinition(parameter.Name, parameter.Attributes, parameter.ParameterType));
            return reference;
        }
        ComparerBindings.Validate(module);
        var comparerDefinition = module.GetType("System.Collections.EqualityComparer`1");
        var closedComparer = new GenericInstanceType(comparerDefinition);
        closedComparer.GenericArguments.Add(module.TypeSystem.String);
        var comparerEquals = comparerDefinition.Methods.Single(m => m.Name == "Equals");
        Check("Equality comparer closes both inputs", ComparerBindings.Bind(Reference(comparerEquals, closedComparer), comparerEquals)?.Arguments.SequenceEqual(new[] { "System.Collections.EqualityComparer<String>", "String", "String" }) == true);
        var wrongComparer = Reference(comparerEquals, closedComparer);
        wrongComparer.ReturnType = module.TypeSystem.Int32;
        Reject("Comparer rejects wrong result", () => ComparerBindings.Bind(wrongComparer, comparerEquals));
        comparerDefinition.GenericParameters[0].Attributes = GenericParameterAttributes.Contravariant;
        Reject("Comparer variance is not inferred", () => ComparerBindings.Validate(module));
        comparerDefinition.GenericParameters[0].Attributes = GenericParameterAttributes.NonVariant;
        var stringComparer = module.GetType("System.StringComparer");
        var ordinalFactory = stringComparer.Methods.Single(m => m.Name == "get_Ordinal");
        Check("Ordinal policy reference binds", ComparerBindings.Bind(Reference(ordinalFactory, stringComparer), ordinalFactory)?.Result == "System.StringComparer");
        var ignoreCaseFactory = stringComparer.Methods.Single(m => m.Name == "get_OrdinalIgnoreCase");
        Check("Ignore-case policy reference binds", ComparerBindings.Bind(Reference(ignoreCaseFactory, stringComparer), ignoreCaseFactory)?.Result == "System.StringComparer");
        EnumBindings.Validate(module, EnumBindings.StringComparison);
        var stringType = module.GetType("System.String");
        var compare = stringType.Methods.Single(m => m.Name == "Compare");
        Check("Explicit string comparison binds nominal enum", StringBindings.Bind(Reference(compare, stringType), compare, false)?.Arguments.Last() == EnumBindings.StringComparison);
        var wrongMode = Reference(compare, stringType);
        wrongMode.Parameters[2].ParameterType = module.TypeSystem.Int32;
        Reject("String comparison rejects raw Int32 metadata", () => StringBindings.Bind(wrongMode, compare, false));
        var shortcut = stringType.Methods.Single(m => m.Name == "CompareOrdinalIgnoreCase");
        Check("Ignore-case shortcut binds", StringBindings.Bind(Reference(shortcut, stringType), shortcut, false)?.Result == "Int32");
        var mode = module.GetType(EnumBindings.StringComparison);
        mode.Fields.Single(f => f.Name == "OrdinalIgnoreCase").Constant = 5;
        Reject("String comparison rejects changed enum literals", () => EnumBindings.Validate(module, EnumBindings.StringComparison));
        mode.Fields.Single(f => f.Name == "OrdinalIgnoreCase").Constant = 1;
        var mapClass = module.GetType("System.Collections.HashMap`2");
        var mapWithPolicy = new GenericInstanceType(mapClass);
        mapWithPolicy.GenericArguments.Add(module.TypeSystem.String);
        mapWithPolicy.GenericArguments.Add(module.TypeSystem.Int32);
        var policyConstructor = mapClass.Methods.Single(m => m.IsConstructor && m.Parameters.Count == 1);
        Check("HashMap accepts equality policy", MapBindings.Construct(Reference(policyConstructor, mapWithPolicy), policyConstructor)?.Arguments.Single() == "System.Collections.EqualityComparer<String>");
        var wrongPolicy = Reference(policyConstructor, mapWithPolicy);
        wrongPolicy.Parameters[0].ParameterType = module.TypeSystem.String;
        Reject("HashMap rejects wrong policy type", () => MapBindings.Construct(wrongPolicy, policyConstructor));
        File.WriteAllText(Path.Combine(output, "comparer-signatures.json"), JsonSerializer.Serialize(checks, new JsonSerializerOptions { WriteIndented = true }));
        Console.WriteLine($"{checks.Count} comparer signature checks passed");
    }
}
