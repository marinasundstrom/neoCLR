"""Cross-page validation for the static feature guide builder."""
import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import json
import subprocess

spec = importlib.util.spec_from_file_location('website_build', Path(__file__).with_name('build-website.py'))
build = importlib.util.module_from_spec(spec)
spec.loader.exec_module(build)


class LegacyApiContent(unittest.TestCase):
    def test_serve_sidecars_keep_native_ids_and_preserve_overload_contracts(self):
        for name, expected in (
            ('http-server-serve.md', 'M:System.Web.Http.HttpServer.Serve(System.Func`2)'),
            ('http-server-serve-cancellation.md',
             'M:System.Web.Http.HttpServer.Serve(System.Func`2,System.Concurrency.CancellationToken)')):
            source = (build.SOURCE / 'api-content' / name).read_text()
            rendered = build.legacy_api_content(source)
            uid = next(line[5:] for line in rendered.splitlines() if line.startswith('uid: '))
            self.assertEqual(uid, expected)
            self.assertIn('Serve(Function', source)
            self.assertEqual(source.split('---', 2)[2], rendered.split('---', 2)[2])


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

    def test_page_url_case_must_match_on_case_insensitive_filesystems_too(self):
        self.page('docs/api/System/Time/TimeZone/index.html', '<main>Zone</main>')
        path, guide = self.page('index.html', '<a href="docs/api/system/time/TimeZone/">Zone</a>')
        with self.assertRaisesRegex(ValueError, 'Missing local asset'):
            guide.check(path, self.pages)
        path, guide = self.page('index.html', '<a href="docs/api/System/Time/TimeZone/">Zone</a>')
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
        site = json.loads((repo / 'website/site.json').read_text())
        result, output = self.publish('# Reference', extra=dict(
            api=str(repo / 'api-docs/reference/NeoCLR.CoreProbe.dll'),
            excludedMembers=list(exclusions), extensionNamespaces=site['extensionNamespaces']))
        self.assertEqual(result.returncode, 0, result.stderr)
        xrefs = {uid.replace('+', '.'): path for uid, path in
                 json.loads((output / 'xref-map.json').read_text()).items()}
        for name in ('System.Array', 'System.Optional.Some`1', 'System.Result.Ok`1',
                     'System.Concurrency.Tasks.TaskOutcome.Cancelled',
                     'Raven.Runtime.CompilerServices.RavenUnionCompanionAttribute'):
            self.assertNotIn('T:' + name, xrefs)
        for name in ('System.Array`1', 'System.Optional`1', 'System.Result`2',
                     'System.Concurrency.Tasks.TaskOutcome`1'):
            self.assertIn('T:' + name, xrefs)
            self.assertTrue((output / xrefs['T:' + name].split('#')[0]).is_file(), name)
        # Companion IDs remain useful aliases, but never create duplicate pages.
        for name, arity in (('System.Optional', 1), ('System.Result', 2), ('System.Concurrency.Tasks.TaskOutcome', 1)):
            self.assertEqual(xrefs['T:' + name], xrefs['T:' + name + '`' + str(arity)])
        case = (output / 'api/System/BooleanParseError/InvalidFormat/index.html').read_text()
        self.assertNotIn('ReferenceEquals', case)
        self.assertIn('Inherited from', case)
        self.assertIn('id="show-inherited-members"', case)
        self.assertIn('id="member-grouping"', case)
        string_page = (output / 'api/System/String/index.html').read_text()
        self.assertIn('Any() -&gt; bool', string_page)
        self.assertIn('Any(predicate: char -&gt; bool)', string_page)
        self.assertNotIn('Any&lt;T&gt;', string_page)
        self.assertIn('Map&lt;U&gt;(selector: char -&gt; U)', string_page)
        factory = (output / 'api/System/Concurrency/Tasks/Task/index.html').read_text()
        self.assertNotIn('data-member-inherited="true"', factory)
        self.assertIn('Run(callback: () -&gt; void) -&gt; Task&lt;void&gt;', factory)
        self.assertIn('Run&lt;T&gt;(callback: () -&gt; Task&lt;T&gt;)', factory)
        self.assertIn('FlatMap&lt;U&gt;(selector: char -&gt; Iterable&lt;U&gt;)', string_page)
        run_page = (output / 'api/System/Concurrency/Tasks/Task/method_Run.html').read_text()
        self.assertNotIn('Func&lt;', run_page)
        self.assertIn('api-parameter-name', run_page)
        self.assertIn('() -&gt; void', run_page)
        for receiver, container, methods in (
                ('System/Concurrency/Tasks/Task`1', 'TaskOperators', ('Map', 'Then')),
                ('System/Result`2', 'ResultOperators', ('Map', 'Then', 'MapError')),
                ('System/Optional`1', 'OptionalOperators', ('Map',))):
            page = (output / 'api' / receiver / 'index.html').read_text()
            self.assertIn('data-member-extension="true"', page, receiver)
            self.assertNotIn('Number&lt;T&gt;.op_', page, receiver)
            for method in methods:
                self.assertIn(container + '/method_' + method + '.html', page, receiver)
                self.assertIn(container + '.' + method, page, receiver)


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
                     'System.Optional.Some`1', 'System.Runtime.CompilerServices.IsReadOnlyAttribute'):
            self.assertIn(name, actual)
        self.assertNotIn('System.Introspection.RuntimeTypeInfo', actual)


class NativePreviewPublication(unittest.TestCase):
    def test_mixed_reference_and_guide_pages_share_final_navigation(self):
        with tempfile.TemporaryDirectory() as directory:
            site = Path(directory)
            shared = site / 'docs/api/api-navigation.html'
            shared.parent.mkdir(parents=True)
            panel = '<nav class="api-navigation-panel" aria-label="API modules and types">'
            canonical = (panel + '<ul><li><details class="api-namespace">'
                         '<summary title="System.Localization"><span>System.Localization</span></summary>'
                         '<ul><li><a href="System/Localization/index.html">Module overview</a></li>'
                         '</ul></details></li></ul></nav>')
            shared.write_text(canonical)
            pages = ['docs/index.html', 'docs/functions.html',
                     'docs/api/System/Localization/index.html', 'docs/api/legacy.html']
            for name in pages:
                page = site / name
                page.parent.mkdir(parents=True, exist_ok=True)
                page.write_text('<main>Preserved API content</main>' + panel + 'Old menu</nav>')
            self.assertEqual(build.share_api_navigation(site), len(pages))
            versions = set()
            for name in pages:
                page = site / name
                content = page.read_text()
                self.assertNotIn('Old menu', content)
                self.assertIn('System.Localization', content)
                self.assertIn('<main>Preserved API content</main>', content)
                source = content.split('data-navigation-src="', 1)[1].split('"', 1)[0]
                path, version = source.split('?v=')
                self.assertEqual((page.parent / path).resolve(), shared.resolve())
                versions.add(version)
            self.assertEqual(len(versions), 1)
            self.assertEqual(shared.read_text(), canonical)
            shared.write_text(canonical.replace('System.Localization', 'System.DependencyInjection'))
            build.share_api_navigation(site)
            refreshed = (site / pages[0]).read_text()
            self.assertNotIn(next(iter(versions)), refreshed)
            self.assertIn('System.DependencyInjection', refreshed)
            self.assertNotIn('System.Localization', refreshed)


    def test_snapshot_banner_uses_current_release_without_rewriting_content(self):
        with tempfile.TemporaryDirectory() as temporary:
            site = Path(temporary)
            page = site / 'api.html'
            page.write_text('<div class="release-notice" role="note">'
                            'Native coverage in progress <a href="https://example.com/old">Release</a></div>'
                            '<main><a href="https://example.com/old">Historical evidence</a></main>')
            build.refresh_release_links(site, 'https://example.com/new?a=1&b=2')
            content = page.read_text()
            self.assertIn('Native coverage in progress', content)
            self.assertIn('href="https://example.com/new?a=1&amp;b=2">Release', content)
            self.assertIn('href="https://example.com/old">Historical evidence', content)

    def test_uncommitted_audit_requires_explicit_selection(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            with patch.object(build, 'NATIVE_SNAPSHOT', root / 'snapshot.json'):
                self.assertIsNone(build.selected_native_preview())
                with self.assertRaisesRegex(ValueError, 'Selected native API audit is missing'):
                    build.selected_native_preview(root / 'audit')
                (root / 'audit/site/docs/api').mkdir(parents=True)
                self.assertEqual((root / 'audit').resolve(), build.selected_native_preview(root / 'audit'))

    def test_clean_checkout_selects_pinned_native_reference(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            snapshot = root / 'snapshot.json'
            snapshot.write_text('{}')
            (root / 'selection.json').write_text(json.dumps({'audit': str(root / 'stale-audit')}))
            with patch.object(build, 'NATIVE_PREVIEW', root / 'selection.json'), \
                    patch.object(build, 'NATIVE_SNAPSHOT', snapshot), \
                    patch.object(build, 'extract_snapshot', return_value=root / 'audit') as extract:
                self.assertEqual(root / 'audit', build.selected_native_preview())
                extract.assert_called_once_with(snapshot, build.ROOT / 'target/native-api-publication')

    def test_native_overlay_completes_before_publication(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            destination, staging = root / 'website', root / 'website-next'
            destination.mkdir()
            staging.mkdir()
            (destination / 'owner').write_text('native previous')
            (staging / 'owner').write_text('legacy')

            def integrate(command, check):
                self.assertTrue(check)
                self.assertEqual(command[-2:], ['--site', str(staging)])
                self.assertEqual((destination / 'owner').read_text(), 'native previous')
                (staging / 'owner').write_text('native updated')

            with patch.object(build.subprocess, 'run', side_effect=integrate):
                build.publish_site(staging, destination, root / 'audit')
            self.assertEqual((destination / 'owner').read_text(), 'native updated')

    def test_failed_overlay_preserves_visible_site(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            destination, staging = root / 'website', root / 'website-next'
            destination.mkdir()
            staging.mkdir()
            (destination / 'owner').write_text('native previous')
            with patch.object(build.subprocess, 'run', side_effect=subprocess.CalledProcessError(1, 'preview')):
                with self.assertRaises(subprocess.CalledProcessError):
                    build.publish_site(staging, destination, root / 'audit')
            self.assertEqual((destination / 'owner').read_text(), 'native previous')


if __name__ == '__main__':
    unittest.main()
