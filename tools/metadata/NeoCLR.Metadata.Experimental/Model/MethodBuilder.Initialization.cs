namespace NeoCLR.Metadata.Experimental.Model;

public sealed partial class MethodBuilder
{
    /// <summary>Loads an owned local address, including an as-yet uninitialized local.</summary>
    /// <param name="local">Local belonging to this method.</param>
    /// <exception cref="ArgumentException">Foreign local.</exception>
    /// <exception cref="ArgumentNullException">Null local.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded.</exception>
    public void LoadLocalAddress(LocalDefinition local) => GetILGenerator().LoadLocalAddress(local);

    /// <summary>Initializes an addressed local using its exact type; validates on write.</summary>
    /// <param name="type">Non-Void supported signature type in the current generic scope.</param>
    /// <exception cref="ArgumentNullException">Null type.</exception>
    /// <exception cref="ArgumentException">Void, foreign owner or out-of-scope parameter.</exception>
    /// <exception cref="InvalidDataException">Instruction limit exceeded or invalid stack on write.</exception>
    public void InitializeObject(SignatureType type) => GetILGenerator().InitializeObject(type);

    /// <summary>Pushes a default value using a fresh scratch local and typed initialization.</summary>
    /// <param name="type">Supported non-Void value type, including scoped method parameters.</param>
    /// <remarks>Uses one local and three instructions. ClearBody retains the scratch local.</remarks>
    /// <exception cref="ArgumentNullException">Null type.</exception>
    /// <exception cref="ArgumentException">Void, foreign owner or out-of-scope parameter.</exception>
    /// <exception cref="InvalidDataException">Local or instruction limit exceeded.</exception>
    public void LoadDefault(SignatureType type) => GetILGenerator().LoadDefault(type);
}
