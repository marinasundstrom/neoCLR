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

    def check(self):
        if self.stack:
            raise ValueError('Unclosed HTML tags: ' + ', '.join(self.stack))
        for link in self.links:
            if link.startswith('#'):
                if link[1:] not in self.ids:
                    raise ValueError('Missing anchor: ' + link)
            elif '://' not in link and not (OUTPUT / link).exists():
                raise ValueError('Missing local asset: ' + link)


def main():
    if OUTPUT.exists():
        shutil.rmtree(OUTPUT)
    OUTPUT.mkdir(parents=True)
    for name in ('style.css', 'mark.svg'):
        shutil.copyfile(SOURCE / name, OUTPUT / name)
    page = (SOURCE / 'index.html').read_text(encoding='utf-8')
    raven = 'docs/experiments/raven-target/samples/'
    samples = {
        'CLOCK_SAMPLE': (raven + 'library-instants.rvn', 'func ShowCurrentTime', '\n}', True),
        'INTROSPECTION_SAMPLE': (raven + 'library-type-preview.rvn', 'func Main', '\n}', True),
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
    for token, source in samples.items():
        page = page.replace('{{' + token + '}}', excerpt(*source))
    # Tokenize complete Raven blocks so imported names and multiline state are retained.
    blocks = list(re.finditer(r'(<pre[^>]*><code>)(.*?)(</code></pre>)', page, re.S))
    raven_blocks = [block for block in blocks if 'neoIL' not in block.group(1)]
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
    (OUTPUT / 'index.html').write_text(page, encoding='utf-8')
    (OUTPUT / '.nojekyll').touch()
    check = PageCheck()
    check.feed(page)
    check.check()
    print('Built and checked:', OUTPUT)


if __name__ == '__main__':
    main()
