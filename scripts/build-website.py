#!/usr/bin/env python3
"""Build the project site and DocFX reference, using executable sample excerpts."""
from html import escape, unescape
from html.parser import HTMLParser
from pathlib import Path
import posixpath
import shutil
import json
import re
import subprocess
import sys
from textwrap import dedent
from urllib.parse import urlsplit, urlunsplit, unquote

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / 'website'
OUTPUT = ROOT / 'target/website'


def excerpt(path, start, end, include_end=True):
    text = (ROOT / path).read_text(encoding='utf-8')
    first = text.index(start)
    last = text.index(end, first) + (len(end) if include_end else 0)
    return escape(dedent(text[first:last]).rstrip())


class PageCheck(HTMLParser):
    def __init__(self):
        super().__init__()
        self.ids = set()
        self.links = []
        self.stack = []

    def handle_starttag(self, tag, attrs):
        if tag not in {'area', 'base', 'br', 'col', 'embed', 'hr', 'img', 'input', 'link', 'meta', 'param', 'source', 'track', 'wbr'}:
            self.stack.append(tag)
        attrs = dict(attrs)
        if 'id' in attrs:
            if attrs['id'] in self.ids:
                raise ValueError('Duplicate anchor: ' + attrs['id'])
            self.ids.add(attrs['id'])
        for attr in ('href', 'src'):
            if attr in attrs:
                self.links.append(attrs[attr])

    def handle_endtag(self, tag):
        if not self.stack or self.stack.pop() != tag:
            raise ValueError('Mismatched closing HTML tag: ' + tag)

    def check(self, path, pages):
        if self.stack:
            raise ValueError('Unclosed HTML tags: ' + ', '.join(self.stack))
        for link in self.links:
            url = urlsplit(link)
            if url.scheme or url.netloc:
                continue
            target = (path.parent / unquote(url.path)).resolve() if url.path else path
            if target.is_dir():
                target /= 'index.html'
            if not target.is_relative_to(OUTPUT.resolve()) or not target.is_file():
                raise ValueError(f'Missing local asset in {path.name}: {link}')
            if url.fragment and target in pages and unquote(url.fragment) not in pages[target].ids:
                raise ValueError(f'Missing anchor in {path.name}: {link}')


def render(page, samples):
    for token, source in samples.items():
        page = page.replace('{{' + token + '}}', excerpt(*source))
    blocks = list(re.finditer(r'(<pre[^>]*><code>)(.*?)(</code></pre>)', page, re.S))
    raven_blocks = [block for block in blocks
                    if 'neoIL' not in block.group(1) and 'data-language="text"' not in block.group(1)]
    result = subprocess.run(
        ['node', str(SOURCE / 'highlight.mjs')],
        input=json.dumps([unescape(block.group(2)) for block in raven_blocks]),
        capture_output=True, text=True, check=True)
    highlighted = dict(zip((block.start() for block in raven_blocks), json.loads(result.stdout)))
    for block in reversed(blocks):
        if block.start() in highlighted:
            page = page[:block.start(2)] + highlighted[block.start()] + page[block.end(2):]
    if '{{' in page:
        raise ValueError('Unexpanded website placeholder')
    return page


def make_reference_links_relative():
    # DocFX treats links into the surrounding site as external root-relative URLs.
    # Relativize HTML and the client-side navigation data for Pages project paths.
    def relative(value, page):
        url = urlsplit(value)
        if url.scheme or url.netloc or not url.path.startswith('/'):
            return value
        parent = page.relative_to(OUTPUT).parent.as_posix()
        path = posixpath.relpath(url.path.lstrip('/') or '.', parent)
        if url.path.endswith('/'):
            path += '/'
        return urlunsplit(('', '', path, url.query, url.fragment))

    attribute = re.compile(r'\b(href|src)=("|\')(/[^"\']*)\2')
    for page in (OUTPUT / 'docs').rglob('*.html'):
        def replace_attribute(match):
            value = relative(unescape(match.group(3)), page)
            return f'{match.group(1)}={match.group(2)}{escape(value, quote=True)}{match.group(2)}'

        html = re.sub(r'<[^>]+>', lambda tag: attribute.sub(replace_attribute, tag.group(0)),
                      page.read_text(encoding='utf-8'))
        page.write_text(html, encoding='utf-8')

    for page in (OUTPUT / 'docs').rglob('toc.json'):
        def visit(value):
            if isinstance(value, dict):
                for key, child in value.items():
                    if key in {'href', 'topicHref', 'tocHref'} and isinstance(child, str):
                        value[key] = relative(child, page)
                    else:
                        visit(child)
            elif isinstance(value, list):
                for child in value:
                    visit(child)
        toc = json.loads(page.read_text(encoding='utf-8'))
        visit(toc)
        page.write_text(json.dumps(toc), encoding='utf-8')


def check_reference_links():
    # DocFX validates its own content graph. Also check links from generated HTML
    # into the surrounding website, including root-relative overview links.
    class ReferenceLinks(HTMLParser):
        def handle_starttag(self, tag, attrs):
            for name, value in attrs:
                if name not in ('href', 'src') or not value:
                    continue
                url = urlsplit(value)
                if url.scheme or url.netloc or not url.path:
                    continue
                path = unquote(url.path)
                target = ((OUTPUT / path.lstrip('/')) if path.startswith('/')
                          else self.page.parent / path).resolve()
                if target.is_dir():
                    target /= 'index.html'
                if not target.is_relative_to(OUTPUT.resolve()) or not target.is_file():
                    raise ValueError(f'Missing API reference link in {self.page.name}: {value}')

    for page in (OUTPUT / 'docs').rglob('*.html'):
        parser = ReferenceLinks()
        parser.page = page
        parser.feed(page.read_text(encoding='utf-8'))


def main():
    if OUTPUT.exists():
        shutil.rmtree(OUTPUT)
    OUTPUT.mkdir(parents=True)
    for name in ('style.css', 'mark.svg'):
        shutil.copyfile(SOURCE / name, OUTPUT / name)
    raven = 'docs/experiments/raven-target/samples/'
    samples = {
        'OBJECT_DISPLAY_SAMPLE': ('docs/experiments/object-display/Main.rvn', 'open class Plain', '\nfunc Main()', False),
        'CONSOLE_PROPAGATION_SAMPLE': ('docs/experiments/console-streams/Propagation.rvn', 'func ReadInput()', '\n}', True),
        'CONSOLE_IF_LET_SAMPLE': ('docs/experiments/console-streams/IfLet.rvn', 'func ReadInput()', '\n}', True),
        'STORAGE_POC_SAMPLE': ('docs/experiments/storage-poc/Main.rvn', 'func Main()', '\n}', True),
        'ARRAY_TOUR': (raven + 'library-array-tour.rvn', 'import System.*', '\n}', True),
        'TASK_AWAIT_SAMPLE': ('website/samples/preview9/library-async-default-queue.rvn', 'func Describe', '\n    return ()\n}', True),
        'TASK_WORKER_SAMPLE': ('website/samples/preview9/library-async-default-queue.rvn', 'import System.*', '\nfunc Main() {\n    _ = Show()\n}', True),
        'TASK_PROMISE_SAMPLE': (raven + 'library-task-producer.rvn', 'import System.*', '\n    promise.Complete(41)\n}', True),
        'TASK_PROPAGATION_SAMPLE': (raven + 'library-task-propagation.rvn', 'async func Read', '\n}\n', True),
        'TASK_RESULT_SAMPLE': (raven + 'library-task-result.rvn', 'import System.*', '\n    promise.Complete(Ok(41))\n}', True),
        'TASK_CANCELLATION_SAMPLE': (raven + 'library-async-cancellation.rvn', 'import System.*', '\n    promise.Cancel()\n}', True),
        'PROJECT_SAMPLE': ('docs/experiments/raven-target/msbuild/Demo.rvnproj', '<Project', '</Project>', True),
        'CLOCK_SAMPLE': (raven + 'library-instants.rvn', 'func ShowCurrentTime', '\n}', True),
        'INTROSPECTION_SAMPLE': (raven + 'library-introspection-tour.rvn', '    let assembly', '    for module', False),
        'UNION_SAMPLE': (raven + 'library-query-terminals.rvn', 'func PrintOptional', '\nfunc OnlyPositive', False),
        'FUNC_SAMPLE': (raven + 'application-delegates.rvn', '    var shared = 7', '    WriteLine(shared)', True),
        'DATE_SAMPLE': (raven + 'library-calendar.rvn', '    CheckDate(Date.Create(day: 29', '    CheckDate(Date.FromDayNumber(-1))', True),
        'RAVEN_SAMPLE': (raven + 'library-propagation.rvn', 'func Normalize', '\n}', True),
        'IL_SAMPLE': ('examples/preview/result-void.neoil', '.function Complete', '.end', True),
        'OPTION_SAMPLE': (raven + 'library-query-terminals.rvn', 'func FirstPositive', '\n}', True),
        'COLLECTION_SAMPLE': (raven + 'library-collection-capabilities.rvn', 'func Read', '\nfunc Main', False),
        'FILE_SAMPLE': (raven + 'library-files.rvn', 'func Load', '\n}', True),
        'TEXT_SAMPLE': (raven + 'library-string-slices.rvn', 'func Extract', '\n}', True),
        'OUTCOME_OPERATORS_SAMPLE': (raven + 'library-outcome-operators.rvn', 'import System.*', '\n}', True),
        'QUERY_BASICS_SAMPLE': (raven + 'library-query-basics.rvn', 'import System.Collections.*', '\n    }\n}', True),
        'QUERY_NAMES_SAMPLE': (raven + 'library-query-names.rvn', 'import System.Linq.*', '\n}', True),
        'QUERY_SAMPLE': (raven + 'library-query-terminals.rvn', 'func OnlyPositive', '\n}', True),
    }
    samples.update({
        'GRAPHEME_COUNTS': (raven + 'library-grapheme-strings.rvn', '    let text =', '    ShowCharacters(text)', False),
        'GRAPHEME_ITERATION': (raven + 'library-grapheme-strings.rvn', 'func ShowCharacters', '\n}', True),
        'UTF8_ROUNDTRIP': (raven + 'library-utf8.rvn', 'func RoundTrip', '\n}', True),
    })
    tour = raven + 'library-introspection-tour.rvn'
    samples.update({
        'TOUR_ACQUISITION': (tour, '    let widget:', '\n    let assembly', False),
        'TOUR_DISCOVERY': (tour, '    let assembly', '\n    if description.MetadataToken', False),
        'TOUR_TOKENS': (tour, '    if description.MetadataToken', '\n    let methods:', False),
        'TOUR_SEQUENCES': (tour, '    let methods:', '\n    let flags', False),
        'TOUR_MATCH': (tour, 'func MemberKind', '\n}', True),
        'TOUR_MEMBERS': (tour, '    let flags', '\n}', False),
    })
    # The full expected output is shared with the saved-project execution check.
    output_text = escape((ROOT / (raven + 'library-introspection-tour.expected.txt')).read_text().rstrip())
    array_output = escape((ROOT / (raven + 'library-array-tour.expected.txt')).read_text().rstrip())
    downloads = OUTPUT / 'samples'
    downloads.mkdir()
    for name in ('library-array-tour.rvn', 'library-array-tour.expected.txt', 'library-task-propagation.rvn', 'library-task-result.rvn', 'library-async-default-queue.rvn', 'library-task-producer.rvn', 'library-async-cancellation.rvn', 'library-outcome-operators.rvn', 'library-outcome-operators.expected.txt', 'library-query-basics.rvn', 'library-query-basics.expected.txt', 'library-query-names.rvn', 'library-query-names.expected.txt', 'library-introspection-tour.rvn', 'library-introspection-tour.expected.txt', 'library-utf8.rvn', 'library-utf8.expected.txt', 'library-instants.rvn', 'library-propagation.rvn', 'library-collection-capabilities.rvn', 'library-files.rvn', 'library-grapheme-strings.rvn', 'library-grapheme-strings.expected.txt'):
        source_sample = ROOT / raven / name
        if name == 'library-async-default-queue.rvn':
            # Published examples retain the release contract during development.
            source_sample = SOURCE / 'samples/preview9' / name
        shutil.copyfile(source_sample, downloads / name)
    storage_downloads = downloads / 'storage-provider'
    storage_downloads.mkdir()
    for name in ('Storage.rvn', 'Path.rvn', 'Streams.rvn', 'ByteRoundTrip.rvn', 'Main.rvn', 'StorageExplorer.rvnproj', 'expected.txt'):
        shutil.copyfile(ROOT / 'docs/experiments/storage-provider' / name, storage_downloads / name)
    poc_downloads = downloads / 'storage-poc'
    poc_downloads.mkdir()
    for name in ('Main.rvn', 'StoragePoc.rvnproj', 'expected.txt'):
        shutil.copyfile(ROOT / 'docs/experiments/storage-poc' / name, poc_downloads / name)
    transformer_sources = OUTPUT / '_transformer-source'
    for folder, names in {
        'file-transformer': ('Main.rvn', 'FileTransformer.rvnproj', 'README.md', 'verify.py'),
        'json-document': ('JsonDocument.rvn', 'JsonValue.rvn'),
        'json-message': ('JsonMessage.rvn',),
    }.items():
        destination = transformer_sources / folder
        destination.mkdir(parents=True)
        for name in names:
            shutil.copyfile(ROOT / 'docs/experiments' / folder / name, destination / name)
    shutil.make_archive(str(downloads / 'file-transformer'), 'zip', transformer_sources)
    shutil.rmtree(transformer_sources)
    pending_sources = OUTPUT / '_pending-read-source' / 'pending-read'
    pending_sources.mkdir(parents=True)
    for name in ('PendingRead.rvn', 'Main.rvn', 'PendingRead.rvnproj', 'expected.txt', 'verify.py', 'README.md'):
        shutil.copyfile(ROOT / 'docs/experiments/pending-read' / name, pending_sources / name)
    shutil.make_archive(str(downloads / 'pending-read'), 'zip', pending_sources.parent)
    shutil.rmtree(pending_sources.parent)
    host_pending_sources = OUTPUT / '_host-pending-source' / 'host-pending-read'
    host_pending_sources.mkdir(parents=True)
    for name in ('Copy.rvn', 'Main.rvn', 'DelayedCopy.rvnproj', 'expected.txt', 'README.md'):
        shutil.copyfile(ROOT / 'docs/experiments/host-pending-read' / name, host_pending_sources / name)
    shutil.make_archive(str(downloads / 'host-pending-read'), 'zip', host_pending_sources.parent)
    shutil.rmtree(host_pending_sources.parent)
    console_sources = OUTPUT / '_console-source' / 'console-streams'
    console_sources.mkdir(parents=True)
    for name in ('Main.rvn', 'Contracts.rvn', 'Propagation.rvn', 'IfLet.rvn', 'ConsoleStreams.rvnproj', 'verify.py', 'README.md'):
        shutil.copyfile(ROOT / 'docs/experiments/console-streams' / name, console_sources / name)
    shutil.make_archive(str(downloads / 'console-streams'), 'zip', console_sources.parent)
    shutil.rmtree(console_sources.parent)
    object_sources = OUTPUT / '_object-source' / 'object-display'
    object_sources.mkdir(parents=True)
    for name in ('Main.rvn', 'Abstract.rvn', 'ObjectDisplay.rvnproj', 'expected.txt', 'verify.py', 'README.md'):
        shutil.copyfile(ROOT / 'docs/experiments/object-display' / name, object_sources / name)
    shutil.make_archive(str(downloads / 'object-display'), 'zip', object_sources.parent)
    equality_sources = OUTPUT / '_object-equality-source' / 'object-equality'
    equality_sources.mkdir(parents=True, exist_ok=True)
    for name in ('Main.rvn', 'ObjectEquality.rvnproj', 'expected.txt', 'verify.py', 'README.md'):
        shutil.copyfile(ROOT / 'docs/experiments/object-equality' / name, equality_sources / name)
    shutil.make_archive(str(downloads / 'object-equality'), 'zip', equality_sources.parent)
    shutil.rmtree(equality_sources.parent)
    shutil.rmtree(object_sources.parent)
    record_sources = OUTPUT / '_records-source' / 'records'
    record_sources.mkdir(parents=True)
    for name in ('Main.rvn', 'Records.rvnproj', 'expected.txt', 'verify.py', 'README.md'):
        shutil.copyfile(ROOT / 'docs/experiments/records' / name, record_sources / name)
    shutil.make_archive(str(downloads / 'records'), 'zip', record_sources.parent)
    shutil.rmtree(record_sources.parent)
    cancel_sources = OUTPUT / '_worker-cancel-source' / 'worker-task-cancellation'
    cancel_sources.mkdir(parents=True)
    for name in ('Workers.rvn', 'Copy.rvn', 'Main.rvn', 'DelayedCopy.rvnproj', 'Fault.rvn', 'Forbidden.rvn', 'Affinity.rvn', 'affinity.expected.txt', 'expected.txt', 'README.md'):
        shutil.copyfile(ROOT / 'docs/experiments/worker-task-cancellation' / name, cancel_sources / name)
    shutil.make_archive(str(downloads / 'worker-task-cancellation'), 'zip', cancel_sources.parent)
    shutil.rmtree(cancel_sources.parent)
    pages = {}
    for source in sorted(SOURCE.rglob('*.html')):
        relative = source.relative_to(SOURCE)
        target = OUTPUT / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        template = (source.read_text(encoding='utf-8')
                    .replace('{{TOUR_OUTPUT}}', output_text)
                    .replace('{{ARRAY_OUTPUT}}', array_output))
        page = render(template, samples)
        target.write_text(page, encoding='utf-8')
        check = PageCheck()
        check.feed(page)
        pages[target.resolve()] = check
    subprocess.run([sys.executable, str(ROOT / 'scripts/build-api-docs.py')], check=True)
    for path, check in pages.items():
        check.check(path, pages)
    make_reference_links_relative()
    check_reference_links()
    (OUTPUT / '.nojekyll').touch()
    print(f'Built and checked {len(pages)} pages:', OUTPUT)


if __name__ == '__main__':
    main()
