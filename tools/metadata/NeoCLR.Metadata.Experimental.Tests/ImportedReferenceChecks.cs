using System.Text.Json;
using NeoCLR.Metadata.Experimental.Model;

internal static class ImportedReferenceChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var library = new AssemblyBuilder(new("Library", new Version(1, 0, 0, 0)), core);
        var twice = library.AddType("Example", "Math").AddMethod("Twice", 1);
        twice.LoadArgument(0); twice.LoadConstant(2); twice.Multiply(); twice.Return();
        var notify = library.AddFunction("Notify", returnsValue: false); notify.Return();
        var image = library.Write();
        var snapshot = AssemblyDefinition.ReadAssembly(image, false);
        var definition = snapshot.MainModule.Types[1].Methods.Single();
        var app = new AssemblyBuilder(new("Consumer", new Version(1, 0, 0, 0)), core);
        var imported = app.ImportReference(definition, core);
        Check(ReferenceEquals(imported.Owner, app) && imported.AssemblyIdentity.Equals(library.Identity) &&
            imported.Namespace == "Example" && imported.DeclaringTypeName == "Math" && imported.Name == "Twice" &&
            imported.ParameterCount == 1 && imported.ReturnsValue, "imported contract");
        Check(ReferenceEquals(imported, app.ImportReference(AssemblyDefinition.ReadAssembly(image, false).MainModule.Types[1].Methods[0], core)), "same snapshot import is interned");
        Array.Clear(image); twice.ClearBody(); // Emission must not inspect the original graph or byte buffer.
        var main = app.AddFunction("Main"); main.LoadConstant(21); main.Call(imported); main.Return(); app.EntryPoint = main;
        var consumer = AssemblyDefinition.ReadAssembly(app.Write(), false);
        Check(ReferenceEquals(consumer.MainModule.MemberReferences.Single().ResolveMethod(new Resolver(snapshot)), definition), "PE import resolves to exact definition");
        using var native = JsonDocument.Parse(app.WriteNativeAssembly());
        Check(native.RootElement.GetProperty("references").GetArrayLength() == 1, "native dependency imported without producer graph");
        var global = app.ImportReference(snapshot.MainModule.Functions.Single(), core);
        Check(global.Namespace is null && global.DeclaringTypeName is null && !global.ReturnsValue, "global no-result contract");
        main.ClearBody(); main.Call(global); main.LoadConstant(21); main.Call(imported); main.Return();
        using var nativeGlobal = JsonDocument.Parse(app.WriteNativeAssembly());
        Check(nativeGlobal.RootElement.GetProperty("functions")[0].GetProperty("body")[0].GetProperty("arg").GetProperty("owner").ValueKind == JsonValueKind.Null, "native imported global owner absent");
        Reject<InvalidDataException>(() => app.Write(), "global PE import");
        var other = new AssemblyBuilder(new("Other", new Version(1, 0, 0, 0)), core);
        Reject<ArgumentException>(() => other.AddFunction("Main").Call(imported), "wrong consuming owner");
        Reject<InvalidDataException>(() => app.ImportReference(definition, new("OtherCore", new Version(1, 0, 0, 0))), "core mismatch");
        Reject<InvalidDataException>(() => library.ImportReference(definition, core), "output identity collision");
        var replacement = new AssemblyBuilder(library.Identity, core);
        var replacementMethod = replacement.AddFunction("Replacement"); replacementMethod.LoadConstant(0); replacementMethod.Return();
        Reject<InvalidDataException>(() => app.ImportReference(AssemblyDefinition.ReadAssembly(replacement.Write(), false).MainModule.Functions.Single(), core), "conflicting MVID");
        Reject<ArgumentNullException>(() => app.ImportReference((MethodDefinition)null!, core), "null definition");
        Reject<ArgumentNullException>(() => app.ImportReference(definition, null!), "null core");
        Reject<ArgumentNullException>(() => main.Call((ImportedMethodReference)null!), "null reference");
        main.ClearBody(); main.Call(imported); main.Return();
        Reject<InvalidDataException>(() => app.WriteNativeAssembly(), "imported call stack underflow");
    }
    private sealed class Resolver(AssemblyDefinition assembly) : IAssemblyResolver
    {
        public AssemblyDefinition? Resolve(AssemblyIdentity identity) => identity.Equals(assembly.Identity) ? assembly : null;
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
    private static void Reject<T>(Action action, string message) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected rejection: " + message);
    }
}
