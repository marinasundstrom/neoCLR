using Mono.Cecil;
using Mono.Cecil.Cil;
using System.Reflection.PortableExecutable;

static class OptionImportChecks
{
    public static Dictionary<string, string> Run(string application, string core, string output)
    {
        using var image = AssemblyDefinition.ReadAssembly(application);
        var methods = image.MainModule.GetTypes().SelectMany(t => t.Methods).ToArray();
        var show = methods.Single(m => m.Name == "ShowPrice");
        var find = methods.Single(m => m.Name == "FindPrice");
        var original = File.ReadAllBytes(application);
        using var pe = new PEReader(new MemoryStream(original));
        int CodeStart(MethodDefinition method)
        {
            var section = pe.PEHeaders.SectionHeaders.Single(s => method.RVA >= s.VirtualAddress && method.RVA < s.VirtualAddress + s.VirtualSize);
            var body = section.PointerToRawData + method.RVA - section.VirtualAddress;
            return body + ((original[body] & 3) == 2 ? 1 : (BitConverter.ToUInt16(original, body) >> 12) * 4);
        }
        Instruction Extract(string name) => show.Body.Instructions.Single(i => i.Operand is MethodReference m && m.Name == "TryGetValue" && m.Parameters[0].ParameterType.FullName.Contains(name));
        Instruction Construct(string name) => find.Body.Instructions.Single(i => i.OpCode.Code == Code.Newobj && i.Operand is MethodReference m && m.DeclaringType.FullName.StartsWith("System.Option`1") && m.Parameters[0].ParameterType.FullName.Contains(name));
        var cases = new (string Name, int Offset, byte[] Bytes, string Diagnostic)[] {
            ("OptionDefaultCarrier", CodeStart(show), [(byte)OpCodes.Ldloc_0.Value], "Read of uninitialized or unsupported default local"),
            ("OptionWrongOutCase", CodeStart(show) + Extract("/Some`1").Offset + 1,
                BitConverter.GetBytes(((MethodReference)Extract("/None").Operand).MetadataToken.ToUInt32()), "Input stack type mismatch"),
            ("OptionWrongConstructorCase", CodeStart(find) + Construct("/Some`1").Offset + 1,
                BitConverter.GetBytes(((MethodReference)Construct("/None").Operand).MetadataToken.ToUInt32()), "Input stack type mismatch")
        };
        var results = new Dictionary<string, string>();
        foreach (var test in cases)
        {
            var bytes = (byte[])original.Clone(); test.Bytes.CopyTo(bytes, test.Offset);
            var path = Path.Combine(output, test.Name + ".dll");
            var destination = Path.Combine(output, test.Name + ".neoil");
            File.WriteAllBytes(path, bytes);
            try { UnionImport.Write(path, core, destination); }
            catch (InvalidDataException error) when (error.Message.Contains(test.Diagnostic))
            {
                if (File.Exists(destination)) throw new Exception("Rejected input produced executable output.");
                results[test.Name] = error.Message; continue;
            }
            throw new Exception("Union importer failed to reject " + test.Name);
        }
        return results;
    }
}
