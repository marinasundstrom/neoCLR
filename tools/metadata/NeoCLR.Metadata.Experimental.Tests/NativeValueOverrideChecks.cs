using System.Diagnostics;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class NativeValueOverrideChecks
{
    internal static async Task RunRuntime(string runtime, string systemPath, string referencePath, string directory)
    {
        Directory.CreateDirectory(directory);
        var reference = AssemblyDefinition.ReadAssembly(File.ReadAllBytes(referencePath), expectedExtended: false);
        var systemImage = File.ReadAllBytes(systemPath);
        var system = NativeLibraryDefinition.ReadAssembly(systemImage);
        var core = reference.Identity;
        AssemblyBuilder Create(NativeLibraryDefinition dependency)
        {
            var graph = new AssemblyBuilder(new("NativeValueOverride", new Version(1, 0, 0, 0)), core);
            graph.BindNativeLibrary(reference, dependency, core);
            foreach (var owner in new[] { graph.AddValueType("Example", "Display"), graph.AddGenericValueType("Example", "GenericDisplay", ["T"]) })
            {
                var body = owner.AddOverride("ToString", new(PrimitiveType.String, [])).GetILGenerator();
                body.Emit(OpCode.Ldstr, "native override"); body.Return();
            }
            return graph;
        }
        var library = Create(system);
        var json = library.WriteNativeAssembly();
        var binary = RuntimeAssemblyContainer.WriteBinary(json, core);
        var snapshot = AssemblyDefinition.ReadNativeAssembly(binary);
        var type = snapshot.MainModule.Types.Single(t => t.Name == "Display");
        foreach (var definition in snapshot.MainModule.Types.Where(t => t.Namespace == "Example").SelectMany(t => t.Methods))
            Check((definition.Attributes & 0x540) == 0x40, "native roundtrip lost override flags");
        var projected = RuntimeAssemblyContainer.ReadCliProjection(binary);
        Check((projected.MainModule.Types.Single(t => t.Name == "Display").Methods.Single().Attributes & 0x540) == 0x40, "projection lost override slot");
        var app = new AssemblyBuilder(new("NativeOverrideConsumer", new Version(1, 0, 0, 0)), core);
        var imported = app.ImportReference(type, core);
        var method = app.ImportReference(type.Methods.Single(), core);
        var main = app.AddFunction("Main"); app.EntryPoint = main;
        var il = main.GetILGenerator();
        var value = il.DeclareLocal(imported);
        il.LoadLocalAddress(value); il.InitializeObject(imported);
        il.LoadLocalAddress(value); il.Call(method); il.WriteConsoleLine(); il.LoadConstant(42); il.Return();
        var libraryPath = Path.Combine(directory, "Library.dll");
        var appPath = Path.Combine(directory, "Consumer.dll");
        File.WriteAllBytes(libraryPath, binary);
        File.WriteAllBytes(appPath, RuntimeAssemblyContainer.WriteBinary(app.WriteNativeAssembly(), core));
        await Execute("verify", appPath, true, libraryPath);
        await Execute("run", appPath, true, libraryPath);
        await Execute("verify", appPath, false, libraryPath, includeSystem: false);

        // Boxing is not yet in the metadata generator profile. This runtime harness
        // consumes the unmodified API-produced library and exercises its actual slot.
        var names = JsonNode.Parse(json)!["types"]!.AsArray().Select(t => t!["name"]!.GetValue<string>()).ToArray();
        var harness = new StringBuilder(".module OverrideHarness\n.entry Main\n.function Main() -> Int32\n");
        // Use fresh default values; no duplicate class declarations or replacement bodies.
        foreach (var name in new[] { names[0], names[1] + "<Int32>" })
            harness.Append($"newobj {name}\nbox {name}\ncallvirt instance System.Object::ToString()\nldstr \"native override\"\ncall neoCLR.Runtime.StringCompareOrdinal(String,String)\nbrtrue failed\n");
        harness.Append("ldc.i4 42\nret\nfailed:\nldc.i4 1\nret\n.end\n");
        var harnessPath = Path.Combine(directory, "Boxed.neoil");
        File.WriteAllText(harnessPath, harness.ToString());
        await Execute("verify", harnessPath, true, libraryPath);
        await Execute("run", harnessPath, true, libraryPath, expectedOutput: "");

        var unrelated = new AssemblyBuilder(new("UnrelatedBootstrap", new Version(1, 0, 0, 0)), core);
        var unrelatedSnapshot = AssemblyDefinition.ReadAssembly(unrelated.Write(), expectedExtended: false);
        var ambiguous = Create(system);
        ambiguous.BindNativeLibrary(unrelatedSnapshot, system, core);
        Reject(() => ambiguous.WriteNativeAssembly());
        var wrongReference = new AssemblyBuilder(new("WrongBootstrap", new Version(1, 0, 0, 0)), core);
        wrongReference.BindNativeLibrary(unrelatedSnapshot, system, core);
        var wrongBody = wrongReference.AddValueType("Example", "Value").AddOverride("ToString", new(PrimitiveType.String, [])).GetILGenerator();
        wrongBody.Emit(OpCode.Ldstr, "wrong"); wrongBody.Return();
        Reject(() => wrongReference.WriteNativeAssembly());

        foreach (var corruption in new[] { "result", "virtual", "receiver", "generic", "missing", "duplicate" })
        {
            var changed = JsonNode.Parse(NativeModuleContainer.Read(systemImage))!;
            var slot = changed["functions"]!.AsArray().Single(f => f!["name"]!.GetValue<string>() == "System.Object.ToString")!;
            if (corruption == "missing") slot["name"] = "System.Object.Unrelated";
            if (corruption == "duplicate") changed["functions"]!.AsArray().Add(slot.DeepClone());
            if (corruption == "result") slot["returns"] = "Int32";
            if (corruption == "virtual") slot["is_virtual"] = false;
            if (corruption == "receiver") slot["receiver_byref"] = true;
            if (corruption == "generic") slot["generic_parameters"] = new JsonArray("T");
            Reject(() => Create(NativeLibraryDefinition.ReadAssembly(NativeModuleContainer.WriteLibraryBinary(Encoding.UTF8.GetBytes(changed.ToJsonString())))).WriteNativeAssembly());
        }
        foreach (var corruption in new[] { "virtual", "override", "name", "binding" })
        {
            var changed = JsonNode.Parse(json)!;
            var slot = changed["functions"]![0]!;
            if (corruption == "virtual") slot.AsObject().Remove("is_virtual");
            if (corruption == "override") slot["is_override"] = false;
            if (corruption == "name") slot["name"] = "NotTheSlot";
            if (corruption == "binding") changed["assemblies"]![0]!["native_module_bindings"] = new JsonArray();
            Reject(() => NativeAssemblyDefinition.ReadAssembly(Encoding.UTF8.GetBytes(changed.ToJsonString())));
        }
        Console.WriteLine("Native override roundtrips, imported direct call and boxed Object dispatch passed (42).");

        async Task Execute(string command, string input, bool success, string dependency, bool includeSystem = true, string expectedOutput = "native override")
        {
            var start = new ProcessStartInfo(runtime) { RedirectStandardOutput = true, RedirectStandardError = true };
            foreach (var argument in new[] { command, input, "--module", dependency }) start.ArgumentList.Add(argument);
            if (includeSystem) { start.ArgumentList.Add("--system"); start.ArgumentList.Add(systemPath); }
            using var process = Process.Start(start)!;
            var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
            await process.WaitForExitAsync(); var output = await stdout; var error = await stderr;
            if (success ? process.ExitCode != (command == "run" ? 42 : 0) || command == "run" && output.Trim() != expectedOutput : process.ExitCode == 0)
                throw new Exception(command + " " + input + ": " + process.ExitCode + " " + output + error);
        }
    }
    private static void Reject(Action action)
    {
        try { action(); } catch (InvalidDataException) { return; }
        throw new Exception("incompatible override accepted");
    }
    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
}
