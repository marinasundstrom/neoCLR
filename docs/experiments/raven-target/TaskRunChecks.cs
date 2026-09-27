using Mono.Cecil;
using System.Text.Json;

static class TaskRunChecks
{
    public static void Write(string output)
    {
        Directory.CreateDirectory(output);
        var path = Path.Combine(output, "NeoCLR.CoreProbe.dll");
        CoreDeclarations.Write(path, unionProbe: true, collectionProbe: true);
        using var core = AssemblyDefinition.ReadAssembly(path);
        var module = core.MainModule;
        var owner = module.GetType("System.Tasks.Task");
        var checks = new List<string>();
        void Check(string name, bool valid) {
            if (!valid) throw new Exception("Task.Run check failed: " + name);
            checks.Add(name);
        }
        void Reject(string name, Action action) {
            try { action(); } catch (InvalidDataException) { checks.Add(name); return; }
            throw new Exception("Task.Run check did not reject: " + name);
        }
        Check("Three public overloads", owner.Methods.Count(m => m.IsPublic && m.Name == "Run") == 3);
        foreach (var method in owner.Methods.Where(m => m.Name == "Run")) {
            MethodReference reference = method;
            if (method.HasGenericParameters) {
                var closed = new GenericInstanceMethod(method);
                closed.GenericArguments.Add(module.TypeSystem.Int32);
                reference = closed;
            }
            var result = method.HasGenericParameters ? "Int32" : "Void";
            Check("Bind " + method.FullName, TaskBindings.Bind(reference, method, false, false)?.Result == $"System.Tasks.Task<{result}>");
            var attributes = method.Attributes;
            method.IsPublic = false;
            method.IsPrivate = true;
            Reject("Reject private " + method.FullName, () => TaskBindings.Bind(reference, method, false, false));
            method.Attributes = attributes;
            Reject("Reject construction " + method.FullName, () => TaskBindings.Bind(reference, method, true, false));
        }
        var sync = owner.Methods.Single(m => !m.HasGenericParameters && m.Name == "Run");
        var old = sync.ReturnType;
        sync.ReturnType = module.TypeSystem.Int32;
        Reject("Reject changed result", () => TaskBindings.Bind(sync, sync, false, false));
        sync.ReturnType = old;
        File.WriteAllText(Path.Combine(output, "task-run-signatures.json"), JsonSerializer.Serialize(checks, new JsonSerializerOptions { WriteIndented = true }));
        Console.WriteLine($"{checks.Count} Task.Run signature checks passed");
    }
}
