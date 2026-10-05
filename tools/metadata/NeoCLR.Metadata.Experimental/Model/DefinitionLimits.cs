namespace NeoCLR.Metadata.Experimental.Model;

// Keep authored CLI/native graphs inside the existing CLI snapshot row budget.
// CLI reserves the first TypeDef row for <Module>; native rows omit that entry.
internal static class DefinitionLimits
{
    internal const int TypeRows = 4096;
    internal const int AuthoredTypes = TypeRows - 1;
}
