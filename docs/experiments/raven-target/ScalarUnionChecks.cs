using Mono.Cecil;

static class ScalarUnionChecks
{
    public static void Verify(string schema, string core)
    {
        using var resolver = new DefaultAssemblyResolver();
        resolver.AddSearchDirectory(Path.GetDirectoryName(Path.GetFullPath(core))!);
        using var module = ModuleDefinition.ReadModule(schema, new ReaderParameters { AssemblyResolver = resolver });
        var union = module.Types.Single(t => StandardUnionLibrary.IsCandidate(t));
        void Check(bool value, string message)
        {
            if (!value) throw new InvalidDataException(message);
        }
        Check(ApplicationTypes.IsInt32CaseUnion(union), "Expected scalar union admission.");
        var tag = union.Fields.Single(f => f.Name == "<Tag>");
        tag.Offset = 1;
        Check(!ApplicationTypes.IsInt32CaseUnion(union), "A misplaced tag was admitted.");
        tag.Offset = 0;
        var slot = union.Fields.First(f => f != tag);
        var offset = slot.Offset;
        slot.Offset = 0;
        Check(!ApplicationTypes.IsInt32CaseUnion(union), "A payload overlapping the tag was admitted.");
        slot.Offset = offset;
        var payload = union.NestedTypes.SelectMany(t => t.Fields).First();
        var attributes = payload.Attributes;
        payload.Attributes = (attributes & ~FieldAttributes.FieldAccessMask) | FieldAttributes.Public;
        Check(!ApplicationTypes.IsInt32CaseUnion(union), "Public overlay payload was admitted.");
        payload.Attributes = attributes;
        var scalar = payload.FieldType;
        payload.FieldType = module.TypeSystem.String;
        Check(!ApplicationTypes.IsInt32CaseUnion(union), "Managed-reference overlay was admitted.");
        payload.FieldType = scalar;
        var marker = union.CustomAttributes.Single(a => a.AttributeType.FullName == "System.Runtime.CompilerServices.UnionAttribute");
        union.CustomAttributes.Remove(marker);
        Check(!ApplicationTypes.IsInt32CaseUnion(union), "An unmarked explicit layout was admitted.");
        union.CustomAttributes.Add(marker);
        Check(ApplicationTypes.IsInt32CaseUnion(union), "Restored scalar union was rejected.");
        Console.WriteLine("Scalar union admission: 7 checks passed.");
    }
}
