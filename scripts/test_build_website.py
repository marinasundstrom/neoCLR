"""Cross-page validation for the static feature guide builder."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('website_build', Path(__file__).with_name('build-website.py'))
build = importlib.util.module_from_spec(spec)
spec.loader.exec_module(build)


class PageLinks(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.previous_output = build.OUTPUT
        self.addCleanup(setattr, build, 'OUTPUT', self.previous_output)
        build.OUTPUT = Path(self.temporary.name).resolve()
        self.pages = {}

    def page(self, name, html):
        path = build.OUTPUT / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(html)
        parser = build.PageCheck()
        parser.feed(html)
        self.pages[path] = parser
        return path, parser

    def test_nested_pages_resolve_directory_links_and_cross_page_anchors(self):
        root, home = self.page('index.html', '<main id="library"><a href="features/info/#types">Guide</a></main>')
        path, guide = self.page('features/info/index.html', '<main id="types"><a href="../../#library">Home</a><a href="#types">Types</a></main>')
        home.check(root, self.pages)
        guide.check(path, self.pages)

    def test_missing_cross_page_anchor_fails(self):
        self.page('index.html', '<main id="library"></main>')
        path, guide = self.page('features/info/index.html', '<a href="../../#missing">Home</a>')
        with self.assertRaisesRegex(ValueError, 'Missing anchor'):
            guide.check(path, self.pages)

    def test_downloads_must_exist_inside_public_output(self):
        path, page = self.page('index.html', '<a href="samples/demo.rvn">Source</a>')
        with self.assertRaisesRegex(ValueError, 'Missing local asset'):
            page.check(path, self.pages)
        (build.OUTPUT / 'samples').mkdir()
        (build.OUTPUT / 'samples/demo.rvn').write_text('func Main() {}')
        page.check(path, self.pages)
        path, page = self.page('index.html', '<a href="../private.txt">Outside</a>')
        with self.assertRaisesRegex(ValueError, 'Missing local asset'):
            page.check(path, self.pages)


    def test_reference_links_work_under_a_project_base_path(self):
        from urllib.parse import urljoin
        self.page('index.html', '<main>Platform</main>')
        self.page('features/tasks/index.html', '<main id="await">Tasks</main>')
        path, _ = self.page('docs/api/Task.html',
            '<a href="/features/tasks/index.html?one=1&amp;two=2#await">Guide</a>'
            '<a href="/index.html">Home</a><a href="//example.com/help">External</a>')
        import json
        toc = build.OUTPUT / 'docs/toc.json'
        toc.write_text(json.dumps({'items': [{'href': '/index.html', 'topicHref': '/index.html'}]}))
        build.make_reference_links_relative()
        self.assertEqual(json.loads(toc.read_text())['items'][0],
                         {'href': '../index.html', 'topicHref': '../index.html'})
        check = build.PageCheck()
        check.feed(path.read_text())
        self.assertEqual(urljoin('https://example.org/neoCLR/docs/api/Task.html', check.links[0]),
                         'https://example.org/neoCLR/features/tasks/index.html?one=1&two=2#await')
        self.assertEqual(check.links[1:], ['../../index.html', '//example.com/help'])
        build.check_reference_links()


class RavenDocPages(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        import importlib.util
        spec = importlib.util.spec_from_file_location('ravendoc', Path(__file__).with_name('ravendoc.py'))
        runner = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(runner)
        cls.runtime = runner.runtime()

    def publish(self, content, extension='.md', extra=None):
        import json
        import subprocess
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        source = root / ('page' + extension)
        source.write_text(content)
        config = dict(name='Example', subtitle='Experimental', footer='Development',
                      output=str(root / 'site'), pages=[dict(source=str(source), output='index.html', title='Fallback')],
                      links=[dict(label='Guides', url='guides/')], apiNavigationRoot='docs', notice='May document unreleased APIs.',
                      releaseUrl='https://example.com/releases', releaseLabel='Published release')
        config.update(extra or {})
        manifest = root / 'site.json'
        manifest.write_text(json.dumps(config))
        result = subprocess.run(['dotnet', str(self.runtime), '--site', str(manifest)], capture_output=True, text=True)
        return result, root / 'site'

    def test_scaffold_types_are_excluded_but_generic_apis_remain(self):
        import json
        repo = Path(__file__).resolve().parent.parent
        exclusions = json.loads((repo / 'api-docs/exclusions.json').read_text())
        result, output = self.publish('# Reference', extra=dict(
            api=str(repo / 'api-docs/reference/NeoCLR.CoreProbe.dll'),
            excludedMembers=list(exclusions)))
        self.assertEqual(result.returncode, 0, result.stderr)
        xrefs = {uid.replace('+', '.'): path for uid, path in
                 json.loads((output / 'xref-map.json').read_text()).items()}
        for name in ('System.Array', 'System.Option', 'System.Result', 'System.Tasks.TaskOutcome',
                     'System.Option.Some`1', 'System.Result.Ok`1', 'System.Tasks.TaskOutcome.Cancelled'):
            self.assertNotIn('T:' + name, xrefs)
        for name in ('System.Array`1', 'System.Option`1', 'System.Result`2',
                     'System.Tasks.TaskOutcome`1'):
            self.assertIn('T:' + name, xrefs)
            self.assertTrue((output / xrefs['T:' + name].split('#')[0]).is_file(), name)

    def test_api_list_labels_and_signature_opt_in(self):
        import shutil
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        assembly = Path(temporary.name) / 'NeoCLR.CoreProbe.dll'
        repo = Path(__file__).resolve().parent.parent
        shutil.copyfile(repo / 'api-docs/reference/NeoCLR.CoreProbe.dll', assembly)
        shutil.copyfile(repo / 'api-docs/NeoCLR.CoreProbe.xml', assembly.with_suffix('.xml'))
        config = dict(api=str(assembly), types=['System.Storage.Path', 'System.Introspection.BindingFlags', 'System.Console', 'System.Introspection.TypeInfo', 'System.IO.StreamError', 'System.Storage.EntryKind', 'System.Storage.File'])
        result, output = self.publish('# Reference', extra=config)
        self.assertEqual(result.returncode, 0, result.stderr)
        namespace = (output / 'api/System/Storage/index.html').read_text()
        self.assertIn('class="member-name">Path</span>', namespace)
        path = (output / 'api/System/Storage/Path/index.html').read_text()
        self.assertIn('class="member-name">Text: string</span>', path)
        self.assertIn('API Browser</h2>', path)
        self.assertIn('aria-label="API namespaces and types"', path)
        self.assertIn('<summary title="System.Storage"><span>System.Storage</span></summary>', path)
        self.assertIn('aria-current="location"', path)
        self.assertIn('aria-controls="api-browser"', path)
        self.assertNotIn('id="api-browser"', (output / 'index.html').read_text())
        self.assertEqual(path.count('class="member-name">Equals('), 2)
        self.assertIn(' -&gt; bool</span>', path)
        self.assertNotIn('class="member-name">func ', path)
        self.assertIn('title="Static member"', path)
        self.assertIn('class="symbol-static-marker">S</span>', path)
        self.assertIn('class="visually-hidden">Static member: </span>', path)
        self.assertNotIn('[static]', path)
        console = (output / 'api/System/Console/index.html').read_text()
        self.assertIn('class="member-name">Write(value0: int) -&gt; ()</span>', console)
        self.assertIn('class="member-name">Write(value0: string) -&gt; ()</span>', console)
        introspection = (output / 'api/System/Introspection/index.html').read_text()
        self.assertIn('symbol-icon--interface" title="Interface"', introspection)
        interface = (output / 'api/System/Introspection/TypeInfo/index.html').read_text()
        self.assertIn('symbol-icon--interface" title="Interface"', interface)
        flags = (output / 'api/System/Introspection/BindingFlags/index.html').read_text()
        self.assertIn('class="member-name">Public: BindingFlags</span>', flags)
        self.assertIn('symbol-icon--enum', flags)
        self.assertIn('symbol-icon--class', console)
        union = (output / 'api/System/IO/StreamError/index.html').read_text()
        self.assertIn('union struct StreamError', union)
        self.assertIn('symbol-icon--union', union)
        self.assertIn('case Closed', union)
        self.assertNotIn('<summary title="StreamError">', union)
        self.assertTrue((output / 'api/System/Storage/File/index.html').exists())
        detail = (output / 'api/System/Storage/Path/property_Text.html').read_text()
        self.assertIn('val Text:', detail)
        result, output = self.publish('# Reference', extra={**config, 'memberListStyle': 'signatures', 'apiNavigationRoot': None})
        self.assertEqual(result.returncode, 0, result.stderr)
        path = (output / 'api/System/Storage/Path/index.html').read_text()
        self.assertIn('class="member-signature">val Text:', path)
        self.assertIn('id="api-browser"', (output / 'index.html').read_text())

    def test_markdown_landing_metadata_omits_outline_and_keeps_notice(self):
        result, output = self.publish('---\ntitle: "A landing page"\nlayout: landing\ntoc: false\n---\n# Welcome\n\n## Features\n\nText.')
        self.assertEqual(result.returncode, 0, result.stderr)
        html = (output / 'index.html').read_text()
        self.assertIn('<title>A landing page · Example</title>', html)
        self.assertIn('layout-landing without-outline', html)
        self.assertNotIn('aria-label="On this page"', html)
        self.assertIn('May document unreleased APIs.', html)
        self.assertIn('<h2 id="features">Features</h2>', html)

    def test_html_body_uses_shared_shell_without_markdown_parsing(self):
        result, output = self.publish('---\ntitle: HTML landing\ntoc: false\n---\n<section><h1>Hero</h1><p>**literal HTML**</p></section>', '.html')
        self.assertEqual(result.returncode, 0, result.stderr)
        html = (output / 'index.html').read_text()
        self.assertIn('<section><h1>Hero</h1>', html)
        self.assertIn('**literal HTML**', html)
        self.assertIn('Main navigation', html)
        self.assertIn('May document unreleased APIs.', html)
        self.assertEqual(html.count('<html'), 1)

    def test_guide_retains_outline_by_default(self):
        result, output = self.publish('# Guide\n\n## Details\n\nText.')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('aria-label="On this page"', (output / 'index.html').read_text())

    def test_invalid_front_matter_fails_instead_of_ignoring_controls(self):
        for content in ('---\ntoc: maybe\n---\n# Bad', '---\ntoc: false\ntoc: true\n---\n# Bad',
                        '---\nlayout: unknown\n---\n# Bad', '---\ntoc: false\n# Bad'):
            with self.subTest(content=content):
                result, _ = self.publish(content)
                self.assertNotEqual(result.returncode, 0)

    def test_full_html_document_is_rejected_as_content(self):
        result, _ = self.publish('<!doctype html><html><body>Nested document</body></html>', '.html')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('body fragment', result.stderr)




class PublicApiCoverage(unittest.TestCase):
    def setUp(self):
        import json
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        for name in ('ROOT', 'OUTPUT'):
            self.addCleanup(setattr, build, name, getattr(build, name))
        build.ROOT = self.root
        build.OUTPUT = self.root / 'site'
        build.OUTPUT.mkdir()
        docs = self.root / 'api-docs'
        docs.mkdir()
        (docs / 'types.json').write_text(json.dumps(['Example.Public']))
        (docs / 'exclusions.json').write_text('{}')
        (docs / 'NeoCLR.CoreProbe.xml').write_text('<doc><members /></doc>')
        (build.OUTPUT / 'xref-map.json').write_text(json.dumps({'T:Example.Public': 'public.html'}))

    def test_manual_member_routes_require_existing_page_and_anchor(self):
        import json
        manifest = self.root / 'api-docs/manual-members.json'
        manifest.write_text(json.dumps({'P:Example.Public.Value': {
            'output': 'value.html#value', 'reason': 'Renderer omits case payloads'}}))
        with self.assertRaisesRegex(ValueError, 'Missing manual member entry'):
            build.register_manual_member_routes()
        (build.OUTPUT / 'value.html').write_text('<h1>Value</h1>')
        with self.assertRaisesRegex(ValueError, 'Missing manual member anchor'):
            build.register_manual_member_routes()
        (build.OUTPUT / 'value.html').write_text('<h1 id="value">Value</h1>')
        build.register_manual_member_routes()
        self.assertEqual(json.loads((build.OUTPUT / 'xref-map.json').read_text())[
            'P:Example.Public.Value'], 'value.html#value')

    def test_public_type_without_comments_remains_publishable(self):
        (build.OUTPUT / 'public.html').write_text('<span class="member-summary--empty"></span>')
        build.check_api_coverage()

    def test_public_type_without_page_fails_even_without_comments(self):
        with self.assertRaisesRegex(ValueError, 'Missing public type reference'):
            build.check_api_coverage()

    def test_exact_type_exclusion_does_not_hide_other_missing_types(self):
        import json
        (self.root / 'api-docs/exclusions.json').write_text(json.dumps({'T:Example.Public': 'Scaffold'}))
        build.check_api_coverage()
        (self.root / 'api-docs/types.json').write_text(json.dumps(['Example.Public', 'Example.Public`1']))
        with self.assertRaisesRegex(ValueError, 'Missing public type reference'):
            build.check_api_coverage()

    def test_manual_entries_repair_phantom_routes_without_hiding_types(self):
        import json
        (build.OUTPUT / 'manual.html').write_text('<h1 id="constructor">Public</h1>')
        (self.root / 'api-docs/manual-types.json').write_text(json.dumps({
            'Example.Public': {'output': 'manual.html', 'reason': 'Renderer limitation',
                               'members': {'M:Example.Public.#ctor': 'constructor'}}
        }))
        build.register_manual_api_routes()
        routes = json.loads((build.OUTPUT / 'xref-map.json').read_text())
        self.assertEqual(routes['T:Example.Public'], 'manual.html')
        self.assertEqual(routes['M:Example.Public.#ctor'], 'manual.html#constructor')
        self.assertIn('manual.html', (build.OUTPUT / 'public.html').read_text())
        build.check_api_coverage()

    def test_manual_entry_requires_a_reason_and_a_real_page(self):
        import json
        (self.root / 'api-docs/manual-types.json').write_text(json.dumps({
            'Example.Public': {'output': 'missing.html', 'reason': '', 'members': {}}
        }))
        with self.assertRaisesRegex(ValueError, 'Missing manual API entry'):
            build.register_manual_api_routes()


class PublicMetadataInventory(unittest.TestCase):
    def test_reference_snapshot_covers_all_public_types_and_no_internal_providers(self):
        import json
        from api_inventory import public_types
        root = Path(__file__).resolve().parent.parent
        actual = public_types(root / 'api-docs/reference/NeoCLR.CoreProbe.dll')
        self.assertEqual(actual, json.loads((root / 'api-docs/types.json').read_text()))
        for name in ('System.Collections.ArrayList`1', 'System.Collections.HashMap`2',
                     'System.Option.Some`1', 'System.Runtime.CompilerServices.IsReadOnlyAttribute'):
            self.assertIn(name, actual)
        self.assertNotIn('System.Introspection.RuntimeTypeInfo', actual)


if __name__ == '__main__':
    unittest.main()
