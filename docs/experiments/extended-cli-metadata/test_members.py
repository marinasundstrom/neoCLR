from dataclasses import replace
from pathlib import Path
import json
import struct
import subprocess
import sys
import tempfile
import unittest

from codec import FormatError, Section, decode, encode
from members import MemberRef, decode_members, encode_members, read_members, resolve_members
from references import Bindings, encode_bindings, read_profile
from signatures import Context, Node, encode_signature
from test_references import CATALOG, fixture as reference_fixture

I = Node('int32')
S = Node('string')
U = Node('unit')


def image(root, members):
    return encode([Section(2, 1, True, encode_bindings(Bindings(()))),
                   Section(3, 1, True, encode_signature(root, references=True)),
                   Section(4, 1, True, encode_members(members))], {2: 1, 3: 1, 4: 1})


def describe(root, members):
    return resolve_members(members, root, Context(), Bindings(()), {})


class MemberTests(unittest.TestCase):
    def test_golden_table(self):
        members = (MemberRef('tuple_element', element=1), MemberRef('tuple_deconstruct'))
        golden = bytes.fromhex('0200' '010002000100' '010003000000')
        self.assertEqual(encode_members(members), golden)
        self.assertEqual(decode_members(golden), members)
        self.assertEqual(decode_members(encode_members(())), ())

    def test_array_contract(self):
        member = MemberRef('array_length')
        owned, reference = [describe(Node(kind, (I,)), [member])[0]
                            for kind in ('array', 'array_ref')]
        self.assertEqual(owned.result, ('intrinsic', 'native_uint'))
        self.assertEqual(owned.parameters, ())
        self.assertFalse(owned.no_result)
        self.assertNotEqual(owned.identity, reference.identity)

    def test_tuple_contracts(self):
        root = Node('tuple', (I, S))
        first, second, deconstruct = describe(root, [MemberRef('tuple_element'),
            MemberRef('tuple_element', element=1), MemberRef('tuple_deconstruct')])
        self.assertEqual(first.result[0], 'int32')
        self.assertEqual(second.result[0], 'string')
        self.assertNotEqual(first.identity, second.identity)
        self.assertEqual(deconstruct.parameters, (first.result, second.result))
        self.assertEqual(deconstruct.modes, ('out', 'out'))
        self.assertTrue(deconstruct.no_result)

    def test_function_contracts(self):
        member = MemberRef('function_invoke')
        for mode in ('value', 'ref', 'readonly_ref', 'out', 'out_when_true'):
            root = Node('function', (I, Node('bool')), modes=(mode,))
            result, = describe(root, [member])
            self.assertEqual(result.parameters[0][0], 'int32')
            self.assertEqual(result.result[0], 'bool')
            self.assertEqual(result.modes, (mode,))
        inhabited, absent = [describe(Node('function', (U,), no_result=flag), [member])[0]
                             for flag in (False, True)]
        self.assertNotEqual(inhabited.identity, absent.identity)
        self.assertFalse(inhabited.no_result)
        self.assertTrue(absent.no_result)

    def test_renumbered_owner_identity(self):
        member = MemberRef('function_invoke')
        keys = []
        for reorder in (False, True):
            sections = decode(reference_fixture(reorder), {2: 1, 3: 1})
            sections.append(Section(4, 1, True, encode_members([member])))
            profile = read_profile(sections)
            members = read_members(sections, profile)
            keys.append(resolve_members(members, *profile, CATALOG)[0])
            with self.assertRaisesRegex(FormatError, 'unresolved definition'):
                resolve_members(members, *profile, {})
        self.assertEqual(keys[0], keys[1])

    def test_wrong_shapes_and_indices(self):
        for root, member in [
            (I, MemberRef('array_length')),
            (Node('array', (I,)), MemberRef('tuple_element')),
            (Node('tuple', (I,)), MemberRef('tuple_element', element=1)),
            (Node('tuple', (I,)), MemberRef('function_invoke')),
            (Node('function', (U,)), MemberRef('tuple_deconstruct')),
        ]:
            sections = decode(image(root, [member]), {2: 1, 3: 1, 4: 1})
            with self.assertRaises(FormatError):
                read_members(sections, read_profile(sections))
            with self.assertRaises(FormatError):
                describe(root, [member])
        for member in (MemberRef('Invoke'), MemberRef('array_length', owner=0),
                       MemberRef('array_length', owner=2), MemberRef('array_length', element=1),
                       MemberRef('tuple_element', element=256)):
            with self.assertRaises(FormatError):
                encode_members([member])

    def test_malformed_table(self):
        golden = encode_members([MemberRef('tuple_element'), MemberRef('tuple_deconstruct')])
        for size in range(len(golden)):
            with self.subTest(size=size), self.assertRaises(FormatError):
                decode_members(golden[:size])
        for offset, code, value in [(0, 'H', 257), (2, 'H', 2), (4, 'B', 255),
                                    (5, 'B', 1), (12, 'H', 1)]:
            data = bytearray(golden)
            struct.pack_into('<' + code, data, offset, value)
            with self.assertRaises(FormatError):
                decode_members(data)
        with self.assertRaises(FormatError):
            decode_members(golden + b'x')
        with self.assertRaises(FormatError):
            encode_members([MemberRef('array_length')] * 2)
        with self.assertRaises(FormatError):
            encode_members([MemberRef('tuple_element')] * 257)
        duplicate = bytes.fromhex('0200') + golden[2:8] * 2
        with self.assertRaisesRegex(FormatError, 'duplicate'):
            decode_members(duplicate)

    def test_profile_and_inspector(self):
        members = (MemberRef('tuple_element', element=1), MemberRef('tuple_deconstruct'))
        raw = image(Node('tuple', (I, S)), members)
        fixture = Path(__file__).parent / 'fixtures' / 'tuple-members.neox'
        self.assertEqual(fixture.read_bytes(), raw)
        sections = decode(raw, {2: 1, 3: 1, 4: 1})
        profile = read_profile(sections)
        self.assertEqual(read_members(list(reversed(sections)), profile), members)
        self.assertEqual(encode(sections, {2: 1, 3: 1, 4: 1}), raw)
        for bad, owner in [([sections[-1]], None),
                           ([replace(sections[-1], required=False)], profile)]:
            with self.assertRaises(FormatError):
                read_members(bad, owner)
        self.assertIsNone(read_members([Section(4, 99, False, b'opaque')], None))
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'members.neox'
            path.write_bytes(raw)
            command = [sys.executable, str(Path(__file__).with_name('codec.py')), str(path)]
            result = subprocess.run(command, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            output = json.loads(result.stdout)['sections'][-1]
            self.assertTrue(output['owner_shape_validated'])
            self.assertFalse(output['resolved'])
            self.assertEqual(output['members'][0]['operation'], 'tuple_element')
            path.write_bytes(image(I, [MemberRef('function_invoke')]))
            result = subprocess.run(command, capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            self.assertIn('owner shape', result.stderr)
            self.assertEqual(result.stdout, '')


if __name__ == '__main__':
    unittest.main()
