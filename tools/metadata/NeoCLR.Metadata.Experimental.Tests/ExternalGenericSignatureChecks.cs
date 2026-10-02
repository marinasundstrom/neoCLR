using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ExternalGenericSignatureChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var library = new AssemblyBuilder(new("ExternalGenericLibrary", new Version(1, 0, 0, 0)), core);
        var box = library.AddGenericClass("Example", "Box", ["T"]);
        var parameter = SignatureType.TypeParameter(0);
        var field = box.AddField("Value", parameter);
        var ctor = box.AddConstructor(new MethodSignature(PrimitiveType.Void, [parameter]));
        ctor.LoadArgument(0); ctor.LoadArgument(1); ctor.StoreField(field); ctor.Return();
        var get = box.AddInstanceMethod("Get", new MethodSignature(parameter, []));
        get.LoadArgument(0); get.LoadField(field); get.Return();
        foreach (var binary in new[] { false, true })
        {
            byte[] Image(AssemblyBuilder builder) => binary ? RuntimeAssemblyContainer.WriteBinary(builder.WriteNativeAssembly(), core) : RuntimeAssemblyContainer.Write(builder.WriteNativeAssembly(), core);
            var dependency = AssemblyDefinition.ReadNativeAssembly(Image(library));
            var definition = dependency.MainModule.Types.Single();
            var bridge = new AssemblyBuilder(new("ExternalGenericBridge", new Version(1, 0, 0, 0)), core);
            var imported = bridge.ImportReference(definition, core);
            var open = imported.MakeGenericInstance(SignatureType.MethodParameter(0));
            var forward = bridge.AddType("Example", "Relay").AddMethod("Forward", new MethodSignature(open, [open], ["T"]));
            forward.LoadArgument(0); forward.Return();
            var snapshot = AssemblyDefinition.ReadNativeAssembly(Image(bridge));
            var method = snapshot.MainModule.Types.Single().Methods.Single();
            Check(method.TryGetSignature(out var signature) && signature!.ReturnType.ReferencedGenericInstance is not null, "external generic signature");
            var construction = signature!.ReturnType.ReferencedGenericInstance!;
            Check(construction.TypeArguments[0].MethodParameterIndex == 0 && ReferenceEquals(construction.Definition.Resolve(new Resolver(dependency)), definition), "external definition and method scope");
            var app = new AssemblyBuilder(new("ExternalGenericApp", new Version(1, 0, 0, 0)), core);
            Reject<InvalidDataException>(() => app.ImportReference(method, core));
            var wrong = new AssemblyBuilder(new(library.Identity.Name, new Version(2, 0, 0, 0)), core);
            Reject<InvalidDataException>(() => app.ImportReference(method, core, new Resolver(AssemblyDefinition.ReadNativeAssembly(Image(wrong)))));
            var reference = app.ImportReference(method, core, new Resolver(dependency));
            var main = app.AddFunction("Main"); app.EntryPoint = main;
            main.LoadConstant(42);
            main.NewObject(app.ImportReference(definition.Methods.Single(m => m.Name == ".ctor"), core).MakeConstructedReference([PrimitiveType.Int32]));
            main.Call(reference.MakeGenericInstance(PrimitiveType.Int32));
            main.Call(app.ImportReference(definition.Methods.Single(m => m.Name == "Get"), core).MakeConstructedReference([PrimitiveType.Int32]));
            main.Return();
            _ = app.WriteNativeAssembly();
            var context = new AssemblyLoadContext("external-generic-" + binary, true);
            try
            {
                context.LoadFromStream(new MemoryStream(library.Write()));
                context.LoadFromStream(new MemoryStream(bridge.Write()));
                Check(Equals(42, context.LoadFromStream(new MemoryStream(app.Write())).EntryPoint!.Invoke(null, null)), "CLR external generic forwarding");
            }
            finally { context.Unload(); }
        }
    }
    private sealed class Resolver(AssemblyDefinition definition) : IAssemblyResolver
    { public AssemblyDefinition? Resolve(AssemblyIdentity identity) => definition; }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
    private static void Reject<T>(Action action) where T : Exception
    { try { action(); } catch (T) { return; } throw new Exception("expected " + typeof(T).Name); }
}
