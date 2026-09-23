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


if __name__ == '__main__':
    unittest.main()
