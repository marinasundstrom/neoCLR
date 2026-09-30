using NeoCLR.Metadata.Experimental;

if (args.Length != 2)
{
    Console.Error.WriteLine("Usage: NeoCLR.Metadata.Translate <module.neo.json> <module.neox>");
    return 2;
}
try
{
    if (Path.GetFullPath(args[0]) == Path.GetFullPath(args[1]))
        throw new InvalidDataException("input and output paths must differ");
    if (new FileInfo(args[0]).Length > 4 * 1024 * 1024)
        throw new InvalidDataException("native JSON exceeds 4 MiB limit");
    var image = NativeModuleContainer.WriteBinary(File.ReadAllBytes(args[0]));
    using var output = new FileStream(args[1], FileMode.CreateNew, FileAccess.Write);
    output.Write(image);
    Console.WriteLine($"Wrote {image.Length} bytes. Runtime semantic validation is required.");
    return 0;
}
catch (Exception error) when (error is IOException or InvalidDataException or UnauthorizedAccessException)
{
    Console.Error.WriteLine(error.Message);
    return 1;
}
