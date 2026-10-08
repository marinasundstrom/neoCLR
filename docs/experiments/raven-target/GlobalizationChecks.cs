using Mono.Cecil;
using System.Text.Json;

static class GlobalizationChecks
{
    public static void Write(string output)
    {
        Directory.CreateDirectory(output);
        var path = Path.Combine(output, "NeoCLR.CoreProbe.dll");
        CoreDeclarations.Write(path, unionProbe: true, collectionProbe: true);
        using var core = AssemblyDefinition.ReadAssembly(path);
        var checks = new List<string>();
        void Check(string name, bool valid) {
            if (!valid) throw new Exception("Globalization check failed: " + name);
            checks.Add(name);
        }
        void Reject(string name, Action action) {
            try { action(); } catch (InvalidDataException) { checks.Add(name); return; }
            throw new Exception("Globalization check did not reject: " + name);
        }
        MethodReference Reference(MethodDefinition method) {
            var reference = new MethodReference(method.Name, method.ReturnType, method.DeclaringType) { HasThis = method.HasThis };
            foreach (var parameter in method.Parameters)
                reference.Parameters.Add(new ParameterDefinition(parameter.Name, parameter.Attributes, parameter.ParameterType));
            return reference;
        }
        var module = core.MainModule;
        ErrorBindings.Reset(module);
        foreach (var type in module.Types.Where(t => GlobalizationBindings.IsName(t.FullName))) {
            Check(type.Name + " has no setters", type.Properties.All(p => p.SetMethod is null));
            foreach (var method in type.Methods.Where(m => m.IsPublic)) {
                var bound = method.IsConstructor
                    ? GlobalizationBindings.Construct(Reference(method), method) is not null
                    : GlobalizationBindings.Bind(Reference(method), method) is not null;
                Check(method.FullName, bound);
            }
        }
        var calendar = module.GetType(GlobalizationBindings.Calendar);
        var project = calendar.Methods.Single(m => m.Name == "GetYear");
        var wrong = Reference(project);
        wrong.ReturnType = module.TypeSystem.String;
        Reject("Calendar rejects wrong projection result", () => GlobalizationBindings.Bind(wrong, project));
        var factory = calendar.Methods.Single(m => m.Name == "get_Hebrew");
        var instanceFactory = Reference(factory);
        instanceFactory.HasThis = true;
        Reject("Calendar rejects instance factory", () => GlobalizationBindings.Bind(instanceFactory, factory));
        calendar.IsSealed = false;
        Reject("Calendar rejects extensible contract", () => GlobalizationBindings.Bind(Reference(project), project));
        calendar.IsSealed = true;
        foreach (var name in new[] { "Date", "TimeOfDay", "Instant", "LocalDateTime", "TimeOffset" }) {
            var type = module.GetType("System.Time." + name);
            foreach (var method in type.Methods.Where(m => m.IsPublic && !m.IsConstructor))
                Check(name + " " + method.FullName, CalendarBindings.Bind(Reference(method), method) is not null);
        }
        var zone = module.GetType(GlobalizationBindings.Zone);
        var mapping = zone.Methods.Single(m => m.Name == "MapLocal");
        var wrongMapping = Reference(mapping);
        wrongMapping.ReturnType = module.TypeSystem.String;
        Reject("TimeZone rejects wrong mapping result", () => GlobalizationBindings.Bind(wrongMapping, mapping));
        var offsetFactory = module.GetType("System.Time.TimeOffset").Methods.Single(m => m.Name == "FromSeconds");
        var badFactory = Reference(offsetFactory);
        badFactory.HasThis = true;
        Reject("TimeOffset rejects instance factory", () => CalendarBindings.Bind(badFactory, offsetFactory));
        var dateTime = module.GetType("System.Time.DateTime");
        Check("DateTime is the exact parenthesized union", RavenUnionMetadata.IsDateTimeUnion(dateTime));
        foreach (var extractor in dateTime.Methods.Where(m => m.Name == "TryGetValue"))
            Check("DateTime conditional extraction " + extractor.FullName, ApplicationTypes.IsConditionalUnionOutput(extractor));
        var variantConstructor = dateTime.Methods.First(m => m.IsConstructor && m.IsPublic);
        var originalVariant = variantConstructor.Parameters[0].ParameterType;
        variantConstructor.Parameters[0].ParameterType = module.GetType("System.Time.TimeOfDay");
        Check("DateTime rejects a foreign variant", !RavenUnionMetadata.IsDateTimeUnion(dateTime));
        variantConstructor.Parameters[0].ParameterType = originalVariant;
        Check("Implementation policies are not public reference types", module.Types.All(t => t.Name is not ("CalendarRules" or "GregorianCalendar" or "HebrewCalendar" or "HebrewDateTimeFormat")));
        File.WriteAllText(Path.Combine(output, "checks.json"), JsonSerializer.Serialize(checks, new JsonSerializerOptions { WriteIndented = true }));
        Console.WriteLine($"Passed {checks.Count} calendar/globalization signature checks");
    }
}
