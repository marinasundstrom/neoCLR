import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

from codec import FormatError, Section, decode, encode
from signatures import Context, Node, decode_signature, encode_signature

I = Node("int32")


def nested():
    return Node("function", (
        Node("array", (Node("tuple", (I, Node("string"))),)),
        Node("intersection", (Node("type_parameter", index=0), Node("self"))),
        Node("union", (Node("nullable", (Node("method_parameter", index=0),)), Node("unit")))
    ), modes=("value", "readonly_ref"))


class SignatureTests(unittest.TestCase):
    def test_golden_function(self):
        # context(0,0,false), Function(length=15), convention/count, mode, int32, string
        golden = bytes.fromhex("0000000000" "0a0f000000" "00000100" "00"
                               "0100000000" "0200000000")
        tree = Node("function", (I, Node("string")), modes=("value",))
        self.assertEqual(encode_signature(tree), golden)
        self.assertEqual(decode_signature(golden), (tree, Context()))

    def test_nested_types_and_context(self):
        context = Context(1, 1, True)
        data = encode_signature(nested(), context)
        self.assertEqual(decode_signature(data), (nested(), context))
        image = encode([Section(1, 1, True, data)], {1: 1})
        fixture = Path(__file__).parent / "fixtures" / "nested.neox"
        self.assertEqual(image, fixture.read_bytes())
        self.assertEqual(decode_signature(decode(image, {1: 1})[0].payload)[0], nested())
        for size in range(len(data)):
            with self.subTest(size=size), self.assertRaises(FormatError):
                decode_signature(data[:size])

    def test_parameter_modes_and_order_preserved(self):
        for mode in ("value", "ref", "readonly_ref", "out"):
            tree = Node("function", (I, Node("unit")), modes=(mode,))
            self.assertEqual(decode_signature(encode_signature(tree))[0], tree)
        tree = Node("union", (Node("string"), I, I))
        self.assertEqual(decode_signature(encode_signature(tree))[0], tree)

    def test_invalid_context_and_shapes(self):
        for tree in (Node("type_parameter"), Node("method_parameter"), Node("self"),
                     Node("tuple"), Node("union", (I,)), Node("intersection"),
                     Node("array"), Node("function", (I, I)), Node("unknown"),
                     Node("int32", (I,)), Node("int32", index=1),
                     Node("function", (I, I), modes=("unknown",))):
            with self.subTest(tree=tree), self.assertRaises(FormatError):
                encode_signature(tree)
        for context in (Context(-1), Context(257), Context(self_allowed=2)):
            with self.assertRaises(FormatError):
                encode_signature(I, context)

    def test_malformed_payloads(self):
        golden = encode_signature(Node("function", (I, I), modes=("value",)))
        for offset, value in [(4, 2), (5, 255), (6, 255), (10, 1), (14, 255), (15, 255)]:
            data = bytearray(golden)
            data[offset] = value
            with self.subTest(offset=offset), self.assertRaises(FormatError):
                decode_signature(data)
        for data in (encode_signature(I) + b"x",
                     bytes.fromhex("0000000000" "05020000000000"),
                     bytes.fromhex("0000000000" "0700000000")):
            with self.assertRaises(FormatError):
                decode_signature(data)

    def test_resource_limits(self):
        tree = I
        for _ in range(34):
            tree = Node("array", (tree,))
        for tree in (tree, Node("tuple", (I,) * 257),
                     Node("tuple", (Node("tuple", (I,) * 256),) * 16)):
            with self.assertRaises(FormatError):
                encode_signature(tree)
        # Malicious nesting must also be bounded while decoding, before tree validation.
        import struct
        raw = bytes.fromhex("0100000000")
        for _ in range(34):
            raw = struct.pack("<BI", 8, len(raw)) + raw
        with self.assertRaisesRegex(FormatError, "resource limit"):
            decode_signature(b"\0" * 5 + raw)

    def test_structural_branch_contracts(self):
        unit_result = Node("function", (Node("unit"),))
        no_result = Node("function", (Node("unit"),), no_result=True)
        self.assertNotEqual(encode_signature(unit_result), encode_signature(no_result))
        for tree in (no_result, Node("function", (I, Node("bool")), modes=("out_when_true",)),
                     Node("array_ref", (I,))):
            self.assertEqual(decode_signature(encode_signature(tree))[0], tree)
        self.assertNotEqual(encode_signature(Node("array", (I,))),
                            encode_signature(Node("array_ref", (I,))))
        for tree in (Node("function", (I,), no_result=True),
                     Node("function", (I, I), modes=("out_when_true",)),
                     Node("int32", no_result=True)):
            with self.assertRaises(FormatError):
                encode_signature(tree)

    def test_inspector_process(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "nested.neox"
            data = encode_signature(nested(), Context(1, 1, True))
            path.write_bytes(encode([Section(1, 1, True, data)], {1: 1}))
            command = [sys.executable, str(Path(__file__).with_name("codec.py")), str(path)]
            result = subprocess.run(command, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            output = json.loads(result.stdout)
            self.assertFalse(output["executable"])
            self.assertEqual(output["sections"][0]["signature"]["kind"], "function")
            path.write_bytes(encode([Section(1, 1, True, b"bad")], {1: 1}))
            result = subprocess.run(command, capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            self.assertIn("invalid signature size", result.stderr)
            self.assertEqual(result.stdout, "")


if __name__ == "__main__":
    unittest.main()
