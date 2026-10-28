using System.Reflection;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;
using AssemblyBuilder = NeoCLR.Metadata.Experimental.Model.AssemblyBuilder;

internal static class InliningChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var graph = new AssemblyBuilder(new("Inlining", new(1, 0, 0, 0)), core);
        foreach (var flags in new[] { 0, 8, 256, 264 })
        {
            var method = graph.AddFunction("Method" + flags);
            method.SetImplementationFlags((MethodImplAttributes)flags);
            method.GetILGenerator().LoadConstant(42);
            method.GetILGenerator().Return();
            if (flags == 8) graph.EntryPoint = method;
        }
        var native = graph.WriteNativeAssembly();
        foreach (var image in new[] {
            AssemblyDefinition.ReadAssembly(graph.Write(), false),
            AssemblyDefinition.ReadNativeAssembly(RuntimeAssemblyContainer.WriteBinary(native, core)),
            AssemblyDefinition.ReadAssembly(NativeAssemblyDefinition.ReadAssembly(native).CreateReferenceAssembly(core), false) })
        {
            foreach (var method in image.MainModule.Methods)
            {
                if (method.ImplementationAttributes != ushort.Parse(method.Name[6..]))
                    throw new Exception("Inlining flags lost across native/CLI/reference round trip");
                try { method.SetImplementationFlags(0); throw new Exception("Loaded method mutated"); }
                catch (InvalidOperationException) { }
            }
        }
        var target = graph.AddFunction("Invalid");
        foreach (var flags in new[] { 1, 64, 512, 4096, -1 })
        {
            try { target.SetImplementationFlags((MethodImplAttributes)flags); throw new Exception("Unsupported flags accepted"); }
            catch (ArgumentOutOfRangeException) { }
        }
        target.SetInternalCall();
        try { target.SetImplementationFlags(MethodImplAttributes.NoInlining); throw new Exception("InternalCall mixed with inlining"); }
        catch (InvalidOperationException) { }
        Console.WriteLine("PASS inlining flags: native, CLI, reference, entry and rejection contracts");
    }
}
