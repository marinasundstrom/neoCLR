using System.Reflection;
using System.Text.Json;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;

internal static class EntryPointChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        foreach (var global in new[] { true, false })
        {
            var graph = new AssemblyBuilder(new("NoResultEntry" + global, new Version(1, 0, 0, 0)), core);
            var entry = global ? graph.AddFunction("Main", returnsValue: false) : graph.AddType("Example", "Program").AddMethod("Main", returnsValue: false);
            var helper = graph.AddFunction("Helper", returnsValue: false);
            helper.Return(); entry.Call(helper); entry.Return(); graph.EntryPoint = entry;
            var pe = graph.Write();
            var read = AssemblyDefinition.ReadAssembly(pe, false);
            Check(read.EntryPoint is { } method && method.TryGetStaticInt32Signature(out var count, out var returns) && count == 0 && !returns, "CLI no-result entry contract");
            Check(Assembly.Load(pe).EntryPoint!.Invoke(null, null) is null, "CLI no-result entry invocation");
            var native = graph.WriteNativeAssembly();
            NativeAssemblyDefinition.ReadAssembly(native);
            var container = RuntimeAssemblyContainer.WriteBinary(native, core);
            Check(RuntimeAssemblyContainer.Read(container).SequenceEqual(native), "native entry container roundtrip");
            using var json = JsonDocument.Parse(native);
            var definition = json.RootElement.GetProperty("functions").EnumerateArray().Single(f => f.GetProperty("name").GetString() == json.RootElement.GetProperty("entry").GetString());
            Check(definition.GetProperty("no_result").GetBoolean() && definition.GetProperty("returns").GetString() == "Void", "native no-result entry signature");
            entry.ClearBody(); entry.LoadConstant(1); entry.Return();
            Reject(() => graph.WriteNativeAssembly());
            entry.ClearBody(); entry.Return();
            var parameterized = graph.AddFunction("WithArgument", 1, false); parameterized.Return(); graph.EntryPoint = parameterized;
            Reject(() => graph.Write()); Reject(() => graph.WriteNativeAssembly());
            var foreign = new AssemblyBuilder(new("Foreign", new Version(1, 0, 0, 0)), core).AddFunction("Main", returnsValue: false);
            foreign.Return(); graph.EntryPoint = foreign;
            Reject(() => graph.WriteNativeAssembly());
        }
    }
    private static void Reject(Action action)
    {
        try { action(); } catch (InvalidDataException) { return; }
        throw new Exception("invalid entry point accepted");
    }
    private static void Check(bool condition, string message) { if (!condition) throw new Exception(message); }
}
