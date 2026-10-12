// Documentation/compiler reference only; native Raven library bodies execute.
static class CryptographyDeclarations
{
    public const string Source = """
namespace Security.Cryptography {
    public struct CryptographyError {
        public struct DestinationTooSmall { }
        public struct ProviderFailure { }
    }
    public static class SHA256 {
        public static Result<int, CryptographyError> HashData(byte[] data, byte[] destination) => default;
    }
    public static class HMACSHA256 {
        public static Result<int, CryptographyError> HashData(byte[] key, byte[] data, byte[] destination) => default;
        public static bool VerifyData(byte[] key, byte[] data, byte[] tag) => default;
    }
    public static class RandomNumberGenerator {
        public static Result<PropagationUnit, CryptographyError> Fill(byte[] destination) => default;
    }
}
""";
}
