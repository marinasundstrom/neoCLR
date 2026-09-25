#!/usr/bin/env python3
"""Build one RavenDoc site from Markdown, reference metadata and tested samples."""
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
    return dedent(text[first:last]).rstrip()


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

    def handle_startendtag(self, tag, attrs):
        self.handle_starttag(tag, attrs)
        if self.stack and self.stack[-1] == tag:
            self.stack.pop()

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


def render(page, samples, html=False):
    for token, source in samples.items():
        value = excerpt(*source)
        page = page.replace('{{' + token + '}}', escape(value) if html else value)
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
    for page in OUTPUT.rglob('*.html'):
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

    for page in OUTPUT.rglob('*.html'):
        parser = ReferenceLinks()
        parser.page = page
        parser.feed(page.read_text(encoding='utf-8'))


def register_manual_api_routes():
    """Keep documented renderer limitations visible, rather than excluding types."""
    path = OUTPUT / 'xref-map.json'
    xrefs = json.loads(path.read_text())
    for name, entry in json.loads((ROOT / 'api-docs/manual-types.json').read_text()).items():
        target = entry['output']
        if not entry['reason'] or not (OUTPUT / target).is_file():
            raise ValueError('Missing manual API entry or reason: ' + name)
        previous = xrefs.get('T:' + name)
        if previous and previous != target and not (OUTPUT / previous).is_file():
            # Namespace promotion leaves a registered container route without a page.
            # Preserve already-rendered links to it with a checked redirect.
            alias = OUTPUT / previous
            alias.parent.mkdir(parents=True, exist_ok=True)
            relative = posixpath.relpath(target, posixpath.dirname(previous))
            alias.write_text('<!doctype html><html lang="en"><head><meta charset="utf-8">'
                             '<title>Public type reference</title></head><body>'
                             f'<p><a href="{escape(relative)}">{escape(name)}</a></p>'
                             '<script>location.replace(' + json.dumps(relative) + ');</script></body></html>')
        xrefs['T:' + name] = target
        for uid, anchor in entry['members'].items():
            xrefs[uid] = target + ('#' + anchor if anchor else '')
    path.write_text(json.dumps(xrefs, indent=2) + '\n')


def check_api_coverage():
    import xml.etree.ElementTree as ET
    xrefs = {uid.replace('+', '.').replace('..ctor', '.#ctor'): path
             for uid, path in json.loads((OUTPUT / 'xref-map.json').read_text()).items()}
    types = json.loads((ROOT / 'api-docs/types.json').read_text())
    exclusions = json.loads((ROOT / 'api-docs/exclusions.json').read_text())
    comments = {node.attrib['name']: node for node in
                ET.parse(ROOT / 'api-docs/NeoCLR.CoreProbe.xml').findall('./members/member')}
    for name in types:
        uid = 'T:' + name
        if uid in exclusions:
            continue
        if uid not in xrefs or not (OUTPUT / xrefs[uid].split('#', 1)[0]).is_file():
            raise ValueError('Missing public type reference: ' + uid)
        page = OUTPUT / xrefs[uid].split('#', 1)[0]
        if uid not in comments or 'member-summary--empty' in page.read_text():
            print('API descriptions still needed: ' + name, flush=True)
    for uid, node in comments.items():
        normalized = re.sub(r'``[0-9]+', '', uid.split('(', 1)[0])
        selected = uid.startswith('T:') and uid[2:] in types or (
            uid[:2] in ('M:', 'P:', 'F:') and any(normalized[2:].startswith(name + '.') for name in types if 'T:' + name not in exclusions))
        if not selected or normalized in exclusions:
            continue
        if normalized not in xrefs:
            raise ValueError('Documented API missing from generated reference: ' + uid)
        summary = node.find('summary')
        if summary is None or not ''.join(summary.itertext()).strip():
            print('API summary still needed: ' + uid, flush=True)


def write_legacy_routes():
    xrefs = {uid.replace('+', '.').replace('..ctor', '.#ctor'): path
             for uid, path in json.loads((OUTPUT / 'xref-map.json').read_text()).items()}
    routes = json.loads((ROOT / 'api-docs/legacy-routes.json').read_text())
    for name, route in routes.items():
        if route['uid'] not in xrefs:
            raise ValueError('Missing legacy API target: ' + route['uid'])
        target = posixpath.relpath(xrefs[route['uid']], 'docs/api')
        anchors = {anchor: posixpath.relpath(xrefs[uid], 'docs/api')
                   for anchor, uid in route['anchors'].items() if uid in xrefs}
        body = ''.join(f'<p id="{escape(anchor)}"><a href="{escape(href)}">Member reference</a></p>'
                       for anchor, href in anchors.items())
        (OUTPUT / 'docs/api' / name).write_text(
            '<!doctype html><html lang="en"><head><meta charset="utf-8">'
            '<meta name="viewport" content="width=device-width, initial-scale=1">'
            '<title>API reference moved · neoCLR</title></head><body>'
            f'<h1>API reference moved</h1><p><a href="{escape(target)}">Open the RavenDoc reference</a></p>'
            + body + '<script>const routes=' + json.dumps(anchors) + ';'
            + 'location.replace(routes[decodeURIComponent(location.hash.slice(1))] || '
            + json.dumps(target) + ');</script></body></html>')


def main():
    global OUTPUT
    publish_destination = OUTPUT
    subprocess.run([sys.executable, str(ROOT / 'scripts/build-api-docs.py'), '--check'], check=True)
    OUTPUT = publish_destination.with_name(publish_destination.name + '-next')
    if OUTPUT.exists():
        shutil.rmtree(OUTPUT)
    OUTPUT.mkdir(parents=True)
    for name in ('custom.css', 'mark.svg', 'favicon.svg'):
        shutil.copyfile(SOURCE / name, OUTPUT / name)
    raven = 'docs/experiments/raven-target/samples/'
    samples = {
        'HTTP_VERB_SAMPLE': ('docs/experiments/http-verbs/Sample.rvn', 'async func ReplaceText(', '\n}', True),
        'HTTP_POST_SAMPLE': ('docs/experiments/http-post/Sample.rvn', 'async func PostText(', '\n}', True),
        'HTTP_JSON_SAMPLE': ('docs/experiments/http-json/Client.rvn', 'async func ReadReport(', '\n}', True),
        'HTTP_SERVER_SAMPLE': ('docs/experiments/http-server/Server.rvn', 'func Respond(', '\n}', True),
        'HTTP_CLIENT_SAMPLE': ('docs/experiments/http-client/Main.rvn', 'async func ReadGreeting(', '\n}', True),
        'SOCKET_SERVER_SAMPLE': ('docs/experiments/socket-echo/Server.rvn', 'async func Serve(', '\n}', True),
        'DNS_RESOLVE_SAMPLE': ('docs/experiments/socket-client/Main.rvn', 'async func ResolveHost()', '\n}', True),
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
    output_text = (ROOT / (raven + 'library-introspection-tour.expected.txt')).read_text().rstrip()
    array_output = (ROOT / (raven + 'library-array-tour.expected.txt')).read_text().rstrip()
    downloads = OUTPUT / 'samples'
    downloads.mkdir()
    for name in ('library-array-tour.rvn', 'library-array-tour.expected.txt', 'library-task-propagation.rvn', 'library-task-result.rvn', 'library-async-default-queue.rvn', 'library-task-producer.rvn', 'library-async-cancellation.rvn', 'library-outcome-operators.rvn', 'library-outcome-operators.expected.txt', 'library-query-basics.rvn', 'library-query-basics.expected.txt', 'library-query-names.rvn', 'library-query-names.expected.txt', 'library-introspection-tour.rvn', 'library-introspection-tour.expected.txt', 'library-utf8.rvn', 'library-utf8.expected.txt', 'library-instants.rvn', 'library-propagation.rvn', 'library-collection-capabilities.rvn', 'library-files.rvn', 'library-grapheme-strings.rvn', 'library-grapheme-strings.expected.txt'):
        source_sample = ROOT / raven / name
        if name == 'library-async-default-queue.rvn':
            # Published examples retain the release contract during development.
            source_sample = SOURCE / 'samples/preview9' / name
        shutil.copyfile(source_sample, downloads / name)
    post_downloads = downloads / 'http-post'
    post_downloads.mkdir()
    for name in ('Sample.rvn', 'SampleChecks.rvn', 'Client.rvn', 'Server.rvn', 'HttpPost.rvnproj', 'Reference.cs', 'README.md', 'verify.py'):
        shutil.copyfile(ROOT / 'docs/experiments/http-post' / name, post_downloads / name)
    shutil.make_archive(str(downloads / 'http-post'), 'zip', post_downloads)
    http_downloads = downloads / 'http-client'
    http_downloads.mkdir()
    for name in ('Main.rvn', 'Handlers.rvn', 'HttpClient.rvnproj', 'Reference.cs', 'README.md', 'verify.py'):
        shutil.copyfile(ROOT / 'docs/experiments/http-client' / name, http_downloads / name)
    shutil.make_archive(str(downloads / 'http-client'), 'zip', http_downloads)
    server_downloads = downloads / 'http-server'
    server_downloads.mkdir(parents=True, exist_ok=True)
    for name in ('Server.rvn', 'Server.rvnproj', 'README.md', 'verify.py'):
        shutil.copyfile(ROOT / 'docs/experiments/http-server' / name, server_downloads / name)
    for name in ('Main.rvn', 'Handlers.rvn', 'HttpClient.rvnproj', 'Reference.cs'):
        shutil.copyfile(ROOT / 'docs/experiments/http-client' / name, server_downloads / name)
    shutil.make_archive(str(downloads / 'http-server'), 'zip', server_downloads)
    json_downloads = downloads / 'http-json'
    for directory, names in (
        ('http-json', ('Client.rvn', 'Server.rvn', 'Client.rvnproj', 'Server.rvnproj', 'README.md', 'verify.py')),
        ('json-message', ('JsonMessage.rvn',)),
        ('json-document', ('JsonValue.rvn', 'JsonDocument.rvn')),
    ):
        destination = json_downloads / directory
        destination.mkdir(parents=True, exist_ok=True)
        for name in names:
            shutil.copyfile(ROOT / 'docs/experiments' / directory / name, destination / name)
    shutil.make_archive(str(downloads / 'http-json'), 'zip', json_downloads)


    socket_downloads = downloads / 'socket-client'
    socket_downloads.mkdir()
    for name in ('Main.rvn', 'SocketClient.rvnproj', 'README.md', 'verify.py'):
        shutil.copyfile(ROOT / 'docs/experiments/socket-client' / name, socket_downloads / name)
    shutil.make_archive(str(downloads / 'socket-client'), 'zip', socket_downloads)
    echo_downloads = downloads / 'socket-echo'
    echo_downloads.mkdir()
    for name in ('Server.rvn', 'Server.rvnproj', 'README.md', 'verify.py'):
        shutil.copyfile(ROOT / 'docs/experiments/socket-echo' / name, echo_downloads / name)
    shutil.copyfile(ROOT / 'docs/experiments/socket-client/Main.rvn', echo_downloads / 'Client.rvn')
    (echo_downloads / 'Client.rvnproj').write_text((ROOT / 'docs/experiments/socket-client/SocketClient.rvnproj').read_text().replace('Main.rvn', 'Client.rvn'))
    shutil.make_archive(str(downloads / 'socket-echo'), 'zip', echo_downloads)

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
    for name in ('Main.rvn', 'Records.rvnproj', 'expected.txt', 'Nested.rvn', 'Nested.rvnproj', 'nested-expected.txt', 'Defaults.rvn', 'Defaults.rvnproj', 'defaults-expected.txt', 'verify.py', 'README.md'):
        shutil.copyfile(ROOT / 'docs/experiments/records' / name, record_sources / name)
    shutil.make_archive(str(downloads / 'records'), 'zip', record_sources.parent)
    shutil.rmtree(record_sources.parent)
    cancel_sources = OUTPUT / '_worker-cancel-source' / 'worker-task-cancellation'
    cancel_sources.mkdir(parents=True)
    for name in ('Workers.rvn', 'Copy.rvn', 'Main.rvn', 'DelayedCopy.rvnproj', 'Fault.rvn', 'Forbidden.rvn', 'Affinity.rvn', 'affinity.expected.txt', 'expected.txt', 'README.md'):
        shutil.copyfile(ROOT / 'docs/experiments/worker-task-cancellation' / name, cancel_sources / name)
    shutil.make_archive(str(downloads / 'worker-task-cancellation'), 'zip', cancel_sources.parent)
    shutil.rmtree(cancel_sources.parent)
    staging = ROOT / 'target/website-content'
    shutil.rmtree(staging, ignore_errors=True)
    staging.mkdir(parents=True)
    config = json.loads((SOURCE / 'site.json').read_text())
    if not config.get('notice') or not config.get('releaseUrl'):
        raise ValueError('The site must explain development/release availability and link the published release')
    publisher_output = ROOT / 'target/website-rendered'
    config.update(output=str(publisher_output), api=str(ROOT / 'api-docs/reference/NeoCLR.CoreProbe.dll'),
                  types=None,  # Generate every public type, regardless of comment availability.
                  excludedMembers=list(json.loads((ROOT / 'api-docs/exclusions.json').read_text())), pages=[])
    sources = [(source, source.relative_to(SOURCE / 'content').with_suffix('.html'))
               for source in sorted((SOURCE / 'content').rglob('*')) if source.suffix in ('.md', '.html')]
    sources += [(source, Path('docs') / source.with_suffix('.html').name)
                for source in sorted((ROOT / 'api-docs').glob('*.md')) if source.name != 'README.md']
    sources += [(ROOT / 'api-docs' / entry['source'], Path(entry['output']))
                for entry in json.loads((ROOT / 'api-docs/manual-types.json').read_text()).values()]
    for source, relative in sources:
        template = source.read_text().replace('{{TOUR_OUTPUT}}', output_text).replace('{{ARRAY_OUTPUT}}', array_output)
        markdown = render(template, samples, html=source.suffix == '.html')
        # Conceptual Markdown links keep their existing published .html routes.
        markdown = re.sub(r'(?<=\])\(([^):]+)\.md([#?][^)]*)?\)',
                          lambda m: '(' + m[1] + '.html' + (m[2] or '') + ')', markdown)
        title = re.search(r'^# (.+)$', markdown, re.M)
        staged = staging / relative.with_suffix(source.suffix)
        staged.parent.mkdir(parents=True, exist_ok=True)
        staged.write_text(markdown)
        config['pages'].append(dict(source=str(staged), output=str(relative), title=title[1].replace('*', '') if title else None))
    shutil.copyfile(SOURCE / 'toc.yml', staging / 'toc.yml')
    manifest = staging / 'site.json'
    manifest.write_text(json.dumps(config))
    subprocess.run([sys.executable, str(ROOT / 'scripts/ravendoc.py'), '--site', str(manifest)], check=True)
    shutil.copytree(publisher_output, OUTPUT, dirs_exist_ok=True)
    register_manual_api_routes()
    check_api_coverage()
    write_legacy_routes()
    # All site links must work at the domain root and under a Pages project prefix.
    make_reference_links_relative()
    pages = {}
    for path in OUTPUT.rglob('*.html'):
        page = path.read_text()
        check = PageCheck()
        try:
            check.feed(page)
        except ValueError as error:
            raise ValueError(f"{path}: {error}") from error
        pages[path.resolve()] = check
    for path, check in pages.items():
        check.check(path, pages)
    (OUTPUT / '.nojekyll').touch()
    shutil.rmtree(publish_destination, ignore_errors=True)
    OUTPUT.rename(publish_destination)
    OUTPUT = publish_destination
    print(f'Built and checked {len(pages)} RavenDoc pages:', OUTPUT)


if __name__ == '__main__':
    main()
