using System.Reflection;
using System.Text;
using System.Text.Json.Nodes;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;

internal static class NamespaceBindingChecks
{
    internal static void Run()
    {
        var snapshot = AssemblyDefinition.ReadAssembly(File.ReadAllBytes(typeof(NamespaceBindingChecks).Assembly.Location), expectedExtended: false);
        var core = snapshot.Identity;
        var library = new AssemblyBuilder(new("NamespaceRuntime", new Version(1, 0, 0, 0)), core);
        var echo = library.AddFunction("Example", "Echo", new MethodSignature(PrimitiveType.Int32, [PrimitiveType.Int32]));
        echo.LoadArgument(0); echo.Return();
        var native = JsonNode.Parse(library.WriteNativeAssembly())!;
        native["functions"]![0]!["name"] = "Example.Echo";
        var runtime = NativeLibraryDefinition.ReadAssembly(NativeModuleContainer.WriteBinary(Encoding.UTF8.GetBytes(native.ToJsonString())));
        AssemblyBuilder App(AssemblyIdentity selectedCore)
        {
            var app = new AssemblyBuilder(new("NamespaceCaller", new Version(1, 0, 0, 0)), selectedCore);
            app.BindNativeLibrary(snapshot, runtime, selectedCore); return app;
        }
        var marked = snapshot.MainModule.Types.Single(t => t.Name == "ArbitraryContainer").Methods.Single(m => m.Name == "Echo");
        var app = App(core);
        var imported = app.ImportReference(marked, core);
        var main = app.AddFunction("Main"); app.EntryPoint = main;
        main.LoadConstant(42); main.Call(imported); main.Return();
        var result = JsonNode.Parse(app.WriteNativeAssembly())!;
        var call = result["functions"]![0]!["body"]!.AsArray().Single(i => i!["op"]!.GetValue<string>() == "call")!["arg"]!;
        if (call["name"]!.GetValue<string>() != "Example.Echo" || call["owner"] is not null) throw new Exception("namespace function owner not removed");
        var cli = AssemblyDefinition.ReadAssembly(app.Write(), expectedExtended: false);
        if (!cli.MainModule.TypeReferences.Any(t => t.Name == "ArbitraryContainer")) throw new Exception("CLI container identity lost");
        void Reject(Action action) { try { action(); } catch (InvalidDataException) { return; } throw new Exception("invalid namespace binding admitted"); }
        var unmarked = snapshot.MainModule.Types.Single(t => t.Name == "UnmarkedContainer").Methods.Single(m => m.Name == "Echo");
        Reject(() => App(core).ImportReference(unmarked, core));
        var wrongCore = new AssemblyIdentity("UnrelatedCore", new Version(1, 0, 0, 0));
        Reject(() => App(wrongCore).ImportReference(marked, wrongCore));
        native["functions"]![0]!["returns"] = "Boolean";
        var mismatch = new AssemblyBuilder(new("Mismatch", new Version(1, 0, 0, 0)), core);
        mismatch.BindNativeLibrary(snapshot, NativeLibraryDefinition.ReadAssembly(NativeModuleContainer.WriteBinary(Encoding.UTF8.GetBytes(native.ToJsonString()))), core);
        Reject(() => mismatch.ImportReference(marked, core));
    }
}

namespace System.Runtime.CompilerServices
{
    [AttributeUsage(AttributeTargets.Class)]
    public sealed class TopLevelAttribute : Attribute { }
}
namespace Example
{
    [System.Runtime.CompilerServices.TopLevel]
    public static class ArbitraryContainer { public static int Echo(int value) => value; }
    public static class UnmarkedContainer { public static int Echo(int value) => value; }
}
