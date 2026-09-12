using Mono.Cecil;
using AssemblyDefinition = Mono.Cecil.AssemblyDefinition;
using Mono.Cecil.Cil;
using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;

static class VoidChecks
{
    public static Dictionary<string, string> Run(string application, string raw, string core, string output)
    {
        using var image = AssemblyDefinition.ReadAssembly(application);
        var methods = image.MainModule.GetTypes().SelectMany(t => t.Methods).ToArray();
        var marker = methods.Single(m => m.Name == "Marker");
        var bytes = File.ReadAllBytes(application);
        using var pe = new PEReader(new MemoryStream(bytes));
        var metadata = pe.GetMetadataReader();
        var signature = metadata.GetBlobReader(metadata.GetMethodDefinition(MetadataTokens.MethodDefinitionHandle((int)marker.MetadataToken.RID)).Signature);
        signature.ReadSignatureHeader();
        if (signature.ReadCompressedInteger() != 1 || signature.ReadByte() != 0x15 || signature.ReadByte() != 0x11)
            throw new Exception("Expected Marker(Int32) returning a generic value carrier.");
        signature.ReadTypeHandle();
        if (signature.ReadCompressedInteger() != 1 || signature.ReadByte() != 0x11)
            throw new Exception("Generic Void must use a value-type token, not a VOID return marker.");
        var handle = signature.ReadTypeHandle();
        var type = metadata.GetTypeReference((TypeReferenceHandle)handle);
        if (metadata.GetString(type.Name) != "Void" || metadata.GetString(type.Namespace) != "System"
            || metadata.GetString(metadata.GetAssemblyReference((AssemblyReferenceHandle)type.ResolutionScope).Name) != CoreDeclarations.Identity
            || signature.ReadByte() != 0x08 || signature.RemainingBytes != 0)
            throw new Exception("Unexpected generic Void signature.");

        var literal = marker.Body.Instructions.Single(i => i.OpCode.Code == Code.Ldsfld);
        var section = pe.PEHeaders.SectionHeaders.Single(s => marker.RVA >= s.VirtualAddress && marker.RVA < s.VirtualAddress + s.VirtualSize);
        var body = section.PointerToRawData + marker.RVA - section.VirtualAddress;
        var start = body + ((bytes[body] & 3) == 2 ? 1 : (BitConverter.ToUInt16(bytes, body) >> 12) * 4);
        var wrongField = image.MainModule.GetTypes().SelectMany(t => t.Fields).First(f => !f.IsStatic);
        var wrongPayload = (byte[])bytes.Clone();
        wrongPayload[start + literal.Offset] = (byte)OpCodes.Ldc_I4.Value;
        BitConverter.GetBytes(1).CopyTo(wrongPayload, start + literal.Offset + 1);
        var nonUnit = (byte[])bytes.Clone();
        BitConverter.GetBytes(wrongField.MetadataToken.ToUInt32()).CopyTo(nonUnit, start + literal.Offset + 1);
        var results = new Dictionary<string, string>();
        foreach (var test in new[] {
            (Name: "VoidRawSignature", Bytes: File.ReadAllBytes(raw), Diagnostic: "Unsupported Result profile type"),
            (Name: "VoidWrongPayload", Bytes: wrongPayload, Diagnostic: "Input stack type mismatch"),
            (Name: "VoidNonUnitField", Bytes: nonUnit, Diagnostic: "Only the empty Raven Unit literal field is admitted") })
        {
            var path = Path.Combine(output, test.Name + ".dll");
            var destination = Path.Combine(output, test.Name + ".neoil");
            File.WriteAllBytes(path, test.Bytes);
            try { UnionImport.Write(path, core, destination); }
            catch (InvalidDataException error) when (error.Message.Contains(test.Diagnostic))
            {
                if (File.Exists(destination)) throw new Exception("Rejected Void input produced executable output.");
                results[test.Name] = error.Message; continue;
            }
            throw new Exception("Void profile did not reject " + test.Name);
        }
        try { typeof(List<>).MakeGenericType(typeof(void)); }
        catch (ArgumentException) { return results; }
        throw new Exception("Host .NET generic Void behavior changed; recheck the platform comparison.");
    }
}
