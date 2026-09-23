using System;
using System.IO;

// .NET library comparison, not a supported neoCLR application frontend.
var root = Path.Combine(Path.GetTempPath(), "neoclr-dotnet-lookup-" + Guid.NewGuid());
Directory.CreateDirectory(root);
try
{
    var path = Path.Combine(root, "item");
    var absoluteRoot = Path.GetPathRoot(root)!;
    Check(Path.Combine(root, absoluteRoot) == absoluteRoot,
        "A rooted later Combine argument should replace the earlier prefix");
    var descriptor = new FileInfo(path); // Construction succeeds before creation.
    Check(!descriptor.Exists, "Missing file should not exist");
    File.WriteAllText(path, "first");
    Check(!descriptor.Exists, "Exists should retain its cached observation");
    descriptor.Refresh();
    Check(descriptor.Exists, "Refresh should observe creation");
    using (var opened = new FileStream(path, FileMode.Open, FileAccess.Read, FileShare.ReadWrite | FileShare.Delete))
    {
        File.Move(path, Path.Combine(root, "moved"));
        File.WriteAllText(path, "replacement");
        using var reader = new StreamReader(opened);
        Check(reader.ReadToEnd() == "first", "Open handle should retain original item");
        Check(File.ReadAllText(descriptor.FullName) == "replacement", "Descriptor should retain address");
    }
    File.Delete(path);
    Check(descriptor.Exists, "Exists should retain observation until refreshed");
    descriptor.Refresh();
    Check(!descriptor.Exists, "Refresh should observe removal");
    Check(!new FileInfo(root).Exists, "FileInfo.Exists treats a directory as false");
    Console.WriteLine(".NET descriptor caching, replacement and lookup: passed");
}
finally
{
    Directory.Delete(root, recursive: true);
}

static void Check(bool condition, string message)
{
    if (!condition) throw new Exception(message);
}
