using Mono.Cecil;
using Mono.Cecil.Cil;
using System.Reflection.PortableExecutable;

static class ResultImportChecks
{
    public static Dictionary<string, string> Run(string application, string core, string output)
    {
        using var image = AssemblyDefinition.ReadAssembly(application);
        var show = image.MainModule.GetTypes().SelectMany(t => t.Methods).Single(m => m.Name == "Show");
        var original = File.ReadAllBytes(application);
        using var pe = new PEReader(new MemoryStream(original));
        var section = pe.PEHeaders.SectionHeaders.Single(s => show.RVA >= s.VirtualAddress && show.RVA < s.VirtualAddress + s.VirtualSize);
        var body = section.PointerToRawData + show.RVA - section.VirtualAddress;
        var header = (original[body] & 3) == 2 ? 1 : (BitConverter.ToUInt16(original, body) >> 12) * 4;
        var code = body + header;
        var okCall = show.Body.Instructions.Single(i => i.Operand is MethodReference m && m.Name == "TryGetValue" && m.Parameters[0].ParameterType.FullName.Contains("/Ok`1"));
        var errorCall = show.Body.Instructions.Single(i => i.Operand is MethodReference m && m.Name == "TryGetValue" && m.Parameters[0].ParameterType.FullName.Contains("/Error`1"));
        var cases = new (string Name, int Offset, byte[] Bytes, string Diagnostic)[] {
            ("ResultDefaultCarrier", code, [(byte)OpCodes.Ldloc_0.Value], "Read of uninitialized or unsupported default local"),
            ("ResultUnassignedReceiver", code + show.Body.Instructions.First(i => i.OpCode.Code == Code.Stloc_3).Offset,
                [(byte)OpCodes.Pop.Value], "Read through uninitialized carrier address"),
            ("ResultMergeType", code + show.Body.Instructions.First(i => i.OpCode.Code == Code.Ldc_I4_1).Offset,
                [(byte)OpCodes.Ldloc_0.Value], "Incompatible branch stack merge"),
            ("ResultWrongOutCase", code + okCall.Offset + 1,
                BitConverter.GetBytes(((MethodReference)errorCall.Operand).MetadataToken.ToUInt32()), "Input stack type mismatch")
        };
        var results = new Dictionary<string, string>();
        foreach (var test in cases)
        {
            var bytes = (byte[])original.Clone();
            test.Bytes.CopyTo(bytes, test.Offset);
            var path = Path.Combine(output, test.Name + ".dll");
            var destination = Path.Combine(output, test.Name + ".neoil");
            File.WriteAllBytes(path, bytes);
            try { UnionImport.Write(path, core, destination); }
            catch (InvalidDataException error) when (error.Message.Contains(test.Diagnostic))
            {
                if (File.Exists(destination)) throw new Exception("Rejected input produced executable output.");
                results[test.Name] = error.Message;
                continue;
            }
            throw new Exception("Result importer failed to reject " + test.Name);
        }
        return results;
    }
}
