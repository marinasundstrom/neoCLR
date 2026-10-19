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
        // Borrowed view methods are documented by the native Raven reference.
    }
    public static class HMACSHA256 {
        // HashData and VerifyData are documented by the native Raven reference.
    }
    public static class RandomNumberGenerator {
        // Borrowed view methods are documented by the native Raven reference.
    }
}
""";
}
