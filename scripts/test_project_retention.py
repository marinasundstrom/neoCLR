"""Project retention resolves fresh identities and keeps capability boundaries."""
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


def load(name, file):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(file))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


builder = load('native_builder', 'build-native-project.py')
runner = load('native_runner', 'run-native-sample.py')


class ProjectRetentionTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix='project retention ')
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name).resolve()
        self.project = self.root / 'App.rvnproj'
        self.file = self.project.with_suffix('.retention.json')
        self.entry = {'name': 'Example.Service', 'retain': ['constructors']}

    def write(self, entries=None):
        self.file.write_text(json.dumps({'schemaVersion': 1, 'types': entries or [self.entry]}))

    def test_optional_default_and_explicit_project_relative_path(self):
        self.assertEqual(builder.project_retention(self.project), (None, None, None))
        self.write()
        self.assertEqual(builder.project_retention(self.project), (self.file, [self.entry], hashlib.sha256(self.file.read_bytes()).hexdigest()))
        renamed = self.root / 'custom.json'
        self.file.rename(renamed)
        self.assertEqual(builder.project_retention(self.project, Path('custom.json')), (renamed, [self.entry], hashlib.sha256(renamed.read_bytes()).hexdigest()))
        with self.assertRaises(FileNotFoundError):
            builder.project_retention(self.project, Path('missing.json'))

    def test_rejects_malformed_or_unsupported_declarations(self):
        valid = {'schemaVersion': 1, 'types': [self.entry]}
        invalid = [[], {}, {**valid, 'schemaVersion': True}, {**valid, 'schemaVersion': 2},
                   {**valid, 'extra': True}, {**valid, 'types': []},
                   {**valid, 'types': [self.entry, self.entry]}]
        for entry in ({'name': '', 'retain': ['constructors']},
                      {'name': ' Example.Service', 'retain': ['constructors']},
                      {'name': 'Example.Service', 'retain': ['all']},
                      {'name': 'Example.Service', 'retain': ['methods', 'methods']},
                      {'name': 'Example.Service', 'retain': [False]},
                      {'name': 'Example.Service', 'retain': []},
                      {**self.entry, 'assembly': 'Unknown'}):
            invalid.append({**valid, 'types': [entry]})
        for policy in invalid:
            with self.subTest(policy=policy):
                self.file.write_text(json.dumps(policy))
                with self.assertRaises(ValueError):
                    builder.project_retention(self.project)
        self.file.write_text('{"schemaVersion":1,"schemaVersion":1,"types":[]}')
        with self.assertRaisesRegex(ValueError, 'Duplicate'):
            builder.project_retention(self.project)

    def test_conflicting_flags_and_unsupported_profile_fail_before_output(self):
        self.write()
        for options in ({'reflection_types': ['Example.Service']},
                        {'reflection_roots': self.root / 'private.json'},
                        {'profile': builder.PROFILES['native-interop']}):
            with self.subTest(options=options), self.assertRaises(ValueError):
                builder.build(self.project, self.root, self.root, self.root / 'out', **options)
            self.assertFalse((self.root / 'out').exists())

    def resolve(self, entries, app_types, library_types=()):
        assembly = self.root / 'App.dll'
        library = self.root / 'Library.dll'
        def disassemble(command):
            is_app = command[2] == assembly
            command[-1].write_text(json.dumps(dict(name='App' if is_app else 'Library',
                revision=3, types=app_types if is_app else library_types)))
        path = builder.resolve_project_retention('runtime', assembly, entries, disassemble,
                                                [library], runner.reflection_roots)
        return json.loads(path.read_text())

    def test_combined_modes_resolve_fresh_ordinals_and_merge_one_root(self):
        entries = [{'name': 'Example.Service', 'retain': ['constructors', 'properties', 'methods']}]
        service = dict(name='opaque', origin=dict(name='Example.Service'))
        other = dict(name='other', origin=dict(name='Example.Other'))
        for types, ordinal in (([service, other], 0), ([other, service], 1)):
            policy = self.resolve(entries, types)
            self.assertEqual(policy['schemaVersion'], 8)
            self.assertEqual(len(policy['types']), 1)
            root = policy['types'][0]
            self.assertEqual(root['definition'], dict(module='App', revision=3, index=ordinal))
            self.assertEqual(policy['methodTypes'], [root['definition']])
            for flag in ('construct', 'constructors', 'properties', 'getters', 'setters'):
                self.assertTrue(root[flag])
            for flag in ('customAttributes', 'fields', 'fieldGetters', 'fieldSetters'):
                self.assertFalse(root[flag])

    def test_library_generic_adapter_and_method_only_capability(self):
        entries = [dict(name='DefaultLocalizer`1', retain=['constructors']),
                   dict(name='Example.Service', retain=['methods'])]
        policy = self.resolve(entries, [dict(name='opaque', origin=dict(name='Example.Service'))],
                              [dict(name='generic', origin=dict(name='DefaultLocalizer`1'))])
        self.assertEqual(len(policy['types']), 1)
        self.assertEqual(policy['types'][0]['definition']['module'], 'Library')
        self.assertFalse(policy['types'][0]['properties'])
        self.assertEqual(policy['methodTypes'], [dict(module='App', revision=3, index=0)])

    def test_missing_and_ambiguous_names_fail(self):
        for types in ([], [dict(name='a', origin=dict(name='Example.Service')),
                           dict(name='b', origin=dict(name='Example.Service'))]):
            with self.assertRaisesRegex(ValueError, 'Expected one reflection root'):
                self.resolve([self.entry], types)

    def test_attribute_only_type_retention_does_not_grant_activation(self):
        entries = [dict(name='Example.Handler', retain=['customAttributes'])]
        policy = self.resolve(entries, [dict(name='handler', origin=dict(name='Example.Handler'))])
        root = policy['types'][0]
        self.assertTrue(root.pop('customAttributes'))
        root.pop('definition')
        self.assertTrue(all(flag is False for flag in root.values()))
        self.assertEqual(policy['methodTypes'], [])

    def test_version_two_module_selection_is_explicit_and_validated(self):
        module = dict(assembly='Logic', name='Example.Web', retain=['customAttributes'])
        policy = dict(schemaVersion=2, types=[self.entry], modules=[module])
        self.file.write_text(json.dumps(policy))
        self.assertEqual(builder.project_retention(self.project)[1], [self.entry, module])
        for modules in ([module, module], [dict(module, retain=['constructors'])],
                        [dict(module, assembly='')], [dict(module, extra=True)]):
            self.file.write_text(json.dumps(dict(policy, modules=modules)))
            with self.assertRaises(ValueError):
                builder.project_retention(self.project)

    def test_module_attributes_resolve_assembly_and_exact_declaration_scope(self):
        assembly = self.root / 'App.dll'
        library = self.root / 'Logic.dll'
        def run(command):
            owners = [] if command[2] == assembly else [dict(name='Logic', full_name='Logic,1',
                declaration_modules=dict(names=['Example.Web', 'Example.Web.Child']))]
            command[-1].write_text(json.dumps(dict(name='fixture', revision=1, types=[], assemblies=owners)))
        entry = dict(assembly='Logic', name='Example.Web', retain=['customAttributes'])
        path = builder.resolve_project_retention('runtime', assembly, [entry], run,
                                                [library], runner.reflection_roots)
        policy = json.loads(path.read_text())
        self.assertEqual(policy['types'], [])
        self.assertEqual(policy['methodTypes'], [])
        self.assertEqual(policy['moduleCatalogs'], ['Logic,1'])
        self.assertEqual(policy['moduleFunctions'], [dict(assembly='Logic,1', name='Example.Web', customAttributes=True)])
        for invalid in (dict(entry, name='Example'), dict(entry, assembly='Missing')):
            with self.assertRaisesRegex(ValueError, 'Expected one declared module'):
                builder.resolve_project_retention('runtime', assembly, [invalid], run, [library], runner.reflection_roots)
        with self.assertRaisesRegex(ValueError, 'Expected one declared module'):
            builder.resolve_project_retention('runtime', assembly, [entry], run, [library, library], runner.reflection_roots)


if __name__ == '__main__':
    unittest.main()
