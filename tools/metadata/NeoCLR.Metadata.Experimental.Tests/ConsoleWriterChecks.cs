using System.Text.Json;
using NeoCLR.Metadata.Experimental;
using NeoCLR.Metadata.Experimental.Model;

internal static class ConsoleWriterChecks
{
    internal static void Run()
    {
        var core = new AssemblyIdentity("Core", new Version(1, 0, 0, 0));
        var graph = new AssemblyBuilder(new("Hello", new Version(1, 0, 0, 0)), core);
        var main = graph.AddFunction("Main");
        main.LoadConstant(0); main.WriteConsoleLine("Hello 世界\n"); main.Return(); graph.EntryPoint = main;
        var native = graph.WriteNativeAssembly();
        using var document = JsonDocument.Parse(native);
        var body = document.RootElement.GetProperty("functions")[0].GetProperty("body");
        if (body.GetArrayLength() != 5 || body[1].GetProperty("arg").GetString() != "Hello 世界\n" ||
            body[2].GetProperty("arg").GetProperty("name").GetString() != "System.Console.WriteLine")
            throw new Exception("console lowering changed");
        var pe = RuntimeAssemblyContainer.Write(native, core);
        if (!RuntimeAssemblyContainer.Read(pe).SequenceEqual(native)) throw new Exception("console container mismatch");
        Throws<InvalidDataException>(() => graph.Write());
        Throws<ArgumentNullException>(() => main.WriteConsoleLine(null!));
        Throws<ArgumentException>(() => main.WriteConsoleLine("\ud800"));
        Throws<ArgumentException>(() => main.WriteConsoleLine(new string('é', 32769)));
        main.ClearBody();
        for (int i = 0; i < 65; i++) main.WriteConsoleLine(new string('a', 65536));
        main.LoadConstant(0); main.Return();
        Throws<InvalidDataException>(() => graph.WriteNativeAssembly());
    }
    private static void Throws<T>(Action action) where T : Exception
    {
        try { action(); } catch (T) { return; }
        throw new Exception("expected " + typeof(T).Name);
    }
}
