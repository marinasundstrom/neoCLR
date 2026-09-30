import struct
import unittest
from codec import ENTRY, HEADER, MAX_SIZE, FormatError, Section, decode, encode


class EnvelopeTests(unittest.TestCase):
    def test_golden_optional_section(self):
        golden = bytes.fromhex(
            "4e454f58000001000100000023000000"
            "07000100000000002000000003000000" "616263")
        sections = [Section(7, 1, False, b"abc")]
        self.assertEqual(encode(sections), golden)
        self.assertEqual(decode(golden), sections)
        self.assertEqual(encode(decode(golden)), golden)

    def test_required_schema_negotiation(self):
        data = encode([Section(7, 2, True, b"abc")], {7: 2})
        self.assertEqual(decode(data, {7: 2})[0].payload, b"abc")
        for supported in ({}, {7: 1}):
            with self.assertRaisesRegex(FormatError, "unsupported required"):
                decode(data, supported)
        self.assertEqual(decode(encode([Section(7, 2, False, b"abc")]), {7: 1})[0].version, 2)

    def test_every_truncation_rejected(self):
        data = encode([Section(7, 1, False, b"abc")])
        for length in range(len(data)):
            with self.subTest(length=length), self.assertRaises(FormatError):
                decode(data[:length])

    def test_invalid_directory(self):
        original = encode([Section(7, 1, False, b"abc"), Section(8, 1, False, b"d")])
        mutations = [(16, "H", 0), (18, "H", 0), (20, "I", 2),
                     (24, "I", 0), (24, "I", 0xffffffff), (28, "I", 0xffffffff),
                     (32, "H", 7), (40, "I", 48), (40, "I", 52)]
        for offset, code, value in mutations:
            data = bytearray(original)
            struct.pack_into("<" + code, data, offset, value)
            with self.subTest(offset=offset, value=value), self.assertRaises(FormatError):
                decode(data)

    def test_version_size_and_limits(self):
        original = encode([])
        for offset, code, value in [(4, "H", 1), (6, "H", 2), (8, "I", 65), (12, "I", 0)]:
            data = bytearray(original)
            struct.pack_into("<" + code, data, offset, value)
            with self.assertRaises(FormatError):
                decode(data)
        for data in (b"NOPE" + original[4:], original + b"x", b"x" * (MAX_SIZE + 1)):
            with self.assertRaises(FormatError):
                decode(data)
        with self.assertRaises(FormatError):
            encode([Section(1, 1, False, b"x" * MAX_SIZE)])
        with self.assertRaises(FormatError):
            encode([Section(1, 1, False, b"")] * 65)

    def test_empty_and_zero_length_sections(self):
        for sections in ([], [Section(1, 1, False, b""), Section(2, 1, False, b"")]):
            self.assertEqual(decode(encode(sections)), sections)


if __name__ == "__main__":
    unittest.main()
