from dataclasses import replace
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import unittest
from uuid import UUID

from codec import FormatError, Section, decode, encode
from references import (Bindings, Definition, Reference, decode_bindings, encode_bindings,
                        read_profile, resolve)
from signatures import Context, Node, decode_signature, encode_signature

ASSEMBLY = UUID('11111111-1111-1111-1111-111111111111')
MODULE = UUID('22222222-2222-2222-2222-222222222222')
TYPE = Reference(ASSEMBLY, MODULE, 0x02000001)
CONTRACT = Reference(ASSEMBLY, MODULE, 0x02000002)
METHOD = Reference(ASSEMBLY, MODULE, 0x06000001)
OTHER_METHOD = Reference(ASSEMBLY, MODULE, 0x06000002)
CATALOG = {TYPE: Definition('type', 1), CONTRACT: Definition('interface'),
           METHOD: Definition('method', 1, TYPE), OTHER_METHOD: Definition('method', 1, TYPE)}
CONTEXT = Context(1, 1, True)


def fixture(reordered=False):
    refs = (CONTRACT, METHOD, TYPE) if reordered else (TYPE, CONTRACT, METHOD)
    type_index, contract_index, method_index = (3, 1, 2) if reordered else (1, 2, 3)
    bindings = Bindings(refs, type_index, method_index, contract_index)
    nominal = Node('nominal', (Node('type_parameter'),), index=type_index)
    root = Node('function', (nominal, Node('tuple', (Node('self'), Node('method_parameter')))),
                modes=('readonly_ref',))
    sections = [Section(2, 1, True, encode_bindings(bindings)),
                Section(3, 1, True, encode_signature(root, CONTEXT, references=True))]
    if reordered:
        sections.reverse()
    return encode(sections, {2: 1, 3: 1})


def resolved(image, catalog=CATALOG):
    return resolve(*read_profile(decode(image, {2: 1, 3: 1})), catalog)


class ReferenceTests(unittest.TestCase):
    def test_golden_reference_and_nominal(self):
        raw = bytes.fromhex('0100000000000000' + '11' * 16 + '22' * 16 + '01000002')
        bindings = Bindings((TYPE,))
        self.assertEqual(encode_bindings(bindings), raw)
        self.assertEqual(decode_bindings(raw), bindings)
        node = Node('nominal', (Node('int32'),), index=1)
        golden = bytes.fromhex('0000000000' '0f09000000' '01000100' '0100000000')
        self.assertEqual(encode_signature(node, references=True), golden)
        self.assertEqual(decode_signature(golden, references=True)[0], node)
        with self.assertRaisesRegex(FormatError, 'reference profile'):
            decode_signature(golden)

    def test_cross_module_reference_renumbering(self):
        first, second = fixture(), fixture(True)
        self.assertNotEqual(first, second)
        self.assertEqual(resolved(first), resolved(second))
        for name, image in [('references-a.neox', first), ('references-b.neox', second)]:
            path = Path(__file__).parent / 'fixtures' / name
            self.assertEqual(path.read_bytes(), image)
            sections = decode(image, {2: 1, 3: 1})
            self.assertEqual(encode(sections, {2: 1, 3: 1}), image)

    def test_owner_and_scope_identity(self):
        root, context, bindings = read_profile(decode(fixture(), {2: 1, 3: 1}))
        baseline = resolve(root, context, bindings, CATALOG)
        for old, new in [(METHOD, OTHER_METHOD),
                         (TYPE, replace(TYPE, assembly=UUID(int=10))),
                         (TYPE, replace(TYPE, module=UUID(int=11))),
                         (CONTRACT, replace(CONTRACT, token=0x02000003))]:
            refs = tuple(new if ref == old else ref for ref in bindings.references)
            catalog = dict(CATALOG)
            catalog[new] = catalog[old]
            if old == TYPE:
                catalog[METHOD] = replace(catalog[METHOD], owner=new)
            key = resolve(root, context, replace(bindings, references=refs), catalog)
            self.assertNotEqual(baseline, key)

    def test_reference_table_rejections(self):
        raw = encode_bindings(Bindings((TYPE,), type_owner=1))
        for size in range(len(raw)):
            with self.subTest(size=size), self.assertRaises(FormatError):
                decode_bindings(raw[:size])
        for bindings in (Bindings((TYPE, TYPE)), Bindings((TYPE,), type_owner=2),
                         Bindings((TYPE,), method_owner=1),
                         Bindings((replace(TYPE, token=0x01000001),)),
                         Bindings((replace(TYPE, token=0x02000000),)),
                         Bindings((replace(TYPE, assembly=UUID(int=0)),)),
                         Bindings((TYPE,) * 257)):
            with self.assertRaises(FormatError):
                encode_bindings(bindings)
        for offset, code, value in [(0, 'H', 257), (2, 'H', 2), (40, 'I', 0x04000001)]:
            broken = bytearray(raw)
            struct.pack_into('<' + code, broken, offset, value)
            with self.assertRaises(FormatError):
                decode_bindings(broken)

    def test_resolution_rejections(self):
        root, context, bindings = read_profile(decode(fixture(), {2: 1, 3: 1}))
        for catalog in ({}, {**CATALOG, TYPE: Definition('type', 2)},
                        {**CATALOG, CONTRACT: Definition('type')},
                        {**CATALOG, CONTRACT: Definition('interface', 1)},
                        {**CATALOG, METHOD: Definition('method', 1, CONTRACT)},
                        {**CATALOG, TYPE: Definition('method', 1, TYPE)}):
            with self.assertRaises(FormatError):
                resolve(root, context, bindings, catalog)
        for bad in (replace(bindings, type_owner=0), replace(bindings, self_owner=0)):
            with self.assertRaises(FormatError):
                resolve(root, context, bad, CATALOG)
        for node in (Node('nominal', index=4), Node('nominal', index=3),
                     Node('nominal', index=1)):
            with self.assertRaises(FormatError):
                resolve(node, context, bindings, CATALOG)

    def test_bounded_compound_identity(self):
        integer, string = Node('int32'), Node('string')
        for kind in ('union', 'intersection'):
            left = Node(kind, (integer, Node(kind, (string, integer))))
            right = Node(kind, (string, integer))
            self.assertEqual(resolve(left, Context(), Bindings(()), {}),
                             resolve(right, Context(), Bindings(()), {}))
        self.assertNotEqual(resolve(Node('tuple', (integer, string)), Context(), Bindings(()), {}),
                            resolve(Node('tuple', (string, integer)), Context(), Bindings(()), {}))

    def test_function_and_array_identity_distinctions(self):
        integer, unit = Node('int32'), Node('unit')
        groups = [
            [Node('function', (integer, unit), modes=(mode,))
             for mode in ('value', 'ref', 'readonly_ref', 'out')],
            [Node('function', (unit,)), Node('function', (unit,), no_result=True)],
            [Node('array', (integer,)), Node('array_ref', (integer,))],
        ]
        for group in groups:
            keys = [resolve(node, Context(), Bindings(()), {}) for node in group]
            self.assertEqual(len(set(keys)), len(group))

    def test_profiles_and_inspector(self):
        sections = decode(fixture(), {2: 1, 3: 1})
        for bad in (sections[:1], sections[1:], [replace(sections[0], required=False), sections[1]],
                    sections + [Section(1, 1, True, encode_signature(Node('int32')))]):
            with self.assertRaises(FormatError):
                read_profile(bad)
        self.assertIsNone(read_profile([Section(2, 99, False, b'opaque')]))
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'reference.neox'
            path.write_bytes(fixture())
            command = [sys.executable, str(Path(__file__).with_name('codec.py')), str(path)]
            result = subprocess.run(command, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn('"resolved": false', result.stdout)
            path.write_bytes(encode(sections[:1], {2: 1}))
            result = subprocess.run(command, capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            self.assertIn('incomplete reference profile', result.stderr)


if __name__ == '__main__':
    unittest.main()
