using System.Diagnostics;
using System.Security.Cryptography;
using System.Text.Json;
using NeoCLR.Metadata.Experimental.Model;

internal static class RuntimeIntegration
{
    internal static async Task Run(string runtime, string bridge, string core, string system, string output)
    {
        runtime = Path.GetFullPath(runtime); bridge = Path.GetFullPath(bridge);
        core = Path.GetFullPath(core); system = Path.GetFullPath(system); output = Path.GetFullPath(output);
        if (Directory.Exists(output)) throw new IOException("runtime-test output must be a fresh directory");
        var identity = AssemblyDefinition.ReadAssembly(File.ReadAllBytes(core), false).Identity;
        WriterChecks.Emit(output, identity);
        var imported = Path.Combine(output, "imported");
        await Command("dotnet", 0, bridge, "--import", Path.Combine(output, "GeneratedApp.dll"), core, imported, Path.Combine(output, "GeneratedLibrary.dll"));
        var native = Path.Combine(output, "GeneratedApp.neo.json");
        await Command(runtime, 0, "assemble", Path.Combine(imported, "App.neoil"), native, "--system", system);
        await Command(runtime, 0, "verify", native, "--system", system);
        var executed = await Command(runtime, 42, "run", native, "--system", system, "--show-result");
        if (!executed.Contains("=> Int32(42)", StringComparison.Ordinal)) throw new Exception("runtime did not report expected value");
        var report = new
        {
            date = "2026-09-30", scope = "API PE emission -> existing CLI bridge -> serialized native assembly -> neoCLR verify/run",
            expectedExitCode = 42, actualExitCode = 42, output = executed.Trim(),
            directNeoStreamLoading = false,
            runtimeSha256 = Hash(runtime), bridgeSha256 = Hash(bridge), coreSha256 = Hash(core), systemSha256 = Hash(system),
            libraryApiSha256 = Hash(typeof(AssemblyDefinition).Assembly.Location),
            appPeSha256 = Hash(Path.Combine(output, "GeneratedApp.dll")), libraryPeSha256 = Hash(Path.Combine(output, "GeneratedLibrary.dll")),
            nativeAssemblySha256 = Hash(native)
        };
        File.WriteAllText(Path.Combine(output, "runtime-validation.json"), JsonSerializer.Serialize(report, new JsonSerializerOptions { WriteIndented = true }) + "\n");
        Console.WriteLine("PASS API-produced PE application and dependency imported, assembled, loaded, verified and executed by neoCLR: 42");
    }
    private static string Hash(string path) => Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(path))).ToLowerInvariant();
    private static async Task<string> Command(string file, int expected, params string[] arguments)
    {
        var start = new ProcessStartInfo(file) { RedirectStandardOutput = true, RedirectStandardError = true, UseShellExecute = false };
        foreach (var argument in arguments) start.ArgumentList.Add(argument);
        using var process = Process.Start(start) ?? throw new IOException("failed to start " + file);
        var stdout = process.StandardOutput.ReadToEndAsync(); var stderr = process.StandardError.ReadToEndAsync();
        using var timeout = new CancellationTokenSource(TimeSpan.FromMinutes(5));
        try { await process.WaitForExitAsync(timeout.Token); }
        catch (OperationCanceledException) { process.Kill(entireProcessTree: true); throw new TimeoutException("process timed out: " + file); }
        string result = await stdout, errors = await stderr;
        if (process.ExitCode != expected) throw new Exception($"{file} returned {process.ExitCode}, expected {expected}: {errors}\n{result}");
        Console.WriteLine("PASS " + Path.GetFileName(file) + " " + arguments[0] + " (exit " + process.ExitCode + ")");
        return result + errors;
    }
}
