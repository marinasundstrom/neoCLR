"""Guard source ownership and dependency direction across relocated API projects."""
from pathlib import Path
import unittest
import json
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]


class ClassLibraryLayout(unittest.TestCase):
    def projects(self):
        return sorted((ROOT / 'runtime').glob('System.*/src/*/*.rvnproj')) + sorted(
            (ROOT / 'extensions').glob('Neo.*/src/*/*.rvnproj'))

    def test_projects_own_sources_and_resolve_references(self):
        owners = {}
        for project in self.projects():
            tree = ET.parse(project)
            sources = set()
            for item in tree.findall('.//Compile'):
                if '$(' in item.attrib['Include']:
                    continue
                included = set(project.parent.glob(item.attrib['Include']))
                self.assertTrue(included, (project, item.attrib['Include']))
                for exclude in item.attrib.get('Exclude', '').split(';'):
                    if exclude:
                        included -= set(project.parent.glob(exclude))
                sources |= {p.resolve() for p in included}
            self.assertTrue(sources, project)
            for source in sources:
                if source.name == 'RuntimeAwait.rvn':
                    continue  # Shared SDK source template, intentionally compiled per consumer.
                self.assertNotIn(source, owners, (source, owners.get(source), project))
                owners[source] = project
            for reference in tree.findall('.//ProjectReference'):
                target = (project.parent / reference.attrib['Include']).resolve()
                self.assertTrue(target.is_file(), (project, target))
                if project.is_relative_to(ROOT / 'runtime'):
                    self.assertTrue(target.is_relative_to(ROOT / 'runtime'), (project, target))
        authored = {p.resolve() for parent, pattern in (('runtime', 'System.*/src'), ('extensions', 'Neo.*/src'))
                    for source in (ROOT / parent).glob(pattern) for p in source.rglob('*.rvn')
                    if not {'bin', 'obj'}.intersection(p.parts) and p.name != 'ObjectIntrospection.rvn'}
        self.assertTrue(authored)
        self.assertFalse(authored - owners.keys(), authored - owners.keys())

    def test_runtime_suite_catalog_resolves_sources(self):
        catalog = json.loads((ROOT / 'scripts/runtime-test-projects.json').read_text())
        for name, relative in catalog.items():
            project = ROOT / relative
            self.assertTrue(project.is_file(), (name, project))
            self.assertIn('test', project.parts)
            for item in ET.parse(project).findall('.//Compile'):
                include = item.attrib['Include']
                if '$(' not in include:
                    self.assertTrue(list(project.parent.glob(include)), (project, include))

    def test_extension_tests_are_separate_projects(self):
        for name in ('Neo.DependencyInjection', 'Neo.Localization'):
            component = ROOT / 'extensions' / name
            source = component / 'src' / name / (name + '.rvnproj')
            tests = component / 'test' / (name + '.Tests') / (name + '.Tests.rvnproj')
            self.assertTrue(source.is_file())
            self.assertTrue(tests.is_file())
            for item in ET.parse(tests).findall('.//Compile'):
                include = item.attrib['Include']
                if '$(' not in include:
                    self.assertTrue((tests.parent / include).is_file(), (tests, include))
            self.assertEqual(name + '.Tests', ET.parse(tests).findtext('.//AssemblyName'))


if __name__ == '__main__':
    unittest.main()
