using System.Reflection.Metadata;
using System.Reflection.Metadata.Ecma335;
using System.Reflection.PortableExecutable;

namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class AssemblyDefinition
{
    // Inspect the exact snapshot. Container spelling is not a semantic contract.
    internal bool HasCoreTopLevelMarker(uint token, AssemblyIdentity core)
    {
        using var stream = new MemoryStream(image, writable: false);
        using var pe = new PEReader(stream);
        var reader = pe.GetMetadataReader();
        var type = reader.GetTypeDefinition((TypeDefinitionHandle)MetadataTokens.EntityHandle((int)token));
        foreach (var handle in type.GetCustomAttributes())
        {
            var attribute = reader.GetCustomAttribute(handle);
            EntityHandle owner;
            string name;
            BlobHandle signature;
            if (attribute.Constructor.Kind == HandleKind.MethodDefinition)
            {
                var method = reader.GetMethodDefinition((MethodDefinitionHandle)attribute.Constructor);
                owner = method.GetDeclaringType(); name = reader.GetString(method.Name); signature = method.Signature;
            }
            else if (attribute.Constructor.Kind == HandleKind.MemberReference)
            {
                var member = reader.GetMemberReference((MemberReferenceHandle)attribute.Constructor);
                owner = member.Parent; name = reader.GetString(member.Name); signature = member.Signature;
            }
            else continue;
            if (name != ".ctor" || !reader.GetBlobBytes(signature).SequenceEqual(new byte[] { 0x20, 0, 1 }) ||
                !reader.GetBlobBytes(attribute.Value).SequenceEqual(new byte[] { 1, 0, 0, 0 })) continue;
            if (owner.Kind == HandleKind.TypeDefinition)
            {
                var marker = reader.GetTypeDefinition((TypeDefinitionHandle)owner);
                if (Identity.Equals(core) && marker.GetDeclaringType().IsNil && reader.GetString(marker.Namespace) == "System.Runtime.CompilerServices" && reader.GetString(marker.Name) == "TopLevelAttribute") return true;
            }
            else if (owner.Kind == HandleKind.TypeReference)
            {
                var marker = reader.GetTypeReference((TypeReferenceHandle)owner);
                if (marker.ResolutionScope.Kind != HandleKind.AssemblyReference || reader.GetString(marker.Namespace) != "System.Runtime.CompilerServices" || reader.GetString(marker.Name) != "TopLevelAttribute") continue;
                var scopeToken = (uint)MetadataTokens.GetToken(marker.ResolutionScope);
                if (MainModule.AssemblyReferences.Any(scope => scope.MetadataToken == scopeToken && scope.Identity.Equals(core))) return true;
            }
        }
        return false;
    }
}
