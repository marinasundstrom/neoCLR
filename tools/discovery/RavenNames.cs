using System.Text;

// Source-linked by the two independent discovery tools; no runtime dependency.
internal static class RavenNames
{
    internal static string Identifier(string name)
    {
        // These are Raven source limitations, not native metadata restrictions.
        if (name.Length == 0 || name.IndexOfAny(['`', '\r', '\n', '\u0085', '\u2028', '\u2029']) >= 0)
            throw new InvalidDataException("Name cannot be represented by a Raven backtick identifier");
        return "`" + name + "`";
    }

    // JSON permits escaped UTF-16 surrogates; Raven literals require scalar text.
    internal static string StringLiteral(string value)
    {
        var result = new StringBuilder("\"");
        foreach (var character in value)
        {
            result.Append(character switch
            {
                '\\' => "\\\\",
                '"' => "\\\"",
                '\r' => "\\r",
                '\n' => "\\n",
                '\t' => "\\t",
                _ when char.IsControl(character) || character is '\u2028' or '\u2029' => "\\u" + ((int)character).ToString("X4"),
                _ => character.ToString()
            });
        }
        return result.Append('"').ToString();
    }

    internal static string Reference(string scope, string name) =>
        (scope.Length == 0 ? "" : string.Join(".", scope.Split('.').Select(Identifier)) + ".") + Identifier(name);
}
