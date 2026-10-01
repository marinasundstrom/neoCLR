using System.Text;

namespace NeoCLR.Metadata.Experimental.Model;

// Temporary CLI global-name projection, not a synthetic native type owner.
internal static class FunctionNamespaceEncoding
{
    internal const string Prefix = "<NeoFunction>";
    internal static void Validate(string value)
    {
        ArgumentNullException.ThrowIfNull(value);
        if (value.Length == 0) return;
        if (value.Length > 1024 || value.Split('.').Any(p => string.IsNullOrWhiteSpace(p)) || value.Any(char.IsControl))
            throw new ArgumentException("invalid function namespace");
        try { _ = new UTF8Encoding(false, true).GetByteCount(value); }
        catch (EncoderFallbackException error) { throw new ArgumentException("invalid namespace Unicode", error); }
    }
    internal static string Encode(string ns, string name) => ns.Length == 0 ? name :
        Prefix + Convert.ToHexString(Encoding.UTF8.GetBytes(ns)) + "_" + Convert.ToHexString(Encoding.UTF8.GetBytes(name));
    internal static (string Namespace, string Name) Decode(string name)
    {
        if (!name.StartsWith(Prefix, StringComparison.Ordinal)) return ("", name);
        try
        {
            var parts = name[Prefix.Length..].Split('_');
            if (parts.Length != 2) throw new FormatException();
            var utf8 = new UTF8Encoding(false, true);
            var ns = utf8.GetString(Convert.FromHexString(parts[0]));
            var simple = utf8.GetString(Convert.FromHexString(parts[1]));
            Validate(ns);
            if (ns.Length == 0 || simple.Length == 0 || Encode(ns, simple) != name) throw new FormatException();
            return (ns, simple);
        }
        catch (Exception error) when (error is ArgumentException or FormatException)
        { throw new InvalidDataException("invalid namespaced global projection", error); }
    }
}
