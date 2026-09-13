using System.Reflection;
using Raven.CodeAnalysis;

// Reuse the editor's project/reference policy; keep emission and admission target-owned.
static class ProjectBuild
{
    public static void Write(string projectPath, string output)
    {
        projectPath = Path.GetFullPath(projectPath);
        output = Path.GetFullPath(output);
        if (Directory.Exists(output)) throw new IOException("Output directory must not exist.");
        var workspace = RavenWorkspace.Create(targetFramework: "net11.0");
        var id = workspace.OpenProject(projectPath);
        var project = workspace.CurrentSolution.GetProject(id)!;
        if (project.CompilationOptions?.MetadataImportOptions?.CoreAssemblyName != CoreDeclarations.Identity)
            throw new InvalidDataException("Project must explicitly target NeoCLR.CoreProbe.");
        var references = project.MetadataReferences.OfType<PortableExecutableReference>().ToArray();
        if (references.Length != 1 || project.ProjectReferences.Any())
            throw new InvalidDataException("This profile admits only the supplied neoCLR core reference.");
        var core = references[0].FilePath!;
        if (AssemblyName.GetAssemblyName(core).Name != CoreDeclarations.Identity)
            throw new InvalidDataException("Unexpected target core identity.");
        var expectedIteration = new RuntimeIterationContract(CoreDeclarations.Identity,
            "System.Collections.Iterable`1", "System.Collections.Iterator`1");
        var iteration = project.CompilationOptions.RuntimeIterationContract;
        if (iteration is not null && (iteration with { ArraysImplementIterable = false, ArrayShapeTypeName = null }) != expectedIteration)
            throw new InvalidDataException("Unsupported neoCLR project iteration contract.");
        if (iteration?.ArrayShapeTypeName is not null and not "System.Array`1")
            throw new InvalidDataException("Unsupported neoCLR array shape.");
        var propagation = project.CompilationOptions.RuntimePropagationContract;
        if (propagation is not null && propagation != new RuntimePropagationContract(CoreDeclarations.Identity, "System.Propagatable`3"))
            throw new InvalidDataException("Unsupported neoCLR project propagation contract.");
        var compilation = workspace.GetCompilation(id)!;
        using var image = new MemoryStream();
        var emitted = compilation.Emit(image, null, new EmitOptions(AssemblyName.GetAssemblyName(core)));
        if (!emitted.Success)
            throw new InvalidDataException(string.Join("\n", emitted.Diagnostics));
        Directory.CreateDirectory(output);
        var raw = Path.Combine(output, "App.raw.dll");
        var projected = Path.Combine(output, "App.dll");
        File.WriteAllBytes(raw, image.ToArray());
        VoidProjection.Write(raw, core, projected);
        UnionImport.Write(projected, core, Path.Combine(output, "App.neoil"), collectionProfile: iteration is not null);
    }
}
