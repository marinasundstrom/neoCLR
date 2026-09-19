#!/usr/bin/env python3
"""Build the dependency-free project site using excerpts from executable samples."""
from html import escape, unescape
from html.parser import HTMLParser
from pathlib import Path
import shutil
import json
import re
import subprocess
from textwrap import dedent
from urllib.parse import urlsplit, unquote

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


def main():
    if OUTPUT.exists():
        shutil.rmtree(OUTPUT)
    OUTPUT.mkdir(parents=True)
    for name in ('style.css', 'mark.svg'):
        shutil.copyfile(SOURCE / name, OUTPUT / name)
    raven = 'docs/experiments/raven-target/samples/'
    samples = {
        'CLOCK_SAMPLE': (raven + 'library-instants.rvn', 'func ShowCurrentTime', '\n}', True),
        'INTROSPECTION_SAMPLE': (raven + 'library-introspection-tour.rvn', '    val assembly', '    for module', False),
        'UNION_SAMPLE': (raven + 'library-query-terminals.rvn', 'func PrintOptional', '\nfunc OnlyPositive', False),
        'FUNC_SAMPLE': (raven + 'application-delegates.rvn', '    var shared = 7', '    WriteLine(shared)', True),
        'DATE_SAMPLE': (raven + 'library-calendar.rvn', '    CheckDate(Date.Create(day: 29', '    CheckDate(Date.FromDayNumber(-1))', True),
        'RAVEN_SAMPLE': (raven + 'library-propagation.rvn', 'func Normalize', '\n}', True),
        'IL_SAMPLE': ('examples/preview/result-void.neoil', '.function Complete', '.end', True),
        'OPTION_SAMPLE': (raven + 'library-query-terminals.rvn', 'func FirstPositive', '\n}', True),
        'COLLECTION_SAMPLE': (raven + 'library-collection-capabilities.rvn', 'func Read', '\nfunc Main', False),
        'TEXT_SAMPLE': (raven + 'library-string-slices.rvn', 'func Extract', '\n}', True),
        'QUERY_SAMPLE': (raven + 'library-query-terminals.rvn', 'func OnlyPositive', '\n}', True),
    }
    tour = raven + 'library-introspection-tour.rvn'
    samples.update({
        'TOUR_ACQUISITION': (tour, '    val widget:', '\n    val assembly', False),
        'TOUR_DISCOVERY': (tour, '    val assembly', '\n    if description.MetadataToken', False),
        'TOUR_TOKENS': (tour, '    if description.MetadataToken', '\n    val methods:', False),
        'TOUR_SEQUENCES': (tour, '    val methods:', '\n    val flags', False),
        'TOUR_MATCH': (tour, 'func MemberKind', '\n}', True),
        'TOUR_MEMBERS': (tour, '    val flags', '\n}', False),
    })
    # The full expected output is shared with the saved-project execution check.
    output_text = escape((ROOT / (raven + 'library-introspection-tour.expected.txt')).read_text().rstrip())
    downloads = OUTPUT / 'samples'
    downloads.mkdir()
    for name in ('library-introspection-tour.rvn', 'library-introspection-tour.expected.txt'):
        shutil.copyfile(ROOT / raven / name, downloads / name)
    pages = {}
    for source in sorted([SOURCE / 'index.html', *(SOURCE / 'features').rglob('*.html')]):
        relative = source.relative_to(SOURCE)
        target = OUTPUT / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        template = source.read_text(encoding='utf-8').replace('{{TOUR_OUTPUT}}', output_text)
        page = render(template, samples)
        target.write_text(page, encoding='utf-8')
        check = PageCheck()
        check.feed(page)
        pages[target.resolve()] = check
    for path, check in pages.items():
        check.check(path, pages)
    (OUTPUT / '.nojekyll').touch()
    print(f'Built and checked {len(pages)} pages:', OUTPUT)


if __name__ == '__main__':
    main()
