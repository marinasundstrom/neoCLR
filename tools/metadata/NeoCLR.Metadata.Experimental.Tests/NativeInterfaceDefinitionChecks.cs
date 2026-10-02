using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class NativeInterfaceDefinitionChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var library = new AssemblyBuilder(new("NativeInterfaces", new Version(1, 0, 0, 0)), core);
        var contract = library.AddInterface("Example", "Value");
        contract.AddInterfaceMethod("Get", new(PrimitiveType.Int32, []));
        var unrelated = library.AddInterface("Example", "Unrelated"); unrelated.AddInterfaceMethod("Get", new(PrimitiveType.Int32, []));
        var derived = library.AddInterface("Example", "Derived"); derived.AddBaseInterface(contract);
        var owner = library.AddClass("Example", "Concrete"); owner.AddInterfaceImplementation(derived);
        var ctor = owner.AddConstructor(Array.Empty<PrimitiveType>()); ctor.Return();
        var get = owner.AddInstanceMethod("Get", new(PrimitiveType.Int32, [])); get.LoadConstant(42); get.Return();
        var factory = library.AddType("Example", "Factory");
        var create = factory.AddMethod("Create", new(derived, Array.Empty<SignatureType>())); create.NewObject(ctor); create.Return();
        foreach (var binary in new[] { false, true })
        {
            var native = library.WriteNativeAssembly();
            var image = binary ? RuntimeAssemblyContainer.WriteBinary(native, core) : RuntimeAssemblyContainer.Write(native, core);
            var snapshot = AssemblyDefinition.ReadNativeAssembly(image);
            var readContract = snapshot.MainModule.Types.Single(t => t.Name == "Value");
            var readDerived = snapshot.MainModule.Types.Single(t => t.Name == "Derived");
            var readOwner = snapshot.MainModule.Types.Single(t => t.Name == "Concrete");
            Check((readContract.Attributes & 0xa0) == 0xa0 && (readContract.Methods.Single().Attributes & 0x5c0) == 0x5c0, "interface flags");
            Check(ReferenceEquals(readDerived.Interfaces.Single().InterfaceType.Resolve(), readContract) &&
                ReferenceEquals(readOwner.Interfaces.Single().InterfaceType.Resolve(), readDerived) &&
                ReferenceEquals(readOwner.Interfaces.Single().DeclaringType, readOwner), "canonical relationships");
            var relationship = readOwner.Interfaces.Single();
            Parallel.For(0, 32, _ => Check(ReferenceEquals(readOwner.Interfaces.Single(), relationship), "stable relationship identity"));
            try { readOwner.Interfaces.Clear(); throw new Exception("native relationships are mutable"); } catch (NotSupportedException) { }
            Check(snapshot.Write().SequenceEqual(image), "native interface image roundtrip");
            var app = new AssemblyBuilder(new("NativeInterfaceApp", new Version(1, 0, 0, 0)), core);
            var imported = app.ImportReference(readContract.Methods.Single(), core);
            Check(imported.IsInterfaceMethod && !imported.IsStatic, "imported dispatch contract");
            var main = app.AddFunction("Main"); app.EntryPoint = main;
            main.Call(app.ImportReference(snapshot.MainModule.Types.Single(t => t.Name == "Factory").Methods.Single(), core));
            main.CallVirtual(imported); main.Return();
            _ = app.WriteNativeAssembly();
            var wrong = new AssemblyBuilder(new("WrongInterfaceApp", new Version(1, 0, 0, 0)), core);
            var wrongMain = wrong.AddFunction("Main"); wrong.EntryPoint = wrongMain;
            wrongMain.Call(wrong.ImportReference(snapshot.MainModule.Types.Single(t => t.Name == "Factory").Methods.Single(), core));
            wrongMain.CallVirtual(wrong.ImportReference(snapshot.MainModule.Types.Single(t => t.Name == "Unrelated").Methods.Single(), core)); wrongMain.Return();
            try { wrong.WriteNativeAssembly(); throw new Exception("unrelated interface accepted"); } catch (InvalidDataException) { }
            var context = new AssemblyLoadContext("native-interface-" + binary, true);
            try
            {
                context.LoadFromStream(new MemoryStream(library.Write()));
                Check((int)context.LoadFromStream(new MemoryStream(app.Write())).EntryPoint!.Invoke(null, null)! == 42, "CLR imported native interface dispatch");
            }
            finally { context.Unload(); }
        }
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
}
