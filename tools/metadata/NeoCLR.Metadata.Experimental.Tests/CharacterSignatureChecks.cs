using System.Runtime.Loader;
using NeoCLR.Metadata.Experimental.Model;

internal static class CharacterSignatureChecks
{
    internal static void Run()
    {
        var host = typeof(object).Assembly.GetName();
        var core = new AssemblyIdentity(host.Name!, host.Version!, "", Convert.ToHexString(host.GetPublicKeyToken()!));
        var reference = AssemblyDefinition.ReadAssembly(File.ReadAllBytes(typeof(CharacterFixture).Assembly.Location), false);
        var graph = new AssemblyBuilder(new("CharacterSignatures", new Version(1, 0, 0, 0)), core);
        var fixture = reference.MainModule.Types.Single(t => t.Name == nameof(CharacterFixture));
        foreach (var name in new[] { "Echo", "Vector" })
        {
            var imported = graph.ImportReference(fixture.Methods.Single(m => m.Name == name), core);
            var type = imported.Signature.ReturnType;
            var character = (type.ArrayElement ?? type).ImportedType!;
            if (character.Name != "Char" || !character.IsValueType || !character.AssemblyIdentity.Equals(core))
                throw new Exception("character signature lost core identity");
            var echo = graph.AddFunction(name, new MethodSignature(type, [type]));
            var il = echo.GetILGenerator(); il.LoadArgument(0); il.Return();
        }
        var image = graph.Write();
        var snapshot = AssemblyDefinition.ReadAssembly(image, false);
        if (!snapshot.MainModule.Functions.Single(m => m.Name == "Echo").GetSignature().SequenceEqual(new byte[] { 0, 1, 3, 3 }) ||
            !snapshot.MainModule.Functions.Single(m => m.Name == "Vector").GetSignature().SequenceEqual(new byte[] { 0, 1, 0x1d, 3, 0x1d, 3 }))
            throw new Exception("Char requires canonical CLI element encoding");
        var context = new AssemblyLoadContext("characters", isCollectible: true);
        try
        {
            var assembly = context.LoadFromStream(new MemoryStream(image));
            var echo = assembly.ManifestModule.GetMethods().Single(m => m.Name == "Echo");
            foreach (var value in new[] { 'A', '\uD800', '\uFFFF' })
                if (!Equals(echo.Invoke(null, [value]), value)) throw new Exception("CLI Char code unit changed");
            var vector = new[] { 'A', '\uD800' };
            if (!ReferenceEquals(assembly.ManifestModule.GetMethods().Single(m => m.Name == "Vector").Invoke(null, [vector]), vector))
                throw new Exception("CLI Char vector identity changed");
        }
        finally { context.Unload(); }
    }
}

public static class CharacterFixture
{
    public static char Echo(char value) => value;
    public static char[] Vector(char[] value) => value;
}
