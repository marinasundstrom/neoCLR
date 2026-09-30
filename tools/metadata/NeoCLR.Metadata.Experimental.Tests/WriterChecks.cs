using System.Reflection.Metadata;
using System.Reflection.PortableExecutable;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyDefinition = NeoCLR.Metadata.Experimental.Model.AssemblyDefinition;

internal static class WriterChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var (app, library, method) = Build(core);
        var bytes = app.Write();
        Check(bytes.SequenceEqual(app.Write()), "repeat write stability");
        var read = AssemblyDefinition.ReadAssembly(bytes, false);
        Check(read.EntryPointToken == 0x06000001 && read.Write().SequenceEqual(bytes), "snapshot entry and preserved PE");
        var copy = read.Write(); Array.Clear(copy);
        Check(read.Write().SequenceEqual(bytes), "owned snapshot emission");
        var dependency = AssemblyDefinition.ReadAssembly(library.Write(), false);
        var reference = read.MainModule.TypeReferences.Single(t => t.Name == "Math");
        Check(reference.Resolve(new Resolver(dependency)).Name == "Math", "imported TypeRef resolves");
        using (var pe = new PEReader(new MemoryStream(bytes)))
        {
            var metadata = pe.GetMetadataReader();
            Check(pe.PEHeaders.CorHeader!.EntryPointTokenOrRelativeVirtualAddress == 0x06000001, "entry token");
            Check(metadata.MemberReferences.Count == 1, "cross-assembly MemberRef imported");
            var entry = metadata.GetMethodDefinition(System.Reflection.Metadata.Ecma335.MetadataTokens.MethodDefinitionHandle(1));
            Check(pe.GetMethodBody(entry.RelativeVirtualAddress).GetILBytes()!.Contains((byte)0x28), "call body emitted");
        }
        method.ClearBody(); method.LoadConstant(7); method.Return();
        Check(!bytes.SequenceEqual(app.Write()), "edited body changes image");
        method.ClearBody(); method.Add(); method.Return(); Reject(() => app.Write());
        method.ClearBody(); method.LoadArgument(0); method.Return(); Reject(() => app.Write());
        method.ClearBody(); method.LoadConstant(1); method.LoadConstant(2); method.Return(); Reject(() => app.Write());
        method.ClearBody(); method.LoadConstant(1); Reject(() => app.Write());
        method.ClearBody(); method.LoadConstant(1); method.Return(); method.Return(); Reject(() => app.Write());
        method.ClearBody(); method.LoadConstant(1); method.Return();
        app.EntryPoint = library.Types[0].Methods[0]; Reject(() => app.Write());
        app.EntryPoint = method;
        var foreign = new AssemblyBuilder(app.Identity, core).AddType("", "Other").AddMethod("F");
        foreign.LoadConstant(1); foreign.Return();
        method.ClearBody(); method.Call(foreign); method.Return(); Reject(() => app.Write());
    }
    internal static void Emit(string output, AssemblyIdentity core)
    {
        Directory.CreateDirectory(output);
        var (app, library, _) = Build(core);
        File.WriteAllBytes(Path.Combine(output, "GeneratedLibrary.dll"), library.Write());
        File.WriteAllBytes(Path.Combine(output, "GeneratedApp.dll"), app.Write());
        Console.WriteLine("Emitted two PE assemblies through the public metadata model; expected entry result: 42");
    }
    private static (AssemblyBuilder App, AssemblyBuilder Library, MethodBuilder Main) Build(AssemblyIdentity core)
    {
        var library = new AssemblyBuilder(new("GeneratedLibrary", new Version(1, 0, 0, 0)), core);
        var twice = library.AddType("Generated", "Math").AddMethod("Twice", 1);
        twice.LoadArgument(0); twice.LoadConstant(2); twice.Multiply(); twice.Return();
        var app = new AssemblyBuilder(new("GeneratedApp", new Version(1, 0, 0, 0)), core);
        var main = app.AddType("Generated", "Program").AddMethod("Main");
        main.LoadConstant(20); main.Call(twice); main.LoadConstant(2); main.Add(); main.Return();
        app.EntryPoint = main;
        return (app, library, main);
    }
    private static void Reject(Action action)
    {
        try { action(); }
        catch (InvalidDataException) { return; }
        throw new Exception("invalid writer graph accepted");
    }
    private static void Check(bool value, string message) { if (!value) throw new Exception(message); }
    private sealed class Resolver(AssemblyDefinition target) : IAssemblyResolver
    {
        public AssemblyDefinition? Resolve(AssemblyIdentity identity) => target;
    }
}
