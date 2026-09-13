// Artifact import is separate from Raven compilation and project evaluation.
static class ApplicationImport
{
    public static void Write(string source, string core, string output)
    {
        if (new FileInfo(source).Length > 16 * 1024 * 1024)
            throw new InvalidDataException("Image exceeds profile limit.");
        Write(File.ReadAllBytes(source), core, output, collectionProfile: true);
    }

    public static void Write(byte[] image, string core, string output, bool collectionProfile)
    {
        output = Path.GetFullPath(output);
        if (Directory.Exists(output)) throw new IOException("Output directory must not exist.");
        Directory.CreateDirectory(output);
        var raw = Path.Combine(output, "App.raw.dll");
        var projected = Path.Combine(output, "App.dll");
        File.WriteAllBytes(raw, image);
        // Named Void storage projection remains a documented temporary adapter.
        VoidProjection.Write(raw, core, projected);
        UnionImport.Write(projected, core, Path.Combine(output, "App.neoil"), collectionProfile);
    }
}
