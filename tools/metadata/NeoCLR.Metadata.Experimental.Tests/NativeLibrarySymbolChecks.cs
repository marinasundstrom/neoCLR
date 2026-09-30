using System.Text;
using System.Text.Json;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class NativeLibrarySymbolChecks
{
    internal static void Run()
    {
        const string source = """
            {"format":5,"name":"System","entry":"","types":[{"name":"System.Date","fields":[]}],"functions":[
            {"name":"System.Date.Count","owner":{"Named":"System.Date"},"parameters":["Int32"],"returns":"Int32","body":[]},
            {"name":"System.Date.Result","owner":{"Named":"System.Date"},"parameters":["Int32"],"returns":{"Named":"System.Result"},"body":[]},
            {"name":"System.Date.Instance","owner":{"Named":"System.Date"},"instance":true,"parameters":[],"returns":"Int32","body":[]}]}
            """;
        NativeLibraryDefinition Read(string text) => NativeLibraryDefinition.ReadAssembly(NativeModuleContainer.WriteLibraryBinary(Encoding.UTF8.GetBytes(text)));
        var genericNames = Read(source.Replace("\"types\":[", "\"types\":[{\"name\":\"System.Date\",\"generic_parameters\":[\"T\"]},"));
        Check(genericNames.TypeNames.Count == 2, "native generic arities share descriptive names");
        Reject(() => Read(source.Replace("\"types\":[", "\"types\":[{\"name\":\"System.Date\"},")));
        var library = Read(source);
        Check(library.ModuleName == "System" && library.TypeNames.Single() == "System.Date" && library.Functions.Count == 3, "inventory");
        var method = library.Functions[0];
        Check(method.TableIndex == 0 && method.TryGetStaticInt32Signature(out var count) && count == 1, "signature");
        Check(!library.Functions[1].TryGetStaticInt32Signature(out _) && !library.Functions[2].TryGetStaticInt32Signature(out _), "unsupported signatures retained");
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, host.CultureName ?? "", Convert.ToHexString(host.GetPublicKeyToken() ?? []));
        var identity = new AssemblyIdentity("NativeStaticView", new Version(1, 0, 0, 0));
        var projection = library.CreateStaticInt32ReferenceAssembly(identity, core, [method]);
        var snapshot = AssemblyDefinition.ReadAssembly(projection, false);
        var projected = snapshot.MainModule.Types.Single(t => t.Name == "Date").Methods.Single();
        Check(projected.Name == "Count" && projected.TryGetStaticInt32Signature(out count, out var result) && count == 1 && result, "projected signature");
        Reject(() => library.CreateStaticInt32ReferenceAssembly(identity, core, []));
        Reject(() => library.CreateStaticInt32ReferenceAssembly(identity, core, [method, method]));
        Reject(() => library.CreateStaticInt32ReferenceAssembly(identity, core, [Read(source).Functions[0]]));
        Reject(() => library.CreateStaticInt32ReferenceAssembly(identity, core, [library.Functions[1]]));
        Reject(() => library.CreateStaticInt32ReferenceAssembly(core, core, [method]));
        Check(!Read(source.Replace("\"body\":[]", "\"visibility\":\"private\",\"body\":[]")).Functions[0].TryGetStaticInt32Signature(out _), "native private visibility");
        var hiddenOwner = Read(source.Replace("\"fields\":[]", "\"visibility\":\"internal\",\"fields\":[]"));
        Reject(() => hiddenOwner.CreateStaticInt32ReferenceAssembly(identity, core, [hiddenOwner.Functions[0]]));
        var privateMethod = Read(source.Replace("\"body\":[]", "\"origin\":{\"member_access\":\"Private\"},\"body\":[]")).Functions[0];
        Check(!privateMethod.TryGetStaticInt32Signature(out _), "private callable");
        var malformed = Read(source.Replace("\"body\":[]", "\"origin\":true,\"body\":[]")).Functions[0];
        Check(!malformed.TryGetStaticInt32Signature(out _), "malformed optional contract");
        var graph = new AssemblyBuilder(new("Consumer", new Version(1, 0, 0, 0)), core);
        var main = graph.AddFunction("Main"); graph.EntryPoint = main;
        main.LoadConstant(42); main.Call(method); main.Return();
        using var native = JsonDocument.Parse(graph.WriteNativeAssembly());
        var call = native.RootElement.GetProperty("functions")[0].GetProperty("body")[1].GetProperty("arg");
        Check(call.GetProperty("name").GetString() == method.Name && call.GetProperty("owner").GetProperty("Named").GetString() == method.DeclaringTypeName, "original native call identity");
        Reject(() => graph.Write());
        Reject(() => main.Call(Read(source.Replace("\"name\":\"System\"", "\"name\":\"Other\"")).Functions[0]));
        main.ClearBody(); main.Call(method); main.Return();
        Reject(() => graph.WriteNativeAssembly());
    }
    private static void Reject(Action action)
    {
        try { action(); } catch (InvalidDataException) { return; }
        throw new Exception("invalid native library operation was accepted");
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
}
