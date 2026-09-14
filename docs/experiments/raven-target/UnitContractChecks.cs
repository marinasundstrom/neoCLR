using Mono.Cecil;
using System.Text.Json;

static class UnitContractChecks
{
    public static void Verify(string path)
    {
        using var image = AssemblyDefinition.ReadAssembly(path);
        var module = image.MainModule;
        if (module.GetTypes().Any(type => type.FullName == "System.Unit")
            || module.GetTypeReferences().Any(type => type.FullName == "System.Unit"))
            throw new InvalidDataException("The neoCLR runtime contract must not emit System.Unit.");
        var values = module.GetTypeReferences().Where(type => type.FullName == "System.Void").ToArray();
        if (values.Length == 0 || values.Any(type => !RuntimeSignatures.IsCore(type.Scope)))
            throw new InvalidDataException("Void values must refer to the target core type.");
        var methods = module.GetTypes().SelectMany(type => type.Methods).ToArray();
        var notify = methods.Single(method => method.Name == "Notify");
        var take = methods.Single(method => method.Name == "Take");
        if (notify.ReturnType.MetadataType != MetadataType.Void || take.Parameters.Single().ParameterType.FullName != "System.Void")
            throw new InvalidDataException("No-result return and Void parameter contracts differ from the probe.");
        Console.WriteLine(JsonSerializer.Serialize(new { UnitDefinitions = 0, UnitReferences = 0,
            TargetVoidReferences = values.Length, NoResultReturn = true, VoidParameter = true }));
    }
}
